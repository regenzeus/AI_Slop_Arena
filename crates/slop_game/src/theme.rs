//! Colours shared by the HUD and the arena.

use bevy::prelude::*;
use slop_sim::DamageType;

/// Beyond the arena walls.
pub const VOID: Color = Color::srgb(0.16, 0.16, 0.22);
pub const WALL: Color = Color::srgb(0.24, 0.24, 0.32);
pub const INK: Color = Color::srgb(0.13, 0.12, 0.17);
pub const PAPER: Color = Color::srgb(0.97, 0.96, 0.93);
pub const PANEL: Color = Color::srgba(0.13, 0.12, 0.17, 0.92);
pub const QUIET: Color = Color::srgb(0.62, 0.62, 0.68);
/// Anything that hurts you, and nothing else (FAIR-04).
pub const DANGER: Color = Color::srgb(1.0, 0.18, 0.58);
/// The progress bar of a video player.
pub const PLAYER_RED: Color = Color::srgb(0.93, 0.16, 0.16);
pub const LIKE: Color = Color::srgb(1.0, 0.31, 0.43);
pub const AGREE: Color = Color::srgb(0.20, 0.55, 0.22);

/// Damage numbers and weapon chips are coloured by damage type.
pub fn type_color(t: DamageType) -> Color {
    match t {
        DamageType::Analog => Color::srgb(0.78, 0.33, 0.24),
        DamageType::Fact => Color::srgb(0.18, 0.36, 0.84),
        DamageType::Sarcasm => Color::srgb(0.80, 0.60, 0.0),
        DamageType::Grass => Color::srgb(0.20, 0.58, 0.27),
    }
}

/// Mental Health bar colour: green when calm, red near Burnout.
pub fn mental_health_color(share: f32) -> Color {
    if share > 0.6 {
        Color::srgb(0.30, 0.72, 0.38)
    } else if share > 0.3 {
        Color::srgb(0.95, 0.70, 0.20)
    } else {
        Color::srgb(0.90, 0.25, 0.25)
    }
}
