//! The client's state: which screen is up and the run being played.

use bevy::prelude::*;
use slop_rules::memory::{Memory, Temperature};
use slop_rules::run::RuleError;
use slop_rules::{Hero, Phase, Run};
use slop_sim::world::World;
use slop_sim::Event;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Title,
    Heroes,
    /// A run: the Feed, with level-ups, chests and machines over it.
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
    /// Level-up pick `i`.
    Choose(usize),
    Reroll,
    /// Close a chest, or walk away from a machine.
    Close,
    /// Buy vending machine offer `i`.
    Buy(usize),
    /// Take shrine blessing `i`.
    Bless(usize),
    /// Take glitch `i` after a boss.
    Glitch(usize),
    /// Throw item offer `i` of a recycle bin away.
    Recycle(usize),
    /// Choose Temperature `i` on the hero screen (TEMP-01).
    SetTemperature(usize),
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
    Chest,
    /// A legendary in a chest, or a glitch: the big fanfare.
    Legendary,
}

#[derive(Resource)]
pub struct Game {
    pub screen: Screen,
    pub run: Option<Run>,
    pub seed: u64,
    /// Hero highlighted on the hero screen.
    pub hero: usize,
    pub paused: bool,
    /// Autoplay holds the run still for a screenshot, without the pause screen.
    pub frozen: bool,
    /// Simulation time owed to the fixed tick.
    pub accumulator: f32,
    /// How far between the last tick and the next the screen is, 0 to 1:
    /// positions are drawn in between, so motion is smooth (FEEL-04).
    pub alpha: f32,
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
    /// Seconds since the run ended, so the end screen waits for the effects.
    pub since_end: f32,
    /// Seconds since a choice opened, so a held key does not pick at once.
    pub since_choice: f32,
    /// A big centred banner: title, line, seconds left.
    pub banner: Option<(String, String, f32)>,
    /// News from the run (patch notes, the next platform), with seconds left.
    pub toasts: Vec<(String, f32)>,
    /// The level the HUD last showed, to flash "LEVEL UP".
    pub shown_level: u32,
    pub level_flash: bool,
    pub cues: Vec<Cue>,
    /// The run the bots play behind the title and hero screens.
    pub demo: Option<Run>,
    /// The Temperature chosen on the hero screen (TEMP-01).
    pub temperature: Temperature,
    /// What the slop remember about you between runs (NEMESIS-01).
    pub memory: Memory,
    /// The run on screen was already written to memory.
    pub remembered: bool,
    /// A dash pressed since the last tick (DASH-01).
    pub dash_queued: bool,
    /// Seconds since a chest opened, for its reveal (LOOT-02).
    pub chest_opened: f32,
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
            alpha: 0.0,
            events: Vec::new(),
            bot: false,
            speed: 1,
            dirty: true,
            message: None,
            since_end: 0.0,
            since_choice: 0.0,
            banner: None,
            toasts: Vec::new(),
            shown_level: 1,
            level_flash: false,
            cues: Vec::new(),
            demo: None,
            temperature: Temperature::Default,
            memory: Memory::default(),
            remembered: false,
            dash_queued: false,
            chest_opened: 10.0,
        }
    }

    /// The run on screen: yours, or the bots' demo behind the menus.
    pub fn shown_run(&self) -> Option<&Run> {
        match self.screen {
            Screen::Title | Screen::Heroes => self.demo.as_ref(),
            Screen::Run => self.run.as_ref(),
        }
    }

    pub fn world(&self) -> Option<&World> {
        self.shown_run().and_then(|r| r.world.as_ref())
    }

    pub fn start_run(&mut self) {
        let hero = Hero::ALL[self.hero.min(Hero::ALL.len() - 1)];
        let run = Run::with(self.seed, hero, self.temperature, self.memory.clone());
        self.remembered = false;
        self.seed += 1;
        self.demo = None;
        let platform = run.platform;
        self.banner = Some((format!("TODAY ON {}", platform.name().to_uppercase()), format!("{} Its boss comes in 4:30.", platform.hazard()), 4.0));
        self.run = Some(run);
        self.screen = Screen::Run;
        self.paused = false;
        self.accumulator = 0.0;
        self.since_end = 0.0;
        self.shown_level = 1;
        self.toasts.clear();
        self.cues.push(Cue::Go);
        self.dirty = true;
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

    /// Whether a choice may be taken: overlays ignore input for a moment so
    /// a key held while playing does not pick something by accident.
    pub fn choice_ready(&self) -> bool {
        self.bot || self.since_choice > 0.5
    }

    /// Does what a button or key asked (UI-08). Returns false to quit.
    pub fn act(&mut self, action: UiAction) -> bool {
        let choosing = matches!(action, UiAction::Choose(_) | UiAction::Buy(_) | UiAction::Bless(_) | UiAction::Glitch(_) | UiAction::Recycle(_) | UiAction::Close | UiAction::Reroll);
        if choosing && !self.choice_ready() {
            return true;
        }
        match action {
            UiAction::ToHeroes => {
                self.screen = Screen::Heroes;
                self.dirty = true;
            }
            UiAction::PickHero(i) => {
                self.hero = i.min(Hero::ALL.len() - 1);
                self.dirty = true;
            }
            UiAction::SetTemperature(i) => {
                self.temperature = Temperature::ALL[i.min(Temperature::ALL.len() - 1)];
                self.dirty = true;
            }
            UiAction::StartRun => self.start_run(),
            UiAction::StartAs(i) => {
                self.hero = i.min(Hero::ALL.len() - 1);
                self.start_run();
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
            UiAction::Quit => return false,
            _ => {
                let Some(run) = self.run.as_mut() else { return true };
                let (result, cue) = match action {
                    UiAction::Choose(i) => (run.choose(i), Cue::Level),
                    UiAction::Reroll => (run.reroll(), Cue::Reroll),
                    UiAction::Close => (run.close(), Cue::Tick),
                    UiAction::Buy(i) => (run.buy(i), Cue::Buy),
                    UiAction::Bless(i) => (run.bless(i), Cue::Level),
                    UiAction::Glitch(i) => (run.take_glitch(i), Cue::Level),
                    UiAction::Recycle(i) => (run.recycle(i), Cue::Buy),
                    _ => (Ok(()), Cue::Tick),
                };
                self.rule(result, cue);
                self.since_choice = 0.0;
            }
        }
        true
    }

    /// The run is waiting for a choice (an overlay is up).
    pub fn choosing(&self) -> bool {
        self.run.as_ref().is_some_and(|r| matches!(r.phase, Phase::LevelUp(_) | Phase::Chest(_) | Phase::Vending { .. } | Phase::Shrine { .. } | Phase::Glitch(_) | Phase::Recycle { .. }))
    }
}

/// Where the slop keep what they remember about you.
fn memory_path() -> Option<std::path::PathBuf> {
    let base = std::env::var_os("APPDATA").or_else(|| std::env::var_os("HOME"))?;
    Some(std::path::PathBuf::from(base).join("AI Slop Arena").join("memory.txt"))
}

/// Reads what the slop remember; nothing there means a first run.
pub fn load_memory() -> Memory {
    memory_path().and_then(|p| std::fs::read_to_string(p).ok()).map_or_else(Memory::default, |t| Memory::from_text(&t))
}

/// Writes what the slop remember. Losing it only costs a grudge.
pub fn save_memory(memory: &Memory) {
    let Some(path) = memory_path() else { return };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Err(e) = std::fs::write(&path, memory.to_text()) {
        eprintln!("cannot save {}: {e}", path.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: UI-08
    #[test]
    fn actions_walk_from_title_into_the_feed_and_report_rule_errors() {
        let mut g = Game::new(1);
        g.act(UiAction::ToHeroes);
        assert_eq!(g.screen, Screen::Heroes);
        g.act(UiAction::PickHero(2));
        g.act(UiAction::StartRun);
        assert_eq!(g.screen, Screen::Run, "straight into the Feed");
        assert_eq!(g.run.as_ref().unwrap().hero, Hero::Teacher);
        assert!(g.banner.is_some());
        g.since_choice = 1.0;
        g.act(UiAction::Reroll);
        assert!(g.message.is_some(), "rerolling while playing is refused with a message");
        assert!(!g.act(UiAction::Quit));
    }
}
