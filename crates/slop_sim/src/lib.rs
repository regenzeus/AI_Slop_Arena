//! Deterministic arena simulation for AI Slop Arena.
//!
//! No engine code lives here: the Bevy client and the headless tools both
//! drive [`World`], one fixed tick at a time. Transcendental maths goes
//! through [`math`] and randomness through [`rng`], so a seed means the same
//! wave on every machine. Between waves, `slop_rules` builds each wave's
//! [`world::WaveSetup`] and your [`world::Loadout`].

mod ai;
mod boss;
pub mod bot;
mod combat;
pub mod content;
pub mod events;
pub mod hazards;
pub mod map;
pub mod math;
mod player;
pub mod rng;
pub mod spatial;
pub mod spec;
pub mod stats;
pub mod vec;
pub mod world;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_v4;

pub use content::{Archetype, BossKind, Look, Platform};
pub use events::Event;
pub use spec::SlopSpec;
pub use stats::{DamageType, Stats};
pub use vec::V2;
pub use world::{Input, Outcome, World};
