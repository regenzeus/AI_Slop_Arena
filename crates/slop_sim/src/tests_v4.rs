//! Behaviour tests for version 4: dashes, Boundaries and barriers,
//! legendary effects, biomes, events and the brainrot icons. Each names the
//! requirement it verifies (see docs/requirements.md).

use crate::content::*;
use crate::events::{Event, StatusKind};
use crate::map;
use crate::spec::{Movement, SlopSpec};
use crate::stats::DamageType;
use crate::vec::V2;
use crate::world::*;

fn sandbox() -> World {
    let mut w = World::sandbox(3, &[]);
    w.player.perks.dash_charges = 2;
    w.player.perks.dash_recharge = 3.0;
    w.player.dashes = 2;
    w
}

fn dash(x: f32, y: f32) -> Input {
    Input { x, y, dash: true }
}

fn run(w: &mut World, ticks: u32) -> Vec<Event> {
    let mut events = Vec::new();
    for _ in 0..ticks {
        w.step(Input::default());
        events.extend(w.drain_events());
    }
    events
}

/// A sturdy slop of `a` at `off` from you, and its index.
fn put(w: &mut World, a: Archetype, off: V2) -> usize {
    let at = w.player.pos + off;
    let id = w.spawn_archetype(a, at);
    let j = w.slop.iter().position(|s| s.id == id).unwrap();
    w.slop[j].hp = 1e6;
    w.slop[j].max_hp = 1e6;
    j
}

// REQ: DASH-01
#[test]
fn a_dash_is_fast_untouchable_and_its_charges_come_back() {
    let mut w = sandbox();
    let start = w.player.pos;
    w.step(dash(1.0, 0.0));
    assert_eq!(w.player.dashes, 1, "a charge is spent");
    assert!(w.player.invulnerable > 0, "you cannot be hurt while dashing");
    let hit = w.hurt(50.0, NO_SPEC);
    assert!(!hit && w.player.mental_health == w.player.stats.max_mental_health);
    run(&mut w, 15);
    let moved = w.player.pos.distance(start);
    assert!(moved > 170.0, "a dash covers about 200 units, moved {moved}");
    w.step(dash(1.0, 0.0));
    w.step(dash(1.0, 0.0));
    assert_eq!(w.player.dashes, 0, "no charge, no dash");
    let events = run(&mut w, 60 * 3 + 5);
    assert!(w.player.dashes >= 1 && events.iter().any(|e| matches!(e, Event::DashReady)), "a charge comes back in 3 s");
}

// REQ: DASH-01, SLOP-02
#[test]
fn a_dash_shakes_off_grabbing_hands() {
    let mut w = sandbox();
    let me = w.player.pos;
    w.spawn_archetype(Archetype::SixFingerHand, me + V2::new(20.0, 0.0));
    run(&mut w, 5);
    assert!(w.player.grabbed > 0);
    w.step(dash(-1.0, 0.0));
    assert_eq!(w.player.grabbed, 0);
    assert!(w.slop.iter().all(|s| s.grab.is_none()));
}

// REQ: DASH-02
#[test]
fn dashes_and_trails_hurt_slop() {
    let mut w = sandbox();
    w.player.perks.dash_damage = 30.0;
    let j = put(&mut w, Archetype::EngagementBait, V2::new(90.0, 0.0));
    w.step(dash(1.0, 0.0));
    run(&mut w, 12);
    assert!(w.slop[j].hp < w.slop[j].max_hp, "Swipe Left hits what you dash through");
    let mut w = sandbox();
    w.player.perks.flame_trail = 6.0;
    for _ in 0..40 {
        w.step(Input::moving(1.0, 0.0));
    }
    assert!(w.zones.iter().any(|z| z.kind == ZoneKind::Fire), "Flame War leaves fire behind");
    let at = w.zones.iter().find(|z| z.kind == ZoneKind::Fire).unwrap().pos;
    let off = at - w.player.pos;
    let k = put(&mut w, Archetype::EngagementBait, off);
    run(&mut w, 20);
    assert!(w.slop[k].hp < w.slop[k].max_hp, "fire burns slop standing in it");
}

// REQ: DEF-01
#[test]
fn boundaries_take_hits_first_and_come_back() {
    let mut w = sandbox();
    w.player.stats.boundaries = 30.0;
    w.player.shield = 30.0;
    w.hurt(20.0, NO_SPEC);
    assert!((w.player.shield - 10.0).abs() < 0.01 && w.player.mental_health == w.player.stats.max_mental_health, "the shield took it");
    w.player.invulnerable = 0;
    w.hurt(20.0, NO_SPEC);
    assert!(w.player.shield <= 0.0 && w.player.mental_health < w.player.stats.max_mental_health, "the rest gets through");
    run(&mut w, 60 * 3 + 60 * 4);
    assert!((w.player.shield - 30.0).abs() < 0.01, "back after a few seconds without a hit");
}

// REQ: DEF-02
#[test]
fn a_firewall_stops_enemy_shots_and_hurts_slop() {
    let firewall = weapon_index("Firewall").unwrap();
    let mut w = World::sandbox(3, &[firewall]);
    let segs = w.barrier_segments();
    assert_eq!(segs.len(), 3);
    let (c, _, _) = segs[0];
    let me = w.player.pos;
    let dir = (c - me).normalized();
    let mut p = Projectile::new(900_000, c + dir * 60.0, -dir * 200.0, BUBBLE_RADIUS, 10.0, DamageType::Analog, true, Look::Bubble, 200);
    p.line = NO_LINE;
    w.projectiles.push(p);
    let events = run(&mut w, 30);
    assert!(events.iter().any(|e| matches!(e, Event::Deflected { .. })), "the barrier stopped the bubble");
    assert_eq!(w.player.mental_health, w.player.stats.max_mental_health);
}

// REQ: ITEM-03
#[test]
fn legendaries_break_the_rules() {
    let rock = weapon_index("Rock").unwrap();
    let mut w = World::sandbox(3, &[rock]);
    w.player.perks.extra_shots = 2;
    put(&mut w, Archetype::ChadGpt, V2::new(200.0, 0.0));
    run(&mut w, 1);
    assert_eq!(w.projectiles.iter().filter(|p| !p.hostile).count(), 3, "Infinite Scroll: two more rocks");
    // Viral Moment: a deletion hurts its neighbours.
    let mut w = sandbox();
    w.player.perks.kill_blast = 0.5;
    let a = w.spawn_archetype(Archetype::ChadGpt, w.player.pos + V2::new(400.0, 0.0));
    let b = put(&mut w, Archetype::EngagementBait, V2::new(440.0, 0.0));
    let b = w.slop[b].id;
    let ja = w.slop.iter().position(|s| s.id == a).unwrap();
    w.slop[ja].hp = 0.0;
    run(&mut w, 1);
    let b = w.slop.iter().find(|s| s.id == b).unwrap();
    assert!(b.hp < b.max_hp, "the explosion hit the slop next to it");
}

// REQ: BIOME-01
#[test]
fn the_map_is_a_patchwork_of_platforms() {
    for seed in [1, 7, 42] {
        for x in -3..3 {
            for y in -3..3 {
                let here = map::biome_of_cell(seed, (x, y));
                assert_ne!(here, map::biome_of_cell(seed, (x + 1, y)), "neighbours differ");
                assert_ne!(here, map::biome_of_cell(seed, (x, y + 1)));
            }
        }
        assert!(map::biome_settled(seed, V2::ZERO, 160.0).is_some(), "a run starts in the middle of a square");
    }
    // Walking into the next square changes where you are, once you are in.
    let mut setup = WaveSetup::sandbox();
    setup.platform = Some(Platform::FaceSlop);
    let mut w = World::new(5, setup, &Loadout::with_weapons(&[]));
    let first = w.biome;
    let mut entered = None;
    for _ in 0..(60 * 14) {
        w.step(Input::moving(1.0, 0.0));
        if let Some(Event::Biome { platform }) = w.drain_events().into_iter().find(|e| matches!(e, Event::Biome { .. })) {
            entered = Some((platform, w.player.pos.x));
        }
    }
    let (platform, x) = entered.expect("walked into the next biome");
    assert_ne!(platform, first);
    assert!(x > map::BIOME / 2.0 + 150.0, "only once properly inside");
}

// REQ: EVENT-01
#[test]
fn a_swarm_crosses_the_screen_and_is_gone() {
    let mut w = sandbox();
    let spec = w.add_spec(SlopSpec::archetype(Archetype::TrampolineBunny).sweeping());
    w.player.invulnerable = u16::MAX;
    let dir = w.sweep(spec, 20);
    assert_eq!(w.slop.len(), 20);
    let before: Vec<V2> = w.slop.iter().map(|s| s.pos).collect();
    run(&mut w, 30);
    for (s, p) in w.slop.iter().zip(&before) {
        assert!((s.pos - *p).dot(dir) > 0.0, "every bunny moves along the swarm's line");
    }
    run(&mut w, 60 * 20);
    assert!(w.slop.is_empty(), "past you and far away, the swarm is gone");
    assert!(w.likes.is_empty() && w.tally.kills == 0, "vanishing is not a kill");
}

// REQ: EVENT-02
#[test]
fn a_livestream_holds_you_in_and_pays_a_chest() {
    let mut w = sandbox();
    w.start_arena(300.0, 5.0);
    let events = run(&mut w, 160);
    assert!(events.iter().any(|e| matches!(e, Event::ArenaClosed { .. })));
    for _ in 0..120 {
        w.step(Input::moving(1.0, 0.0));
    }
    let a = w.arena.expect("still live");
    assert!(w.player.pos.distance(a.centre) <= a.radius, "the ring holds you inside");
    let events = run(&mut w, 60 * 4);
    assert!(events.iter().any(|e| matches!(e, Event::ArenaCleared { .. })) && w.arena.is_none());
    assert!(w.pickups.iter().any(|p| p.kind == PickupKind::Chest && p.value == 2), "a big chest for surviving the stream");
}

// REQ: ICON-01
#[test]
fn brainrot_icons_each_have_their_trick() {
    for a in Archetype::ICONS {
        let s = SlopSpec::archetype(a);
        assert!(s.is_brainrot() && s.hp > 0.0, "{}", s.name);
    }
    // Lirilì Larilò stands still and fires needles.
    let mut w = sandbox();
    w.player.invulnerable = u16::MAX;
    let j = put(&mut w, Archetype::Lirili, V2::new(300.0, 0.0));
    let at = w.slop[j].pos;
    run(&mut w, 300);
    assert_eq!(w.slop[j].pos, at, "it never moves");
    let mut needles = false;
    for _ in 0..300 {
        w.step(Input::default());
        needles |= w.projectiles.iter().any(|p| p.look == Look::Needle);
    }
    assert!(needles, "it fires rings of needles");
    // Tralalelo aims, then runs in a straight line.
    let mut w = sandbox();
    put(&mut w, Archetype::Tralalelo, V2::new(400.0, 0.0));
    let events = run(&mut w, 60);
    assert!(events.iter().any(|e| matches!(e, Event::ChargeAim { .. })), "the charge is telegraphed");
    // Cappuccino Assassinello marks a spot behind you, then appears there.
    let mut w = sandbox();
    let k = put(&mut w, Archetype::Assassinello, V2::new(500.0, 0.0));
    w.slop[k].timer = 1;
    let events = run(&mut w, 5);
    let Some(Event::BlinkWarning { at }) = events.into_iter().find(|e| matches!(e, Event::BlinkWarning { .. })) else { panic!("no warning") };
    run(&mut w, 50);
    assert!(w.slop[k].pos.distance(at) < 60.0, "it blinked to the marked spot");
    assert_eq!(SlopSpec::archetype(Archetype::Lirili).movement, Movement::Stand);
}

// REQ: STATUS-02
#[test]
fn a_brainrot_zone_flips_you_only_if_you_stay_for_the_drop() {
    let mut w = sandbox();
    let me = w.player.pos;
    w.zones.push(Zone::new(1, ZoneKind::Brainrot, me, 100.0, 30));
    let events = run(&mut w, 31);
    assert!(w.player.status.brainrot > 0 && events.iter().any(|e| matches!(e, Event::Status { kind: StatusKind::Brainrot })));
    let mut w = sandbox();
    w.zones.push(Zone::new(1, ZoneKind::Brainrot, me + V2::new(500.0, 0.0), 100.0, 30));
    run(&mut w, 31);
    assert_eq!(w.player.status.brainrot, 0, "outside the zone, nothing happens");
}

// REQ: LOOT-02, LOOT-03
#[test]
fn the_vacuum_pulls_every_like_and_coin() {
    let mut w = sandbox();
    let me = w.player.pos;
    w.drop_pickup(PickupKind::Clout, me + V2::new(1200.0, 300.0), 12);
    w.likes.push(Like { id: 5, pos: me + V2::new(-1300.0, 0.0), value: 3, pulled: false, fake: false });
    w.drop_pickup(PickupKind::Magnet, me, 0);
    let events = run(&mut w, 150);
    assert!(events.iter().any(|e| matches!(e, Event::Vacuum)));
    assert_eq!((w.xp, w.clout), (3, 12), "the far Like and the bag of coins came to you");
}

// REQ: MAP-04
#[test]
fn a_fence_waits_while_you_stand_where_it_would_close() {
    let mut w = sandbox();
    let me = w.player.pos;
    w.map.add_fence(me, V2::new(100.0, 100.0), 10, 300);
    run(&mut w, 60);
    assert!(w.map.fences.iter().all(|f| !f.solid()), "it does not close on top of you");
    for _ in 0..60 {
        w.step(Input::moving(1.0, 0.0));
    }
    run(&mut w, 2);
    assert!(w.map.fences.iter().any(|f| f.solid()), "it closes once you have left");
}

// REQ: FAIR-05
#[test]
fn nothing_reverses_drags_or_shoves_you() {
    for status in [StatusKind::Brainrot, StatusKind::Doomscroll] {
        let mut w = sandbox();
        w.apply_status(status);
        run(&mut w, 50);
        let x = w.player.pos.x;
        for _ in 0..30 {
            w.step(Input::moving(1.0, 0.0));
        }
        assert!(w.player.pos.x > x + 40.0, "{status:?}: you still go where you steer");
        let y = w.player.pos.y;
        run(&mut w, 30);
        assert!((w.player.pos.y - y).abs() < 1.0, "{status:?}: nothing drags you");
    }
    // A post that hits you slows you; it does not push you.
    let mut w = sandbox();
    let me = w.player.pos;
    let mut p = Projectile::new(900_001, me + V2::new(-30.0, 0.0), V2::new(300.0, 0.0), 16.0, 1.0, DamageType::Analog, true, Look::Post, 60);
    p.shove = 420.0;
    w.projectiles.push(p);
    run(&mut w, 3);
    assert!(w.player.pos.distance(me) < 2.0 && w.player.status.doomscroll > 0);
}

// REQ: LOOT-04
#[test]
fn the_loot_goblin_runs_and_pays_if_caught() {
    let mut w = sandbox();
    let me = w.player.pos;
    let j = w.spawn_archetype(Archetype::LootGoblin, me + V2::new(200.0, 0.0));
    run(&mut w, 30);
    let g = w.slop.iter().find(|s| s.id == j).unwrap();
    assert!(g.pos.distance(me) > 210.0, "he runs away");
    let k = w.slop.iter().position(|s| s.id == j).unwrap();
    w.slop[k].hp = 0.0;
    run(&mut w, 1);
    assert!(w.pickups.iter().filter(|p| p.kind == PickupKind::Clout).count() >= 6);
    assert!(w.pickups.iter().any(|p| p.kind == PickupKind::Chest));
    // Left alone, he gets away with nothing dropped.
    let mut w = sandbox();
    w.spawn_archetype(Archetype::LootGoblin, me + V2::new(200.0, 0.0));
    run(&mut w, (GOBLIN_TICKS + 5) as u32);
    assert!(w.slop.is_empty() && w.pickups.is_empty());
}

// REQ: ELITE-02, CHEST-02
#[test]
fn summoners_call_minions_and_split_copies_carry_no_chests() {
    let mut w = sandbox();
    let mut spec = SlopSpec::archetype(Archetype::ChadGpt).elite();
    spec.traits.0 |= crate::spec::Traits::SUMMONER | crate::spec::Traits::VIRAL;
    let i = w.add_spec(spec);
    w.place(i, w.player.pos + V2::new(400.0, 0.0), 0, 1);
    w.player.invulnerable = u16::MAX;
    run(&mut w, 320);
    assert!(w.slop.len() > 1, "a summoner called minions");
    // Viral copies of an elite never drop chests of their own.
    for s in w.slop.iter_mut() {
        if s.spec as usize == i {
            s.hp = 0.0;
        }
    }
    run(&mut w, 1);
    let copies: Vec<usize> = (0..w.slop.len()).filter(|&k| w.slop[k].spec as usize == i).collect();
    for k in copies {
        w.slop[k].hp = 0.0;
    }
    run(&mut w, 1);
    assert!(w.pickups.iter().filter(|p| p.kind == PickupKind::Chest).count() <= 1);
}

// REQ: MAP-05
#[test]
fn a_power_outlet_refills_dashes_and_speeds_you_up() {
    let mut w = sandbox();
    w.player.dashes = 0;
    let me = w.player.pos;
    w.map.props.push(crate::map::Prop { id: 77, kind: crate::map::PropKind::PowerOutlet, pos: me, charge: 0, state: crate::map::PropState::Idle, left: 60, price: 0 });
    run(&mut w, 2);
    assert_eq!(w.player.dashes, w.player.perks.dash_charges);
    assert!(w.player.charged > 0);
}

// REQ: INJECT-01
#[test]
fn a_prompt_injection_turns_the_slop_on_each_other() {
    let mut w = sandbox();
    let me = w.player.pos;
    let j = put(&mut w, Archetype::ChadGpt, V2::new(100.0, 0.0));
    let id = w.slop[j].id;
    w.slop[j].hp = 100.0;
    w.slop[j].max_hp = 100.0;
    w.drop_pickup(PickupKind::Inject, me, 0);
    let events = run(&mut w, 40);
    assert!(events.iter().any(|e| matches!(e, Event::PromptInjection { .. })));
    let s = w.slop.iter().find(|s| s.id == id).unwrap();
    assert!(s.hp < 100.0 && s.confused > 0, "it stopped attacking and is being torn at");
    assert_eq!(w.player.mental_health, w.player.stats.max_mental_health, "and it leaves you alone");
}

// REQ: TEMP-01
#[test]
fn hallucinated_likes_are_fakes() {
    let mut w = sandbox();
    w.hallucinate = 1.0;
    let id = w.spawn_archetype(Archetype::ChadGpt, w.player.pos + V2::new(600.0, 0.0));
    w.slop.iter_mut().find(|s| s.id == id).unwrap().hp = 0.0;
    run(&mut w, 1);
    assert!(!w.likes.is_empty() && w.likes.iter().all(|l| l.fake));
}

// REQ: NEMESIS-01
#[test]
fn a_nemesis_always_drops_a_golden_chest() {
    let mut w = sandbox();
    let mut spec = SlopSpec::archetype(Archetype::GlassFruit);
    spec.traits.0 |= crate::spec::Traits::NEMESIS;
    let i = w.add_spec(spec);
    let id = w.place(i, w.player.pos + V2::new(600.0, 0.0), 0, 1);
    w.slop.iter_mut().find(|s| s.id == id).unwrap().hp = 0.0;
    run(&mut w, 1);
    assert!(w.nemesis_down);
    assert!(w.pickups.iter().any(|p| p.kind == PickupKind::Chest && p.value == 3));
}
