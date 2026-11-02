//! The shop that opens after every wave (SHOP-01 to SHOP-04): offers, tier
//! odds, prices, rerolls and locks.

use slop_sim::content::WEAPONS;
use slop_sim::rng::Rng;

use crate::items::{tier_price, ITEMS};

/// What an offer sells.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Goods {
    Weapon { def: usize, tier: u8 },
    Item { item: usize },
}

impl Goods {
    pub fn name(&self) -> &'static str {
        match *self {
            Goods::Weapon { def, .. } => WEAPONS[def].name,
            Goods::Item { item } => ITEMS[item].name,
        }
    }

    pub fn tier(&self) -> u8 {
        match *self {
            Goods::Weapon { tier, .. } => tier,
            Goods::Item { item } => ITEMS[item].tier,
        }
    }
}

/// Why an offer is special.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Note {
    /// Opened from a Loot Box an elite dropped: free.
    LootBox,
    /// You ended the last wave below 30% Mental Health: half price.
    Mercy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Offer {
    pub goods: Goods,
    pub price: u32,
    pub locked: bool,
    pub sold: bool,
    pub note: Option<Note>,
}

#[derive(Clone, Debug, Default)]
pub struct Shop {
    pub offers: Vec<Offer>,
    /// Rerolls paid for in this shop.
    pub rerolls: u32,
    pub free_rerolls: u32,
}

impl Shop {
    /// 5 Clout, then 3 more each time in the same shop; free rerolls first.
    pub fn reroll_cost(&self) -> u32 {
        if self.free_rerolls > 0 {
            0
        } else {
            5 + 3 * self.rerolls
        }
    }
}

/// Chance of tiers I to IV in a wave's shop (SHOP-02).
pub fn tier_odds(wave: u32) -> [f32; 4] {
    match wave {
        0..=5 => [0.85, 0.15, 0.0, 0.0],
        6..=10 => [0.55, 0.35, 0.10, 0.0],
        11..=15 => [0.30, 0.40, 0.25, 0.05],
        _ => [0.15, 0.35, 0.38, 0.12],
    }
}

/// A tier for an offer; each point of Luck moves 1% of the odds up a tier.
pub fn roll_tier(rng: &mut Rng, wave: u32, luck: f32) -> u8 {
    let odds = tier_odds(wave);
    let top = odds.iter().rposition(|p| *p > 0.0).unwrap_or(0);
    let roll = rng.f32() + luck.max(0.0) * 0.01;
    let mut acc = 0.0;
    for (t, p) in odds.iter().enumerate().take(top + 1) {
        acc += p;
        if roll < acc {
            return t as u8 + 1;
        }
    }
    top as u8 + 1
}

/// Price of goods in a wave: weapons cost a third more than items of the
/// same tier, and prices rise 4% per wave (SHOP-03).
pub fn price(goods: Goods, wave: u32, discount: f32, markup: f32) -> u32 {
    let base = match goods {
        Goods::Weapon { tier, .. } => tier_price(tier) * 4 / 3,
        Goods::Item { item } => tier_price(ITEMS[item].tier),
    };
    let inflation = 1.04f32.powi(wave.saturating_sub(1) as i32);
    (base as f32 * inflation * (1.0 - discount) * (1.0 + markup)).round().max(1.0) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: SHOP-02
    #[test]
    fn later_waves_and_luck_bring_better_tiers() {
        let mean = |wave: u32, luck: f32| {
            let mut rng = Rng::new(9, 1);
            (0..2000).map(|_| roll_tier(&mut rng, wave, luck) as f32).sum::<f32>() / 2000.0
        };
        assert!(mean(1, 0.0) < 1.3);
        assert!(mean(18, 0.0) > 2.3);
        assert!(mean(8, 30.0) > mean(8, 0.0));
        let mut rng = Rng::new(1, 1);
        assert!((0..500).all(|_| roll_tier(&mut rng, 1, 0.0) <= 2));
    }

    // REQ: SHOP-03
    #[test]
    fn prices_rise_with_waves_and_discounts_apply() {
        let rock = Goods::Weapon { def: 0, tier: 1 };
        assert_eq!(price(rock, 1, 0.0, 0.0), 20);
        assert!((41..=43).contains(&price(rock, 20, 0.0, 0.0)));
        assert!(price(rock, 10, 0.08, 0.0) < price(rock, 10, 0.0, 0.0));
        let shop = Shop { rerolls: 2, ..Shop::default() };
        assert_eq!(shop.reroll_cost(), 11);
    }
}
