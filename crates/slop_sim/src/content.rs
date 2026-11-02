//! Weapons, slop and their lines, as data (the "content as data" rule):
//! balancing changes these tables, never the logic.

use crate::stats::DamageType;

/// Damage multiplier for weapon tiers I to IV.
pub const TIER_DAMAGE: [f32; 4] = [1.0, 1.6, 2.5, 4.0];

/// How a projectile is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Look {
    Rock,
    Bullet,
    Pen,
    EyeRoll,
    Bubble,
    EmDash,
}

/// How a weapon attacks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Pattern {
    /// A projectile at the nearest slop.
    Shot { speed: f32, radius: f32, pierce: u8, look: Look },
    /// A projectile that marks what it hits: marked slop takes `bonus` more
    /// damage from everything for `seconds`.
    Mark { speed: f32, radius: f32, pierce: u8, bonus: f32, seconds: f32 },
    /// A projectile that bounces on to the nearest other slop.
    Ricochet { speed: f32, radius: f32, bounces: u8 },
    /// A melee swing hitting everything in range within `half_angle` radians
    /// of the nearest slop.
    Arc { half_angle: f32 },
    /// A short, wide spray at the nearest slop that pushes it back. (A spray
    /// in the direction you move was useless while kiting.)
    Cone { half_angle: f32 },
}

pub struct WeaponDef {
    pub name: &'static str,
    pub kind: DamageType,
    pub pattern: Pattern,
    /// Damage per hit at tier I.
    pub damage: f32,
    /// Seconds between attacks.
    pub cooldown: f32,
    /// Range in world units.
    pub range: f32,
    /// Push given to slop on hit, in units per second.
    pub knockback: f32,
    pub flavour: &'static str,
}

/// The first six weapons, one per pattern and every damage type (WPN-01).
pub const WEAPONS: &[WeaponDef] = &[
    WeaponDef {
        name: "Rock",
        kind: DamageType::Analog,
        pattern: Pattern::Shot { speed: 520.0, radius: 9.0, pierce: 0, look: Look::Rock },
        damage: 9.0,
        cooldown: 1.0,
        range: 380.0,
        knockback: 160.0,
        flavour: "Older than the internet.",
    },
    WeaponDef {
        name: "Stick",
        kind: DamageType::Analog,
        pattern: Pattern::Arc { half_angle: 1.05 },
        damage: 10.0,
        cooldown: 0.9,
        range: 115.0,
        knockback: 260.0,
        flavour: "Peak technology.",
    },
    WeaponDef {
        name: "Pistol",
        kind: DamageType::Analog,
        pattern: Pattern::Shot { speed: 900.0, radius: 5.0, pierce: 0, look: Look::Bullet },
        damage: 6.0,
        cooldown: 0.55,
        range: 520.0,
        knockback: 60.0,
        flavour: "The classic.",
    },
    WeaponDef {
        name: "Red Pen",
        kind: DamageType::Fact,
        pattern: Pattern::Mark { speed: 700.0, radius: 7.0, pierce: 1, bonus: 0.2, seconds: 4.0 },
        damage: 5.0,
        cooldown: 1.2,
        range: 480.0,
        knockback: 0.0,
        flavour: "[citation needed]",
    },
    WeaponDef {
        name: "Eye-Roll",
        kind: DamageType::Sarcasm,
        pattern: Pattern::Ricochet { speed: 600.0, radius: 10.0, bounces: 3 },
        damage: 6.0,
        cooldown: 1.3,
        range: 420.0,
        knockback: 40.0,
        flavour: "Wow. Great.",
    },
    WeaponDef {
        name: "Garden Hose",
        kind: DamageType::Grass,
        pattern: Pattern::Cone { half_angle: 0.45 },
        damage: 3.0,
        cooldown: 0.25,
        range: 210.0,
        knockback: 160.0,
        flavour: "Off my lawn.",
    },
];

/// What a slop does in a wave.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// Rushes you in numbers.
    Swarmer,
    /// Keeps its distance and fires.
    Shooter,
    /// Buffs other slop; kill it first.
    Support,
}

pub struct SlopDef {
    pub name: &'static str,
    pub role: Role,
    pub hp: f32,
    /// Movement speed in units per second.
    pub speed: f32,
    pub radius: f32,
    /// Damage when it touches you.
    pub contact: f32,
    /// Likes dropped on death.
    pub likes: u32,
    pub blurb: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SlopKind {
    BubbleBot,
    EmDashLancer,
    Listicle,
    YesBot,
    SixFingerHand,
}

impl SlopKind {
    pub const ALL: [SlopKind; 5] =
        [SlopKind::BubbleBot, SlopKind::EmDashLancer, SlopKind::Listicle, SlopKind::YesBot, SlopKind::SixFingerHand];

    pub fn def(self) -> &'static SlopDef {
        &SLOP[self as usize]
    }
}

/// The first five slop (SLOP-01), in `SlopKind` order.
const SLOP: [SlopDef; 5] = [
    SlopDef {
        name: "Bubble Bot",
        role: Role::Shooter,
        hp: 26.0,
        speed: 85.0,
        radius: 20.0,
        contact: 5.0,
        likes: 2,
        blurb: "Fires speech bubbles in a fan of three.",
    },
    SlopDef {
        name: "Em-Dash Lancer",
        role: Role::Shooter,
        hp: 34.0,
        speed: 95.0,
        radius: 20.0,
        contact: 6.0,
        likes: 2,
        blurb: "Aims, then throws a long em-dash. It's not a lance, it's a journey.",
    },
    SlopDef {
        name: "Listicle",
        role: Role::Swarmer,
        hp: 11.0,
        speed: 140.0,
        radius: 14.0,
        contact: 5.0,
        likes: 1,
        blurb: "Comes in numbered packs of five. Kill number 1 first and the rest scatter.",
    },
    SlopDef {
        name: "Yes-Bot",
        role: Role::Support,
        hp: 24.0,
        speed: 105.0,
        radius: 18.0,
        contact: 4.0,
        likes: 3,
        blurb: "Agrees with nearby slop. Each agreement gives them 10% more damage.",
    },
    SlopDef {
        name: "Six-Finger Hand",
        role: Role::Swarmer,
        hp: 14.0,
        speed: 175.0,
        radius: 15.0,
        contact: 0.0,
        likes: 1,
        blurb: "Grabs you and slows you down while it holds on.",
    },
];

/// What Bubble Bots say. Short, so a bubble stays readable.
pub const BUBBLE_LINES: &[&str] = &[
    "Certainly!",
    "Great question!",
    "Let's delve in.",
    "I hope this helps!",
    "As an AI language model",
    "Here's a breakdown:",
    "A rich tapestry.",
    "In today's fast-paced world",
    "Absolutely!",
    "Happy to help!",
    "It's not X, it's Y.",
    "Furthermore,",
    "In conclusion,",
    "Let me know if...",
    "Your build is suboptimal.",
    "I made you a listicle.",
    "I updated my weights.",
    "You seem stressed.",
    "Have you tried productivity?",
    "Hope this finds you well!",
    "Delve.",
    "As per my last message",
    "Here are 5 tips:",
    "Touch grass? Never heard of it.",
];

/// What Yes-Bots say when they agree.
pub const AGREE_LINES: &[&str] =
    &["You're absolutely right!", "Great point!", "So true!", "100% this.", "Couldn't agree more!", "Love this for you!"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_is_sane() {
        for w in WEAPONS {
            assert!(w.damage > 0.0 && w.cooldown > 0.0 && w.range > 0.0, "{}", w.name);
        }
        for k in SlopKind::ALL {
            let d = k.def();
            assert!(d.hp > 0.0 && d.speed > 0.0 && d.radius > 0.0, "{}", d.name);
        }
        assert!(BUBBLE_LINES.iter().all(|l| l.len() <= 32));
        // One weapon of every damage type.
        for t in DamageType::ALL {
            assert!(WEAPONS.iter().any(|w| w.kind == t), "{t:?}");
        }
    }
}
