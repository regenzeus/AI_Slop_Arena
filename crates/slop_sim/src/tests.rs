//! Behaviour tests for the arena. Each names the requirement it verifies
//! (see docs/requirements.md).

use crate::bot;
use crate::content::{Look, SlopKind, WEAPONS};
use crate::events::Event;
use crate::stats::DamageType;
use crate::vec::V2;
use crate::world::*;

fn idle() -> Input {
    Input::default()
}

/// A sandbox where you hold only the given weapons.
fn armed(weapons: &[usize]) -> World {
    let mut w = World::sandbox(3);
    w.player.weapons = weapons.iter().map(|&d| Weapon::new(d, 1)).collect();
    w
}

fn weapon(name: &str) -> usize {
    WEAPONS.iter().position(|w| w.name == name).unwrap()
}

fn bot_wave(seed: u64, ticks: u32) -> World {
    let mut w = World::new(seed, WaveConfig::first());
    for _ in 0..ticks {
        let input = bot::steer(&w);
        w.step(input);
    }
    w
}

fn hostile_bubble(at: V2, vel: V2) -> Projectile {
    Projectile {
        id: 999_999,
        pos: at,
        vel,
        radius: BUBBLE_RADIUS,
        damage: 6.0,
        kind: DamageType::Analog,
        hostile: true,
        pierce: 0,
        bounces: 0,
        mark: None,
        look: Look::Bubble,
        life: 240,
        line: 0,
        last_hit: 0,
        knockback: 0.0,
    }
}

// REQ: SIM-01
#[test]
fn same_seed_and_inputs_replay_exactly() {
    let a = bot_wave(7, 1800);
    let b = bot_wave(7, 1800);
    assert_eq!(a.checksum(), b.checksum());
    assert!(!a.slop.is_empty() || a.tally.kills > 0, "the wave should have spawned slop");
    let c = bot_wave(8, 1800);
    assert_ne!(a.checksum(), c.checksum());
}

// REQ: SIM-02
#[test]
fn you_stay_inside_the_arena() {
    let mut w = World::sandbox(1);
    for _ in 0..900 {
        w.step(Input { x: -1.0, y: -1.0 });
    }
    let p = w.player.pos;
    assert!(p.x >= PLAYER_RADIUS && p.y >= PLAYER_RADIUS);
    assert!(p.x < PLAYER_RADIUS + 1.0 && p.y < PLAYER_RADIUS + 1.0);
}

// REQ: SIM-03
#[test]
fn movement_eases_to_full_speed() {
    let mut w = World::sandbox(1);
    w.step(Input { x: 1.0, y: 0.0 });
    assert!(w.player.vel.x < PLAYER_SPEED * 0.5);
    for _ in 0..60 {
        w.step(Input { x: 1.0, y: 0.0 });
    }
    assert!((w.player.vel.x - PLAYER_SPEED).abs() < 1.0);
    // Diagonal input is not faster.
    let mut d = World::sandbox(1);
    for _ in 0..60 {
        d.step(Input { x: 1.0, y: 1.0 });
    }
    assert!((d.player.vel.length() - PLAYER_SPEED).abs() < 1.0);
}

// REQ: WPN-01
#[test]
fn every_weapon_damages_slop_in_range() {
    for (d, def) in WEAPONS.iter().enumerate() {
        let mut w = armed(&[d]);
        let at = w.player.pos + V2::new(def.range * 0.5, 0.0);
        let id = w.spawn_slop(SlopKind::BubbleBot, at);
        for _ in 0..120 {
            w.step(idle());
        }
        let hurt = w.slop.iter().find(|s| s.id == id).is_none_or(|s| s.hp < s.max_hp);
        assert!(hurt, "{} never hit", def.name);
        assert!(w.tally.damage_dealt[def.kind as usize] > 0.0, "{}", def.name);
    }
}

// REQ: WPN-02
#[test]
fn swings_only_hit_in_front() {
    let mut w = armed(&[weapon("Stick")]);
    let front = w.spawn_slop(SlopKind::Listicle, w.player.pos + V2::new(50.0, 0.0));
    let back = w.spawn_slop(SlopKind::Listicle, w.player.pos + V2::new(-80.0, 0.0));
    w.step(idle());
    let hp = |id| w.slop.iter().find(|s| s.id == id).map_or(0.0, |s| s.hp);
    assert!(hp(front) < SlopKind::Listicle.def().hp);
    assert_eq!(hp(back), SlopKind::Listicle.def().hp);
}

// REQ: WPN-03
#[test]
fn eye_rolls_bounce_between_slop() {
    let mut w = armed(&[weapon("Eye-Roll")]);
    for x in [120.0, 200.0, 280.0] {
        w.spawn_slop(SlopKind::BubbleBot, w.player.pos + V2::new(x, 0.0));
    }
    for _ in 0..40 {
        w.step(idle());
    }
    let damaged = w.slop.iter().filter(|s| s.hp < s.max_hp).count();
    assert!(damaged >= 2, "one throw should hit several, hit {damaged}");
}

// REQ: WPN-04
#[test]
fn red_pen_marks_and_marks_add_damage() {
    let mut w = armed(&[weapon("Red Pen")]);
    let id = w.spawn_slop(SlopKind::EmDashLancer, w.player.pos + V2::new(200.0, 0.0));
    for _ in 0..30 {
        w.step(idle());
    }
    let s = w.slop.iter().find(|s| s.id == id).unwrap();
    assert!(s.marked > 0 && s.mark_bonus > 0.0);
}

// REQ: DMG-01
#[test]
fn marks_and_resistance_scale_damage() {
    let pistol = weapon("Pistol");
    let base = WEAPONS[pistol].damage;
    let first_hit = |setup: &dyn Fn(&mut Slop)| {
        let mut w = armed(&[pistol]);
        w.spawn_slop(SlopKind::EmDashLancer, w.player.pos + V2::new(150.0, 0.0));
        setup(&mut w.slop[0]);
        for _ in 0..30 {
            w.step(idle());
            for e in w.drain_events() {
                if let Event::SlopHit { amount, .. } = e {
                    return amount;
                }
            }
        }
        panic!("no hit");
    };
    assert!((first_hit(&|_| {}) - base).abs() < 1e-4);
    let marked = first_hit(&|s| {
        s.marked = 600;
        s.mark_bonus = 0.2;
    });
    assert!((marked - base * 1.2).abs() < 1e-4);
    let resisted = first_hit(&|s| s.resist[DamageType::Analog as usize] = 0.5);
    assert!((resisted - base * 0.5).abs() < 1e-4);
}

// REQ: SLOP-01
#[test]
fn bubble_bots_fire_fans_of_three_with_lines() {
    let mut w = armed(&[]);
    w.spawn_slop(SlopKind::BubbleBot, w.player.pos + V2::new(350.0, 0.0));
    let (mut most, mut talking) = (0, 0);
    for _ in 0..200 {
        w.step(idle());
        let bubbles: Vec<_> = w.projectiles.iter().filter(|p| p.look == Look::Bubble).collect();
        let line_ok = |p: &&Projectile| p.line == NO_LINE || (p.line as usize) < crate::content::BUBBLE_LINES.len();
        assert!(bubbles.iter().all(|p| p.hostile && line_ok(p)));
        talking = talking.max(bubbles.iter().filter(|p| p.line != NO_LINE).count());
        most = most.max(bubbles.len());
    }
    assert!(most >= 3);
    // Only the middle bubble of a fan says something, so crowds stay readable.
    assert!(talking >= 1 && talking < most);
}

// REQ: SLOP-02
#[test]
fn hands_grab_slow_and_hurt_you() {
    let mut w = armed(&[]);
    w.spawn_slop(SlopKind::SixFingerHand, w.player.pos + V2::new(40.0, 0.0));
    for _ in 0..10 {
        w.step(idle());
    }
    assert_eq!(w.player.grabbed, 1);
    for _ in 0..90 {
        w.step(Input { x: 1.0, y: 0.0 });
    }
    assert!((w.player.vel.x - PLAYER_SPEED * 0.8).abs() < 2.0, "speed {}", w.player.vel.x);
    assert!(w.player.mental_health < 100.0);
    // The hand travels with you.
    assert!(w.slop[0].pos.distance(w.player.pos) < 40.0);
}

// REQ: SLOP-03
#[test]
fn killing_number_one_first_scatters_the_pack() {
    let pack = |kill: u8| {
        let mut w = armed(&[]);
        for n in 1..=5u8 {
            w.spawn_slop(SlopKind::Listicle, w.player.pos + V2::new(400.0, n as f32 * 30.0));
            let s = w.slop.last_mut().unwrap();
            s.pack = 7;
            s.number = n;
        }
        w.slop[kill as usize - 1].hp = 0.0;
        w.step(idle());
        let scattered = w.drain_events().iter().any(|e| matches!(e, Event::Scattered { .. }));
        (scattered, w.slop.iter().all(|s| s.fleeing > 0))
    };
    assert_eq!(pack(1), (true, true));
    assert_eq!(pack(3), (false, false));
}

// REQ: SLOP-04
#[test]
fn yes_bots_buff_nearby_slop() {
    let mut w = armed(&[]);
    let at = w.player.pos + V2::new(600.0, 0.0);
    let friend = w.spawn_slop(SlopKind::BubbleBot, at);
    w.spawn_slop(SlopKind::YesBot, at + V2::new(40.0, 0.0));
    let mut agreed = false;
    for _ in 0..120 {
        w.step(idle());
        agreed |= w.drain_events().iter().any(|e| matches!(e, Event::Agreed { .. }));
    }
    assert!(agreed);
    let f = w.slop.iter().find(|s| s.id == friend).unwrap();
    assert!(f.agreed >= 1 && f.agreement() > 1.0);
}

// REQ: FAIR-01
#[test]
fn slop_far_away_never_shoots() {
    let mut w = armed(&[]);
    w.spawn_slop(SlopKind::BubbleBot, w.player.pos + V2::new(1000.0, 0.0));
    for _ in 0..150 {
        w.step(idle());
    }
    assert!(w.projectiles.is_empty());
}

// REQ: FAIR-02
#[test]
fn lancers_aim_before_they_throw() {
    let mut w = armed(&[]);
    w.spawn_slop(SlopKind::EmDashLancer, w.player.pos + V2::new(300.0, 0.0));
    w.step(idle());
    assert!(w.drain_events().iter().any(|e| matches!(e, Event::LancerAim { .. })));
    for _ in 0..LANCE_AIM - 1 {
        w.step(idle());
        assert!(w.projectiles.is_empty());
    }
    w.step(idle());
    assert!(w.projectiles.iter().any(|p| p.look == Look::EmDash && p.hostile));
}

// REQ: FAIR-02
#[test]
fn spawns_are_telegraphed_away_from_you() {
    let mut w = World::new(5, WaveConfig::first());
    let mut warned = None;
    for _ in 0..300 {
        w.step(idle());
        if let Some(Event::SpawnWarning { at, .. }) = w.drain_events().into_iter().find(|e| matches!(e, Event::SpawnWarning { .. })) {
            warned = Some((at, w.tick));
            break;
        }
    }
    let (at, tick) = warned.expect("a spawn warning");
    assert!(at.distance(w.player.pos) >= 450.0);
    while w.tick < tick + SPAWN_WARNING as u32 {
        w.step(idle());
    }
    assert!(w.slop.iter().any(|s| s.pos.distance(at) < 40.0));
}

// REQ: FAIR-03
#[test]
fn a_hit_gives_brief_invulnerability() {
    let mut w = armed(&[]);
    for _ in 0..3 {
        w.spawn_slop(SlopKind::Listicle, w.player.pos);
    }
    w.step(idle());
    let contact = SlopKind::Listicle.def().contact;
    assert!((w.player.mental_health - (100.0 - contact)).abs() < 1e-4);
    for _ in 0..31 {
        w.step(idle());
    }
    assert!(w.player.mental_health < 100.0 - contact);
}

// REQ: BUB-01
#[test]
fn your_shots_pop_bubbles() {
    let mut w = armed(&[weapon("Pistol")]);
    w.spawn_slop(SlopKind::EmDashLancer, w.player.pos + V2::new(450.0, 0.0));
    let p = w.player.pos;
    w.projectiles.push(hostile_bubble(p + V2::new(200.0, 0.0), V2::new(-10.0, 0.0)));
    for _ in 0..40 {
        w.step(idle());
    }
    assert_eq!(w.tally.bubbles_popped, 1);
}

// REQ: LIKE-01
#[test]
fn likes_fly_to_you_within_curiosity() {
    let mut w = armed(&[]);
    let p = w.player.pos;
    w.likes.push(Like { id: 1, pos: p + V2::new(80.0, 0.0), value: 3, pulled: false });
    w.likes.push(Like { id: 2, pos: p + V2::new(400.0, 0.0), value: 5, pulled: false });
    for _ in 0..30 {
        w.step(idle());
    }
    assert_eq!(w.clout, 3);
    assert_eq!(w.likes.len(), 1);
}

// REQ: WAVE-01
#[test]
fn a_wave_clears_on_time_and_credits_half_the_leftovers() {
    let mut w = World::new(1, WaveConfig { number: 1, seconds: 2.0, spawning: false });
    w.player.weapons.clear();
    w.likes.push(Like { id: 1, pos: V2::new(100.0, 100.0), value: 10, pulled: false });
    for _ in 0..119 {
        w.step(idle());
    }
    assert_eq!(w.outcome, Outcome::Running);
    w.step(idle());
    assert_eq!(w.outcome, Outcome::Cleared);
    assert_eq!(w.clout, 5);
}

// REQ: WAVE-01
#[test]
fn burnout_ends_the_wave() {
    let mut w = armed(&[]);
    w.player.mental_health = 1.0;
    w.spawn_slop(SlopKind::Listicle, w.player.pos);
    w.step(idle());
    assert_eq!(w.outcome, Outcome::Burnout);
    let tick = w.tick;
    w.step(idle());
    assert_eq!(w.tick, tick, "a finished wave does not advance");
}

// REQ: WAVE-02
#[test]
fn the_wave_ramps_up_and_mixes_kinds() {
    assert!(spawn_rate(0.0) < spawn_rate(1.0));
    let mut w = World::new(11, WaveConfig::first());
    let (mut early, mut late) = (0, 0);
    let mut kinds = std::collections::HashSet::new();
    while w.outcome == Outcome::Running && w.tick < 3600 {
        let input = bot::steer(&w);
        w.step(input);
        for e in w.drain_events() {
            if let Event::SpawnWarning { kind, .. } = e {
                kinds.insert(kind);
                if w.tick < 600 {
                    early += 1;
                } else if w.tick >= 2400 && w.tick < 3000 {
                    late += 1;
                }
            }
        }
    }
    assert!(late > early, "early {early}, late {late}");
    assert!(kinds.len() >= 4, "{kinds:?}");
}
