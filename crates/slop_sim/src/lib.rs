//! Deterministic arena simulation for AI Slop Arena.
//!
//! No engine code lives here: the Bevy client and the headless lab both drive
//! [`World`], one fixed tick at a time. Transcendental maths goes through
//! [`math`] and randomness through [`rng`], so a seed means the same wave on
//! every machine.

pub mod bot;
pub mod content;
pub mod events;
pub mod math;
pub mod rng;
pub mod spatial;
pub mod stats;
pub mod vec;
pub mod world;

#[cfg(test)]
mod tests;

pub use content::{Look, SlopKind};
pub use events::Event;
pub use stats::{DamageType, Stats};
pub use vec::V2;
pub use world::{Input, Outcome, World};
