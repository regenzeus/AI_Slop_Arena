//! Tests of a run as a whole. Each names the requirement it verifies.

use slop_sim::content::{weapon_index, BossKind, WEAPONS};
use slop_sim::map::{PropKind, PropState};
use slop_sim::world::MAX_LEVEL;

use crate::bot::{self, Policy};
use crate::heroes::Hero;
use crate::items::{item_index, ITEMS};
use crate::run::{Choice, OwnedWeapon, Phase, Run, ITEM_SLOTS, WEAPON_SLOTS};

/// Makes you unkillable.
fn immortal(run: &mut Run) {
    let w = run.world.as_mut().unwrap();
    w.player.stats.max_mental_health = 1e7;
    w.player.mental_health = 1e7;
    run.base.max_mental_health = 1e7;
}

/// Plays with the bots for `seconds`, taking every choice.
fn play(run: &mut Run, seconds: f32) {
    let mut rng = slop_sim::rng::Rng::new(3, 3);
    let end = run.seconds() + seconds;
    while run.seconds() < end && !run.is_over() {
        if matches!(run.phase, Phase::Playing) {
            let input = slop_sim::bot::steer(run.world.as_ref().unwrap());
            run.step(input);
        } else {
            bot::decide(run, Policy::Heuristic, &mut rng);
        }
    }
}

// REQ: RUN-01, HERO-01, PLAT-01
#[test]
fn a_run_starts_with_the_heroes_loadout_on_one_of_six_platforms() {
    let run = Run::new(1, Hero::Teacher);
    assert_eq!(WEAPONS[run.weapons[0].def].name, "Red Pen");
    assert_eq!(run.items[0].0, item_index("Library Card").unwrap());
    assert!(matches!(run.phase, Phase::Playing));
    let mut order = run.platforms.clone();
    assert_eq!(order[0], run.platform);
    order.sort_by_key(|p| *p as usize);
    order.dedup();
    assert_eq!(order.len(), 6, "every platform once");
    assert_eq!(run.world.as_ref().unwrap().player.weapons.len(), 1);
}

// REQ: LEVEL-01, LEVEL-02
#[test]
fn likes_level_you_up_and_every_choice_is_valid() {
    let mut run = Run::new(2, Hero::Boomer);
    run.world.as_mut().unwrap().xp += 400;
    run.step(Default::default());
    assert!(run.pending_levels >= 3, "400 Likes are several levels");
    let Phase::LevelUp(choices) = run.phase.clone() else { panic!("a level-up opens") };
    assert_eq!(choices.len(), 3);
    for (i, a) in choices.iter().enumerate() {
        for b in &choices[i + 1..] {
            assert_ne!(a, b, "three different picks");
        }
    }
    // Full weapon slots: no new weapons; maxed weapons: no levels for them.
    let six: Vec<usize> = (0..WEAPONS.len()).filter(|&d| !WEAPONS[d].evolved).take(WEAPON_SLOTS).collect();
    run.weapons = six.iter().map(|&def| OwnedWeapon { def, level: MAX_LEVEL, agent: false }).collect();
    for _ in 0..30 {
        run.reroll_free_for_test();
        let Phase::LevelUp(choices) = run.phase.clone() else { panic!() };
        for c in choices {
            assert!(!matches!(c, Choice::NewWeapon(_) | Choice::WeaponLevel(_)), "{c:?} is not valid with six maxed weapons");
        }
    }
}

// REQ: RUN-02, LEVEL-02
#[test]
fn a_choice_changes_the_world_at_once() {
    let mut run = Run::new(3, Hero::Zoomer);
    let rock = weapon_index("Rock").unwrap();
    run.pending_levels = 1;
    run.phase = Phase::LevelUp(vec![Choice::NewWeapon(rock), Choice::Heal, Choice::Clout(5)]);
    run.choose(0).unwrap();
    let w = run.world.as_ref().unwrap();
    assert!(w.player.weapons.iter().any(|x| x.def == rock), "the new weapon fires right away");
    assert_eq!(run.weapons.iter().find(|w| w.def == rock).unwrap().level, 1);
    assert!(matches!(run.phase, Phase::Playing));
}

// REQ: WPN-12, CHEST-01
#[test]
fn chests_merge_two_max_weapons_into_one() {
    let mut run = Run::new(4, Hero::Boomer);
    let (rock, sling, meteor) = (weapon_index("Rock").unwrap(), weapon_index("Slingshot").unwrap(), weapon_index("Meteor Shower").unwrap());
    run.weapons = vec![OwnedWeapon { def: rock, level: MAX_LEVEL, agent: false }, OwnedWeapon { def: sling, level: MAX_LEVEL, agent: false }];
    run.world.as_mut().unwrap().chests.push(1);
    run.step(Default::default());
    let Phase::Chest(reward) = run.phase.clone() else { panic!("the chest opens") };
    assert_eq!(reward.merged, Some((rock, sling, meteor)));
    assert_eq!(run.weapons.len(), 1, "the merge frees a slot");
    assert_eq!(run.weapons[0].def, meteor);
    assert!(run.world.as_ref().unwrap().player.weapons.iter().any(|w| w.def == meteor));
    run.close().unwrap();
    // Without a merge, a chest levels what you have.
    run.weapons = vec![OwnedWeapon { def: rock, level: 2, agent: false }];
    run.world.as_mut().unwrap().chests.push(1);
    run.step(Default::default());
    let Phase::Chest(reward) = run.phase.clone() else { panic!() };
    assert!(!reward.upgrades.is_empty() && reward.merged.is_none());
}

// REQ: RUN-11, BOSS-01
#[test]
fn each_platform_ends_with_its_boss_and_the_feed_moves_on() {
    let mut run = Run::new(5, Hero::Programmer);
    immortal(&mut run);
    let first = run.platform;
    run.platform_since = -crate::waves::BOSS_AFTER;
    play(&mut run, 0.1);
    assert_eq!(run.boss_out, Some(first.boss()));
    let id = run.world.as_ref().unwrap().boss.as_ref().unwrap().id;
    run.world.as_mut().unwrap().slop.iter_mut().find(|s| s.id == id).unwrap().hp = 0.0;
    play(&mut run, 0.2);
    assert_eq!(run.bosses_beaten, 1);
    assert_ne!(run.platform, first, "the Feed scrolls on");
    assert!(run.toasts.iter().any(|t| t.contains("Next up")));
    run.stage = 6;
    assert_eq!(run.next_boss(), BossKind::TheModel, "after six platforms comes The Model");
}

// REQ: MAP-03
#[test]
fn a_vending_machine_sells_an_item_once() {
    let mut run = Run::new(6, Hero::Artist);
    immortal(&mut run);
    run.clout = 500;
    let w = run.world.as_mut().unwrap();
    let pos = w.player.pos + slop_sim::V2::new(5000.0, 5000.0);
    let v = slop_sim::map::Prop { id: 999_001, kind: PropKind::Vending, pos, charge: 0, state: PropState::Idle, left: 0, price: 0 };
    w.player.pos = v.pos;
    w.map.props.push(v);
    for _ in 0..80 {
        run.step(Default::default());
        if !matches!(run.phase, Phase::Playing) {
            break;
        }
    }
    let Phase::Vending { offers, .. } = run.phase.clone() else { panic!("standing in the ring opens the machine: {:?}", run.phase) };
    assert!(!offers.is_empty());
    let levels: u32 = run.items.iter().map(|i| i.1).sum();
    run.buy(0).unwrap();
    assert!(run.items.iter().map(|i| i.1).sum::<u32>() > levels);
    assert!(run.clout < 500);
    let state = run.world.as_ref().unwrap().map.props.iter().find(|p| p.id == 999_001).unwrap().state;
    assert_eq!(state, PropState::Done);
}

// REQ: ITEM-01
#[test]
fn items_level_up_to_their_max_and_six_fit() {
    let mut run = Run::new(7, Hero::Boomer);
    run.items = (0..ITEM_SLOTS).map(|i| (i, ITEMS[i].max_level())).collect();
    run.world.as_mut().unwrap().xp += 200;
    run.step(Default::default());
    let Phase::LevelUp(choices) = run.phase.clone() else { panic!() };
    for c in choices {
        assert!(!matches!(c, Choice::NewItem(_) | Choice::ItemLevel(_)), "{c:?}: items are full and maxed");
    }
}

// REQ: EVO-01, EVO-03
#[test]
fn the_slop_retrains_every_two_minutes_and_says_so() {
    let mut run = Run::new(8, Hero::Teacher);
    immortal(&mut run);
    play(&mut run, 125.0);
    assert!(run.notes.is_some(), "patch notes after two minutes");
    assert!(run.toasts.iter().any(|t| t.contains("patch notes")));
    assert!(run.level > 3, "a bot levels up in two minutes: {}", run.level);
}

// REQ: SIM-01, BAL-02
#[test]
fn the_same_seed_plays_the_same_run() {
    let a = bot::play(9, Hero::Zoomer, Policy::Heuristic, 1.5);
    let b = bot::play(9, Hero::Zoomer, Policy::Heuristic, 1.5);
    assert_eq!((a.kills, a.level), (b.kills, b.level));
    assert!(a.kills > 20);
}

// REQ: ITEM-02
#[test]
fn side_hustles_and_the_creator_fund_pay_every_minute() {
    let mut run = Run::new(10, Hero::Boomer);
    immortal(&mut run);
    run.items.push((item_index("Side Hustle").unwrap(), 2));
    run.offset = 59.9;
    run.step(Default::default());
    assert_eq!(run.clout, 0, "nothing before the minute");
    run.offset = 60.5;
    // A minute after the start the hustles pay 2 x 6, the Fund pays 2.
    let mut rng = slop_sim::rng::Rng::new(1, 1);
    while run.seconds() < 61.0 {
        if !matches!(run.phase, Phase::Playing) {
            bot::decide(&mut run, Policy::Heuristic, &mut rng);
        }
        run.step(Default::default());
    }
    assert!(run.clout >= 12 + crate::run::payout(1), "{}", run.clout);
}

// REQ: ITEM-01
#[test]
fn an_items_drawback_counts_once() {
    let mut run = Run::new(11, Hero::Zoomer);
    let drink = item_index("Energy Drink").unwrap();
    run.items = vec![(drink, 1)];
    let one = run.stats();
    run.items = vec![(drink, 5)];
    let five = run.stats();
    assert_eq!(one.max_mental_health, five.max_mental_health, "-15 Mental Health once, not five times");
    assert!(five.speed > one.speed, "the upside grows");
}
