//! The client's state: which screen is up and the wave being played.

use bevy::prelude::*;
use slop_sim::world::{WaveConfig, World};
use slop_sim::Event;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Title,
    Wave,
    /// The wave is over: cleared or Burnout.
    End,
}

#[derive(Resource)]
pub struct Game {
    pub screen: Screen,
    pub world: World,
    pub seed: u64,
    pub paused: bool,
    /// Simulation time owed to the fixed tick.
    pub accumulator: f32,
    /// Seconds since the wave ended, so the end screen can wait for the effects.
    pub since_end: f32,
    /// Events of the ticks run this frame, read by effects and sound.
    pub events: Vec<Event>,
    /// The bot steers instead of the keyboard.
    pub bot: bool,
}

impl Game {
    pub fn new(seed: u64) -> Game {
        Game {
            screen: Screen::Title,
            world: World::new(seed, WaveConfig::first()),
            seed,
            paused: false,
            accumulator: 0.0,
            since_end: 0.0,
            events: Vec::new(),
            bot: false,
        }
    }

    /// Starts a fresh wave; each new attempt uses the next seed.
    pub fn start(&mut self) {
        self.world = World::new(self.seed, WaveConfig::first());
        self.seed += 1;
        self.screen = Screen::Wave;
        self.paused = false;
        self.accumulator = 0.0;
        self.since_end = 0.0;
    }
}
