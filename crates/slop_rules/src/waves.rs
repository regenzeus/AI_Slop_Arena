//! The 20 waves of a run (WAVE-03 to WAVE-06): how long each lasts, how hard
//! it is, who spawns on each platform, and the bosses of waves 10 and 20.

use slop_sim::content::{Archetype, Body, BossKind, Look, Pattern, Platform, Role, WEAPONS};
use slop_sim::spec::SlopSpec;
use slop_sim::world::{ModelSetup, Spawn, WaveSetup};

use crate::training::Training;

pub const WAVES: u32 = 20;

/// Seconds to survive; 0 for boss waves, which last until the boss dies.
pub fn seconds(wave: u32) -> f32 {
    const S: [f32; 20] = [30., 35., 40., 45., 50., 55., 60., 65., 70., 0., 75., 75., 80., 80., 85., 85., 90., 90., 90., 0.];
    S[(wave.clamp(1, WAVES) - 1) as usize]
}

pub fn is_boss(wave: u32) -> bool {
    wave == 10 || wave == 20
}

/// Viral waves: twice the slop at half the HP.
pub fn is_viral(wave: u32) -> bool {
    wave == 5 || wave == 15
}

/// HP, damage and speed multipliers of slop in a wave.
pub fn scaling(wave: u32, platform: Platform) -> (f32, f32, f32) {
    let w = (wave - 1) as f32;
    let hp = 1.0 + 0.13 * w + 0.009 * w * w;
    let damage = 1.0 + 0.07 * w;
    let mut speed = 1.0 + 0.01 * w;
    if platform == Platform::LinkedOut {
        speed += 0.01 * w;
    }
    (hp, damage, speed)
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

/// A wave ready to play, and which specs are which brainrot genome.
pub struct Built {
    pub setup: WaveSetup,
    pub genomes: Vec<(usize, usize)>,
}

/// Builds wave `wave` on `platform` against the current training. `damage`
/// is the damage each weapon has dealt over the run, for The Model.
pub fn build(wave: u32, platform: Platform, training: &Training, damage: &[f32]) -> Built {
    let (mut hp, dmg, speed) = scaling(wave, platform);
    let viral = is_viral(wave);
    if viral {
        hp *= 0.5;
    }
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
    let mut add = |spec: SlopSpec, w: u32, from: f32, group: u8| -> usize {
        // A credit buys 20 base HP: tough slop and packs cost more.
        let cost = spec.hp / hp * group as f32 / 20.0;
        specs.push(spec);
        spawns.push(Spawn { spec: specs.len() - 1, weight: w, from, group, cost });
        specs.len() - 1
    };
    let archetype = |a: Archetype| {
        let mut s = SlopSpec::archetype(a).scaled(hp, dmg, speed);
        s.resist = training.model;
        s
    };
    if !clocktok {
        add(archetype(Archetype::ChadGpt), weight(Role::Shooter, 3), 0.0, 1);
        add(archetype(Archetype::ListicleDrone), weight(Role::Swarmer, 3), 0.0, 5);
        if wave >= 3 {
            add(archetype(Archetype::Clawed), weight(Role::Shooter, 2), 4.0, 1);
        }
        if wave >= 4 {
            add(archetype(Archetype::YesBot), weight(Role::Support, 1), 8.0, 1);
        }
        if wave >= 6 {
            add(archetype(Archetype::Hallucinator), weight(Role::Special, 1), 10.0, 1);
        }
        if wave >= 2 {
            for &a in platform.locals() {
                add(archetype(a), weight(a.def().role, 2), 5.0, 1);
            }
        }
    }
    if wave >= 3 || clocktok {
        for (g, genome) in training.pool.iter().enumerate() {
            let spec = SlopSpec::brainrot(genome.name.clone(), genome.body, genome.object, genome.accessory, genome.toughness, genome.pace, genome.resist)
                .scaled(hp, dmg, speed);
            let group = if genome.body == Body::Shrimp { 4 } else { 1 };
            let w = if clocktok { weight(spec.role, 2) } else { 1 };
            let index = add(spec, w, 2.0, group);
            genomes.push((index, g));
        }
    }

    let w = (wave - 1) as f32;
    let mut rate = (0.35 + 0.12 * w, 0.7 + 0.32 * w);
    if viral {
        rate = (rate.0 * 1.6, rate.1 * 1.6);
    }
    if clocktok {
        // All brainrot, much of it fast: fewer of them.
        rate = (rate.0 * 0.8, rate.1 * 0.8);
    }
    let mut setup = WaveSetup {
        number: wave,
        seconds: seconds(wave),
        platform: Some(platform),
        specs,
        spawns,
        rate,
        elite_chance: if wave >= 6 { 0.015 + 0.003 * (wave - 6) as f32 } else { 0.0 },
        boss: None,
        repost_chance: if platform == Platform::Thredd { 0.25 } else { 0.0 },
        model: ModelSetup::default(),
    };
    if is_boss(wave) {
        setup.rate = (0.5 + 0.03 * w, 1.2 + 0.05 * w);
        let kind = if wave == 20 { BossKind::TheModel } else { platform.boss() };
        let best = training.best();
        setup.model.parts = Some((best.body, best.object, best.accessory));
        if wave == 20 {
            setup.model.collapse = crate::training::collapse(&training.kills);
            let mut order: Vec<usize> = (0..damage.len()).filter(|&d| damage[d] > 0.0).collect();
            order.sort_by(|&a, &b| damage[b].total_cmp(&damage[a]));
            for (k, d) in order.into_iter().take(3).enumerate() {
                setup.model.copies[k] = Some(weapon_look(d));
            }
        }
        let (base_hp, _, _) = kind.base();
        // Platform bosses come at wave 10, when a build is still young.
        let boss_hp = if wave == 20 { base_hp } else { base_hp * 0.55 };
        let mut boss = SlopSpec::boss(kind, boss_hp).scaled(1.0, dmg, 1.0);
        boss.resist = training.model.map(|r| r * 0.5);
        setup.specs.push(boss);
        setup.boss = Some(setup.specs.len() - 1);
    }
    Built { setup, genomes }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slop_sim::rng::Rng;

    // REQ: WAVE-03
    #[test]
    fn twenty_waves_ramp_up_with_bosses_at_ten_and_twenty() {
        let total: f32 = (1..=WAVES).map(seconds).sum();
        assert!((1100.0..1300.0).contains(&total), "{total} s of timed waves, plus two boss fights");
        assert!(is_boss(10) && is_boss(20) && !is_boss(9));
        assert_eq!(seconds(10), 0.0);
        let (hp1, d1, _) = scaling(1, Platform::FaceSlop);
        let (hp20, d20, _) = scaling(20, Platform::FaceSlop);
        assert!(hp20 > hp1 * 4.0 && d20 > d1 * 2.0);
    }

    // REQ: WAVE-04
    #[test]
    fn house_rules_shape_the_spawn_table() {
        let mut rng = Rng::new(1, 1);
        let t = Training::new(&mut rng);
        let clock = build(3, Platform::ClockTok, &t, &[]);
        assert!(clock.setup.specs.iter().all(|s| s.is_brainrot()), "ClockTok is all brainrot");
        let face = build(1, Platform::FaceSlop, &t, &[]);
        assert!(face.setup.specs.iter().all(|s| !s.is_brainrot()), "wave 1 is chatbots only");
        let thredd = build(8, Platform::Thredd, &t, &[]);
        assert!(thredd.setup.repost_chance > 0.0);
        let insta = build(8, Platform::InstaSlop, &t, &[]);
        let tank_weight = |b: &Built| -> u32 {
            b.setup.spawns.iter().filter(|s| b.setup.specs[s.spec].role == Role::Tank).map(|s| s.weight).sum()
        };
        assert!(tank_weight(&insta) > tank_weight(&build(8, Platform::MeTube, &t, &[])));
    }

    // REQ: WAVE-05, PLAT-01
    #[test]
    fn boss_waves_carry_their_boss_and_the_model_learns_your_weapons() {
        let mut rng = Rng::new(1, 1);
        let t = Training::new(&mut rng);
        let ten = build(10, Platform::MeTube, &t, &[]);
        let boss = &ten.setup.specs[ten.setup.boss.unwrap()];
        assert_eq!(boss.name, BossKind::ContentFarm.name());
        let mut dmg = vec![0.0; WEAPONS.len()];
        dmg[0] = 500.0;
        dmg[8] = 300.0;
        let twenty = build(20, Platform::MeTube, &t, &dmg);
        assert_eq!(twenty.setup.specs[twenty.setup.boss.unwrap()].name, "The Model");
        assert_eq!(twenty.setup.model.copies[0], Some(Look::Rock));
        assert_eq!(twenty.setup.model.copies[1], Some(Look::Pen));
    }
}
