//! Your ten stats and the four damage types (STAT-01, DMG-01).

/// The four kinds of damage a human has against slop. The slop will later
/// learn to resist each of them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DamageType {
    Analog,
    Fact,
    Sarcasm,
    Grass,
}

impl DamageType {
    pub const ALL: [DamageType; 4] = [DamageType::Analog, DamageType::Fact, DamageType::Sarcasm, DamageType::Grass];

    pub fn name(self) -> &'static str {
        match self {
            DamageType::Analog => "Analog",
            DamageType::Fact => "Fact",
            DamageType::Sarcasm => "Sarcasm",
            DamageType::Grass => "Grass",
        }
    }
}

/// Composure at which damage taken is halved.
pub const COMPOSURE_HALF: f32 = 15.0;
/// Scepticism never dodges more than this share of hits.
pub const SCEPTICISM_CAP: f32 = 0.6;
/// Mental Health healed per second for each point of Self-Care.
pub const SELF_CARE_RATE: f32 = 0.2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stats {
    pub max_mental_health: f32,
    /// Armour: damage taken is multiplied by 15 / (15 + Composure).
    pub composure: f32,
    pub self_care: f32,
    /// Attack speed bonus, as a fraction (0.1 = +10%).
    pub focus: f32,
    /// Damage bonus, as a fraction.
    pub conviction: f32,
    /// Range bonus, as a fraction.
    pub reach: f32,
    /// Movement speed multiplier.
    pub speed: f32,
    /// Chance to ignore a hit entirely.
    pub scepticism: f32,
    /// Pickup radius for Likes, in world units.
    pub curiosity: f32,
    pub luck: f32,
}

impl Default for Stats {
    fn default() -> Stats {
        Stats {
            max_mental_health: 100.0,
            composure: 0.0,
            self_care: 0.0,
            focus: 0.0,
            conviction: 0.0,
            reach: 0.0,
            speed: 1.0,
            scepticism: 0.0,
            curiosity: 140.0,
            luck: 0.0,
        }
    }
}

impl Stats {
    /// Damage left after Composure. Negative Composure makes hits hurt more,
    /// up to double.
    pub fn damage_taken(&self, raw: f32) -> f32 {
        raw * COMPOSURE_HALF / (COMPOSURE_HALF + self.composure.max(-COMPOSURE_HALF / 2.0))
    }

    /// Chance to ignore a hit.
    pub fn dodge_chance(&self) -> f32 {
        self.scepticism.clamp(0.0, SCEPTICISM_CAP)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: STAT-01
    #[test]
    fn fifteen_composure_halves_damage() {
        let mut s = Stats::default();
        assert_eq!(s.damage_taken(10.0), 10.0);
        s.composure = 15.0;
        assert!((s.damage_taken(10.0) - 5.0).abs() < 1e-5);
        s.composure = -100.0;
        assert!((s.damage_taken(10.0) - 20.0).abs() < 1e-4);
    }

    // REQ: STAT-01
    #[test]
    fn scepticism_is_capped() {
        let s = Stats { scepticism: 2.0, ..Stats::default() };
        assert_eq!(s.dodge_chance(), SCEPTICISM_CAP);
    }
}
