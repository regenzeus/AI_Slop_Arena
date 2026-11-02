//! A simple bot that steers the human: it keeps away from slop, sidesteps
//! incoming bubbles, avoids walls and grabs Likes when it is safe. The lab
//! uses it to measure waves, and the client uses it for autoplay. It is also
//! the seed of the AI-controlled hero planned for later.

use crate::vec::V2;
use crate::world::{Input, World, ZoneKind, ARENA};

/// Slop closer than this pushes the bot away.
const FEAR: f32 = 340.0;
/// Bubbles closer than this are dodged.
const DODGE: f32 = 220.0;
/// Distance from a wall where it starts to push back.
const WALL: f32 = 260.0;
/// Likes are collected only when no slop is this close.
const SAFE: f32 = 200.0;
const GREED: f32 = 500.0;
/// Even a melee build keeps this much room.
const MIN_FEAR: f32 = 110.0;

pub fn steer(world: &World) -> Input {
    let me = world.player.pos;
    let mut force = V2::ZERO;
    let mut closest = f32::MAX;
    // Fight at the range your weapons reach: melee builds let slop come closer.
    let weapons = &world.player.weapons;
    let reach = if weapons.is_empty() {
        FEAR
    } else {
        weapons.iter().map(|w| w.def().range).sum::<f32>() / weapons.len() as f32 * (1.0 + world.player.stats.reach)
    };
    let fear = (reach * 0.9).clamp(MIN_FEAR, FEAR);
    for s in &world.slop {
        let away = me - s.pos;
        let d = away.length();
        closest = closest.min(d);
        if d < fear {
            let w = (fear - d) / fear;
            force += away.normalized() * (w * w * 3.0);
        }
    }
    for p in world.projectiles.iter().filter(|p| p.hostile) {
        let rel = me - p.pos;
        let d = rel.length();
        if d < DODGE && p.vel.dot(rel) > 0.0 {
            let side = p.vel.perp().normalized();
            let sign = if side.dot(rel) >= 0.0 { 1.0 } else { -1.0 };
            force += side * (sign * 2.0 * (DODGE - d) / DODGE);
        }
    }
    let edge = |v: f32| -> f32 {
        if v < WALL {
            (WALL - v) / WALL
        } else if v > ARENA - WALL {
            -(v - (ARENA - WALL)) / WALL
        } else {
            0.0
        }
    };
    force += V2::new(edge(me.x), edge(me.y)) * 2.5;
    force += (V2::new(ARENA / 2.0, ARENA / 2.0) - me) * 0.0003;
    // Get out of bomb spots and downvote zones.
    for z in world.zones.iter().filter(|z| matches!(z.kind, ZoneKind::Bomb | ZoneKind::Downvote)) {
        let away = me - z.pos;
        let d = away.length();
        if d < z.radius + 40.0 {
            force += away.normalized() * 2.5;
        }
    }
    // Nothing in reach: close in on the nearest slop, or the boss.
    if closest > reach * 1.05 {
        let boss = world.boss.as_ref().and_then(|b| world.slop.iter().find(|s| s.id == b.id));
        let target = boss.map(|s| s.pos).or_else(|| world.slop.iter().map(|s| s.pos).min_by(|a, b| a.distance_sq(me).total_cmp(&b.distance_sq(me))));
        if let Some(at) = target {
            force += (at - me).normalized() * 0.7;
        }
    }
    if closest > SAFE {
        let like = world
            .likes
            .iter()
            .filter(|l| !l.fake)
            .map(|l| (l.pos, l.pos.distance_sq(me)))
            .filter(|(_, d)| *d < GREED * GREED)
            .min_by(|a, b| a.1.total_cmp(&b.1));
        if let Some((at, _)) = like {
            force += (at - me).normalized() * 0.8;
        }
    }
    if force.length() < 0.05 {
        // Nothing to do: drift in a slow circle so weapons keep facing new slop.
        force = world.player.facing.rotated(0.02);
    }
    let dir = force.normalized();
    Input { x: dir.x, y: dir.y }
}
