//! The client's state: which screen is up and the run being played.

use bevy::prelude::*;
use slop_rules::run::RuleError;
use slop_rules::{Hero, Run};
use slop_sim::world::World;
use slop_sim::Event;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Title,
    Heroes,
    /// Once per run, before wave 1: the platform, its slop and its boss.
    Briefing,
    /// A run: waves and everything between them.
    Run,
}

/// Everything a button or key can ask for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiAction {
    ToHeroes,
    PickHero(usize),
    /// Pick hero `i` and start a run with it.
    StartAs(usize),
    StartRun,
    /// Leave the briefing and play wave 1.
    BeginRun,
    LevelPick(usize),
    Buy(usize),
    Lock(usize),
    Reroll,
    /// Sell weapon slot `i` if asked twice in a row.
    SellAsk(usize),
    NextWave,
    Resume,
    NewRun,
    Quit,
}

/// Sounds the menus ask for; played and cleared by `audio`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cue {
    Buy,
    Reroll,
    Level,
    Error,
    Go,
    Tick,
}

#[derive(Resource)]
pub struct Game {
    pub screen: Screen,
    pub run: Option<Run>,
    pub seed: u64,
    /// Hero highlighted on the hero screen.
    pub hero: usize,
    pub paused: bool,
    /// Autoplay holds the wave still for a screenshot, without the pause screen.
    pub frozen: bool,
    /// Simulation time owed to the fixed tick.
    pub accumulator: f32,
    /// Events of the ticks run this frame, read by effects and sound.
    pub events: Vec<Event>,
    /// The bots play instead of you (autoplay).
    pub bot: bool,
    /// Ticks per frame multiplier (autoplay fast-forward).
    pub speed: u32,
    /// The screens need rebuilding.
    pub dirty: bool,
    /// A message for a few seconds, such as "Not enough Clout."
    pub message: Option<(String, f32)>,
    /// Seconds since the last wave ended, so overlays wait for the effects.
    pub since_end: f32,
    /// A big centred banner: title, line, seconds left.
    pub banner: Option<(String, String, f32)>,
    /// A weapon slot waiting for a second press to be sold, and seconds left.
    pub confirm_sell: Option<(usize, f32)>,
    /// Levels earned so far in this wave, and whether one was earned this frame.
    pub wave_levels: u32,
    pub level_flash: bool,
    pub cues: Vec<Cue>,
    /// The run the bots play behind the title and hero screens.
    pub demo: Option<Run>,
}

impl Game {
    pub fn new(seed: u64) -> Game {
        Game {
            screen: Screen::Title,
            run: None,
            seed,
            hero: 0,
            paused: false,
            frozen: false,
            accumulator: 0.0,
            events: Vec::new(),
            bot: false,
            speed: 1,
            dirty: true,
            message: None,
            since_end: 0.0,
            banner: None,
            confirm_sell: None,
            wave_levels: 0,
            level_flash: false,
            cues: Vec::new(),
            demo: None,
        }
    }

    /// The run on screen: yours, or the bots' demo behind the menus.
    pub fn shown_run(&self) -> Option<&Run> {
        match self.screen {
            Screen::Title | Screen::Heroes => self.demo.as_ref(),
            Screen::Briefing | Screen::Run => self.run.as_ref(),
        }
    }

    pub fn world(&self) -> Option<&World> {
        self.shown_run().and_then(|r| r.world.as_ref())
    }

    /// Levels the Likes of the running wave are worth so far, and progress
    /// towards the next one (0 to 1).
    pub fn levels_in_wave(&self) -> (u32, f32) {
        let Some(run) = self.run.as_ref() else { return (0, 0.0) };
        let in_wave = matches!(run.phase, slop_rules::Phase::Wave);
        let mut xp = run.xp + if in_wave { self.world().map_or(0, |w| w.clout) } else { 0 };
        let mut level = run.level;
        let mut gained = 0;
        while xp >= slop_rules::levels::xp_for(level + 1) {
            xp -= slop_rules::levels::xp_for(level + 1);
            level += 1;
            gained += 1;
        }
        (gained, xp as f32 / slop_rules::levels::xp_for(level + 1) as f32)
    }

    pub fn start_run(&mut self) {
        let hero = Hero::ALL[self.hero.min(4)];
        let run = Run::new(self.seed, hero);
        self.seed += 1;
        self.demo = None;
        self.run = Some(run);
        self.screen = Screen::Briefing;
        self.paused = false;
        self.accumulator = 0.0;
        self.since_end = 0.0;
        self.dirty = true;
    }

    /// The banner shown as a wave starts.
    pub fn wave_banner(&mut self, run: &Run) {
        let platform = run.platform;
        let line = if run.wave == 1 {
            format!("{}: {} {}", platform.name(), platform.hazard(), platform.house_rule())
        } else if slop_rules::waves::is_viral(run.wave) {
            "Viral wave: twice the slop, half the HP.".to_string()
        } else if slop_rules::waves::is_boss(run.wave) {
            "Boss wave: it ends when the boss is deleted.".to_string()
        } else {
            format!("Survive {} seconds.", slop_rules::waves::seconds(run.wave) as u32)
        };
        self.banner = Some((format!("WAVE {}", run.wave), line, 3.0));
    }

    pub fn say(&mut self, words: impl Into<String>) {
        self.message = Some((words.into(), 2.5));
        self.dirty = true;
    }

    fn rule(&mut self, result: Result<(), RuleError>, cue: Cue) {
        match result {
            Ok(()) => self.cues.push(cue),
            Err(e) => {
                self.cues.push(Cue::Error);
                self.say(e.to_string());
            }
        }
        self.dirty = true;
    }

    /// Does what a button or key asked (UI-08).
    pub fn act(&mut self, action: UiAction) -> bool {
        match action {
            UiAction::ToHeroes => {
                self.screen = Screen::Heroes;
                self.dirty = true;
            }
            UiAction::PickHero(i) => {
                self.hero = i.min(4);
                self.dirty = true;
            }
            UiAction::StartRun => self.start_run(),
            UiAction::StartAs(i) => {
                self.hero = i.min(4);
                self.start_run();
            }
            UiAction::BeginRun => {
                self.cues.push(Cue::Go);
                if let Some(run) = self.run.take() {
                    self.wave_banner(&run);
                    self.run = Some(run);
                }
                self.screen = Screen::Run;
                self.accumulator = 0.0;
                self.dirty = true;
            }
            UiAction::Resume => {
                self.paused = false;
                self.dirty = true;
            }
            UiAction::NewRun => {
                self.run = None;
                self.screen = Screen::Heroes;
                self.dirty = true;
            }
            UiAction::SellAsk(i) => {
                let Some(run) = self.run.as_mut() else { return true };
                if self.confirm_sell.is_some_and(|(s, _)| s == i) {
                    self.confirm_sell = None;
                    let result = run.sell(i);
                    self.rule(result, Cue::Buy);
                } else if i < run.weapons.len() {
                    let words = format!("Sell {} for {} Clout? Press again.", slop_sim::content::WEAPONS[run.weapons[i].def].name, run.sell_value(i));
                    self.confirm_sell = Some((i, 3.0));
                    self.say(words);
                }
            }
            UiAction::Quit => return false,
            other => {
                let Some(run) = self.run.as_mut() else { return true };
                let cue = match other {
                    UiAction::Buy(_) => Cue::Buy,
                    UiAction::Reroll => Cue::Reroll,
                    UiAction::LevelPick(_) => Cue::Level,
                    UiAction::NextWave => Cue::Go,
                    _ => Cue::Tick,
                };
                let result = match other {
                    UiAction::LevelPick(i) => run.choose_level(i),
                    UiAction::Buy(i) => run.buy(i),
                    UiAction::Lock(i) => run.toggle_lock(i),
                    UiAction::Reroll => run.reroll(),
                    UiAction::NextWave => {
                        let r = run.next_wave();
                        if r.is_ok() {
                            let run = self.run.take().expect("still here");
                            self.wave_banner(&run);
                            self.run = Some(run);
                            self.accumulator = 0.0;
                        }
                        r
                    }
                    _ => Ok(()),
                };
                self.rule(result, cue);
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slop_rules::Phase;

    // REQ: UI-08
    #[test]
    fn actions_walk_from_title_to_a_run_and_report_rule_errors() {
        let mut g = Game::new(1);
        g.act(UiAction::ToHeroes);
        assert_eq!(g.screen, Screen::Heroes);
        g.act(UiAction::PickHero(2));
        g.act(UiAction::StartRun);
        assert_eq!(g.screen, Screen::Briefing, "a run opens with its briefing");
        g.act(UiAction::BeginRun);
        assert_eq!(g.screen, Screen::Run);
        assert_eq!(g.run.as_ref().unwrap().hero, Hero::Teacher);
        assert!(matches!(g.run.as_ref().unwrap().phase, Phase::Wave) && g.banner.is_some());
        g.act(UiAction::Reroll);
        assert!(g.message.is_some(), "rerolling during a wave is refused with a message");
        assert!(!g.act(UiAction::Quit));
    }
}
