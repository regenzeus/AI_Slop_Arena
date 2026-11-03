//! Bots that play whole runs: they steer with `slop_sim::bot` and make every
//! choice the Feed asks for. The lab measures balance with them (BAL-02) and
//! the client's autoplay and title-screen demo use them.

use slop_sim::content::{merge_of, WEAPONS};
use slop_sim::rng::Rng;

use crate::heroes::Hero;
use crate::items::{ItemKind, StatKind, ITEMS};
use crate::run::{Choice, Phase, Run};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    /// Levels what it has, chases merges, spreads damage types.
    Heuristic,
    /// Picks at random.
    Random,
}

/// How a bot's run ended.
#[derive(Clone, Debug)]
pub struct Summary {
    /// The Model fell before Burnout.
    pub victory: bool,
    /// Minutes survived.
    pub minutes: f32,
    pub bosses: u32,
    pub level: u32,
    pub kills: u32,
    pub weapons: Vec<(String, u8)>,
}

fn stat_value(stat: StatKind, run: &Run) -> f32 {
    let s = run.stats();
    match stat {
        StatKind::MentalHealth if s.max_mental_health < 180.0 => 3.0,
        StatKind::MentalHealth => 1.5,
        StatKind::Conviction | StatKind::Focus => 3.0,
        StatKind::Composure => 2.5,
        StatKind::Scepticism => 2.0,
        StatKind::SelfCare | StatKind::Reach | StatKind::Speed => 1.5,
        StatKind::Luck => 0.8,
        StatKind::Curiosity => 1.0,
        StatKind::Boundaries => 2.0,
        StatKind::Hustle => 0.6,
    }
}

/// How much the heuristic bot wants a choice.
fn value(run: &Run, c: Choice) -> f32 {
    match c {
        Choice::WeaponLevel(d) => {
            let partner = merge_of(d).is_some_and(|(o, _)| run.weapons.iter().any(|w| w.def == o));
            10.0 + if partner { 3.0 } else { 0.0 }
        }
        Choice::NewWeapon(d) => {
            let partner = merge_of(d).is_some_and(|(o, _)| run.weapons.iter().any(|w| w.def == o));
            let new_type = !run.weapons.iter().any(|w| WEAPONS[w.def].kind == WEAPONS[d].kind);
            let room = if run.weapons.len() + 1 < run.weapon_slots() { 3.0 } else { 0.0 };
            8.0 + room + if partner { 4.0 } else { 0.0 } + if new_type { 1.5 } else { 0.0 }
        }
        Choice::ItemLevel(i) | Choice::NewItem(i) => {
            let d = &ITEMS[i];
            let base = match d.kind {
                ItemKind::Active => 6.0,
                ItemKind::Healthy => 6.5,
                ItemKind::Hustle => 3.5,
                ItemKind::Risky => 5.0,
                ItemKind::Movement => 5.5,
                ItemKind::Defence => 6.5,
                ItemKind::Legendary => 12.0,
            };
            base + d.tier as f32
        }
        Choice::Stat(up) => stat_value(up.stat, run) * (1.0 + up.tier as f32),
        Choice::Heal => 2.0,
        Choice::Clout(_) => 1.0,
        Choice::Glitch(_) => 14.0,
    }
}

fn best(run: &Run, choices: &[Choice]) -> usize {
    (0..choices.len()).max_by(|&a, &b| value(run, choices[a]).total_cmp(&value(run, choices[b]))).unwrap_or(0)
}

/// Makes the decision the run is waiting for. Returns false when there is
/// nothing to decide.
pub fn decide(run: &mut Run, policy: Policy, rng: &mut Rng) -> bool {
    match run.phase.clone() {
        Phase::LevelUp(choices) => {
            let pick = match policy {
                Policy::Random => rng.below(choices.len() as u32) as usize,
                Policy::Heuristic => {
                    let i = best(run, &choices);
                    // Nothing worth it: one reroll if it is free.
                    if value(run, choices[i]) < 6.0 && run.reroll_cost() == 0 && run.reroll().is_ok() {
                        return true;
                    }
                    i
                }
            };
            run.choose(pick).is_ok()
        }
        Phase::Vending { offers, .. } => {
            let affordable: Vec<usize> = (0..offers.len()).filter(|&i| offers[i].1 <= run.clout).collect();
            let pick = match policy {
                Policy::Random => affordable.first().copied(),
                Policy::Heuristic => affordable.into_iter().max_by(|&a, &b| value(run, offers[a].0).total_cmp(&value(run, offers[b].0))),
            };
            match pick {
                Some(i) => run.buy(i).is_ok(),
                None => run.close().is_ok(),
            }
        }
        Phase::Shrine { offers, .. } => {
            let i = rng.below(offers.len() as u32) as usize;
            run.bless(i).is_ok()
        }
        Phase::Chest(_) => run.close().is_ok(),
        // The bots keep their items.
        Phase::Recycle { .. } => run.close().is_ok(),
        Phase::Glitch(offers) => run.take_glitch(rng.below(offers.len() as u32) as usize).is_ok(),
        Phase::Playing | Phase::Over { .. } => false,
    }
}

/// Plays a whole run with the movement bot and `policy`, up to `limit`
/// minutes.
pub fn play(seed: u64, hero: Hero, policy: Policy, limit: f32) -> Summary {
    let mut run = Run::new(seed, hero);
    let mut rng = Rng::new(seed, 11);
    loop {
        match run.phase {
            Phase::Over { .. } => break,
            Phase::Playing => {
                if run.minutes() >= limit {
                    break;
                }
                let input = slop_sim::bot::steer(run.world.as_ref().expect("the Feed runs"));
                run.step(input);
            }
            _ => {
                decide(&mut run, policy, &mut rng);
            }
        }
    }
    Summary {
        victory: run.model_beaten,
        minutes: run.minutes(),
        bosses: run.bosses_beaten,
        level: run.level,
        kills: run.kills(),
        weapons: run.weapons.iter().map(|w| (WEAPONS[w.def].name.to_string(), w.level)).collect(),
    }
}
