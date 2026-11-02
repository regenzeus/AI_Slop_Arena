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
    /// A run: waves and everything between them.
    Run,
}

/// Everything a button or key can ask for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiAction {
    ToHeroes,
    PickHero(usize),
    StartRun,
    LevelPick(usize),
    ClosePatchNotes,
    Buy(usize),
    Lock(usize),
    Reroll,
    Sell(usize),
    NextWave,
    NewRun,
    Quit,
}

#[derive(Resource)]
pub struct Game {
    pub screen: Screen,
    pub run: Option<Run>,
    pub seed: u64,
    /// Hero highlighted on the hero screen.
    pub hero: usize,
    pub paused: bool,
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
}

impl Game {
    pub fn new(seed: u64) -> Game {
        Game {
            screen: Screen::Title,
            run: None,
            seed,
            hero: 0,
            paused: false,
            accumulator: 0.0,
            events: Vec::new(),
            bot: false,
            speed: 1,
            dirty: true,
            message: None,
            since_end: 0.0,
            banner: None,
        }
    }

    pub fn world(&self) -> Option<&World> {
        self.run.as_ref().and_then(|r| r.world.as_ref())
    }

    pub fn start_run(&mut self) {
        let hero = Hero::ALL[self.hero.min(4)];
        let run = Run::new(self.seed, hero);
        self.seed += 1;
        self.wave_banner(&run);
        self.run = Some(run);
        self.screen = Screen::Run;
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

    fn rule(&mut self, result: Result<(), RuleError>) {
        if let Err(e) = result {
            self.say(e.to_string());
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
            UiAction::NewRun => {
                self.run = None;
                self.screen = Screen::Heroes;
                self.dirty = true;
            }
            UiAction::Quit => return false,
            other => {
                let Some(run) = self.run.as_mut() else { return true };
                let result = match other {
                    UiAction::LevelPick(i) => run.choose_level(i),
                    UiAction::ClosePatchNotes => run.close_patch_notes(),
                    UiAction::Buy(i) => run.buy(i),
                    UiAction::Lock(i) => run.toggle_lock(i),
                    UiAction::Reroll => run.reroll(),
                    UiAction::Sell(i) => run.sell(i),
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
                self.rule(result);
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
        assert_eq!(g.screen, Screen::Run);
        assert_eq!(g.run.as_ref().unwrap().hero, Hero::Teacher);
        assert!(matches!(g.run.as_ref().unwrap().phase, Phase::Wave) && g.banner.is_some());
        g.act(UiAction::Reroll);
        assert!(g.message.is_some(), "rerolling during a wave is refused with a message");
        assert!(!g.act(UiAction::Quit));
    }
}
