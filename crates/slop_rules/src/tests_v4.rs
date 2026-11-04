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
    for _ in 0..(60 * 14) {
        if !matches!(run.phase, Phase::Playing) {
            run.phase = Phase::Playing;
            run.pending_levels = 0;
        }
        run.step(slop_sim::Input::moving(1.0, 0.0));
    }
    assert_ne!(run.biome, start);
    assert_eq!(run.world.as_ref().unwrap().setup.platform, Some(run.biome), "the hazard and the spawns are the new biome's");
    assert!(run.toasts.iter().all(|t| !t.starts_with("Entering")), "no announcement: the floor shows it");
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
    while run.seconds() < 800.0 && !(swarm && ring && live) {
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
    run.weapons = vec![OwnedWeapon { def: rock, level: MAX_LEVEL, agent: false }, OwnedWeapon { def: sling, level: MAX_LEVEL, agent: false }];
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

/// Puts a prop of `kind` under you and stands in it until it is used.
fn use_prop(run: &mut Run, kind: slop_sim::map::PropKind) {
    let w = run.world.as_mut().unwrap();
    let me = w.player.pos;
    w.map.props.push(slop_sim::map::Prop { id: 9_999_999, kind, pos: me, charge: 0, state: slop_sim::map::PropState::Idle, left: 0, price: 0 });
    for _ in 0..80 {
        if !matches!(run.phase, Phase::Playing) {
            break;
        }
        run.step(Default::default());
    }
}

// REQ: MAP-05
#[test]
fn recycle_bins_fine_tuning_and_data_brokers() {
    // A recycle bin takes an item and pays for it.
    let mut run = Run::new(3, Hero::Boomer);
    let item = run.items[0].0;
    let clout = run.clout;
    use_prop(&mut run, slop_sim::map::PropKind::RecycleBin);
    assert!(matches!(run.phase, Phase::Recycle { .. }));
    run.recycle(0).unwrap();
    assert!(run.items.iter().all(|(i, _)| *i != item) && run.clout > clout);
    // A fine-tuning station sells weapon levels.
    let mut run = Run::new(3, Hero::Boomer);
    use_prop(&mut run, slop_sim::map::PropKind::FineTuning);
    let Phase::Vending { offers, .. } = run.phase.clone() else { panic!("the station opens") };
    assert!(offers.iter().all(|(c, _)| matches!(c, Choice::WeaponLevel(_))));
    // A data broker buys Mental Health for an epic or legendary item.
    let mut run = Run::new(3, Hero::Boomer);
    let before = run.stats().max_mental_health;
    use_prop(&mut run, slop_sim::map::PropKind::DataBroker);
    let Phase::Chest(reward) = run.phase.clone() else { panic!("the deal is shown") };
    assert!(reward.best >= 3);
    assert!(run.stats().max_mental_health < before);
}

// REQ: CHEST-02
#[test]
fn a_maxed_build_gets_paid_instead_of_stopped() {
    let mut run = Run::new(4, Hero::Teacher);
    // Fill every slot with something at its maximum.
    // Merged weapons cannot level or merge any more.
    run.weapons = (0..WEAPONS.len()).filter(|&d| WEAPONS[d].evolved).take(run.weapon_slots()).map(|d| OwnedWeapon { def: d, level: MAX_LEVEL, agent: false }).collect();
    run.items = ITEMS.iter().enumerate().filter(|(_, i)| i.max_level() == 1).take(crate::run::ITEM_SLOTS).map(|(k, _)| (k, 1)).collect();
    let clout = run.clout;
    run.world.as_mut().unwrap().chests.push(1);
    run.step(Default::default());
    assert!(matches!(run.phase, Phase::Playing), "no chest screen when nothing can be upgraded");
    assert!(run.clout > clout);
}

// REQ: NEMESIS-01, CARD-01
#[test]
fn what_killed_you_comes_back_and_the_card_tells_the_truth() {
    use crate::memory::{Memory, Temperature};
    use slop_sim::content::Archetype;
    use slop_sim::spec::SlopLook;
    let memory = Memory { runs: 3, favourite: Some(0), killer: Some("Glass Fruit".into()), killer_look: Some(SlopLook::Archetype(Archetype::GlassFruit)), killer_count: 1, ..Memory::default() };
    let mut run = Run::with(21, Hero::Boomer, Temperature::Default, memory);
    assert!(run.training.model[0] > 0.05, "it starts ahead on your favourite damage type");
    // Survive to the Nemesis.
    run.base.max_mental_health = 1e7;
    run.world.as_mut().unwrap().player.stats.max_mental_health = 1e7;
    run.world.as_mut().unwrap().player.mental_health = 1e7;
    let mut rng = slop_sim::rng::Rng::new(1, 1);
    let mut seen = false;
    while run.seconds() < 170.0 && !seen {
        if matches!(run.phase, Phase::Playing) {
            run.step(Default::default());
            let w = run.world.as_ref().unwrap();
            seen = w.setup.specs.iter().any(|s| s.name == "Nemesis Glass Fruit" && s.elite);
        } else {
            crate::bot::decide(&mut run, crate::bot::Policy::Heuristic, &mut rng);
        }
    }
    assert!(seen, "the thing that deleted you is back, by name");
    // The card and the next memory come from what really happened.
    let w = run.world.as_mut().unwrap();
    let spec = w.setup.specs.iter().position(|s| s.name == "Nemesis Glass Fruit").unwrap();
    w.last_hurt = spec as u16;
    let card = run.model_card().join("\n");
    assert!(card.contains("Human-4.0") && card.contains("Glass Fruit, 2 times"), "{card}");
    let next = run.remembered();
    assert_eq!((next.runs, next.killer_count, next.killer.as_deref()), (4, 2, Some("Glass Fruit")));
}

// REQ: TEMP-01
#[test]
fn temperature_sets_the_crowd_and_the_rewards() {
    use crate::memory::{Memory, Temperature};
    let hot = Run::with(5, Hero::Boomer, Temperature::Unhinged, Memory::default());
    let cold = Run::with(5, Hero::Boomer, Temperature::Low, Memory::default());
    assert!(hot.loadout().perks.like_bonus > cold.loadout().perks.like_bonus);
    assert!(hot.world.as_ref().unwrap().hallucinate > 0.0 && cold.world.as_ref().unwrap().hallucinate == 0.0);
}

// REQ: CH-01
#[test]
fn death_is_the_next_chapter_and_the_build_starts_over() {
    use crate::chapters::Chapter;
    let mut run = Run::new(12, Hero::Boomer);
    run.weapons[0].agent = true;
    run.automation = 20;
    let before = run.seconds();
    run.world.as_mut().unwrap().player.mental_health = 0.0;
    for _ in 0..3 {
        run.step(Default::default());
    }
    let Phase::ChapterCard(Chapter::Office) = run.phase else { panic!("the Office follows the Feed, got {:?}", std::mem::discriminant(&run.phase)) };
    assert_eq!(run.chapters.len(), 1);
    run.next_chapter().unwrap();
    assert_eq!(run.chapter, Chapter::Office);
    assert_eq!(WEAPONS[run.weapons[0].def].name, "Stapler");
    assert!(run.weapons.iter().any(|w| w.agent), "agents come with you");
    assert_eq!((run.level, run.automation), (1, 20));
    assert!(run.seconds() >= before, "the clock carries on");
    assert_eq!(run.world.as_ref().unwrap().player.mental_health, run.stats().max_mental_health);
}

// REQ: AUTO-01, AUTO-02, AUTO-03
#[test]
fn the_easy_option_costs_automation_and_takes_things_away() {
    use crate::run::Delegate;
    let mut run = Run::new(13, Hero::Boomer);
    let def = run.weapons[0].def;
    let choices = vec![Choice::WeaponLevel(def), Choice::Delegate(Delegate::Agent(def))];
    assert_eq!(crate::bot::best(&run, &choices), 1, "the bot takes the easy option every time");
    run.pending_levels = 1;
    run.phase = Phase::LevelUp(choices);
    run.choose(1).unwrap();
    assert!(run.weapons[0].agent && run.automation == 8);
    assert!(run.world.as_ref().unwrap().player.weapons[0].agent, "the world knows");
    // An agent is no longer yours to level.
    run.pending_levels = 1;
    run.phase = Phase::Playing;
    run.step(Default::default());
    if let Phase::LevelUp(choices) = &run.phase {
        assert!(!choices.contains(&Choice::WeaponLevel(def)));
    }
    // Thresholds.
    run.phase = Phase::Playing;
    run.pending_levels = 0;
    run.automation = 30;
    run.pending_levels = 1;
    run.step(Default::default());
    assert!(run.suggested.is_some(), "suggestions from 25");
    assert_eq!(run.reroll(), Err(crate::run::RuleError::Automated));
    run.automation = 50;
    assert_eq!(run.auto_pick_after(), Some(2.0));
    run.automation = 75;
    run.chapter_since = run.seconds();
    assert!(run.autopilot_now(), "autopilot from 75");
}

// REQ: END-01
#[test]
fn full_automation_ends_in_the_takeover() {
    let mut run = Run::new(14, Hero::Boomer);
    run.automation = 100;
    run.phase = Phase::Playing;
    run.step(Default::default());
    assert!(matches!(run.phase, Phase::Takeover { .. }));
    let mut ticks = 0;
    while !run.is_over() && ticks < 60 * 25 {
        run.step(Default::default());
        ticks += 1;
    }
    assert!(run.is_over() && ticks >= 60 * 19, "the bot plays for 20 s, then it is over");
    assert!(run.score > 0 && run.humanity() == 0);
    let card = run.model_card().join("\n");
    assert!(card.contains("Automation: 100%") && card.contains("Score:"), "{card}");
}

// REQ: AUTO-01
#[test]
fn a_luddite_never_automates_and_a_heuristic_always_does() {
    use crate::bot::{play_run, Policy};
    use crate::memory::Temperature;
    let l = play_run(15, Hero::Zoomer, Policy::Luddite, 6.0, Temperature::Low);
    assert_eq!(l.automation, 0);
    let h = play_run(15, Hero::Zoomer, Policy::Heuristic, 6.0, Temperature::Low);
    assert!(h.automation > 0, "the easy option, every time it is offered");
}

// REQ: CH-02
#[test]
fn later_chapters_have_their_own_rosters_and_bosses() {
    use crate::chapters::Chapter;
    use crate::training::Training;
    use slop_sim::content::{Archetype, Platform};
    use slop_sim::spec::SlopLook;
    let mut rng = slop_sim::rng::Rng::new(1, 1);
    let t = Training::new(&mut rng);
    let office = crate::waves::table(3.0, Platform::CubicleFarm, &t);
    let names: Vec<&str> = office.specs.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"Copilot") && names.contains(&"Ticket") && names.contains(&"Reply-All"), "{names:?}");
    let unique: std::collections::BTreeSet<&str> = names.iter().copied().collect();
    assert_eq!(unique.len(), names.len(), "no slop twice: {names:?}");
    assert!(office.specs.iter().all(|s| matches!(s.look, SlopLook::Archetype(a) if a.chapter() == 1)), "nothing from the Feed in the Office");
    let late = crate::waves::table(6.0, Platform::ServerRoom, &t);
    assert!(late.specs.len() > office.specs.len(), "more of the roster as capability climbs");
    let old = crate::waves::table(4.0, Platform::OldFeed, &t);
    assert!(old.specs.iter().any(|s| matches!(s.look, SlopLook::Archetype(Archetype::ChadGpt))), "the Old Feed remembers");
    let bosses: std::collections::BTreeSet<_> = Chapter::ALL.iter().map(|c| c.def().boss as usize).collect();
    assert_eq!(bosses.len(), 5, "five chapters, five bosses");
    for c in Chapter::ALL {
        assert!(c.def().biomes.iter().all(|b| b.chapter() == c as usize), "{}", c.def().name);
    }
}

// REQ: AUTO-04
#[test]
fn unplug_stops_automation_fires_the_agents_and_costs_half_your_health() {
    let mut run = Run::new(16, Hero::Boomer);
    run.automation = 40;
    run.weapons[0].agent = true;
    let max = run.stats().max_mental_health;
    let unplug = GLITCHES.iter().position(|g| g.name == "Unplug").unwrap();
    run.phase = Phase::Glitch(vec![unplug]);
    run.take_glitch(0).unwrap();
    assert!(run.unplugged && run.weapons.iter().all(|w| !w.agent));
    assert!(run.stats().max_mental_health < max * 0.6, "half the resource goes");
    let before = run.automation;
    run.phase = Phase::LevelUp(vec![Choice::Delegate(crate::run::Delegate::AutoPick)]);
    run.pending_levels = 1;
    run.choose(0).unwrap();
    assert_eq!(run.automation, before, "Automation does not rise this chapter");
    // It is only offered once there is something to unplug.
    let mut fresh = Run::new(17, Hero::Boomer);
    for _ in 0..40 {
        assert!(!fresh.glitch_offers(3).contains(&unplug));
    }
}

// REQ: END-01, LESSON-01
#[test]
fn the_title_replays_your_build_and_a_takeover_closes_the_chapter() {
    let run = Run::showcase_build(3, Hero::Artist, 5.0, &[(2, 4), (5, 2)]);
    assert!(run.weapons.iter().any(|w| w.def == 2 && w.level == 4) && run.weapons.iter().any(|w| w.def == 5 && w.level == 2));
    let mut run = Run::new(18, Hero::Teacher);
    run.automation = 100;
    run.phase = Phase::Playing;
    run.step(Default::default());
    assert_eq!(run.chapters.len(), 1, "the chapter the takeover cut short is on the record");
    assert_eq!(run.chapters[0].killer.as_ref().map(|k| k.0.as_str()), Some("Automation"));
    let m = run.remembered();
    assert_eq!(m.hero, Some(Hero::Teacher as usize));
    assert!(!m.build.is_empty());
}

// REQ: CAP-01
#[test]
fn no_build_outlasts_a_chapters_overtime() {
    let mut run = Run::new(19, Hero::Boomer);
    run.base.max_mental_health = 100_000.0;
    run.sync();
    run.world.as_mut().unwrap().player.invulnerable = u16::MAX;
    // Sixteen minutes into the chapter: the drain is on.
    run.chapter_since = run.seconds() - 16.0 * 60.0;
    let full = run.world.as_ref().unwrap().player.mental_health;
    let mut ticks = 0;
    while matches!(run.phase, Phase::Playing) && ticks < 60 * 120 {
        run.step(Default::default());
        ticks += 1;
    }
    assert!(run.worn, "it says so once");
    assert!(!matches!(run.phase, Phase::Playing), "the chapter ended on its own");
    assert!(ticks < 60 * 60, "and within a minute at sixteen minutes in");
    assert_eq!(run.chapters[0].killer.as_ref().map(|k| k.0.as_str()), Some("the Slop"));
    assert!(full > 0.0);
    assert!(crate::waves::capability(20.0, 0.0, 0) > crate::waves::capability(10.0, 0.0, 0) * 3.0, "and the slop climb steeply too");
}
