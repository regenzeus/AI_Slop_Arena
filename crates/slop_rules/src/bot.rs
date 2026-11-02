//! Bots that play whole runs: they steer with `slop_sim::bot` and make every
//! choice between waves. The lab measures balance with them (BAL-02) and the
//! client's autoplay uses them.

use slop_sim::content::WEAPONS;
use slop_sim::rng::Rng;

use crate::heroes::Hero;
use crate::items::{ItemKind, StatKind, ITEMS};
use crate::run::{Phase, Run, WEAPON_SLOTS};
use crate::shop::Goods;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    /// Upgrades what it has, spreads damage types, saves for good items.
    Heuristic,
    /// Buys and picks at random.
    Random,
}

/// How a bot's run ended.
#[derive(Clone, Debug)]
pub struct Summary {
    pub victory: bool,
    /// The wave it reached (20 when it won).
    pub wave: u32,
    pub level: u32,
    pub weapons: Vec<(String, u8)>,
}

fn stat_value(stat: StatKind, run: &Run) -> f32 {
    let s = run.stats();
    match stat {
        StatKind::MentalHealth => {
            if s.max_mental_health < 160.0 {
                3.0
            } else {
                1.5
            }
        }
        StatKind::Conviction | StatKind::Focus => 3.0,
        StatKind::Composure => 2.5,
        StatKind::Scepticism => 2.0,
        StatKind::SelfCare | StatKind::Reach | StatKind::Speed => 1.5,
        StatKind::Luck => 0.8,
        StatKind::Curiosity => 0.5,
    }
}

/// How much the heuristic bot wants an offer.
fn offer_value(run: &Run, i: usize) -> f32 {
    let o = &run.shop.offers[i];
    match o.goods {
        Goods::Weapon { def, tier } => {
            if let Some(w) = run.weapons.iter().find(|w| w.def == def) {
                if w.tier >= 4 {
                    0.0
                } else {
                    9.0 + w.tier as f32
                }
            } else if run.weapons.len() < WEAPON_SLOTS {
                let new_type = !run.weapons.iter().any(|w| WEAPONS[w.def].kind == WEAPONS[def].kind);
                7.0 + tier as f32 + if new_type { 2.0 } else { 0.0 }
            } else {
                0.0
            }
        }
        Goods::Item { item } => {
            let d = &ITEMS[item];
            let base = match d.kind {
                ItemKind::Active => 5.0,
                ItemKind::Healthy => 4.5,
                ItemKind::Hustle if run.wave <= 8 => 5.0,
                ItemKind::Hustle => 1.0,
                ItemKind::Risky => 3.0,
            };
            base + d.tier as f32
        }
    }
}

/// Makes one decision outside a wave. Returns false when there is nothing to decide.
pub fn decide(run: &mut Run, policy: Policy, rng: &mut Rng) -> bool {
    match run.phase.clone() {
        Phase::LevelUp(offers) => {
            let pick = match policy {
                Policy::Random => rng.below(offers.len() as u32) as usize,
                Policy::Heuristic => (0..offers.len())
                    .max_by(|&a, &b| {
                        let v = |k: usize| stat_value(offers[k].stat, run) * offers[k].tier as f32;
                        v(a).total_cmp(&v(b))
                    })
                    .unwrap_or(0),
            };
            run.choose_level(pick).is_ok()
        }
        Phase::PatchNotes(_) => run.close_patch_notes().is_ok(),
        Phase::Shop => {
            match policy {
                Policy::Random => {
                    for i in 0..run.shop.offers.len() {
                        if rng.chance(0.5) {
                            let _ = run.buy(i);
                        }
                    }
                }
                Policy::Heuristic => {
                    let mut rerolled = 0;
                    loop {
                        let best = (0..run.shop.offers.len())
                            .filter(|&i| !run.shop.offers[i].sold && run.shop.offers[i].price <= run.clout)
                            .map(|i| (i, offer_value(run, i)))
                            .filter(|(_, v)| *v >= 6.0)
                            .max_by(|a, b| a.1.total_cmp(&b.1));
                        match best {
                            Some((i, _)) => {
                                let _ = run.buy(i);
                            }
                            None if rerolled < 2 && run.clout >= run.shop.reroll_cost() * 4 + 20 => {
                                let _ = run.reroll();
                                rerolled += 1;
                            }
                            None => break,
                        }
                    }
                }
            }
            run.next_wave().is_ok()
        }
        Phase::Wave | Phase::Over { .. } => false,
    }
}

/// Plays a whole run with the movement bot and `policy`.
pub fn play(seed: u64, hero: Hero, policy: Policy) -> Summary {
    let mut run = Run::new(seed, hero);
    let mut rng = Rng::new(seed, 11);
    loop {
        match run.phase {
            Phase::Over { .. } => break,
            Phase::Wave => {
                let input = slop_sim::bot::steer(run.world.as_ref().expect("a wave is running"));
                run.step(input);
            }
            _ => {
                decide(&mut run, policy, &mut rng);
            }
        }
    }
    Summary {
        victory: matches!(run.phase, Phase::Over { victory: true }),
        wave: run.reports.last().map_or(1, |r| r.wave),
        level: run.level,
        weapons: run.weapons.iter().map(|w| (WEAPONS[w.def].name.to_string(), w.tier)).collect(),
    }
}
