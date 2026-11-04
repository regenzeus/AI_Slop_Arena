//! The seven bosses (BOSS-01 to BOSS-07). Each poses one clear puzzle and
//! has two phases; the second starts at half health.

use crate::content::{Accessory, Archetype, Body, BossKind, Look, Object, AMEN, BUBBLE_LINES};
use crate::events::{Event, StatusKind};
use crate::spec::{Attack, Movement, SlopLook, SlopSpec, Traits};
use crate::vec::V2;
use crate::world::*;

const RING_SPEED: f32 = 170.0;
const HUSTLE_TICKS: u32 = 1200;
const HUSTLE_STEP: f32 = 1.06;
const DASH_AIM: i32 = 40;
const DASH_TICKS: u16 = 24;
const DASH_SPEED: f32 = 900.0;
const AD_TICKS: u16 = 300;
const MODEL_CYCLE: u32 = 600;
const MODEL_THINK: u32 = 150;
const MODEL_ANSWER: u32 = 390;
/// Your Replacement closes a ticket near it this often, and heals this much.
const CLOSE_EVERY: u32 = 90;
const CLOSE_RANGE: f32 = 340.0;
const CLOSE_HEAL: f32 = 0.03;
/// The Board's seats, and how many make a quorum.
const BOARD_SEATS: u32 = 5;
const QUORUM: usize = 3;
/// A promise lands this far ahead of you, this big, after this long.
const PROMISE_LEAD: f32 = 1.1;
const PROMISE_RADIUS: f32 = 95.0;
const PROMISE_WARNING: u16 = 110;

impl World {
    /// Places the boss of a boss wave and whatever it starts with.
    pub(crate) fn start_boss(&mut self, spec: usize) {
        let Movement::Boss(kind) = self.setup.specs[spec].movement else { return };
        let me = self.player.pos;
        let at = match kind {
            BossKind::ContentFarm | BossKind::TheBoard => self.map.resolve(me + V2::new(0.0, 600.0), 80.0),
            _ => self.map.resolve(me + V2::new(0.0, 520.0), 80.0),
        };
        if kind == BossKind::Influencer {
            self.setup.specs[spec].traits.0 |= Traits::FRONT_SHIELD;
        }
        let id = self.place(spec, at, 0, 1);
        let minion = match kind {
            BossKind::CrustaceanSaviour | BossKind::SupremeBrainrot => {
                let parts = self.setup.model.parts.filter(|_| kind == BossKind::SupremeBrainrot);
                let (body, object, accessory) = parts.unwrap_or((Body::Shrimp, Object::Baguette, Accessory::Headphones));
                let name = if kind == BossKind::CrustaceanSaviour { "Shrimp Disciple".to_string() } else { "Brainrot Fan".to_string() };
                self.ensure_spec(SlopSpec::brainrot(name, body, object, accessory, 1.0, 1.0, [0.0; 4]))
            }
            BossKind::HustleGuru => self.ensure_spec(SlopSpec::archetype(Archetype::ThoughtLeader)),
            BossKind::ContentFarm => self.ensure_spec(SlopSpec::archetype(Archetype::SurpriseEgg)),
            BossKind::RepostHydra => {
                let mut repost = SlopSpec::archetype(Archetype::RepostBot);
                repost.name = "Repost of a Repost".into();
                repost.traits.0 &= !Traits::RESPAWN;
                self.ensure_spec(repost)
            }
            BossKind::Influencer => {
                let (_, radius, speed) = kind.base();
                let clone = SlopSpec {
                    name: "Not the Influencer".into(),
                    look: SlopLook::Boss(BossKind::Influencer),
                    role: crate::content::Role::Special,
                    movement: Movement::Kite,
                    attack: Some(Attack::Fan),
                    hp: 60.0,
                    speed,
                    radius: radius * 0.85,
                    contact: 6.0,
                    likes: 0,
                    resist: [0.0; 4],
                    traits: Traits(Traits::CLONE | Traits::NO_KNOCKBACK),
                    elite: false,
                };
                self.ensure_spec(clone)
            }
            BossKind::TheModel => self.ensure_spec(SlopSpec::archetype(Archetype::ChadGpt)),
            BossKind::YourReplacement => self.ensure_spec(SlopSpec::archetype(Archetype::Ticket)),
            BossKind::TheBoard => self.ensure_spec(SlopSpec::archetype(Archetype::Analyst)),
            BossKind::TheCandidate => self.ensure_spec(SlopSpec::archetype(Archetype::Voter)),
            BossKind::TheState => self.ensure_spec(SlopSpec::archetype(Archetype::Drone)),
        };
        if kind == BossKind::CrustaceanSaviour {
            let mut totem = SlopSpec::archetype(Archetype::EngagementBait);
            totem.name = "Amen Totem".into();
            totem.traits = Traits(Traits::HEAL_BOSS | Traits::NO_KNOCKBACK);
            let totem = self.ensure_spec(totem);
            for k in 0..4 {
                let spot = at + V2::from_angle(k as f32 * std::f32::consts::FRAC_PI_2 + 0.6) * 280.0;
                self.place(totem, spot, 0, 1);
            }
        }
        if kind == BossKind::TheBoard {
            // Five seats around the table: each heals it while it sits, and
            // three of them are a quorum.
            let mut seat = SlopSpec::archetype(Archetype::Consultant);
            seat.name = "Board Member".into();
            seat.movement = Movement::Stand;
            seat.hp = 90.0;
            seat.traits = Traits(Traits::HEAL_BOSS | Traits::NO_KNOCKBACK);
            let seat = self.ensure_spec(seat);
            for k in 0..BOARD_SEATS {
                let spot = at + V2::from_angle(k as f32 * std::f32::consts::TAU / BOARD_SEATS as f32 + 0.3) * 250.0;
                self.place(seat, spot, 0, 1);
            }
        }
        self.boss = Some(BossState { id, kind, phase: 0, timer: 180, timer2: 240, regenerated: false, minion, shielded: 0 });
        self.events.push(Event::BossIntro { kind });
    }

    /// A ring of speech bubbles around `at`; one of them says `line`.
    fn bubble_ring(&mut self, at: V2, count: u32, offset: f32, damage: f32, line: u16, spec: u16) {
        for k in 0..count {
            let dir = V2::from_angle(offset + k as f32 * std::f32::consts::TAU / count as f32);
            let l = if k == 0 { line } else { NO_LINE };
            self.hostile_shot(at + dir * 40.0, dir * RING_SPEED, damage, Look::Bubble, l, spec);
        }
    }

    pub(crate) fn boss_think(&mut self) {
        let Some(mut b) = self.boss.clone() else { return };
        let Some(i) = self.slop.iter().position(|s| s.id == b.id) else { return };
        let me = self.player.pos;
        let (pos, hp, max_hp, spec) = {
            let s = &self.slop[i];
            (s.pos, s.hp, s.max_hp, s.spec)
        };
        let speed = self.setup.specs[spec as usize].speed * self.hustle;
        if b.phase == 0 && hp < max_hp * 0.5 {
            b.phase = 1;
            self.events.push(Event::BossPhase { kind: b.kind, phase: 1 });
        }
        let to = me - pos;
        let dir = to.normalized();
        b.timer -= 1;
        b.timer2 -= 1;
        let mut want = dir * speed;
        let mut armor = 1.0;
        match b.kind {
            BossKind::CrustaceanSaviour => {
                if b.timer <= 0 {
                    b.timer = if b.phase == 1 { 260 } else { 360 };
                    for k in 0..5 {
                        self.pending.push((b.minion, pos + V2::from_angle(k as f32 * 1.256) * 70.0, 0.0, 0, 1));
                    }
                }
                if b.phase == 1 && b.timer2 <= 0 {
                    b.timer2 = 240;
                    self.bubble_ring(pos, 12, self.tick as f32 * 0.1, 7.0, AMEN, spec);
                }
            }
            BossKind::HustleGuru => {
                if self.tick % HUSTLE_TICKS == 0 {
                    self.hustle *= HUSTLE_STEP;
                    self.events.push(Event::Hustle { level: self.hustle });
                }
                let s = &mut self.slop[i];
                if s.charging > 0 {
                    s.charging -= 1;
                    want = s.aim * DASH_SPEED;
                } else if b.timer2 > 0 && b.timer2 <= DASH_AIM {
                    want = V2::ZERO;
                    if b.timer2 == 1 {
                        s.charging = DASH_TICKS;
                    }
                } else if b.timer <= 0 {
                    b.timer = 360;
                    b.timer2 = DASH_AIM;
                    s.aim = dir;
                    self.events.push(Event::LancerAim { from: pos, dir });
                }
                if b.phase == 1 && self.tick % 420 == 0 {
                    // A wall of thought leadership with one gap.
                    let origin = me - dir * 520.0;
                    let gap = self.rng_ai.below(13) as i32 + 1;
                    for k in -7..=7 {
                        if k == gap - 7 || k == gap - 6 {
                            continue;
                        }
                        let at = origin + dir.perp() * (k as f32 * 64.0);
                        self.hostile_shot(at, dir * 170.0, 9.0, Look::Post, NO_LINE, spec);
                    }
                }
            }
            BossKind::ContentFarm => {
                want = V2::ZERO;
                if b.timer <= 0 {
                    b.timer = if b.phase == 1 { 160 } else { 240 };
                    let off = V2::from_angle(self.rng_ai.range(0.0, std::f32::consts::TAU)) * self.rng_ai.range(120.0, 380.0);
                    self.pending.push((b.minion, pos + off, 0.0, 0, 1));
                }
                if b.timer2 <= 0 {
                    b.timer2 = 1500;
                    b.shielded = AD_TICKS;
                    self.ad_break = AD_TICKS;
                    self.events.push(Event::AdBreak);
                    self.bubble_ring(pos, 10, 0.0, 7.0, NO_LINE, spec);
                }
                if b.shielded > 0 {
                    b.shielded -= 1;
                    armor = 0.0;
                }
            }
            BossKind::RepostHydra => {
                if b.phase == 1 {
                    want *= 1.3;
                }
            }
            BossKind::Influencer => {
                let dist = to.length();
                want = if dist > 420.0 {
                    dir * speed
                } else if dist < 300.0 {
                    -dir * speed
                } else {
                    dir.perp() * speed * 0.6
                };
                if b.timer <= 0 {
                    b.timer = 480;
                    let count = if b.phase == 1 { 5 } else { 3 };
                    let spin = self.rng_ai.range(0.0, std::f32::consts::TAU);
                    let real = self.rng_ai.below(count + 1);
                    for k in 0..=count {
                        let spot = me + V2::from_angle(spin + k as f32 * std::f32::consts::TAU / (count + 1) as f32) * 380.0;
                        if k == real {
                            self.slop[i].pos = spot;
                        } else {
                            self.pending.push((b.minion, spot, 0.0, 0, 1));
                        }
                    }
                }
                if b.timer2 <= 0 {
                    b.timer2 = 200;
                    let line = self.rng_ai.below(BUBBLE_LINES.len() as u32) as u16;
                    let at = self.slop[i].pos;
                    self.bubble_ring(at, 8, self.tick as f32 * 0.05, 7.0, line, spec);
                }
            }
            BossKind::SupremeBrainrot => {
                if self.beat == 0 {
                    b.timer2 = (b.timer2 + 1).rem_euclid(4);
                    match b.timer2 {
                        0 => {
                            for k in 0..8 {
                                let d = V2::from_angle(k as f32 * std::f32::consts::FRAC_PI_4);
                                self.hostile_shot(pos + d * 50.0, d * 300.0, 8.0, Look::Toast, NO_LINE, spec);
                            }
                        }
                        1 => self.drop_bomb(me + self.player.vel * 0.5, 14.0, spec),
                        2 => {
                            for k in 0..3 {
                                self.pending.push((b.minion, pos + V2::from_angle(k as f32 * 2.1) * 60.0, 0.0, 0, 1));
                            }
                        }
                        _ => want *= 2.5,
                    }
                }
                if b.timer <= 0 {
                    b.timer = 600;
                    self.apply_status(StatusKind::BrainrotWarning);
                }
            }
            BossKind::YourReplacement => {
                // It does your job: it kites, it raises tickets, and it closes
                // them before you can. Each one closed heals it.
                let dist = to.length();
                want = if dist > 460.0 {
                    dir * speed
                } else if dist < 300.0 {
                    -dir * speed
                } else {
                    dir.perp() * speed * 0.5
                };
                if b.timer <= 0 {
                    b.timer = if b.phase == 1 { 300 } else { 420 };
                    for k in 0..3 {
                        self.pending.push((b.minion, pos + V2::from_angle(k as f32 * 2.1 + self.tick as f32 * 0.01) * 90.0, 0.0, 0, 1));
                    }
                }
                if self.tick % CLOSE_EVERY == 0 {
                    let boss_id = b.id;
                    let specs = &self.setup.specs;
                    let ticket = self
                        .slop
                        .iter()
                        .filter(|s| s.id != boss_id && s.hp > 0.0 && !specs[s.spec as usize].traits.has(Traits::HEAL_BOSS) && !matches!(specs[s.spec as usize].movement, Movement::Boss(_)))
                        .filter(|s| s.pos.distance_sq(pos) < CLOSE_RANGE * CLOSE_RANGE)
                        .min_by(|a, c| a.pos.distance_sq(pos).total_cmp(&c.pos.distance_sq(pos)))
                        .map(|s| s.id);
                    if let Some(id) = ticket {
                        if let Some(t) = self.slop.iter_mut().find(|s| s.id == id) {
                            t.hp = 0.0;
                            t.vanish = true;
                            let at = t.pos;
                            self.events.push(Event::Closed { at });
                        }
                        let boss = &mut self.slop[i];
                        boss.hp = (boss.hp + boss.max_hp * CLOSE_HEAL).min(boss.max_hp);
                        self.events.push(Event::Healed { at: pos });
                    }
                }
                if b.phase == 1 && b.timer2 <= 0 {
                    b.timer2 = 45;
                    let copies: Vec<Look> = self.setup.model.copies.iter().flatten().copied().collect();
                    let look = copies.get((self.tick / 45) as usize % copies.len().max(1)).copied().unwrap_or(Look::Staple);
                    self.hostile_shot(pos + dir * 60.0, dir * 400.0, 9.0, look, NO_LINE, spec);
                }
            }
            BossKind::TheBoard => {
                // It sits. While three seats are filled it has a quorum and
                // takes half damage; every vote is a volley from each seat.
                want = V2::ZERO;
                let specs = &self.setup.specs;
                let seats: Vec<V2> = self.slop.iter().filter(|s| specs[s.spec as usize].traits.has(Traits::HEAL_BOSS) && s.hp > 0.0).map(|s| s.pos).collect();
                if seats.len() >= QUORUM {
                    armor = 0.5;
                    b.shielded = 1;
                } else {
                    b.shielded = 0;
                }
                if b.timer <= 0 {
                    b.timer = if b.phase == 1 { 300 } else { 480 };
                    self.events.push(Event::Vote { at: pos, seats: seats.len() as u8 });
                    for seat in &seats {
                        let d = (me - *seat).normalized();
                        for n in -1..=1 {
                            let at = *seat + d * 30.0 + d.perp() * (n as f32 * 34.0);
                            self.hostile_shot(at, d * 170.0, 8.0, Look::Post, NO_LINE, spec);
                        }
                    }
                    if seats.is_empty() {
                        // No quorum: it calls in analysts to fill the room.
                        for k in 0..3 {
                            self.pending.push((b.minion, pos + V2::from_angle(k as f32 * 2.1) * 120.0, 0.0, 0, 1));
                        }
                    }
                }
                if b.timer2 <= 0 {
                    b.timer2 = 240;
                    let line = self.rng_ai.below(BUBBLE_LINES.len() as u32) as u16;
                    self.bubble_ring(pos, if b.phase == 1 { 10 } else { 6 }, self.tick as f32 * 0.07, 7.0, line, spec);
                }
            }
            BossKind::TheCandidate => {
                // It keeps its distance, makes promises that land where you
                // are going, and holds rallies.
                let dist = to.length();
                want = if dist > 440.0 {
                    dir * speed
                } else if dist < 320.0 {
                    -dir * speed
                } else {
                    dir.perp() * speed * 0.6
                };
                if b.timer <= 0 {
                    b.timer = if b.phase == 1 { 200 } else { 360 };
                    let at = me + self.player.vel * PROMISE_LEAD;
                    self.blast(at, PROMISE_RADIUS, PROMISE_WARNING, 14.0, spec);
                    self.events.push(Event::Promise { at });
                }
                if b.timer2 <= 0 {
                    b.timer2 = 900;
                    let count = if b.phase == 1 { 10 } else { 6 };
                    for k in 0..count {
                        let spot = me + V2::from_angle(k as f32 * std::f32::consts::TAU / count as f32) * 420.0;
                        self.pending.push((b.minion, spot, 0.0, 0, 1));
                    }
                    self.events.push(Event::Rally { at: me });
                }
            }
            BossKind::TheModel | BossKind::TheState => {
                let collapse = self.setup.model.collapse;
                let c = self.tick % MODEL_CYCLE;
                let dist = to.length();
                want = if dist > 260.0 { dir * speed } else { V2::ZERO };
                armor = 1.0 + 0.5 * collapse;
                let glitch = |w: &mut World| collapse > 0.5 && w.rng_ai.chance(collapse * 0.4);
                if c < MODEL_THINK {
                    armor *= 0.3;
                    b.shielded = 1;
                    want = V2::ZERO;
                } else if c < MODEL_ANSWER {
                    b.shielded = 0;
                    let every = if b.phase == 1 { 6 } else { 10 };
                    if c % every == 0 && !glitch(self) {
                        let a = self.tick as f32 * 0.21;
                        for d in [V2::from_angle(a), V2::from_angle(a + std::f32::consts::PI)] {
                            let line = if c % 60 == 0 { self.rng_ai.below(BUBBLE_LINES.len() as u32) as u16 } else { NO_LINE };
                            self.hostile_shot(pos + d * 60.0, d * 190.0, 8.0, Look::Bubble, line, spec);
                        }
                    }
                } else if c % 36 == 0 && !glitch(self) {
                    // It copies your favourite weapons.
                    let copies: Vec<Look> = self.setup.model.copies.iter().flatten().copied().collect();
                    let look = copies.get((c / 36) as usize % copies.len().max(1)).copied().unwrap_or(Look::EmDash);
                    self.hostile_shot(pos + dir * 60.0, dir * 430.0, 10.0, look, NO_LINE, spec);
                }
                if self.tick % 900 == 0 {
                    for k in 0..3 {
                        self.pending.push((b.minion, pos + V2::from_angle(k as f32 * 2.1) * 120.0, 0.0, 0, 1));
                    }
                }
                if b.kind == BossKind::TheState && self.tick % 1200 == 600 {
                    // It sends your own agents after you.
                    let agent = self.ensure_spec(SlopSpec::archetype(Archetype::YourAgent));
                    for k in 0..2 {
                        self.pending.push((agent, pos + V2::from_angle(k as f32 * 3.1 + 1.0) * 160.0, 0.0, 0, 1));
                    }
                }
            }
        }
        let s = &mut self.slop[i];
        s.armor = armor;
        s.moving = want;
        s.pos += want * DT;
        self.boss = Some(b);
    }

    /// The boss is down: the wave is won and the rest of the slop is deleted.
    /// The boss is gone: a big chest drops, and the rules move the Feed on.
    pub(crate) fn boss_defeated(&mut self, at: V2) {
        if let Some(b) = self.boss.take() {
            self.events.push(Event::BossDefeated { kind: b.kind, at });
        }
        self.drop_pickup(PickupKind::Chest, at, 2);
        self.hustle = 1.0;
        // Its helpers leave with it.
        let specs = &self.setup.specs;
        for s in self.slop.iter_mut().filter(|s| specs[s.spec as usize].traits.has(Traits::HEAL_BOSS) || specs[s.spec as usize].traits.has(Traits::CLONE)) {
            s.hp = 0.0;
        }
        if self.setup.seconds > 0.0 && self.outcome == Outcome::Running {
            self.clear();
        }
    }
}
