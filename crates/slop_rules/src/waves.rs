//! The director of the Feed (RUN-10, RUN-11): who spawns, how many, and how
//! tough, as the minutes go by; and the bosses at the end of each platform.
//! Nothing here counts waves any more: difficulty is a function of time.

use slop_sim::content::{Archetype, Body, BossKind, Look, Pattern, Platform, Role, WEAPONS};
use slop_sim::spec::{SlopSpec, Traits};
use slop_sim::world::{ModelSetup, Spawn};

use crate::training::Training;

/// Seconds on a platform before its boss arrives.
pub const BOSS_AFTER: f32 = 270.0;
/// Seconds after the sixth boss before The Model arrives.
pub const MODEL_AFTER: f32 = 90.0;

/// How far into the old 20-wave curve `minutes` of the Feed are: wave 20's
/// strength arrives around half an hour in, and it keeps climbing.
pub fn stage(minutes: f32) -> f32 {
    1.0 + minutes * 0.62
}

/// HP, damage and speed multipliers of slop at `stage`.
pub fn scaling(stage: f32, platform: Platform) -> (f32, f32, f32) {
    let w = (stage - 1.0).max(0.0);
    // Fragile at first, like a horde should be; steep later.
    let hp = 0.3 + 0.12 * w + 0.05 * w * w;
    let damage = 1.0 + 0.13 * w;
    let mut speed = 1.0 + 0.01 * w.min(30.0);
    if platform == Platform::LinkedOut {
        speed += 0.15;
    }
    (hp, damage, speed)
}

/// Spawn credits per second at `stage`; a credit buys 20 base HP of slop.
pub fn rate(stage: f32) -> f32 {
    let w = stage - 1.0;
    2.0 + 0.6 * w + 0.03 * w * w
}

/// What a weapon looks like when The Model copies it.
pub fn weapon_look(def: usize) -> Look {
    match WEAPONS[def].pattern {
        Pattern::Shot { look, .. } | Pattern::Burst { look, .. } | Pattern::Lob { look, .. } => look,
        Pattern::Mark { .. } => Look::Pen,
        Pattern::Ricochet { .. } => Look::EyeRoll,
        Pattern::Boomerang { .. } => Look::Frisbee,
        Pattern::Stun { .. } => Look::Captcha,
        Pattern::Chain { .. } | Pattern::Reflect => Look::Bubble,
        _ => Look::Rock,
    }
}

/// A spawn table ready to hand to the world, and which specs are which
/// brainrot genome.
pub struct Table {
    pub specs: Vec<SlopSpec>,
    pub spawns: Vec<Spawn>,
    pub rate: f32,
    pub elite_chance: f32,
    pub repost_chance: f32,
    pub genomes: Vec<(usize, usize)>,
}

/// Who spawns on `platform` at `stage`, against the current training.
pub fn table(stage: f32, platform: Platform, training: &Training) -> Table {
    let (hp, dmg, speed) = scaling(stage, platform);
    let mut specs = Vec::new();
    let mut spawns = Vec::new();
    let mut genomes = Vec::new();
    let clocktok = platform == Platform::ClockTok;
    // Weights are in halves, so house rules can multiply by 1.5.
    let weight = |role: Role, base: u32| -> u32 {
        let k = match (platform, role) {
            (Platform::FaceSlop, Role::Support) => 4,
            (Platform::MeTube, Role::Shooter) => 3,
            (Platform::InstaSlop, Role::Tank) => 4,
            _ => 2,
        };
        base * k
    };
    let mut add = |spec: SlopSpec, w: u32, group: u8| -> usize {
        let cost = spec.hp / hp * group as f32 / 20.0;
        specs.push(spec);
        spawns.push(Spawn { spec: specs.len() - 1, weight: w, from: 0.0, group, cost });
        specs.len() - 1
    };
    let archetype = |a: Archetype| {
        let mut s = SlopSpec::archetype(a).scaled(hp, dmg, speed);
        s.resist = training.model;
        s
    };
    if !clocktok {
        add(archetype(Archetype::ChadGpt), weight(Role::Shooter, 3), 1);
        add(archetype(Archetype::ListicleDrone), weight(Role::Swarmer, 4), 5);
        if stage >= 2.0 {
            add(archetype(Archetype::Clawed), weight(Role::Shooter, 2), 1);
        }
        if stage >= 3.0 {
            add(archetype(Archetype::YesBot), weight(Role::Support, 1), 1);
        }
        if stage >= 4.5 {
            add(archetype(Archetype::Hallucinator), weight(Role::Special, 1), 1);
        }
        for &a in platform.locals() {
            add(archetype(a), weight(a.def().role, 2), 1);
        }
    }
    if stage >= 2.5 || clocktok {
        for (g, genome) in training.pool.iter().enumerate() {
            let spec = SlopSpec::brainrot(genome.name.clone(), genome.body, genome.object, genome.accessory, genome.toughness, genome.pace, genome.resist).scaled(hp, dmg, speed);
            let group = if genome.body == Body::Shrimp { 4 } else { 1 };
            let w = if clocktok { weight(spec.role, 2) } else { 1 + (stage / 6.0) as u32 };
            let index = add(spec, w, group);
            genomes.push((index, g));
        }
    }
    let mut r = rate(stage);
    if clocktok {
        // All brainrot, much of it fast: fewer of them.
        r *= 0.8;
    }
    Table {
        specs,
        spawns,
        rate: r,
        elite_chance: if stage >= 4.0 { 0.006 + 0.0012 * stage } else { 0.0 },
        repost_chance: if platform == Platform::Thredd { 0.25 } else { 0.0 },
        genomes,
    }
}

/// Elite affixes (ELITE-01).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Affix {
    /// Takes half damage from everything but Fact.
    Verified,
    /// Bursts into four copies.
    Viral,
    /// Drops Clout.
    Sponsored,
    /// Fast.
    Trending,
    /// Leaves slowing puddles.
    Toxic,
}

impl Affix {
    pub const ALL: [Affix; 5] = [Affix::Verified, Affix::Viral, Affix::Sponsored, Affix::Trending, Affix::Toxic];

    pub fn name(self) -> &'static str {
        ["Verified", "Viral", "Sponsored", "Trending", "Toxic"][self as usize]
    }

    pub fn blurb(self) -> &'static str {
        [
            "Half damage from everything but Fact.",
            "Bursts into four copies when deleted.",
            "Drops a pile of Clout.",
            "Fast. Very fast.",
            "Leaves puddles of downvotes behind.",
        ][self as usize]
    }
}

/// An elite of `base` with an affix: bigger, tougher, and it drops a chest.
pub fn elite(base: &SlopSpec, affix: Affix) -> SlopSpec {
    let mut s = base.clone().elite();
    s.name = format!("{} {}", affix.name(), base.name);
    match affix {
        Affix::Verified => s.traits.0 |= Traits::VERIFIED,
        Affix::Viral => s.traits.0 |= Traits::VIRAL,
        Affix::Sponsored => s.traits.0 |= Traits::SPONSORED,
        Affix::Trending => s.speed *= 1.6,
        Affix::Toxic => s.traits.0 |= Traits::TOXIC,
    }
    s
}

/// The boss of `platform` (or The Model) arriving `minutes` into the Feed,
/// with what The Model knows about you. `damage` is the damage each weapon
/// has dealt over the run.
pub fn boss(kind: BossKind, minutes: f32, training: &Training, damage: &[f32]) -> (SlopSpec, ModelSetup) {
    let mut model = ModelSetup::default();
    let best = training.best();
    model.parts = Some((best.body, best.object, best.accessory));
    if kind == BossKind::TheModel {
        model.collapse = crate::training::collapse(&training.kills);
        let mut order: Vec<usize> = (0..damage.len()).filter(|&d| damage[d] > 0.0).collect();
        order.sort_by(|&a, &b| damage[b].total_cmp(&damage[a]));
        for (k, d) in order.into_iter().take(3).enumerate() {
            model.copies[k] = Some(weapon_look(d));
        }
    }
    let (base_hp, _, _) = kind.base();
    let k = if kind == BossKind::TheModel { 0.45 + 0.05 * minutes } else { 0.3 + 0.09 * minutes };
    let (_, dmg, _) = scaling(stage(minutes), Platform::FaceSlop);
    let mut spec = SlopSpec::boss(kind, base_hp * k).scaled(1.0, dmg.min(3.0), 1.0);
    spec.resist = training.model.map(|r| r * 0.5);
    (spec, model)
}

#[cfg(test)]
mod tests {
    use super::*;
    use slop_sim::rng::Rng;

    // REQ: RUN-10
    #[test]
    fn the_feed_gets_harder_with_time() {
        let (hp1, d1, _) = scaling(stage(0.0), Platform::FaceSlop);
        let (hp30, d30, _) = scaling(stage(30.0), Platform::FaceSlop);
        assert!(hp30 > hp1 * 8.0 && d30 > d1 * 2.0);
        assert!(rate(stage(30.0)) > rate(stage(0.0)) * 5.0);
        let (hp60, ..) = scaling(stage(60.0), Platform::FaceSlop);
        assert!(hp60 > hp30 * 2.5, "it never stops climbing");
    }

    // REQ: WAVE-04
    #[test]
    fn house_rules_shape_the_spawn_table() {
        let mut rng = Rng::new(1, 1);
        let t = Training::new(&mut rng);
        let clock = table(3.0, Platform::ClockTok, &t);
        assert!(clock.specs.iter().all(|s| s.is_brainrot()), "ClockTok is all brainrot");
        let face = table(1.0, Platform::FaceSlop, &t);
        assert!(face.specs.iter().all(|s| !s.is_brainrot()), "the first minutes are chatbots");
        assert!(table(8.0, Platform::Thredd, &t).repost_chance > 0.0);
        let tank_weight = |b: &Table| -> u32 { b.spawns.iter().filter(|s| b.specs[s.spec].role == Role::Tank).map(|s| s.weight).sum() };
        assert!(tank_weight(&table(8.0, Platform::InstaSlop, &t)) > tank_weight(&table(8.0, Platform::MeTube, &t)));
    }

    // REQ: RUN-11, BOSS-07
    #[test]
    fn bosses_grow_with_time_and_the_model_copies_your_weapons() {
        let mut rng = Rng::new(1, 1);
        let t = Training::new(&mut rng);
        let (early, _) = boss(BossKind::ContentFarm, 4.5, &t, &[]);
        let (late, _) = boss(BossKind::ContentFarm, 25.0, &t, &[]);
        assert!(late.hp > early.hp * 2.0);
        let mut dmg = vec![0.0; WEAPONS.len()];
        dmg[0] = 500.0;
        dmg[8] = 300.0;
        let (model, setup) = boss(BossKind::TheModel, 30.0, &t, &dmg);
        assert_eq!(model.name, "The Model");
        assert_eq!(setup.copies[0], Some(Look::Rock));
        assert_eq!(setup.copies[1], Some(Look::Pen));
    }

    // REQ: ELITE-01
    #[test]
    fn elites_carry_their_affix() {
        let base = SlopSpec::archetype(Archetype::ChadGpt);
        let e = elite(&base, Affix::Verified);
        assert!(e.elite && e.traits.has(Traits::VERIFIED) && e.hp > base.hp * 2.0);
        assert_eq!(e.name, "Verified ChadGPT");
        assert!(elite(&base, Affix::Trending).speed > base.speed * 1.5);
    }
}
