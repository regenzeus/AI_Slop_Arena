//! What the slop does each tick: move by its movement, fire its attack,
//! use its traits, and touch you (SLOP-01 to SLOP-05).

use crate::content::{Archetype, Look, AGREE_LINES, BUBBLE_LINES, POST_LINES};
use crate::events::{Event, StatusKind};
use crate::spec::{Attack, Movement, SlopSpec, Traits};
use crate::stats::DamageType;
use crate::vec::V2;
use crate::world::*;

const KNOCKBACK_DECAY: f32 = 0.85;
const KITE_FAR: f32 = 420.0;
const KITE_NEAR: f32 = 300.0;
const LANCE_SPEED: f32 = 760.0;
const LANCE_RADIUS: f32 = 8.0;
const LANCE_RANGE: f32 = 440.0;
const LANCE_RECOVER: i32 = 140;
const BUBBLE_SPEED: f32 = 200.0;
const BUBBLE_LIFE: u16 = 200;
const BUBBLE_SPREAD: f32 = 0.28;
const FOLLOW_RANGE: f32 = 600.0;
const AGREE_RANGE: f32 = 220.0;
const AGREE_INTERVAL: i32 = 180;
const AGREE_TICKS: u16 = 300;
const AGREE_MAX: u8 = 3;
const HEAL_RANGE: f32 = 160.0;
const HEAL_INTERVAL: i32 = 120;
const BAIT_RANGE: f32 = 220.0;
const MAX_GRABS: u8 = 3;
const GRAB_DAMAGE: f32 = 1.5;
/// Ticks a hand holds on before it lets go and backs off.
const GRAB_HOLD: u16 = 180;
const GRAB_INTERVAL: i32 = 30;
const SPIN_REACH: f32 = 55.0;
const SPEED_AURA: f32 = 1.35;
const BEAT_BOOST: f32 = 1.25;
/// Ticks after a beat during which brainrot moves faster.
const BEAT_TICKS: u16 = 12;
const FAKE_LIKE_TICKS: i32 = 240;
const HATCHLINGS: u8 = 3;

/// The parts of a spec the AI needs, copied so the slop can be borrowed mutably.
#[derive(Clone, Copy)]
struct Brief {
    movement: Movement,
    attack: Option<Attack>,
    traits: Traits,
    speed: f32,
    radius: f32,
    contact: f32,
    brainrot: bool,
}

impl Brief {
    fn of(sp: &SlopSpec) -> Brief {
        Brief {
            movement: sp.movement,
            attack: sp.attack,
            traits: sp.traits,
            speed: sp.speed,
            radius: sp.radius,
            contact: sp.contact,
            brainrot: sp.is_brainrot(),
        }
    }
}

impl World {
    /// Each slop decides where to move and whether to attack.
    pub(crate) fn think(&mut self) {
        let player = self.player.pos;
        let briefs: Vec<Brief> = self.setup.specs.iter().map(Brief::of).collect();
        // Espresso brainrot speed up the brainrot around them.
        let hasted: Vec<bool> = self
            .slop
            .iter()
            .map(|s| {
                let mut near = false;
                self.grid.for_each_near(s.pos, 150.0, |j| {
                    let o = &self.slop[j];
                    near |= briefs[o.spec as usize].traits.has(Traits::SPEED_AURA) && o.pos.distance_sq(s.pos) < 150.0 * 150.0;
                });
                near && briefs[s.spec as usize].brainrot
            })
            .collect();
        let on_beat = self.beat < BEAT_TICKS && self.setup.platform == Some(crate::content::Platform::ClockTok);
        self.crowd = self.slop.iter().filter(|s| s.pos.distance_sq(player) < SHOOT_RANGE * SHOOT_RANGE).count() as u32;
        let kills = self.tally.kills;
        let grow = kills.saturating_sub(self.karma_seen).min(3) as f32;
        self.karma_seen = kills;

        let mut shots: Vec<(V2, V2, f32, Look, u16, usize)> = Vec::new();
        let mut agree = Vec::new();
        let mut queued: Vec<Queued> = Vec::new();
        let mut fakes = Vec::new();
        let mut spin_hits = Vec::new();
        for i in 0..self.slop.len() {
            let b = briefs[self.slop[i].spec as usize];
            let pos = self.slop[i].pos;
            // Followers look around first, before anything is borrowed mutably.
            let mut friend: Option<V2> = None;
            let mut nearby: Vec<usize> = Vec::new();
            if b.movement == Movement::Follow || b.traits.has(Traits::BUFF_AURA) || b.traits.has(Traits::HEAL_BOSS) {
                let range = if b.traits.has(Traits::BUFF_AURA) { BAIT_RANGE } else { AGREE_RANGE };
                let mut best = f32::MAX;
                self.grid.for_each_near(pos, FOLLOW_RANGE, |j| {
                    let o = &self.slop[j];
                    let ob = briefs[o.spec as usize];
                    if ob.movement == Movement::Follow || j == i {
                        return;
                    }
                    let d = o.pos.distance_sq(pos);
                    if d < best && d < FOLLOW_RANGE * FOLLOW_RANGE {
                        best = d;
                        friend = Some(o.pos);
                    }
                    if d < range * range && nearby.len() < 6 {
                        nearby.push(j);
                    }
                });
            }

            let s = &mut self.slop[i];
            if s.stunned > 0 {
                s.moving = V2::ZERO;
                s.pos += s.push * DT;
                s.push *= KNOCKBACK_DECAY;
                continue;
            }
            let to = player - s.pos;
            let dist = to.length();
            let dir = to.normalized();
            let side = if s.id % 2 == 0 { 1.0 } else { -1.0 };
            let mut speed = b.speed * self.hustle;
            if hasted[i] {
                speed *= SPEED_AURA;
            }
            if on_beat && b.brainrot {
                speed *= BEAT_BOOST;
            }
            if b.traits.has(Traits::GROWS) && grow > 0.0 && s.grown < 2.0 {
                s.grown = (s.grown + 0.05 * grow).min(2.0);
                s.max_hp += 3.0 * grow;
                s.hp += 3.0 * grow;
                s.radius = b.radius * s.grown.sqrt();
            }
            let mut want = V2::ZERO;
            match b.movement {
                Movement::Rush | Movement::Pack => {
                    want = if s.fleeing > 0 { -dir * (speed * 1.3) } else { dir * speed };
                }
                Movement::Hop => {
                    let phase = (self.tick + s.id * 7) % 60;
                    want = dir * if phase < 14 { speed * 3.0 } else { speed * 0.25 };
                }
                Movement::Kite => {
                    want = if dist > KITE_FAR {
                        dir * speed
                    } else if dist < KITE_NEAR {
                        -dir * speed
                    } else {
                        dir.perp() * (side * speed * 0.5)
                    };
                }
                Movement::Lance => {
                    if s.charging > 0 {
                        s.charging -= 1;
                        if s.charging == 0 {
                            shots.push((s.pos + s.aim * b.radius, s.aim * LANCE_SPEED, b.contact * 2.0 * s.agreement(), Look::EmDash, NO_LINE, s.spec as usize));
                            s.timer = LANCE_RECOVER;
                        }
                    } else if s.timer > 0 {
                        s.timer -= 1;
                        want = dir.perp() * (side * speed * 0.6);
                    } else if dist < LANCE_RANGE {
                        s.charging = LANCE_AIM;
                        s.aim = dir;
                        self.events.push(Event::LancerAim { from: s.pos, dir });
                    } else {
                        want = dir * speed;
                    }
                }
                Movement::Follow => {
                    want = match friend {
                        Some(f) if f.distance(s.pos) > 70.0 => (f - s.pos).normalized() * speed,
                        Some(_) => V2::ZERO,
                        None if dist > 260.0 => dir * speed,
                        None => V2::ZERO,
                    };
                    s.timer -= 1;
                    if s.timer <= 0 {
                        if b.traits.has(Traits::AGREE) {
                            s.timer = AGREE_INTERVAL;
                            if !nearby.is_empty() {
                                let line = self.rng_ai.below(AGREE_LINES.len() as u32) as u16;
                                self.events.push(Event::Agreed { at: s.pos, line });
                                agree.extend(nearby.iter().take(3).copied());
                            }
                        } else if b.traits.has(Traits::HEALER) {
                            s.timer = HEAL_INTERVAL;
                            queued.push(Queued::Heal { at: s.pos });
                        }
                    }
                }
                Movement::Grab => {
                    if let Some(offset) = s.grab {
                        s.pos = player + offset;
                        s.moving = V2::ZERO;
                        continue;
                    }
                    want = dir * speed;
                }
                Movement::Stand => {
                    s.timer -= 1;
                    if s.timer <= 0 {
                        if b.traits.has(Traits::HATCH) {
                            for n in 1..=HATCHLINGS {
                                let at = s.pos + V2::from_angle(n as f32 * 2.1) * 26.0;
                                queued.push(Queued::Hatch { at, pack: s.id, number: n });
                            }
                            s.hp = 0.0;
                            self.events.push(Event::Hatched { at: s.pos });
                        } else if b.traits.has(Traits::BUFF_AURA) {
                            s.timer = HEAL_INTERVAL;
                            self.events.push(Event::Agreed { at: s.pos, line: NO_LINE });
                            agree.extend(nearby.iter().copied());
                        } else if b.traits.has(Traits::HEAL_BOSS) {
                            s.timer = HEAL_INTERVAL;
                            self.events.push(Event::Agreed { at: s.pos, line: NO_LINE });
                            queued.push(Queued::HealBoss);
                        } else {
                            s.timer = HEAL_INTERVAL;
                        }
                    }
                }
                Movement::Boss(_) => continue,
            }
            if b.traits.has(Traits::FAKE_LIKES) {
                s.timer -= 1;
                if s.timer <= 0 {
                    s.timer = FAKE_LIKE_TICKS;
                    fakes.push(s.pos);
                }
            }
            if b.traits.has(Traits::SPIN_AURA) && dist < b.radius + PLAYER_RADIUS + SPIN_REACH && self.tick % 30 == 0 {
                spin_hits.push((b.contact * 0.6, s.spec));
            }
            // Damage-type effects (DMG-02).
            if s.staggered > 0 {
                want = V2::ZERO;
            }
            if s.rooted > 0 {
                want *= 0.35;
            }
            if s.confused > 0 {
                want = -want;
            }
            if b.traits.has(Traits::TOXIC) && (self.tick + s.id) % 80 == 0 {
                queued.push(Queued::Toxic { at: s.pos });
            }
            if let Some(attack) = b.attack.filter(|_| s.confused == 0) {
                s.attack_timer -= 1;
                if s.attack_timer <= 0 && dist < SHOOT_RANGE {
                    s.attack_timer = attack.interval() + self.rng_ai.below(40) as i32;
                    let k = s.agreement();
                    let spec = s.spec as usize;
                    match attack {
                        Attack::Fan => {
                            for n in -1..=1 {
                                let aim = dir.rotated(n as f32 * BUBBLE_SPREAD);
                                // Only the middle bubble talks, so crowds stay readable.
                                let line = if n == 0 { self.rng_ai.below(BUBBLE_LINES.len() as u32) as u16 } else { NO_LINE };
                                shots.push((s.pos + aim * b.radius, aim * BUBBLE_SPEED, b.contact * 1.2 * k, Look::Bubble, line, spec));
                            }
                        }
                        Attack::Toast => shots.push((s.pos + dir * b.radius, dir * 320.0, b.contact * k, Look::Toast, NO_LINE, spec)),
                        Attack::Arrow => shots.push((s.pos + dir * b.radius, dir * 380.0, b.contact * 1.6 * k, Look::Arrow, NO_LINE, spec)),
                        Attack::Wall => {
                            for n in -2..=2 {
                                let at = s.pos + dir * b.radius + dir.perp() * (n as f32 * 34.0);
                                let line = if n == 0 { self.rng_ai.below(POST_LINES.len() as u32) as u16 } else { NO_LINE };
                                shots.push((at, dir * 160.0, b.contact * 0.8 * k, Look::Post, line, spec));
                            }
                        }
                        Attack::Bomb => {
                            let at = player + self.player.vel * 0.5;
                            self.events.push(Event::BombWarning { at, radius: 60.0 });
                            queued.push(Queued::Bomb { at, damage: b.contact * 1.6 * k, spec: spec as u16 });
                        }
                    }
                }
            }
            s.moving = want;
            if b.traits.has(Traits::NO_KNOCKBACK) {
                s.push = V2::ZERO;
            }
            if b.movement != Movement::Stand {
                s.pos += (want + s.push) * DT;
            }
            s.push *= KNOCKBACK_DECAY;
        }

        for j in agree {
            let s = &mut self.slop[j];
            s.agreed = (s.agreed + 1).min(AGREE_MAX);
            s.agreed_ticks = AGREE_TICKS;
        }
        for q in queued {
            match q {
                Queued::Heal { at } => {
                    for s in self.slop.iter_mut().filter(|s| s.pos.distance_sq(at) < HEAL_RANGE * HEAL_RANGE) {
                        s.hp = (s.hp + s.max_hp * 0.08).min(s.max_hp);
                    }
                    self.events.push(Event::Healed { at });
                }
                Queued::HealBoss => {
                    let Some(id) = self.boss.as_ref().map(|b| b.id) else { continue };
                    if let Some(boss) = self.slop.iter_mut().find(|s| s.id == id) {
                        boss.hp = (boss.hp + boss.max_hp * 0.008).min(boss.max_hp);
                        self.events.push(Event::Healed { at: boss.pos });
                    }
                }
                Queued::Hatch { at, pack, number } => {
                    let drone = self.ensure_spec(SlopSpec::archetype(Archetype::ListicleDrone));
                    self.pending.push((drone, at, 0.0, pack, number));
                }
                Queued::Bomb { at, damage, spec } => self.drop_bomb(at, damage, spec),
                Queued::Toxic { at } => {
                    let id = self.id();
                    self.zones.push(Zone::new(id, ZoneKind::Downvote, at, 46.0, 300));
                }
            }
        }
        for at in fakes {
            let id = self.id();
            self.likes.push(Like { id, pos: at, value: 1, pulled: false, fake: true });
        }
        let bubbles = self.projectiles.iter().filter(|p| p.look == Look::Bubble).count();
        for (pos, vel, damage, look, line, spec) in shots {
            if look == Look::Bubble && bubbles >= MAX_BUBBLES {
                continue;
            }
            self.hostile_shot(pos, vel, damage, look, line, spec as u16);
        }
        for (damage, spec) in spin_hits {
            self.hurt(damage, spec);
        }
    }

    /// Fires a projectile at you.
    pub(crate) fn hostile_shot(&mut self, pos: V2, vel: V2, damage: f32, look: Look, line: u16, spec: u16) {
        let id = self.id();
        let (radius, life) = match look {
            Look::Bubble => (BUBBLE_RADIUS, BUBBLE_LIFE),
            Look::EmDash => (LANCE_RADIUS, 90),
            Look::Arrow => (12.0, 150),
            Look::Post => (16.0, 240),
            Look::Letter => (18.0, 900),
            _ => (10.0, 180),
        };
        let mut p = Projectile::new(id, pos, vel, radius, damage, DamageType::Analog, true, look, life);
        p.line = line;
        p.spec = spec;
        if look == Look::Post {
            p.shove = 420.0;
        }
        if look == Look::Arrow && self.setup.specs.get(spec as usize).is_some_and(|s| s.traits.has(Traits::DOOMSCROLL)) {
            p.doomscroll = true;
        }
        self.projectiles.push(p);
    }

    /// Pushes overlapping slop apart so crowds spread out instead of stacking.
    pub(crate) fn separate(&mut self) {
        let mut shove = vec![V2::ZERO; self.slop.len()];
        let specs = &self.setup.specs;
        let fixed = |s: &Slop| {
            let sp = &specs[s.spec as usize];
            s.grab.is_some() || sp.traits.has(Traits::FLIES) || matches!(sp.movement, Movement::Stand | Movement::Boss(_))
        };
        for (i, s) in self.slop.iter().enumerate() {
            if fixed(s) {
                continue;
            }
            self.grid.for_each_near(s.pos, s.radius * 2.0 + 30.0, |j| {
                if j == i {
                    return;
                }
                let o = &self.slop[j];
                if specs[o.spec as usize].traits.has(Traits::FLIES) {
                    return;
                }
                let gap = s.radius + o.radius;
                let d = s.pos - o.pos;
                let len = d.length();
                if len < gap {
                    let away = if len > 1e-3 { d * (1.0 / len) } else { V2::from_angle(s.id as f32) };
                    shove[i] += away * ((gap - len) * 0.5);
                }
            });
        }
        for (s, push) in self.slop.iter_mut().zip(shove) {
            if push.length_sq() > 0.0 {
                s.pos += push.clamp_length(6.0);
            }
        }
    }

    /// Slop touching you: contact damage, statuses, and hands that grab (SLOP-02).
    pub(crate) fn touch(&mut self) {
        let p = self.player.pos;
        let mut holding = self.slop.iter().filter(|s| s.grab.is_some()).count() as u8;
        let mut contact = Vec::new();
        let mut grab_hits = 0;
        for (i, s) in self.slop.iter_mut().enumerate() {
            let sp = &self.setup.specs[s.spec as usize];
            if s.stunned > 0 || s.confused > 0 {
                continue;
            }
            let reach = s.radius + PLAYER_RADIUS + if sp.traits.has(Traits::LONG_REACH) { 22.0 } else { 0.0 };
            let touching = s.pos.distance_sq(p) < reach * reach;
            if sp.movement == Movement::Grab {
                if s.grab.is_some() {
                    s.timer -= 1;
                    if s.timer <= 0 {
                        s.timer = GRAB_INTERVAL;
                        grab_hits += 1;
                    }
                    s.charging = s.charging.saturating_sub(1);
                    if s.charging == 0 {
                        s.grab = None;
                        s.fleeing = 90;
                        holding = holding.saturating_sub(1);
                    }
                } else if touching && holding < MAX_GRABS && self.player.dash == 0 && s.fleeing == 0 {
                    s.grab = Some((s.pos - p).clamp_length(PLAYER_RADIUS + 6.0));
                    s.timer = GRAB_INTERVAL;
                    s.charging = GRAB_HOLD;
                    holding += 1;
                    self.events.push(Event::Grabbed);
                }
                continue;
            }
            if touching && sp.contact > 0.0 {
                contact.push((sp.contact * s.agreement(), s.spec, sp.traits, i));
            }
        }
        self.player.grabbed = holding;
        for (dmg, spec, traits, i) in contact {
            if self.hurt(dmg, spec) {
                if traits.has(Traits::POPUP) {
                    self.apply_status(StatusKind::Popup);
                }
                if traits.has(Traits::RATIO) {
                    self.apply_status(StatusKind::Ratio);
                }
                if traits.has(Traits::HEAL_ON_HIT) {
                    if let Some(s) = self.slop.get_mut(i) {
                        s.hp = (s.hp + s.max_hp * 0.2).min(s.max_hp);
                    }
                }
            }
        }
        for _ in 0..grab_hits {
            self.lose(GRAB_DAMAGE);
        }
    }

    /// Marks a spot where a bomb lands after `BOMB_WARNING` ticks.
    pub(crate) fn drop_bomb(&mut self, at: V2, damage: f32, spec: u16) {
        let id = self.id();
        self.events.push(Event::BombWarning { at, radius: 60.0 });
        let mut z = Zone::new(id, ZoneKind::Bomb, at, 60.0, BOMB_WARNING);
        z.damage = damage;
        z.spec = spec;
        self.zones.push(z);
    }
}

/// Work the AI queues until every slop has thought.
enum Queued {
    /// A cake heals the slop around it.
    Heal { at: V2 },
    /// A totem heals its boss.
    HealBoss,
    /// A Surprise Egg hatches a drone.
    Hatch { at: V2, pack: u32, number: u8 },
    Bomb { at: V2, damage: f32, spec: u16 },
    /// A Toxic elite leaves a slowing puddle.
    Toxic { at: V2 },
}
