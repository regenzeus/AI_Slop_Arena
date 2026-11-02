//! Behaviour tests for the arena. Each names the requirement it verifies
//! (see docs/requirements.md).

use crate::bot;
use crate::content::*;
use crate::events::{Event, StatusKind};
use crate::spec::{SlopSpec, Traits};
use crate::stats::DamageType;
use crate::vec::V2;
use crate::world::*;

fn idle() -> Input {
    Input::default()
}

fn weapon(name: &str) -> usize {
    weapon_index(name).unwrap()
}

fn armed(weapons: &[&str]) -> World {
    let w: Vec<usize> = weapons.iter().map(|n| weapon(n)).collect();
    World::sandbox(3, &w)
}

fn run(w: &mut World, ticks: u32) -> Vec<Event> {
    let mut events = Vec::new();
    for _ in 0..ticks {
        w.step(idle());
        events.extend(w.drain_events());
    }
    events
}

fn all_weapons() -> Loadout {
    Loadout::with_weapons(&[0, 2, 9, 12, 16, 18])
}

fn bot_wave(seed: u64, ticks: u32) -> World {
    let mut w = World::new(seed, WaveSetup::first(), &all_weapons());
    for _ in 0..ticks {
        let input = bot::steer(&w);
        w.step(input);
    }
    w
}

fn bubble(at: V2, vel: V2) -> Projectile {
    let mut p = Projectile::new(999_999, at, vel, BUBBLE_RADIUS, 6.0, DamageType::Analog, true, Look::Bubble, 240);
    p.line = 0;
    p
}

fn by_id(w: &World, id: u32) -> Option<&Slop> {
    w.slop.iter().find(|s| s.id == id)
}

// REQ: SIM-01
#[test]
fn same_seed_and_inputs_replay_exactly() {
    let a = bot_wave(7, 1800);
    let b = bot_wave(7, 1800);
    assert_eq!(a.checksum(), b.checksum());
    assert!(a.tally.kills > 0);
    assert_ne!(a.checksum(), bot_wave(8, 1800).checksum());
}

// REQ: SIM-02
#[test]
fn you_stay_inside_the_arena() {
    let mut w = armed(&[]);
    for _ in 0..900 {
        w.step(Input { x: -1.0, y: -1.0 });
    }
    assert!(w.player.pos.x >= PLAYER_RADIUS && w.player.pos.x < PLAYER_RADIUS + 1.0);
}

// REQ: SIM-03
#[test]
fn movement_eases_to_full_speed_and_diagonals_are_not_faster() {
    let mut w = armed(&[]);
    w.step(Input { x: 1.0, y: 0.0 });
    assert!(w.player.vel.x < PLAYER_SPEED * 0.5);
    for _ in 0..60 {
        w.step(Input { x: 1.0, y: 1.0 });
    }
    assert!((w.player.vel.length() - PLAYER_SPEED).abs() < 1.0);
}

// REQ: WPN-01
#[test]
fn every_weapon_damages_slop() {
    for (d, def) in WEAPONS.iter().enumerate() {
        let mut w = World::sandbox(3, &[d]);
        let dist = match def.pattern {
            Pattern::Orbit { radius } => radius,
            _ => (def.range * 0.5).min(200.0),
        };
        w.spawn_archetype(Archetype::EngagementBait, w.player.pos + V2::new(dist, 0.0));
        if def.pattern == Pattern::Reflect {
            let p = w.player.pos;
            w.projectiles.push(bubble(p + V2::new(-150.0, 0.0), V2::new(40.0, 0.0)));
        }
        run(&mut w, 240);
        assert!(w.tally.damage_dealt[def.kind as usize] > 0.0, "{} never hit", def.name);
    }
}

// REQ: WPN-02
#[test]
fn swings_only_hit_in_front() {
    let mut w = armed(&["Stick"]);
    let front = w.spawn_archetype(Archetype::ListicleDrone, w.player.pos + V2::new(50.0, 0.0));
    let back = w.spawn_archetype(Archetype::ListicleDrone, w.player.pos + V2::new(-80.0, 0.0));
    w.step(idle());
    assert!(by_id(&w, front).is_none_or(|s| s.hp < s.max_hp));
    assert!(by_id(&w, back).is_some_and(|s| s.hp == s.max_hp));
}

// REQ: WPN-03
#[test]
fn eye_rolls_bounce_and_air_quotes_chain() {
    for name in ["Eye-Roll", "Air Quotes"] {
        let mut w = armed(&[name]);
        for x in [120.0, 200.0, 280.0] {
            w.spawn_archetype(Archetype::EngagementBait, w.player.pos + V2::new(x, 0.0));
        }
        run(&mut w, 40);
        let damaged = w.slop.iter().filter(|s| s.hp < s.max_hp).count();
        assert!(damaged >= 2, "{name} hit {damaged}");
    }
}

// REQ: WPN-04
#[test]
fn marks_resistance_and_type_bonus_scale_damage() {
    let pistol = weapon("Pistol");
    let base = WEAPONS[pistol].damage;
    let first_hit = |setup: &dyn Fn(&mut World)| {
        let mut w = World::sandbox(3, &[pistol]);
        w.spawn_archetype(Archetype::EngagementBait, w.player.pos + V2::new(150.0, 0.0));
        setup(&mut w);
        for e in run(&mut w, 30) {
            if let Event::SlopHit { amount, .. } = e {
                return amount;
            }
        }
        panic!("no hit");
    };
    assert!((first_hit(&|_| {}) - base).abs() < 1e-4);
    let marked = first_hit(&|w| {
        w.slop[0].marked = 600;
        w.slop[0].mark_bonus = 0.2;
    });
    assert!((marked - base * 1.2).abs() < 1e-4);
    let resisted = first_hit(&|w| w.slop[0].resist[DamageType::Analog as usize] = 0.5);
    assert!((resisted - base * 0.5).abs() < 1e-4);
    let bonus = first_hit(&|w| w.player.perks.type_bonus[DamageType::Analog as usize] = 0.25);
    assert!((bonus - base * 1.25).abs() < 1e-4);
}

// REQ: WPN-05
#[test]
fn bricks_splash_and_ban_hammers_delete_small_slop() {
    let mut w = armed(&["Brick"]);
    for y in [-30.0, 0.0, 30.0] {
        w.spawn_archetype(Archetype::EngagementBait, w.player.pos + V2::new(250.0, y));
    }
    run(&mut w, 60);
    assert_eq!(w.slop.iter().filter(|s| s.hp < s.max_hp).count(), 3);

    let mut w = armed(&["Ban Hammer"]);
    w.spawn_archetype(Archetype::ListicleDrone, w.player.pos + V2::new(150.0, 0.0));
    run(&mut w, 3);
    assert!(w.slop.is_empty(), "small slop is deleted outright");
}

// REQ: WPN-06
#[test]
fn fact_checks_strip_beauty_filters() {
    let shot_at_front = |weapon_name: &str| {
        let mut w = armed(&[weapon_name]);
        let id = w.spawn_archetype(Archetype::FilterInfluencer, w.player.pos + V2::new(200.0, 0.0));
        run(&mut w, 20);
        by_id(&w, id).map_or(f32::MAX, |s| s.max_hp - s.hp)
    };
    // The influencer walks at you, so its filter faces you.
    assert!(shot_at_front("Fact-Check Flashlight") > shot_at_front("Pistol"));
}

// REQ: WPN-07
#[test]
fn frisbees_come_back_and_reflected_bubbles_hurt_slop() {
    let mut w = armed(&["Frisbee"]);
    w.spawn_archetype(Archetype::EngagementBait, w.player.pos + V2::new(200.0, 0.0));
    w.step(idle());
    let id = w.projectiles[0].id;
    let (mut gone, mut farthest) = (false, 0.0f32);
    for _ in 0..300 {
        w.step(idle());
        match w.projectiles.iter().find(|p| p.id == id) {
            Some(p) => farthest = farthest.max(p.pos.distance(w.player.pos)),
            None => {
                gone = true;
                break;
            }
        }
    }
    assert!(gone, "the frisbee comes back to you and disappears");
    assert!(farthest > 250.0 && farthest < 450.0, "it flew out {farthest}");

    let mut w = armed(&["Uno Reverse Card"]);
    w.spawn_archetype(Archetype::EngagementBait, w.player.pos + V2::new(250.0, 0.0));
    let p = w.player.pos;
    w.projectiles.push(bubble(p + V2::new(-200.0, 0.0), V2::new(50.0, 0.0)));
    let events = run(&mut w, 120);
    assert!(events.iter().any(|e| matches!(e, Event::Reflected { .. })));
    assert!(w.tally.damage_dealt[DamageType::Sarcasm as usize] > 0.0);
}

// REQ: WPN-08
#[test]
fn seed_bombs_heal_you_and_the_dog_orbits() {
    let mut w = armed(&["Seed Bomb"]);
    w.player.mental_health = 50.0;
    w.spawn_archetype(Archetype::EngagementBait, w.player.pos + V2::new(40.0, 0.0));
    run(&mut w, 120);
    assert!(w.player.mental_health > 50.0, "standing in your own patch heals");

    let mut w = armed(&["Dog Walk"]);
    let start = w.player.weapons[0].phase;
    run(&mut w, 60);
    assert!(w.player.weapons[0].phase > start);
}

// REQ: SLOP-01
#[test]
fn chadgpt_fires_fans_of_three_and_only_one_talks() {
    let mut w = armed(&[]);
    w.spawn_archetype(Archetype::ChadGpt, w.player.pos + V2::new(350.0, 0.0));
    let (mut most, mut talking) = (0, 0);
    for _ in 0..240 {
        w.step(idle());
        let bubbles: Vec<_> = w.projectiles.iter().filter(|p| p.look == Look::Bubble).collect();
        assert!(bubbles.iter().all(|p| p.hostile && (p.line == NO_LINE || (p.line as usize) < BUBBLE_LINES.len())));
        talking = talking.max(bubbles.iter().filter(|p| p.line != NO_LINE).count());
        most = most.max(bubbles.len());
    }
    assert!(most >= 3 && talking >= 1 && talking < most);
}

// REQ: SLOP-02
#[test]
fn hands_grab_slow_and_hurt_you() {
    let mut w = armed(&[]);
    w.spawn_archetype(Archetype::SixFingerHand, w.player.pos + V2::new(40.0, 0.0));
    run(&mut w, 10);
    assert_eq!(w.player.grabbed, 1);
    for _ in 0..90 {
        w.step(Input { x: 1.0, y: 0.0 });
    }
    assert!((w.player.vel.x - PLAYER_SPEED * 0.8).abs() < 2.0);
    assert!(w.player.mental_health < 100.0);
}

// REQ: SLOP-03
#[test]
fn killing_drone_number_one_first_scatters_the_pack() {
    let pack = |kill: usize| {
        let mut w = armed(&[]);
        let drone = Archetype::ListicleDrone as usize;
        for n in 1..=5u8 {
            w.place(drone, w.player.pos + V2::new(400.0, n as f32 * 30.0), 7, n);
        }
        w.slop[kill - 1].hp = 0.0;
        let scattered = run(&mut w, 1).iter().any(|e| matches!(e, Event::Scattered { .. }));
        (scattered, w.slop.iter().all(|s| s.fleeing > 0))
    };
    assert_eq!(pack(1), (true, true));
    assert_eq!(pack(3), (false, false));
}

// REQ: SLOP-04
#[test]
fn yes_bots_buff_and_cakes_heal() {
    let mut w = armed(&[]);
    let at = w.player.pos + V2::new(600.0, 0.0);
    let friend = w.spawn_archetype(Archetype::ChadGpt, at);
    w.spawn_archetype(Archetype::YesBot, at + V2::new(40.0, 0.0));
    let events = run(&mut w, 120);
    assert!(events.iter().any(|e| matches!(e, Event::Agreed { .. })));
    assert!(by_id(&w, friend).unwrap().agreed >= 1);

    let mut w = armed(&[]);
    let hurt = w.spawn_archetype(Archetype::ChadGpt, at);
    w.spawn_archetype(Archetype::GrandmaCake, at + V2::new(40.0, 0.0));
    w.slop[0].hp = 5.0;
    run(&mut w, 200);
    assert!(by_id(&w, hurt).unwrap().hp > 5.0);
}

// REQ: SLOP-05
#[test]
fn traits_split_repost_and_dodge() {
    let mut w = armed(&["Pistol"]);
    w.spawn_archetype(Archetype::RecruiterBot, w.player.pos + V2::new(300.0, 0.0));
    let events = run(&mut w, 30);
    assert!(events.iter().any(|e| matches!(e, Event::Split { .. })));

    let mut w = armed(&[]);
    w.spawn_archetype(Archetype::RepostBot, w.player.pos + V2::new(300.0, 0.0));
    w.slop[0].hp = 0.0;
    run(&mut w, 1);
    assert_eq!(w.slop.len(), 1);
    assert!(w.slop[0].reposted);
    w.slop[0].hp = 0.0;
    run(&mut w, 1);
    assert!(w.slop.is_empty(), "a repost does not come back again");

    let mut w = armed(&["Pistol"]);
    let cat = w.ensure_spec(SlopSpec::brainrot("Gatto".into(), Body::Cat, Object::Baguette, Accessory::Crown, 1.0, 0.1, [0.0; 4]));
    w.place(cat, w.player.pos + V2::new(200.0, 0.0), 0, 1);
    let events = run(&mut w, 30);
    assert!(events.iter().any(|e| matches!(e, Event::SlopDodged { .. })), "a cat dodges its first hit");
}

// REQ: BRAIN-01
#[test]
fn brainrot_parts_shape_the_spec() {
    let make = |b, o, a| SlopSpec::brainrot("x".into(), b, o, a, 1.0, 1.0, [0.1, 0.0, 0.0, -0.1]);
    let elephant = make(Body::Elephant, Object::Baguette, Accessory::Headphones);
    let shark = make(Body::Shark, Object::Baguette, Accessory::Headphones);
    assert!(elephant.hp > shark.hp * 2.0 && elephant.speed < shark.speed);
    assert!(make(Body::Shark, Object::Baguette, Accessory::Sneakers).speed > shark.speed);
    assert_eq!(make(Body::Shark, Object::Toaster, Accessory::Crown).attack, Some(crate::spec::Attack::Toast));
    assert!(make(Body::Shark, Object::Baguette, Accessory::PartyHat).traits.has(Traits::EXPLODE));
    let shades = make(Body::Shark, Object::Baguette, Accessory::Sunglasses);
    assert!((shades.resist[0] - 0.3).abs() < 1e-5, "sunglasses add to the strongest resistance");
    assert!(shark.is_brainrot());
}

// REQ: STATUS-01
#[test]
fn statuses_hurt_reverse_and_can_be_prevented() {
    let mut w = armed(&[]);
    w.add_anxiety();
    run(&mut w, 120);
    assert!(w.player.mental_health < 100.0, "anxiety hurts over time");

    let mut w = armed(&[]);
    w.apply_status(StatusKind::Brainrot);
    run(&mut w, 50);
    assert!(w.player.status.brainrot > 0, "brainrot lands after its warning");
    let x = w.player.pos.x;
    for _ in 0..20 {
        w.step(Input { x: 1.0, y: 0.0 });
    }
    assert!(w.player.pos.x < x, "controls are reversed");

    let mut w = armed(&[]);
    w.player.perks.ad_blocker = true;
    w.player.perks.dumbphone = true;
    w.apply_status(StatusKind::Popup);
    w.apply_status(StatusKind::Doomscroll);
    assert_eq!(w.player.status.popup + w.player.status.doomscroll, 0);
}

// REQ: ACT-01
#[test]
fn actives_fire_on_their_triggers() {
    let mut w = armed(&[]);
    w.player.perks.actives = vec![Active::LogOff, Active::BlockButton];
    w.player.active_cooldown = vec![0.0, 0.0];
    for k in 0..9 {
        w.spawn_archetype(Archetype::ListicleDrone, w.player.pos + V2::from_angle(k as f32) * 60.0);
    }
    let p = w.player.pos;
    w.projectiles.push(bubble(p + V2::new(60.0, 0.0), V2::new(-50.0, 0.0)));
    let events = run(&mut w, 2);
    assert!(events.iter().any(|e| matches!(e, Event::ActiveUsed { active: Active::LogOff })));
    assert!(events.iter().any(|e| matches!(e, Event::ActiveUsed { active: Active::BlockButton })));

    let mut w = armed(&[]);
    w.player.second_winds = 1;
    w.player.mental_health = 1.0;
    w.spawn_archetype(Archetype::ListicleDrone, w.player.pos);
    let events = run(&mut w, 1);
    assert!(events.iter().any(|e| matches!(e, Event::SecondWind)));
    assert_eq!(w.outcome, Outcome::Running);
}

// REQ: FAIR-01
#[test]
fn slop_far_away_never_shoots() {
    let mut w = armed(&[]);
    w.spawn_archetype(Archetype::ChadGpt, w.player.pos + V2::new(1000.0, 0.0));
    run(&mut w, 150);
    assert!(w.projectiles.is_empty());
}

// REQ: FAIR-02
#[test]
fn em_dashes_and_bombs_are_telegraphed() {
    let mut w = armed(&[]);
    w.spawn_archetype(Archetype::Clawed, w.player.pos + V2::new(300.0, 0.0));
    let events = run(&mut w, 1);
    assert!(events.iter().any(|e| matches!(e, Event::LancerAim { .. })));
    run(&mut w, LANCE_AIM as u32 - 1);
    assert!(w.projectiles.is_empty());
    run(&mut w, 1);
    assert!(w.projectiles.iter().any(|p| p.look == Look::EmDash));

    let mut w = armed(&[]);
    w.drop_bomb(w.player.pos, 10.0, NO_SPEC);
    run(&mut w, BOMB_WARNING as u32 - 1);
    assert_eq!(w.player.mental_health, 100.0);
    run(&mut w, 1);
    assert!(w.player.mental_health < 100.0);
}

// REQ: FAIR-03
#[test]
fn a_hit_gives_brief_invulnerability() {
    let mut w = armed(&[]);
    for _ in 0..3 {
        w.spawn_archetype(Archetype::ListicleDrone, w.player.pos);
    }
    w.step(idle());
    let contact = Archetype::ListicleDrone.def().contact;
    assert!((w.player.mental_health - (100.0 - contact)).abs() < 1e-4);
}

// REQ: BUB-01
#[test]
fn your_shots_pop_bubbles() {
    let mut w = armed(&["Pistol"]);
    w.spawn_archetype(Archetype::EngagementBait, w.player.pos + V2::new(450.0, 0.0));
    let p = w.player.pos;
    w.projectiles.push(bubble(p + V2::new(200.0, 0.0), V2::new(-10.0, 0.0)));
    run(&mut w, 40);
    assert_eq!(w.tally.bubbles_popped, 1);
}

// REQ: LIKE-01
#[test]
fn likes_fly_to_you_and_fakes_explode() {
    let mut w = armed(&[]);
    let p = w.player.pos;
    w.likes.push(Like { id: 1, pos: p + V2::new(80.0, 0.0), value: 3, pulled: false, fake: false });
    w.likes.push(Like { id: 2, pos: p + V2::new(400.0, 0.0), value: 5, pulled: false, fake: false });
    w.likes.push(Like { id: 3, pos: p + V2::new(-60.0, 0.0), value: 1, pulled: false, fake: true });
    run(&mut w, 30);
    assert_eq!(w.clout, 3);
    assert!(w.player.mental_health < 100.0 && w.player.status.anxiety > 0);
}

// REQ: WAVE-01
#[test]
fn a_wave_clears_on_time_and_burnout_ends_it() {
    let mut setup = WaveSetup::sandbox();
    setup.seconds = 2.0;
    let mut w = World::new(1, setup, &Loadout::with_weapons(&[]));
    w.likes.push(Like { id: 1, pos: V2::new(100.0, 100.0), value: 10, pulled: false, fake: false });
    run(&mut w, 120);
    assert_eq!(w.outcome, Outcome::Cleared);
    assert_eq!(w.clout, 5);

    let mut w = armed(&[]);
    w.player.mental_health = 1.0;
    w.spawn_archetype(Archetype::ListicleDrone, w.player.pos);
    run(&mut w, 1);
    assert_eq!(w.outcome, Outcome::Burnout);
}

// REQ: WAVE-02, SLOP-06
#[test]
fn the_wave_ramps_up_and_elites_drop_loot_boxes() {
    let mut setup = WaveSetup::first();
    setup.elite_chance = 0.5;
    let mut w = World::new(11, setup, &all_weapons());
    let (mut early, mut late) = (0, 0);
    while w.outcome == Outcome::Running {
        let input = bot::steer(&w);
        w.step(input);
        for e in w.drain_events() {
            if let Event::SpawnWarning { .. } = e {
                if w.tick < 600 {
                    early += 1;
                } else if (2400..3000).contains(&w.tick) {
                    late += 1;
                }
            }
        }
    }
    assert!(late > early, "early {early}, late {late}");
    assert!(w.setup.specs.iter().any(|s| s.elite));
}

// REQ: BOSS-01
#[test]
fn every_boss_fights_and_can_be_beaten() {
    for kind in BossKind::ALL {
        let mut setup = WaveSetup::sandbox();
        setup.seconds = 0.0;
        setup.specs.push(SlopSpec::boss(kind, 600.0));
        setup.boss = Some(setup.specs.len() - 1);
        let mut loadout = Loadout::with_weapons(&[0, 5, 9, 12, 16, 17]);
        loadout.stats.max_mental_health = 1e6;
        let mut w = World::new(5, setup, &loadout);
        w.player.mental_health = 1e6;
        let mut intro = false;
        for _ in 0..(60 * 120) {
            let input = bot::steer(&w);
            w.step(input);
            intro |= w.drain_events().iter().any(|e| matches!(e, Event::BossIntro { .. }));
            if w.outcome != Outcome::Running {
                break;
            }
        }
        assert!(intro, "{kind:?}");
        assert_eq!(w.outcome, Outcome::Cleared, "{kind:?} should fall within two minutes");
    }
}

// REQ: BOSS-07
#[test]
fn the_model_regenerates_unless_your_build_was_varied() {
    let fight = |collapse: f32| {
        let mut setup = WaveSetup::sandbox();
        setup.seconds = 0.0;
        setup.model.collapse = collapse;
        setup.specs.push(SlopSpec::boss(BossKind::TheModel, 100.0));
        setup.boss = Some(setup.specs.len() - 1);
        let mut w = World::new(5, setup, &Loadout::with_weapons(&[]));
        w.step(idle());
        let i = w.slop.iter().position(|s| w.boss.as_ref().is_some_and(|b| b.id == s.id)).unwrap();
        w.damage_slop(i, 1000.0, DamageType::Analog, V2::ZERO, V2::ZERO, None);
        w.slop[i].hp > 0.0
    };
    assert!(fight(0.0), "a one-trick build faces a regenerating Model");
    assert!(!fight(0.9), "Model Collapse stops the regenerate");
}

// REQ: PLAT-02
#[test]
fn every_platform_has_its_hazard() {
    for platform in Platform::ALL {
        let mut setup = WaveSetup::sandbox();
        setup.platform = Some(platform);
        let mut w = World::new(9, setup, &Loadout::with_weapons(&[]));
        let mut seen = false;
        for _ in 0..1600 {
            w.step(idle());
            let events = w.drain_events();
            seen |= !w.zones.is_empty()
                || w.projectiles.iter().any(|p| p.look == Look::Letter)
                || events.iter().any(|e| matches!(e, Event::AdBreak | Event::Status { kind: StatusKind::BrainrotWarning }));
        }
        assert!(seen, "{platform:?}");
    }
}
