//! Your weapons, every projectile, damage to slop, and zones on the floor
//! (WPN-01 to WPN-08, DMG-01, BUB-01).

use crate::content::{level_extra, Look, Pattern, LEVEL_DAMAGE, WEAPONS};
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
/// Damage-type effects (DMG-02): ticks of Analog stagger, Sarcasm confusion
/// and Grass roots, and the chance of confusion.
const STAGGER_TICKS: u16 = 10;
const CONFUSE_TICKS: u16 = 120;
const CONFUSE_CHANCE: f32 = 0.12;
const ROOT_TICKS: u16 = 75;
/// Verified elites take this share of non-Fact damage.
const VERIFIED: f32 = 0.5;
/// Radians per second a barrier turns around you.
const BARRIER_SPEED: f32 = 1.6;
/// How far your Alt Account stands from you.
const ALT_DISTANCE: f32 = 120.0;
/// Ticks before Echo Chamber repeats an attack.
const ECHO_TICKS: u16 = 18;
/// Fire burns slop inside this often.
const FIRE_EVERY: u16 = 12;
/// Ticks of flipped controls when a brainrot zone drops on you.
const BRAINROT_TICKS: u16 = 90;

impl World {
    /// Applies a hit to one slop (DMG-01): Conviction, the type bonus, a
    /// mark, its resistance, filters and shields all multiply the damage.
    pub(crate) fn damage_slop(&mut self, j: usize, base: f32, kind: DamageType, push: V2, from: V2, weapon: Option<usize>) {
        let stats = self.player.stats;
        let ratio = self.player.status.ratio > 0;
        let perks = &self.player.perks;
        let fact = kind == DamageType::Fact;
        // Fact cuts through filters and shields.
        let filtered = !fact && self.zones.iter().any(|z| z.kind == ZoneKind::Filter && z.contains(self.slop[j].pos, 0.0));
        let sets = perks.type_count;
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
        if self.player.god > 0 {
            mult *= 1.5;
        }
        if perks.speed_damage {
            mult *= stats.speed.max(1.0);
        }
        if filtered {
            mult *= FILTERED;
        }
        if sp.traits.has(Traits::VERIFIED) && !fact {
            mult *= VERIFIED;
        }
        if sp.traits.has(Traits::FRONT_SHIELD) && s.shield_off == 0 && !fact {
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
        let steady = sp.traits.has(Traits::NO_KNOCKBACK);
        // What each damage type does besides damage (DMG-02); four weapons of
        // a type make its effect stronger (DMG-03).
        let strong = sets[k] >= 4;
        match kind {
            DamageType::Analog if !steady => s.staggered = s.staggered.max(if strong { STAGGER_TICKS * 2 } else { STAGGER_TICKS }),
            DamageType::Sarcasm => {
                let chance = if strong { CONFUSE_CHANCE * 2.0 } else { CONFUSE_CHANCE };
                if s.confused == 0 && !steady && self.rng_combat.chance(chance) {
                    s.confused = CONFUSE_TICKS;
                    self.events.push(Event::Confused { at: s.pos });
                }
            }
            DamageType::Grass => s.rooted = s.rooted.max(if strong { ROOT_TICKS * 2 } else { ROOT_TICKS }),
            DamageType::Fact => s.shield_off = s.shield_off.max(if strong { SHIELD_OFF_TICKS } else { 30 }),
            _ => {}
        }
        let s = &mut self.slop[j];
        if s.grab.is_none() && !steady {
            let shove = match (kind, strong) {
                (DamageType::Analog, true) => 1.8,
                (DamageType::Analog, false) => 1.4,
                _ => 1.0,
            };
            s.push += push * shove;
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

    /// Where your Alt Account stands: circling you, a little way off.
    pub fn alt_position(&self) -> V2 {
        self.player.pos + V2::from_angle(self.tick as f32 * 0.015) * ALT_DISTANCE
    }

    /// The segments of barrier weapons around you: centre and radius (DEF-02).
    pub fn barrier_segments(&self) -> Vec<(V2, f32, usize)> {
        let mut out = Vec::new();
        for (wi, w) in self.player.weapons.iter().enumerate() {
            if let Pattern::Barrier { count, radius, size } = w.def().pattern {
                let (shots, area, _) = level_extra(w.level);
                let n = count + if w.def().evolved { 0 } else { shots };
                let r = radius * (1.0 + self.player.stats.reach) * area;
                for k in 0..n {
                    let a = w.phase + k as f32 * std::f32::consts::TAU / n as f32;
                    out.push((self.player.pos + V2::from_angle(a) * r, size * area, wi));
                }
            }
        }
        out
    }

    /// Every weapon whose cooldown is up attacks (WPN-01).
    pub(crate) fn fire_weapons(&mut self) {
        let stats = self.player.stats;
        let at = self.player.pos;
        // Echo Chamber: attacks come back a moment later at half damage.
        let mut due = Vec::new();
        self.player.echoes.retain_mut(|e| {
            e.0 = e.0.saturating_sub(1);
            if e.0 == 0 {
                due.push(e.1);
                false
            } else {
                true
            }
        });
        for wi in due {
            if let Some(w) = self.player.weapons.get(wi) {
                let (def_index, level) = (w.def, w.level);
                let def = &WEAPONS[def_index];
                let (_, area, _) = level_extra(level);
                let range = def.range * (1.0 + stats.reach) * area;
                let damage = def.damage * LEVEL_DAMAGE[level as usize - 1] * 0.5;
                self.attack(wi, def_index, damage, range, level, at);
            }
        }
        for wi in 0..self.player.weapons.len() {
            let (def_index, level, enabled) = {
                let w = &self.player.weapons[wi];
                (w.def, w.level, w.enabled)
            };
            if !enabled {
                continue;
            }
            let def = &WEAPONS[def_index];
            let (_, area, haste) = level_extra(level);
            let range = def.range * (1.0 + stats.reach) * area;
            let damage = def.damage * LEVEL_DAMAGE[level as usize - 1];
            let mut rate = (1.0 + stats.focus) / haste;
            if matches!(def.pattern, Pattern::Aura) {
                let crowd = self.around(at, range).len().min(10);
                rate *= 1.0 + 0.1 * crowd as f32;
            }
            match def.pattern {
                Pattern::Orbit { .. } => self.player.weapons[wi].phase += ORBIT_SPEED * DT * (1.0 + stats.focus),
                Pattern::Barrier { .. } => self.player.weapons[wi].phase += BARRIER_SPEED * DT,
                _ => {}
            }
            self.player.weapons[wi].cooldown -= rate;
            if self.player.weapons[wi].cooldown > 0.0 {
                continue;
            }
            let fired = self.attack(wi, def_index, damage, range, level, at);
            if let Some(to) = fired {
                self.events.push(Event::Attacked { weapon: wi as u8, to });
                let perks = &self.player.perks;
                // The Alt Account fires too, from where it stands.
                if perks.alt_account > 0.0 && !matches!(def.pattern, Pattern::Orbit { .. } | Pattern::Barrier { .. }) {
                    let share = perks.alt_account;
                    let alt = self.alt_position();
                    self.attack(wi, def_index, damage * share, range, level, alt);
                }
                if self.player.perks.echo && !matches!(def.pattern, Pattern::Orbit { .. } | Pattern::Barrier { .. }) {
                    self.player.echoes.push((ECHO_TICKS, wi));
                }
            }
            let w = &mut self.player.weapons[wi];
            if fired.is_some() {
                w.cooldown += def.cooldown * TICK_RATE as f32;
            } else {
                // Ready, waiting for something to hit.
                w.cooldown = 0.0;
            }
        }
    }

    /// One attack of weapon slot `wi`, returning where it was aimed; None when
    /// there was nothing to attack.
    fn attack(&mut self, wi: usize, def_index: usize, damage: f32, range: f32, level: u8, at: V2) -> Option<V2> {
        let def = &WEAPONS[def_index];
        // Merged weapons are built big already: no extra copies from levels.
        let extra = if def.evolved { 0 } else { level_extra(level).0 } + self.player.perks.extra_shots;
        let slot = Some(wi);
        match def.pattern {
            Pattern::Barrier { .. } => {
                let mut any = None;
                for (c, size, w) in self.barrier_segments() {
                    if w != wi {
                        continue;
                    }
                    for j in self.around(c, size) {
                        let push = (self.slop[j].pos - self.player.pos).normalized() * def.knockback;
                        self.damage_slop(j, damage, def.kind, push, c, slot);
                        any = Some(c);
                    }
                }
                return any;
            }
            Pattern::Orbit { radius } => {
                let phase = self.player.weapons[wi].phase;
                let dog = at + V2::from_angle(phase) * radius;
                let hits = self.around(dog, ORBIT_REACH);
                for j in &hits {
                    let push = (self.slop[*j].pos - at).normalized() * def.knockback;
                    self.damage_slop(*j, damage, def.kind, push, dog, slot);
                }
                return Some(dog);
            }
            Pattern::Ring | Pattern::Aura => {
                let hits = self.around(at, range);
                if hits.is_empty() {
                    return None;
                }
                for j in hits {
                    let push = (self.slop[j].pos - at).normalized() * def.knockback;
                    self.damage_slop(j, damage, def.kind, push, at, slot);
                }
                self.events.push(Event::Ring { at, radius: range });
                return Some(at);
            }
            Pattern::Reflect => {
                let bubble = self
                    .projectiles
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| p.hostile && p.life > 0 && p.pos.distance_sq(at) < range * range)
                    .min_by(|a, b| a.1.pos.distance_sq(at).total_cmp(&b.1.pos.distance_sq(at)))
                    .map(|(i, _)| i);
                let b = bubble?;
                let from = self.projectiles[b].pos;
                let t = nearest_slop(&self.grid, &self.slop, from, 600.0, None)?;
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
                return Some(from);
            }
            _ => {}
        }
        let target = nearest_slop(&self.grid, &self.slop, at, range, None)?;
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
                return Some(goal);
            }
            Pattern::Chain { links } => {
                self.chain_hit(at, target, links, damage, def.kind, slot);
                return Some(goal);
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
                return Some(goal);
            }
            Pattern::Zone { radius, seconds, heal } => {
                let id = self.id();
                let ticks = (seconds * TICK_RATE as f32) as u16;
                let mut z = Zone::new(id, ZoneKind::Seed { heal }, goal, radius, ticks);
                z.damage = damage;
                z.spec = wi as u16;
                self.zones.push(z);
                return Some(goal);
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
                return Some(goal);
            }
            Pattern::Orbit { .. } | Pattern::Ring | Pattern::Aura | Pattern::Reflect | Pattern::Barrier { .. } => unreachable!("handled above"),
        }
        // Higher levels throw more of the same, fanned out (WPN-10).
        if extra > 0 && !shots.is_empty() {
            let base = shots.clone();
            for n in 1..=extra {
                let off = if n % 2 == 1 { 0.22 } else { -0.22 } * n.div_ceil(2) as f32;
                for p in &base {
                    let mut q = p.clone();
                    q.vel = p.vel.rotated(off);
                    q.pos = at + (q.pos - at).rotated(off);
                    shots.push(q);
                }
            }
        }
        let bounces = self.player.perks.extra_bounces;
        for mut p in shots {
            p.id = self.id();
            p.knockback = def.knockback;
            p.weapon = wi as u8;
            if !p.boomerang && p.splash == 0.0 {
                p.bounces += bounces;
            }
            self.projectiles.push(p);
        }
        self.events.push(Event::Fired { weapon: wi as u8 });
        Some(goal)
    }

    pub(crate) fn move_projectiles(&mut self) {
        let player = self.player.pos;
        let mut hits = Vec::new();
        let mut on_you = Vec::new();
        let mut splashes = Vec::new();
        let mut deflected = Vec::new();
        let noise = self.player.perks.noise_cancel;
        let barriers = self.barrier_segments();
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
            if p.pos.distance_sq(player) > SHOT_RANGE_MAX * SHOT_RANGE_MAX {
                p.life = 0;
            }
            // Walls stop shots; lobbed ones fly over (MAP-02).
            if p.splash == 0.0 && p.life > 0 && self.map.walls_near(p.pos, p.radius).any(|w| w.contains(p.pos)) {
                p.life = 0;
                continue;
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
                // Barriers stop enemy shots (DEF-02).
                if barriers.iter().any(|(c, size, _)| c.distance_sq(p.pos) < (size + p.radius) * (size + p.radius)) {
                    p.life = 0;
                    deflected.push(p.pos);
                    continue;
                }
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
            // Ratio Machine: a hit may jump on to three more slop.
            let chance = self.player.perks.chain_chance;
            if chance > 0.0 && self.rng_combat.chance(chance) {
                self.chain_hit(p.pos, j, 3, p.damage * 0.6, DamageType::Sarcasm, weapon);
            }
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
        for at in deflected {
            self.events.push(Event::Deflected { at });
        }
        let armor = 1.0 - self.player.perks.shot_armor.clamp(0.0, 0.8);
        for (damage, spec, shove, doomscroll, vel) in on_you {
            if self.hurt(damage * armor, spec) {
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
        let mut fires = Vec::new();
        let mut flipped = false;
        for z in &mut self.zones {
            z.ticks = z.ticks.saturating_sub(1);
            z.pos += z.vel * DT;
            match z.kind {
                ZoneKind::Bomb if z.ticks == 0 => bombs.push((z.pos, z.radius, z.damage, z.spec)),
                ZoneKind::Seed { heal } if z.ticks % 30 == 0 => seeds.push((z.pos, z.radius, z.damage, heal, z.spec)),
                ZoneKind::Fire if z.ticks % FIRE_EVERY == 0 => fires.push((z.pos, z.radius, z.damage)),
                // The drop: inside a brainrot zone, your controls flip.
                ZoneKind::Brainrot if z.ticks == 0 && z.contains(me, PLAYER_RADIUS * 0.5) => flipped = true,
                ZoneKind::Banner if z.pos.distance_sq(me) > 2000.0 * 2000.0 => z.ticks = 0,
                _ => {}
            }
        }
        self.zones.retain(|z| z.ticks > 0);
        for (at, radius, damage) in fires {
            for j in self.around(at, radius) {
                self.damage_slop(j, damage, DamageType::Sarcasm, V2::ZERO, at, None);
            }
        }
        if flipped && !self.player.perks.fluent && self.player.status.brainrot == 0 {
            self.player.status.brainrot = BRAINROT_TICKS;
            self.events.push(Event::Status { kind: StatusKind::Brainrot });
        }
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
            // Brr Brr Patapum's stomp leaves roots behind.
            if self.setup.specs.get(spec as usize).is_some_and(|s| s.attack == Some(crate::spec::Attack::Stomp)) {
                let id = self.id();
                self.zones.push(Zone::new(id, ZoneKind::Roots, at, radius, 240));
            }
        }
    }
}
