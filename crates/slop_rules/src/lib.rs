//! The run of AI Slop Arena: everything between waves.
//!
//! A [`run::Run`] owns the 20 waves, your weapons and items, levels, the
//! shop, and the [`training::Training`] that makes the slop evolve against
//! you. It builds each wave for `slop_sim` and books its results.

pub mod bot;
pub mod heroes;
pub mod items;
pub mod levels;
pub mod run;
pub mod shop;
pub mod training;
pub mod waves;

#[cfg(test)]
mod tests;

pub use heroes::Hero;
pub use run::{Phase, Run};
