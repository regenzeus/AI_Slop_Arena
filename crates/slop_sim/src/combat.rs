//! Your weapons, every projectile, damage to slop, and zones on the floor
//! (WPN-01 to WPN-08, DMG-01, BUB-01).

use crate::content::{Look, Pattern, TIER_DAMAGE, WEAPONS};
use crate::events::{Event, StatusKind};
use crate::spec::{Movement, Traits};
use crate::stats::DamageType;
use crate::vec::V2;
use crate::world::*;

/// How far a ricochet looks for its next target.
const RICOCHET_RANGE: f32 = 300.0;
const CHAIN_RANGE: f32 = 220.0;
/// Pull of a Frisbee back towards you, in units per second squared.
const BOOMERANG_PULL: f32 = 430.0;
const ORBIT_SPEED: f32 = 2.6;
const ORBIT_REACH: f32 = 26.0;
/// Damage taken through a beauty filter from the front.
const SHIELDED: f32 = 0.4;
const FILTERED: f32 = 0.7;
const RATIO_DAMAGE: f32 = 0.8;
const SHIELD_OFF_TICKS: u16 = 180;
/// A Repost Hydra splits off a repost when one hit takes this share of its HP.
const HYDRA_SPLIT: f32 = 0.05;
const MAX_REPOSTS: usize = 20;

impl World {
    /// Applies a hit to one slop (DMG-01): Conviction, the type bonus, a
    /// mark, its resistance, filters and shields all multiply the damage.
    pub(crate) fn damage_slop(&mut self, j: usize, base: f32, kind: DamageType, push: V2, from: V2, weapon: Option<usize>) {
        let stats = self.player.stats;
        let ratio = self.player.status.ratio > 0;
        let perks = &self.player.perks;
        let filtered = self.zones.iter().any(|z| z.kind == ZoneKind::Filter && z.pos.distance_sq(self.slop[j].pos) < z.radius * z.radius);
        let s = &mut self.slop[j];
        if s.hp <= 0.0 {
            return;
        }
        let sp = &self.setup.specs[s.spec as usize];
        if sp.traits.has(Traits::DODGE_FIRST) && !s.dodged {
            s.dodged = true;
            self.events.push(Event::SlopDodged { at: s.pos });
            return;
        }
        let k = kind as usize;
        let mut mult = (1.0 + stats.conviction) * (1.0 + perks.type_bonus[k]) * (1.0 - s.resist[k]).max(0.1) * s.armor;
        if sp.is_brainrot() {
            mult *= 1.0 + perks.brainrot_bonus;
        }
        if s.marked > 0 {
            mult *= 1.0 + s.mark_bonus;
        }
        if perks.group_chat > 0 {
            mult *= 1.0 + 0.01 * perks.group_chat as f32 * (self.crowd / 5) as f32;
        }
        if ratio {
            mult *= RATIO_DAMAGE;
        }
        if filtered {
            mult *= FILTERED;
        }
        if sp.traits.has(Traits::FRONT_SHIELD) && s.shield_off == 0 {
            let facing = if s.moving.length_sq() > 1.0 { s.moving.normalized() } else { (self.player.pos - s.pos).normalized() };
            if facing.dot((from - s.pos).normalized()) > 0.5 {
                mult *= SHIELDED;
            }
        }
        let amount = base * mult;
        s.hp -= amount;
        s.flash = FLASH_TICKS;
        s.last_hit = Some(self.tick);
        s.last_type = k as u8;
        if s.grab.is_none() && !sp.traits.has(Traits::NO_KNOCKBACK) {
            s.push += push;
        }
        self.tally.damage_dealt[k] += amount;
        if let Some(w) = weapon.and_then(|w| self.tally.weapon_damage.get_mut(w)) {
            *w += amount;
        }
        self.events.push(Event::SlopHit { at: s.pos, amount, kind });

        // Things a hit sets off.
        let (pos, spec, max_hp) = (s.pos, s.spec as usize, s.max_hp);
        if sp.traits.has(Traits::SPLIT) && !s.split && s.hp > 0.0 {
            s.split = true;
            let half = s.hp / 2.0;
            s.hp = half;
            self.pending.push((spec, pos + V2::new(20.0, 0.0), half, 0, 1));
            self.events.push(Event::Split { at: pos });
        }
        if let Some(b) = &mut self.boss {
            if b.id == s.id {
                if b.kind == crate::content::BossKind::RepostHydra && amount > max_hp * HYDRA_SPLIT {
                    let minion = b.minion;
                    let reposts = self.slop.iter().filter(|o| o.spec as usize == minion).count();
                    if reposts < MAX_REPOSTS {
                        self.pending.push((minion, pos, max_hp * 0.05, 0, 1));
                        self.events.push(Event::Reposted { at: pos });
                    }
                }
                let s = &mut self.slop[j];
                if b.kind == crate::content::BossKind::TheModel && s.hp <= 0.0 && !b.regenerated && self.setup.model.collapse < 0.6 {
                    b.regenerated = true;
                    s.hp = s.max_hp * 0.5;
                    self.events.push(Event::Regenerate { at: s.pos });
                }
            }
        }
    }

    /// Hits `first`, then jumps to `links - 1` more slop nearby.
    pub(crate) fn chain_hit(&mut self, from: V2, first: usize, links: u8, damage: f32, kind: DamageType, weapon: Option<usize>) {
        let mut hit = vec![first];
        let mut at = from;
        let mut current = first;
        for _ in 0..links {
            let to = self.slop[current].pos;
            self.events.push(Event::Chain { from: at, to });
            self.damage_slop(current, damage, kind, V2::ZERO, at, weapon);
            at = to;
            let mut best: Option<(usize, f32)> = None;
            self.grid.for_each_near(at, CHAIN_RANGE, |j| {
                let s = &self.slop[j];
                let d = s.pos.distance_sq(at);
                if s.hp > 0.0 && !hit.contains(&j) && d < CHAIN_RANGE * CHAIN_RANGE && best.is_none_or(|(_, b)| d < b) {
                    best = Some((j, d));
                }
            });
            let Some((next, _)) = best else { break };
            hit.push(next);
            current = next;
        }
    }

    /// Every slop within `radius` of `at`.
    fn around(&self, at: V2, radius: f32) -> Vec<usize> {
        let mut out = Vec::new();
        self.grid.for_each_near(at, radius + 40.0, |j| {
            let s = &self.slop[j];
            if s.hp > 0.0 && s.pos.distance_sq(at) < (radius + s.radius) * (radius + s.radius) {
                out.push(j);
            }
        });
        out
    }

    /// Every weapon whose cooldown is up attacks (WPN-01).
    pub(crate) fn fire_weapons(&mut self) {
        let stats = self.player.stats;
        let at = self.player.pos;
        for wi in 0..self.player.weapons.len() {
            let (def_index, tier, enabled) = {
                let w = &self.player.weapons[wi];
                (w.def, w.tier, w.enabled)
            };
            if !enabled {
                continue;
            }
            let def = &WEAPONS[def_index];
            let range = def.range * (1.0 + stats.reach);
            let damage = def.damage * TIER_DAMAGE[tier as usize - 1];
            let mut rate = 1.0 + stats.focus;
            if matches!(def.pattern, Pattern::Aura) {
                let crowd = self.around(at, range).len().min(10);
                rate *= 1.0 + 0.1 * crowd as f32;
            }
            if let Pattern::Orbit { .. } = def.pattern {
                self.player.weapons[wi].phase += ORBIT_SPEED * DT * (1.0 + stats.focus);
            }
            self.player.weapons[wi].cooldown -= rate;
            if self.player.weapons[wi].cooldown > 0.0 {
                continue;
            }
            let fired = self.attack(wi, def_index, damage, range);
            let w = &mut self.player.weapons[wi];
            if fired {
                w.cooldown += def.cooldown * TICK_RATE as f32;
            } else {
                // Ready, waiting for something to hit.
                w.cooldown = 0.0;
            }
        }
    }

    /// One attack of weapon slot `wi`. Returns false when there was nothing to attack.
    fn attack(&mut self, wi: usize, def_index: usize, damage: f32, range: f32) -> bool {
        let def = &WEAPONS[def_index];
        let at = self.player.pos;
        let slot = Some(wi);
        match def.pattern {
            Pattern::Orbit { radius } => {
                let phase = self.player.weapons[wi].phase;
                let dog = at + V2::from_angle(phase) * radius;
                let hits = self.around(dog, ORBIT_REACH);
                for j in &hits {
                    let push = (self.slop[*j].pos - at).normalized() * def.knockback;
                    self.damage_slop(*j, damage, def.kind, push, dog, slot);
                }
                return true;
            }
            Pattern::Ring | Pattern::Aura => {
                let hits = self.around(at, range);
                if hits.is_empty() {
                    return false;
                }
                for j in hits {
                    let push = (self.slop[j].pos - at).normalized() * def.knockback;
                    self.damage_slop(j, damage, def.kind, push, at, slot);
                }
                self.events.push(Event::Ring { at, radius: range });
                return true;
            }
            Pattern::Reflect => {
                let bubble = self
                    .projectiles
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| p.hostile && p.life > 0 && p.pos.distance_sq(at) < range * range)
                    .min_by(|a, b| a.1.pos.distance_sq(at).total_cmp(&b.1.pos.distance_sq(at)))
                    .map(|(i, _)| i);
                let Some(b) = bubble else { return false };
                let from = self.projectiles[b].pos;
                let Some(t) = nearest_slop(&self.grid, &self.slop, from, 600.0, None) else { return false };
                let dir = (self.slop[t].pos - from).normalized();
                let p = &mut self.projectiles[b];
                p.hostile = false;
                p.vel = dir * 520.0;
                p.damage = damage;
                p.kind = def.kind;
                p.life = 90;
                p.weapon = wi as u8;
                p.doomscroll = false;
                p.shove = 0.0;
                self.events.push(Event::Reflected { at: from });
                return true;
            }
            _ => {}
        }
        let Some(target) = nearest_slop(&self.grid, &self.slop, at, range, None) else { return false };
        let goal = self.slop[target].pos;
        let aim = (goal - at).normalized();
        let life = |speed: f32| (range / speed * TICK_RATE as f32 * 1.15) as u16 + 4;
        let mut shots = Vec::new();
        match def.pattern {
            Pattern::Shot { speed, radius, pierce, look } => {
                let mut p = Projectile::new(0, at + aim * 20.0, aim * speed, radius, damage, def.kind, false, look, life(speed));
                p.pierce = pierce;
                shots.push(p);
            }
            Pattern::Burst { speed, radius, count, spread, look } => {
                for k in 0..count {
                    let off = (k as f32 - (count as f32 - 1.0) / 2.0) * spread;
                    let dir = aim.rotated(off);
                    shots.push(Projectile::new(0, at + dir * 20.0, dir * speed, radius, damage, def.kind, false, look, life(speed)));
                }
            }
            Pattern::Lob { speed, splash, look } => {
                let dist = goal.distance(at);
                let mut p = Projectile::new(0, at, aim * speed, 8.0, damage, def.kind, false, look, (dist / speed * TICK_RATE as f32) as u16 + 1);
                p.splash = splash;
                shots.push(p);
            }
            Pattern::Mark { speed, radius, pierce, bonus, seconds } => {
                let bonus = bonus + self.player.perks.mark_bonus;
                let mut p = Projectile::new(0, at + aim * 20.0, aim * speed, radius, damage, def.kind, false, Look::Pen, life(speed));
                p.pierce = pierce;
                p.mark = Some((bonus, (seconds * TICK_RATE as f32) as u16));
                shots.push(p);
            }
            Pattern::Ricochet { speed, radius, bounces } => {
                let mut p = Projectile::new(0, at + aim * 20.0, aim * speed, radius, damage, def.kind, false, Look::EyeRoll, life(speed));
                p.bounces = bounces;
                shots.push(p);
            }
            Pattern::Boomerang { speed, radius } => {
                let mut p = Projectile::new(0, at + aim * 20.0, aim * speed, radius, damage, def.kind, false, Look::Frisbee, 260);
                p.boomerang = true;
                p.pierce = 99;
                p.bounces = 1;
                shots.push(p);
            }
            Pattern::Stun { speed, radius, seconds } => {
                let mut p = Projectile::new(0, at + aim * 20.0, aim * speed, radius, damage, def.kind, false, Look::Captcha, life(speed));
                p.stun = (seconds * TICK_RATE as f32) as u16;
                shots.push(p);
            }
            Pattern::Arc { half_angle } | Pattern::Cone { half_angle } => {
                let cos = crate::math::cos(half_angle);
                let hits: Vec<usize> = self
                    .around(at, range)
                    .into_iter()
                    .filter(|&j| {
                        let d = self.slop[j].pos - at;
                        let len = d.length();
                        len < self.slop[j].radius + PLAYER_RADIUS || d.dot(aim) >= cos * len
                    })
                    .collect();
                for j in hits {
                    let push = (self.slop[j].pos - at).normalized() * def.knockback;
                    self.damage_slop(j, damage, def.kind, push, at, slot);
                }
                self.events.push(Event::Swing { at, angle: aim.angle(), range, half_angle, weapon: wi as u8 });
                return true;
            }
            Pattern::Chain { links } => {
                self.chain_hit(at, target, links, damage, def.kind, slot);
                return true;
            }
            Pattern::Beam { width } => {
                let end = at + aim * range;
                let hits: Vec<usize> = (0..self.slop.len())
                    .filter(|&j| {
                        let s = &self.slop[j];
                        let along = (s.pos - at).dot(aim).clamp(0.0, range);
                        s.hp > 0.0 && (at + aim * along).distance(s.pos) < width + s.radius
                    })
                    .collect();
                for j in hits {
                    self.slop[j].shield_off = SHIELD_OFF_TICKS;
                    self.damage_slop(j, damage, def.kind, V2::ZERO, at, slot);
                }
                self.events.push(Event::Beam { from: at, to: end });
                return true;
            }
            Pattern::Zone { radius, seconds, heal } => {
                let id = self.id();
                let ticks = (seconds * TICK_RATE as f32) as u16;
                self.zones.push(Zone { id, kind: ZoneKind::Seed { heal }, pos: goal, radius, ticks, max_ticks: ticks, vel: V2::ZERO, damage, spec: wi as u16 });
                return true;
            }
            Pattern::Slam { radius, execute } => {
                let hits = self.around(goal, radius);
                for j in hits {
                    let s = &self.slop[j];
                    let boss = matches!(self.setup.specs[s.spec as usize].movement, Movement::Boss(_));
                    let push = (s.pos - goal).normalized() * def.knockback;
                    if !boss && s.hp <= execute {
                        // Deleted outright: the full remaining HP counts as one hit.
                        let rest = s.hp / (1.0 - s.resist[def.kind as usize]).max(0.1);
                        self.damage_slop(j, rest.max(damage) * 4.0, def.kind, push, at, slot);
                    } else {
                        self.damage_slop(j, damage, def.kind, push, at, slot);
                    }
                }
                self.events.push(Event::Slam { at: goal, radius });
                return true;
            }
            Pattern::Orbit { .. } | Pattern::Ring | Pattern::Aura | Pattern::Reflect => unreachable!("handled above"),
        }
        for mut p in shots {
            p.id = self.id();
            p.knockback = def.knockback;
            p.weapon = wi as u8;
            self.projectiles.push(p);
        }
        self.events.push(Event::Fired { weapon: wi as u8 });
        true
    }

    pub(crate) fn move_projectiles(&mut self) {
        let player = self.player.pos;
        let mut hits = Vec::new();
        let mut on_you = Vec::new();
        let mut splashes = Vec::new();
        let noise = self.player.perks.noise_cancel;
        for i in 0..self.projectiles.len() {
            let p = &mut self.projectiles[i];
            if p.boomerang {
                let back = (player - p.pos).normalized();
                p.vel += back * (BOOMERANG_PULL * DT);
                if p.bounces == 1 && p.vel.dot(back) > 0.0 {
                    // Turned around: it may hit the same slop again on the way back.
                    p.bounces = 0;
                    p.last_hit = 0;
                }
                if p.bounces == 0 && p.pos.distance_sq(player) < 30.0 * 30.0 {
                    p.life = 0;
                    continue;
                }
            }
            p.pos += p.vel * DT;
            p.life = p.life.saturating_sub(1);
            if p.pos.x < -50.0 || p.pos.y < -50.0 || p.pos.x > ARENA + 50.0 || p.pos.y > ARENA + 50.0 {
                p.life = 0;
            }
            if p.splash > 0.0 {
                if p.life == 0 {
                    splashes.push((p.pos, p.splash, p.damage, p.kind, p.knockback, p.weapon));
                }
                continue;
            }
            if p.life == 0 {
                continue;
            }
            if p.hostile {
                let reach = p.radius + PLAYER_RADIUS * 0.8;
                if p.pos.distance_sq(player) < reach * reach {
                    if noise && p.look == Look::Bubble && self.player.noise_block == 0 {
                        self.player.noise_block = 180;
                        self.events.push(Event::Blocked);
                    } else {
                        on_you.push((p.damage, p.spec, p.shove, p.doomscroll, p.vel));
                    }
                    p.life = 0;
                }
                continue;
            }
            let (at, radius, last) = (p.pos, p.radius, p.last_hit);
            let mut hit = None;
            self.grid.for_each_near(at, radius + 40.0, |j| {
                let s = &self.slop[j];
                if hit.is_none() && s.hp > 0.0 && s.id != last && s.pos.distance_sq(at) < (s.radius + radius) * (s.radius + radius) {
                    hit = Some(j);
                }
            });
            if let Some(j) = hit {
                hits.push((i, j));
            }
        }
        for (i, j) in hits {
            let p = self.projectiles[i].clone();
            if self.slop[j].hp <= 0.0 {
                continue;
            }
            let weapon = (p.weapon != u8::MAX).then_some(p.weapon as usize);
            self.damage_slop(j, p.damage, p.kind, p.vel.normalized() * p.knockback, p.pos - p.vel * 0.05, weapon);
            let chatbot = self.setup.specs[self.slop[j].spec as usize].is_chatbot();
            let s = &mut self.slop[j];
            if let Some((bonus, ticks)) = p.mark {
                s.marked = ticks;
                s.mark_bonus = bonus;
            }
            if p.stun > 0 && chatbot {
                s.stunned = s.stunned.max(p.stun);
            }
            let id = s.id;
            let pr = &mut self.projectiles[i];
            pr.last_hit = id;
            if pr.boomerang {
                continue;
            }
            if pr.bounces > 0 {
                pr.bounces -= 1;
                let from = pr.pos;
                match nearest_slop(&self.grid, &self.slop, from, RICOCHET_RANGE, Some(j)) {
                    Some(k) => {
                        let to = self.slop[k].pos;
                        let pr = &mut self.projectiles[i];
                        let speed = pr.vel.length();
                        pr.vel = (to - from).normalized() * speed;
                        pr.life = pr.life.max(40);
                    }
                    None => self.projectiles[i].life = 0,
                }
            } else if pr.pierce > 0 {
                pr.pierce -= 1;
            } else {
                pr.life = 0;
            }
        }
        for (at, radius, damage, kind, knock, weapon) in splashes {
            let w = (weapon != u8::MAX).then_some(weapon as usize);
            for j in self.around(at, radius) {
                let push = (self.slop[j].pos - at).normalized() * knock;
                self.damage_slop(j, damage, kind, push, at, w);
            }
            self.events.push(Event::Splash { at, radius });
        }
        self.pop_bubbles();
        for (damage, spec, shove, doomscroll, vel) in on_you {
            if self.hurt(damage, spec) {
                if shove > 0.0 {
                    self.player.vel += vel.normalized() * shove;
                }
                if doomscroll {
                    self.apply_status(StatusKind::Doomscroll);
                }
            }
        }
        self.projectiles.retain(|p| p.life > 0);
    }

    /// Your projectiles pop speech bubbles they pass through (BUB-01).
    fn pop_bubbles(&mut self) {
        let mine: Vec<(V2, f32)> = self.projectiles.iter().filter(|p| !p.hostile && p.life > 0).map(|p| (p.pos, p.radius)).collect();
        if mine.is_empty() {
            return;
        }
        let mut dropped = Vec::new();
        for b in self.projectiles.iter_mut().filter(|p| p.hostile && p.look == Look::Bubble && p.life > 0) {
            if mine.iter().any(|(at, r)| at.distance_sq(b.pos) < (r + b.radius) * (r + b.radius)) {
                b.life = 0;
                self.tally.bubbles_popped += 1;
                self.events.push(Event::BubblePopped { at: b.pos, line: b.line });
                if self.rng_combat.below(5) == 0 {
                    dropped.push(b.pos);
                }
            }
        }
        for at in dropped {
            let id = self.id();
            self.likes.push(Like { id, pos: at, value: 1, pulled: false, fake: false });
        }
    }

    /// Seed patches, bombs, banners and the other zones (ZONE-01).
    pub(crate) fn tick_zones(&mut self) {
        let me = self.player.pos;
        let mut bombs = Vec::new();
        let mut seeds = Vec::new();
        for z in &mut self.zones {
            z.ticks = z.ticks.saturating_sub(1);
            z.pos += z.vel * DT;
            match z.kind {
                ZoneKind::Bomb if z.ticks == 0 => bombs.push((z.pos, z.radius, z.damage, z.spec)),
                ZoneKind::Seed { heal } if z.ticks % 30 == 0 => seeds.push((z.pos, z.radius, z.damage, heal, z.spec)),
                ZoneKind::Banner if z.pos.x < -200.0 || z.pos.y < -200.0 || z.pos.x > ARENA + 200.0 || z.pos.y > ARENA + 200.0 => z.ticks = 0,
                _ => {}
            }
        }
        self.zones.retain(|z| z.ticks > 0);
        for (at, radius, damage, heal, slot) in seeds {
            for j in self.around(at, radius) {
                self.damage_slop(j, damage, DamageType::Grass, V2::ZERO, at, Some(slot as usize));
            }
            if me.distance_sq(at) < radius * radius {
                let p = &mut self.player;
                p.mental_health = (p.mental_health + heal * 0.5).min(p.stats.max_mental_health);
            }
        }
        for (at, radius, damage, spec) in bombs {
            self.events.push(Event::Splash { at, radius });
            if me.distance_sq(at) < (radius + PLAYER_RADIUS) * (radius + PLAYER_RADIUS) {
                self.hurt(damage, spec);
            }
        }
    }
}
