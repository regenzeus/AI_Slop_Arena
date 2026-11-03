//! You: movement, status effects, getting hurt, and actives that fire on
//! their own (STATUS-01, ACT-01, FAIR-03).

use crate::content::{Pattern, WEAPONS};
use crate::events::{Event, StatusKind};
use crate::spec::Movement;
use crate::stats::{DamageType, SELF_CARE_RATE};
use crate::vec::V2;
use crate::world::*;

/// Share of the gap to the wanted velocity closed each tick, so movement eases.
const ACCELERATION: f32 = 0.5;
const HIT_INVULNERABLE: u16 = 30;
const DODGE_INVULNERABLE: u16 = 15;
const GRAB_SLOW: f32 = 0.2;
const MIN_GRAB_SPEED: f32 = 0.4;
const DOWNVOTE_SLOW: f32 = 0.6;
const BANNER_PUSH: f32 = 230.0;
const DASH_SPEED: f32 = 1300.0;
const DASH_TICKS: u16 = 12;
const ANXIETY_TICKS: u16 = 180;
const ANXIETY_MAX: u8 = 5;
/// Mental Health per second per Anxiety stack.
const ANXIETY_DAMAGE: f32 = 2.0;
const COFFEE_TICKS: u16 = 1200;
const LETTER_TICKS: u16 = 600;
const LETTER_DAMAGE: f32 = 8.0;
/// Mental Health per second Doomscrolling costs while slop is on screen.
const DOOMSCROLL_DRAIN: f32 = 0.5;
const DOG_BLOCK_TICKS: u16 = 600;

impl World {
    pub(crate) fn move_player(&mut self, input: Input) {
        let zones = &self.zones;
        let p = &mut self.player;
        let mut dir = V2::new(input.x, input.y).clamp_length(1.0);
        if p.status.brainrot > 0 {
            dir = -dir;
        }
        if dir.length_sq() > 0.01 {
            p.facing = dir.normalized();
        }
        let mut slow = (1.0 - GRAB_SLOW * p.grabbed as f32).max(MIN_GRAB_SPEED);
        if zones.iter().any(|z| z.kind == ZoneKind::Downvote && z.contains(p.pos, 0.0)) {
            slow *= DOWNVOTE_SLOW;
        }
        let mut want = dir * (PLAYER_SPEED * p.stats.speed * slow);
        if p.status.doomscroll > 0 {
            want = want * 0.4 + p.facing * (PLAYER_SPEED * 0.9);
        }
        p.vel += (want - p.vel) * ACCELERATION;
        let mut step = p.vel * DT;
        for z in zones.iter().filter(|z| z.kind == ZoneKind::Banner) {
            if z.contains(p.pos, 0.0) {
                step += z.vel.normalized() * (BANNER_PUSH * DT);
            }
        }
        if p.dash > 0 {
            p.dash -= 1;
            step += p.dash_dir * (DASH_SPEED * DT);
        }
        p.pos += step;
        p.mental_health = (p.mental_health + p.stats.self_care * SELF_CARE_RATE * DT).min(p.stats.max_mental_health);
    }

    /// Statuses count down; items with side effects tick (STATUS-01).
    pub(crate) fn tick_status(&mut self) {
        let mut anxiety_hit = 0.0;
        {
            let s = &mut self.player.status;
            if s.anxiety_ticks > 0 {
                s.anxiety_ticks -= 1;
                if s.anxiety_ticks % 30 == 0 {
                    anxiety_hit = ANXIETY_DAMAGE * 0.5 * s.anxiety as f32;
                }
                if s.anxiety_ticks == 0 {
                    s.anxiety = 0;
                }
            }
            s.doomscroll = s.doomscroll.saturating_sub(1);
            s.popup = s.popup.saturating_sub(1);
            s.ratio = s.ratio.saturating_sub(1);
            s.brainrot = s.brainrot.saturating_sub(1);
        }
        if self.player.status.brainrot_warning > 0 {
            self.player.status.brainrot_warning -= 1;
            if self.player.status.brainrot_warning == 0 {
                self.player.status.brainrot = 60;
                self.events.push(Event::Status { kind: StatusKind::Brainrot });
            }
        }
        if anxiety_hit > 0.0 {
            self.lose(anxiety_hit);
        }
        let perks = self.player.perks.clone();
        if perks.coffee > 0 {
            self.player.coffee_timer += 1;
            if self.player.coffee_timer >= COFFEE_TICKS / perks.coffee as u16 {
                self.player.coffee_timer = 0;
                self.add_anxiety();
            }
        }
        if perks.doomscrolling > 0 {
            let me = self.player.pos;
            if self.slop.iter().any(|s| s.pos.distance_sq(me) < SHOOT_RANGE * SHOOT_RANGE) {
                let drain = DOOMSCROLL_DRAIN * perks.doomscrolling as f32 * DT;
                self.player.mental_health -= drain;
                self.tally.damage_taken += drain;
            }
        }
    }

    /// Loses Mental Health directly, with no dodge or invulnerability.
    pub(crate) fn lose(&mut self, amount: f32) {
        let amount = self.player.stats.damage_taken(amount);
        self.player.mental_health -= amount;
        self.tally.damage_taken += amount;
        self.events.push(Event::PlayerHurt { amount });
        self.save_from_burnout();
    }

    fn save_from_burnout(&mut self) {
        let p = &mut self.player;
        if p.mental_health <= 0.0 && p.second_winds > 0 {
            p.second_winds -= 1;
            p.mental_health = 1.0;
            p.invulnerable = 120;
            self.events.push(Event::SecondWind);
        }
    }

    /// A hit on you: an item may block it, Scepticism may ignore it,
    /// Composure softens it, and you are briefly invulnerable afterwards.
    /// Returns whether it landed.
    pub(crate) fn hurt(&mut self, raw: f32, spec: u16) -> bool {
        let p = &self.player;
        if p.invulnerable > 0 || p.dash > 0 || raw <= 0.0 {
            return false;
        }
        let has_dog = p.weapons.iter().any(|w| matches!(WEAPONS[w.def].pattern, Pattern::Orbit { .. }));
        if p.perks.therapy_dog && has_dog && p.dog_block == 0 {
            self.player.dog_block = DOG_BLOCK_TICKS;
            self.player.invulnerable = DODGE_INVULNERABLE;
            self.events.push(Event::Blocked);
            return false;
        }
        if self.rng_combat.chance(self.player.stats.dodge_chance()) {
            self.player.invulnerable = DODGE_INVULNERABLE;
            self.tally.dodges += 1;
            self.events.push(Event::Dodged);
            return false;
        }
        let mut amount = self.player.stats.damage_taken(raw);
        if self.player.perks.hot_take {
            amount *= 1.1;
        }
        self.player.mental_health -= amount;
        self.player.invulnerable = HIT_INVULNERABLE;
        self.tally.damage_taken += amount;
        if let Some(t) = self.spec_tally.get_mut(spec as usize) {
            t.damage += amount;
        }
        self.events.push(Event::PlayerHurt { amount });
        self.save_from_burnout();
        true
    }

    pub(crate) fn add_anxiety(&mut self) {
        let s = &mut self.player.status;
        s.anxiety = (s.anxiety + 1).min(ANXIETY_MAX);
        s.anxiety_ticks = ANXIETY_TICKS;
        self.events.push(Event::Status { kind: StatusKind::Anxiety });
    }

    /// Puts a status on you, unless an item or your hero protects you.
    pub(crate) fn apply_status(&mut self, kind: StatusKind) {
        let perks = &self.player.perks;
        let s = &mut self.player.status;
        match kind {
            StatusKind::Anxiety => return self.add_anxiety(),
            StatusKind::Doomscroll if perks.dumbphone => return,
            StatusKind::Doomscroll => s.doomscroll = 60,
            StatusKind::Popup if perks.ad_blocker => return,
            StatusKind::Popup => s.popup = if perks.fluent { 60 } else { 120 },
            StatusKind::Ratio => s.ratio = 180,
            StatusKind::Brainrot | StatusKind::BrainrotWarning if perks.fluent => return,
            StatusKind::Brainrot | StatusKind::BrainrotWarning => {
                if s.brainrot > 0 || s.brainrot_warning > 0 {
                    return;
                }
                s.brainrot_warning = 45;
                self.events.push(Event::Status { kind: StatusKind::BrainrotWarning });
                return;
            }
        }
        self.events.push(Event::Status { kind });
    }

    /// Actives fire on their own when their trigger is met (ACT-01).
    pub(crate) fn use_actives(&mut self) {
        // The Boomer's chain letter (it needs this tick's grid).
        if self.player.perks.chain_letter {
            self.player.letter_timer += 1;
            if self.player.letter_timer >= LETTER_TICKS {
                self.player.letter_timer = 0;
                let at = self.player.pos;
                if let Some(first) = nearest_slop(&self.grid, &self.slop, at, 500.0, None) {
                    self.chain_hit(at, first, 5, LETTER_DAMAGE, DamageType::Sarcasm, None);
                }
            }
        }
        let actives = self.player.perks.actives.clone();
        for (i, active) in actives.into_iter().enumerate() {
            if self.player.active_cooldown[i] > 0.0 {
                continue;
            }
            let fired = match active {
                Active::LogOff => self.log_off(),
                Active::BlockButton => self.block_button(),
                Active::DigitalDetox => self.digital_detox(),
                Active::Unsubscribe => self.unsubscribe(),
                Active::PhoneAFriend => self.phone_a_friend(),
            };
            if fired {
                self.player.active_cooldown[i] = active.cooldown();
                self.events.push(Event::ActiveUsed { active });
            }
        }
    }

    fn log_off(&mut self) -> bool {
        let me = self.player.pos;
        let near: Vec<V2> = self.slop.iter().filter(|s| s.pos.distance_sq(me) < 120.0 * 120.0).map(|s| s.pos).collect();
        if near.len() < 8 {
            return false;
        }
        let centre = near.iter().fold(V2::ZERO, |a, &b| a + b) * (1.0 / near.len() as f32);
        let away = (me - centre).normalized();
        self.player.dash_dir = if away.length_sq() > 0.0 { away } else { self.player.facing };
        self.player.dash = DASH_TICKS;
        self.player.invulnerable = self.player.invulnerable.max(DASH_TICKS + 6);
        for s in &mut self.slop {
            s.grab = None;
        }
        true
    }

    fn block_button(&mut self) -> bool {
        let me = self.player.pos;
        let threat = self
            .projectiles
            .iter()
            .position(|p| p.hostile && p.pos.distance_sq(me) < 70.0 * 70.0 && p.vel.dot(me - p.pos) > 0.0);
        let Some(i) = threat else { return false };
        let at = self.projectiles[i].pos;
        self.projectiles[i].life = 0;
        if let Some(j) = self.projectiles.iter().position(|p| p.hostile && p.life > 0 && p.pos.distance_sq(at) < 150.0 * 150.0) {
            self.projectiles[j].life = 0;
        }
        self.events.push(Event::Blocked);
        true
    }

    fn digital_detox(&mut self) -> bool {
        if self.player.status.count() < 2 {
            return false;
        }
        self.player.status = Status::default();
        let me = self.player.pos;
        for s in &mut self.slop {
            let d = s.pos - me;
            if d.length_sq() < 220.0 * 220.0 {
                s.push += d.normalized() * 600.0;
                s.grab = None;
            }
        }
        true
    }

    fn unsubscribe(&mut self) -> bool {
        let p = &self.player;
        if p.mental_health > p.stats.max_mental_health * 0.3 {
            return false;
        }
        let me = p.pos;
        for s in &mut self.slop {
            let boss = matches!(self.setup.specs[s.spec as usize].movement, Movement::Boss(_));
            if !boss && s.pos.distance_sq(me) < SHOOT_RANGE * SHOOT_RANGE {
                s.hp -= s.max_hp * 0.1;
                s.stunned = s.stunned.max(120);
                s.last_hit = Some(self.tick);
            }
        }
        true
    }

    fn phone_a_friend(&mut self) -> bool {
        let me = self.player.pos;
        let target = self.slop.iter().position(|s| {
            let sp = &self.setup.specs[s.spec as usize];
            (sp.elite || matches!(sp.movement, Movement::Boss(_))) && s.pos.distance_sq(me) < 450.0 * 450.0
        });
        let Some(t) = target else { return false };
        let dir = (self.slop[t].pos - me).normalized();
        let (from, to) = (me - dir * 300.0, self.slop[t].pos + dir * 300.0);
        let hits: Vec<usize> = (0..self.slop.len())
            .filter(|&j| {
                let s = &self.slop[j];
                let along = (s.pos - from).dot(dir).clamp(0.0, (to - from).length());
                (from + dir * along).distance(s.pos) < s.radius + 30.0
            })
            .collect();
        for j in hits {
            self.damage_slop(j, 40.0, DamageType::Analog, dir * 300.0, me, None);
        }
        self.events.push(Event::Friend { from, to });
        true
    }
}
