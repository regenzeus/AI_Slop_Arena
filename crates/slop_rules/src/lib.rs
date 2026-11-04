//! The rules of AI Slop Arena around the simulation.
//!
//! A [`run::Run`] owns one endless trip through the Feed: the world, your
//! weapons, items and levels, chests and merges, the map's machines, and the
//! [`training::Training`] that makes the slop evolve against you. The
//! director in [`waves`] decides who spawns as time goes by.

pub mod bot;
pub mod heroes;
pub mod items;
pub mod levels;
pub mod memory;
pub mod run;
pub mod training;
pub mod waves;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_v4;

pub use heroes::Hero;
pub use run::{Phase, Run};
