//! Tests of a run as a whole. Each names the requirement it verifies.

use slop_sim::content::{weapon_index, WEAPONS};
use slop_sim::world::Outcome;

use crate::heroes::Hero;
use crate::items::{item_index, StatKind};
use crate::run::{Phase, RuleError, Run, WEAPON_SLOTS};
use crate::shop::{Goods, Note, Offer};

/// Plays the current wave to its end with the movement bot, unkillable.
fn survive(run: &mut Run) {
    let w = run.world.as_mut().unwrap();
    w.player.stats.max_mental_health = 1e6;
    w.player.mental_health = 1e6;
    while matches!(run.phase, Phase::Wave) {
        let input = slop_sim::bot::steer(run.world.as_ref().unwrap());
        run.step(input);
    }
}

/// From the end of a wave through the level-ups into the shop.
fn to_shop(run: &mut Run) {
    while let Phase::LevelUp(_) = run.phase {
        run.choose_level(0).unwrap();
    }
    assert!(matches!(run.phase, Phase::Shop));
}

fn offer(run: &mut Run, goods: Goods, price: u32) -> usize {
    run.shop.offers.push(Offer { goods, price, locked: false, sold: false, note: None });
    run.shop.offers.len() - 1
}

// REQ: RUN-01, HERO-01
#[test]
fn a_run_starts_with_the_heroes_loadout_and_walks_through_the_phases() {
    let mut run = Run::new(1, Hero::Teacher);
    assert_eq!(WEAPONS[run.weapons[0].def].name, "Red Pen");
    assert_eq!(run.items[0].0, item_index("Library Card").unwrap());
    assert!(matches!(run.phase, Phase::Wave));
    assert_eq!(run.wave, 1);
    survive(&mut run);
    assert!(matches!(run.phase, Phase::LevelUp(_) | Phase::Shop), "{:?}", run.phase);
    to_shop(&mut run);
    assert_eq!(run.wave, 2, "the shop sells for the next wave");
    assert!(run.shop.offers.len() >= 5);
    run.next_wave().unwrap();
    assert!(matches!(run.phase, Phase::Wave));
    assert_eq!(run.world.as_ref().unwrap().setup.number, 2);
}

// REQ: RUN-02
#[test]
fn items_and_heroes_shape_the_loadout() {
    let run = Run::new(1, Hero::Programmer);
    let l = run.loadout();
    // Programmer: +5% Focus, and Coffee: +15% Focus and Anxiety.
    assert!((l.stats.focus - 0.20).abs() < 1e-5, "{}", l.stats.focus);
    assert_eq!(l.perks.coffee, 1);
    let zoomer = Run::new(1, Hero::Zoomer).loadout();
    assert!(zoomer.perks.fluent);
    assert!((zoomer.stats.max_mental_health - 65.0).abs() < 1e-4, "100 - 20 - 15 from the Energy Drink");
}

// REQ: RUN-03
#[test]
fn clearing_a_wave_pays_clout_interest_and_xp() {
    let mut run = Run::new(2, Hero::Boomer);
    survive(&mut run);
    let report = run.reports.last().unwrap().clone();
    assert!(report.cleared && report.likes > 0);
    assert_eq!(report.payout, crate::run::payout(1));
    assert_eq!(run.clout, report.likes + report.payout + report.interest + report.income as u32);
    assert!(run.level >= 1 || run.xp == report.likes + report.payout);
}

// REQ: RUN-03
#[test]
fn burnout_ends_the_run() {
    let mut run = Run::new(3, Hero::Artist);
    run.world.as_mut().unwrap().player.mental_health = 0.5;
    let at = run.world.as_ref().unwrap().player.pos;
    run.world.as_mut().unwrap().spawn_archetype(slop_sim::Archetype::ListicleDrone, at);
    run.step(slop_sim::Input::default());
    assert!(matches!(run.phase, Phase::Over { victory: false }));
}

// REQ: SHOP-01
#[test]
fn buying_upgrades_owned_weapons_and_respects_slots() {
    let mut run = Run::new(4, Hero::Boomer);
    survive(&mut run);
    to_shop(&mut run);
    run.clout = 10_000;
    let owned = run.weapons[0].def;
    let i = offer(&mut run, Goods::Weapon { def: owned, tier: 1 }, 30);
    run.buy(i).unwrap();
    assert_eq!(run.weapons[0].tier, 2, "a duplicate upgrades");
    assert_eq!(run.buy(i), Err(RuleError::Sold));
    for d in 0..WEAPONS.len() {
        if run.weapons.len() >= WEAPON_SLOTS {
            break;
        }
        if run.weapons.iter().all(|w| w.def != d) {
            let k = offer(&mut run, Goods::Weapon { def: d, tier: 1 }, 20);
            run.buy(k).unwrap();
        }
    }
    let free = (0..WEAPONS.len()).find(|d| run.weapons.iter().all(|w| w.def != *d)).unwrap();
    let k = offer(&mut run, Goods::Weapon { def: free, tier: 1 }, 20);
    assert_eq!(run.buy(k), Err(RuleError::NoWeaponSlot));
    let before = run.clout;
    let refund = run.sell_value(0);
    run.sell(0).unwrap();
    assert_eq!(run.clout, before + refund);
    assert!(refund > 0);
    run.clout = 0;
    let k = offer(&mut run, Goods::Item { item: 0 }, 50);
    assert_eq!(run.buy(k), Err(RuleError::NotEnoughClout));
}

// REQ: SHOP-04
#[test]
fn locked_offers_survive_rerolls_and_the_next_shop() {
    let mut run = Run::new(5, Hero::Zoomer);
    survive(&mut run);
    to_shop(&mut run);
    run.clout = 1000;
    run.toggle_lock(0).unwrap();
    let kept = run.shop.offers[0].goods;
    run.reroll().unwrap();
    assert!(run.shop.offers.iter().any(|o| o.locked && o.goods == kept));
    let cost = run.reroll_cost();
    run.reroll().unwrap();
    assert!(run.reroll_cost() > cost, "rerolls get pricier");
    run.next_wave().unwrap();
    survive(&mut run);
    to_shop(&mut run);
    assert!(run.shop.offers.iter().any(|o| o.locked && o.goods == kept));
}

// REQ: SHOP-01
#[test]
fn shops_offer_your_damage_type_loot_boxes_and_mercy() {
    let mut run = Run::new(6, Hero::Teacher);
    survive(&mut run);
    run.loot_boxes = 1;
    to_shop(&mut run);
    assert_eq!(run.shop.offers[0].note, Some(Note::LootBox));
    assert_eq!(run.shop.offers[0].price, 0);
    let kind = WEAPONS[run.weapons[0].def].kind;
    assert!(run.shop.offers.iter().any(|o| matches!(o.goods, Goods::Weapon { def, .. } if WEAPONS[def].kind == kind)));
}

// REQ: LEVEL-01
#[test]
fn level_ups_raise_the_chosen_stat() {
    let mut run = Run::new(7, Hero::Boomer);
    run.xp = 50;
    survive(&mut run);
    if let Phase::LevelUp(offers) = run.phase.clone() {
        let before = run.base;
        run.choose_level(0).unwrap();
        let mut expected = before;
        offers[0].stat.apply(&mut expected, offers[0].amount);
        assert_eq!(run.base, expected);
    } else {
        panic!("50 XP and a wave should give a level: {:?}", run.phase);
    }
    let _ = StatKind::ALL;
}

// REQ: EVO-03
#[test]
fn the_slop_trains_between_waves() {
    let mut run = Run::new(8, Hero::Boomer);
    survive(&mut run);
    while let Phase::LevelUp(_) = run.phase {
        run.choose_level(0).unwrap();
    }
    assert!(matches!(run.phase, Phase::Shop));
    let notes = run.notes.as_ref().expect("patch notes after a wave");
    assert_eq!(notes.version, "Slop v1.1");
    assert!(!notes.lines.is_empty());
    // The Boomer's Newspaper Roll kills with Fact, so Fact resistance rose.
    assert!(run.training.model[slop_sim::DamageType::Fact as usize] > 0.0, "{:?}", run.training.model);
    let _ = weapon_index("Rock");
    let _ = Outcome::Running;
}
