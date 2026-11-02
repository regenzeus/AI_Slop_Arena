//! Levels: Clout earned is also XP. Each level gained offers four stat
//! upgrades after the wave; you pick one (LEVEL-01).

use slop_sim::rng::Rng;

use crate::items::StatKind;

/// XP needed to go from level `n` to `n + 1`.
pub fn xp_for(level: u32) -> u32 {
    10 + 8 * level
}

/// One upgrade on offer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatUp {
    pub stat: StatKind,
    pub amount: f32,
    /// 1 to 3: plain, good, great. Luck makes better rolls likelier.
    pub tier: u8,
}

impl StatUp {
    pub fn describe(&self) -> String {
        self.stat.describe(self.amount)
    }
}

/// The plain amount of each stat upgrade.
fn base(stat: StatKind) -> f32 {
    match stat {
        StatKind::MentalHealth => 8.0,
        StatKind::Composure => 2.0,
        StatKind::SelfCare => 1.0,
        StatKind::Focus => 6.0,
        StatKind::Conviction => 6.0,
        StatKind::Reach => 10.0,
        StatKind::Speed => 4.0,
        StatKind::Scepticism => 4.0,
        StatKind::Curiosity => 30.0,
        StatKind::Luck => 2.0,
    }
}

/// Four different upgrades.
pub fn offers(rng: &mut Rng, luck: f32) -> Vec<StatUp> {
    let mut pool: Vec<StatKind> = StatKind::ALL.to_vec();
    let mut out = Vec::new();
    while out.len() < 4 {
        let stat = pool.swap_remove(rng.below(pool.len() as u32) as usize);
        let roll = rng.f32() - luck * 0.01;
        let tier = if roll < 0.08 {
            3
        } else if roll < 0.30 {
            2
        } else {
            1
        };
        let amount = base(stat) * [1.0, 1.5, 2.0][tier as usize - 1];
        out.push(StatUp { stat, amount, tier });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: LEVEL-01
    #[test]
    fn four_different_offers_and_luck_helps() {
        let mut rng = Rng::new(1, 1);
        let o = offers(&mut rng, 0.0);
        assert_eq!(o.len(), 4);
        for i in 0..4 {
            for j in i + 1..4 {
                assert_ne!(o[i].stat, o[j].stat);
            }
        }
        let great = |luck: f32| {
            let mut rng = Rng::new(2, 1);
            (0..500).flat_map(|_| offers(&mut rng, luck)).filter(|u| u.tier == 3).count()
        };
        assert!(great(20.0) > great(0.0));
        assert!(xp_for(1) < xp_for(10));
    }
}
