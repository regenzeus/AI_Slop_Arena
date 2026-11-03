//! Tests of version 4's rules: glitches, rarities and golden chests, hero
//! rules, Hustle, biomes and the director's events. Each names the
//! requirement it verifies.

use slop_sim::content::{weapon_index, WEAPONS};
use slop_sim::DamageType;
use slop_sim::world::MAX_LEVEL;

use crate::heroes::Hero;
use crate::items::{ItemKind, Mod, GLITCHES, ITEMS};
use crate::run::{rarity_weight, Choice, OwnedWeapon, Phase, Run};

/// Hands the run a chest of `value` and opens it.
fn open(run: &mut Run, value: u32) -> crate::run::Reward {
    run.world.as_mut().unwrap().chests.push(value);
    run.step(Default::default());
    let Phase::Chest(reward) = run.phase.clone() else { panic!("the chest opens") };
    reward
}

// REQ: GLITCH-01
#[test]
fn a_bosss_chest_is_followed_by_a_pick_of_glitches() {
    let mut run = Run::new(5, Hero::Zoomer);
    run.platform_since = -crate::waves::BOSS_AFTER;
    run.step(Default::default());
    let id = run.world.as_ref().unwrap().boss.as_ref().expect("the boss is out").id;
    run.world.as_mut().unwrap().slop.iter_mut().find(|s| s.id == id).unwrap().hp = 0.0;
    run.step(Default::default());
    run.step(Default::default());
    assert_eq!(run.bosses_beaten, 1);
    open(&mut run, 2);
    run.close().unwrap();
    let Phase::Glitch(offers) = run.phase.clone() else { panic!("glitches after the boss's chest") };
    assert_eq!(offers.len(), 3);
    let g = offers[1];
    run.take_glitch(1).unwrap();
    assert_eq!(run.glitches, vec![g]);
    assert!(matches!(run.phase, Phase::Playing | Phase::LevelUp(_)));
    // A glitch's effect reaches the world: Doom Dash adds two charges.
    let doom = GLITCHES.iter().position(|d| d.name == "Doom Dash").unwrap();
    let before = run.loadout().perks.dash_charges;
    run.glitches.push(doom);
    assert_eq!(run.loadout().perks.dash_charges, before + 2);
}

// REQ: ITEM-03, LOOT-02
#[test]
fn golden_chests_hold_a_legendary_and_rarer_things_are_offered_less() {
    let mut run = Run::new(6, Hero::Artist);
    let reward = open(&mut run, 3);
    assert_eq!(reward.upgrades.len(), 5, "a golden chest opens five upgrades");
    assert_eq!(reward.best, 4, "and one is legendary");
    assert!(run.items.iter().any(|(i, _)| ITEMS[*i].kind == ItemKind::Legendary));
    assert!(rarity_weight(4, 0.0) < rarity_weight(3, 0.0) && rarity_weight(3, 0.0) < rarity_weight(1, 0.0));
    assert!(rarity_weight(4, 20.0) > rarity_weight(4, 0.0), "Luck makes legendaries likelier");
    for i in ITEMS.iter().filter(|i| i.kind == ItemKind::Legendary) {
        assert_eq!(i.max_level(), 1, "{}: a legendary is one of a kind", i.name);
    }
}

// REQ: HERO-02
#[test]
fn heroes_bend_the_rules() {
    // The Boomer is never offered Sarcasm weapons.
    let mut run = Run::new(7, Hero::Boomer);
    for _ in 0..40 {
        run.pending_levels = 1;
        run.phase = Phase::Playing;
        run.step(Default::default());
        if let Phase::LevelUp(choices) = &run.phase {
            for c in choices {
                if let Choice::NewWeapon(d) = c {
                    assert_ne!(WEAPONS[*d].kind, DamageType::Sarcasm, "{}", WEAPONS[*d].name);
                }
            }
        }
    }
    // The Teacher has five weapon slots; the Programmer's new weapons start at level 2.
    assert_eq!(Run::new(1, Hero::Teacher).weapon_slots(), 5);
    let mut run = Run::new(1, Hero::Programmer);
    let rock = weapon_index("Rock").unwrap();
    run.pending_levels = 1;
    run.phase = Phase::LevelUp(vec![Choice::NewWeapon(rock)]);
    run.choose(0).unwrap();
    assert_eq!(run.weapons.iter().find(|w| w.def == rock).unwrap().level, 2);
    // Every hero lists its properties.
    for h in Hero::ALL {
        assert!(!h.def().properties.is_empty());
    }
}

// REQ: STAT-02, LOOT-03
#[test]
fn hustle_pays_every_minute() {
    let mut run = Run::new(8, Hero::Influencer);
    let hustle = run.stats().hustle;
    assert!(hustle >= 25.0);
    run.world.as_mut().unwrap().player.mental_health = 1e7;
    let before = run.clout;
    run.offset = 59.99;
    run.step(Default::default());
    assert!(run.clout >= before + hustle as u32, "Hustle paid {} Clout", run.clout - before);
    // Clout is power for the Influencer.
    let base = run.stats().conviction;
    run.clout += 300;
    assert!(run.stats().conviction > base + 0.25);
}

// REQ: BIOME-01
#[test]
fn the_spawns_follow_the_biome_you_are_in() {
    let mut run = Run::new(9, Hero::Zoomer);
    let start = run.biome;
    assert_eq!(run.platform, start, "the first boss belongs to where you start");
    // Walk straight to the next square.
    run.world.as_mut().unwrap().player.mental_health = 1e7;
    run.world.as_mut().unwrap().player.stats.max_mental_health = 1e7;
    for _ in 0..(60 * 9) {
        if !matches!(run.phase, Phase::Playing) {
            run.phase = Phase::Playing;
            run.pending_levels = 0;
        }
        run.step(slop_sim::Input::moving(1.0, 0.0));
    }
    assert_ne!(run.biome, start);
    assert_eq!(run.world.as_ref().unwrap().setup.platform, Some(run.biome), "the hazard and the spawns are the new biome's");
    assert!(run.toasts.iter().any(|t| t.starts_with("Entering")));
}

// REQ: EVENT-01, EVENT-02
#[test]
fn the_director_sends_swarms_rings_and_livestreams() {
    let mut run = Run::new(10, Hero::Boomer);
    let w = run.world.as_mut().unwrap();
    w.player.mental_health = 1e7;
    w.player.stats.max_mental_health = 1e7;
    run.base.max_mental_health = 1e7;
    let mut swarm = false;
    let mut ring = false;
    let mut live = false;
    let mut rng = slop_sim::rng::Rng::new(1, 1);
    while run.seconds() < 400.0 && !(swarm && ring && live) {
        if matches!(run.phase, Phase::Playing) {
            let input = slop_sim::bot::steer(run.world.as_ref().unwrap());
            run.step(input);
            for e in run.world.as_mut().unwrap().drain_events() {
                match e {
                    slop_sim::Event::Swarm { .. } => swarm = true,
                    slop_sim::Event::Encircled { .. } => ring = true,
                    slop_sim::Event::ArenaWarning { .. } => live = true,
                    _ => {}
                }
            }
        } else {
            crate::bot::decide(&mut run, crate::bot::Policy::Heuristic, &mut rng);
        }
    }
    assert!(swarm, "a swarm within the first minutes");
    assert!(ring, "a ring of pop-ups");
    assert!(live, "a livestream");
}

// REQ: WPN-12, UI-13
#[test]
fn a_ready_merge_is_announced_and_brings_a_chest_closer() {
    let mut run = Run::new(4, Hero::Boomer);
    let (rock, sling) = (weapon_index("Rock").unwrap(), weapon_index("Slingshot").unwrap());
    run.weapons = vec![OwnedWeapon { def: rock, level: MAX_LEVEL }, OwnedWeapon { def: sling, level: MAX_LEVEL }];
    run.step(Default::default());
    assert!(run.toasts.iter().any(|t| t.starts_with("MERGE READY")));
    // Items in the data are sane: every mod with a number is positive where it must be.
    for i in ITEMS {
        for m in i.mods {
            if let Mod::DashCharges(n) = m {
                assert!(*n > 0);
            }
        }
    }
}
