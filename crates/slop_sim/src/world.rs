//! The arena: one human against a wave of slop, stepped at a fixed 60 ticks
//! per second (SIM-01). Everything the client shows is read from here, and a
//! seed plus the input of every tick replays a wave exactly.

use crate::content::{Look, Pattern, SlopKind, WeaponDef, AGREE_LINES, BUBBLE_LINES, TIER_DAMAGE, WEAPONS};
use crate::events::Event;
use crate::rng::Rng;
use crate::spatial::Grid;
use crate::stats::{DamageType, Stats, SELF_CARE_RATE};
use crate::vec::V2;

pub const TICK_RATE: u32 = 60;
pub const DT: f32 = 1.0 / TICK_RATE as f32;
/// Side of the square arena in world units; the client shows 1600 x 900 of it.
pub const ARENA: f32 = 2400.0;
pub const PLAYER_RADIUS: f32 = 16.0;
/// Base movement speed in units per second.
pub const PLAYER_SPEED: f32 = 300.0;
/// Share of the gap to the wanted velocity closed each tick, so movement eases.
const ACCELERATION: f32 = 0.3;
/// Ticks of invulnerability after a hit, and after a dodge.
const HIT_INVULNERABLE: u16 = 30;
const DODGE_INVULNERABLE: u16 = 15;
/// Slop only shoots when this close to you: roughly "on screen" (FAIR-01).
pub const SHOOT_RANGE: f32 = 760.0;
pub const MAX_SLOP: usize = 400;
pub const MAX_BUBBLES: usize = 300;
/// Ticks a spawn is telegraphed before the slop appears (FAIR-02).
pub const SPAWN_WARNING: u16 = 60;
const SPAWN_MIN: f32 = 700.0;
const SPAWN_MAX: f32 = 920.0;
const SPAWN_CLOSEST: f32 = 480.0;
/// No new spawns in the last seconds of a wave.
const SPAWN_STOP: f32 = 2.0;
const LIKE_PULL_SPEED: f32 = 700.0;
/// Share of uncollected Likes credited when a wave is cleared.
const LEFTOVER_SHARE: f32 = 0.5;
/// Ticks a health bar stays after the last hit.
pub const HEALTH_BAR_TICKS: u32 = 180;
/// Ticks a slop flashes white after a hit.
pub const FLASH_TICKS: u8 = 4;
const KNOCKBACK_DECAY: f32 = 0.85;
const MAX_GRABS: u8 = 4;
const GRAB_DAMAGE: f32 = 2.0;
const GRAB_INTERVAL: i32 = 30;
/// Speed lost per hand holding on, and the floor.
const GRAB_SLOW: f32 = 0.2;
const MIN_GRAB_SPEED: f32 = 0.4;
const AGREE_BONUS: f32 = 0.1;
const AGREE_MAX: u8 = 3;
const AGREE_TICKS: u16 = 300;
const AGREE_RANGE: f32 = 220.0;
const AGREE_INTERVAL: i32 = 180;
const AGREE_TARGETS: usize = 3;
const YES_FOLLOW: f32 = 600.0;
const BUBBLE_SPEED: f32 = 200.0;
const BUBBLE_DAMAGE: f32 = 6.0;
pub const BUBBLE_RADIUS: f32 = 13.0;
const BUBBLE_LIFE: u16 = 200;
/// Line index of a bubble that says nothing ("...").
pub const NO_LINE: u16 = u16::MAX;
const BUBBLE_SPREAD: f32 = 0.28;
const BUBBLE_INTERVAL: i32 = 180;
const BUBBLE_KEEP_FAR: f32 = 420.0;
const BUBBLE_KEEP_NEAR: f32 = 300.0;
const LANCE_SPEED: f32 = 760.0;
const LANCE_DAMAGE: f32 = 12.0;
const LANCE_RADIUS: f32 = 8.0;
const LANCE_LIFE: u16 = 90;
/// Ticks an Em-Dash Lancer aims before it throws (FAIR-02).
pub const LANCE_AIM: u16 = 48;
const LANCE_RANGE: f32 = 440.0;
const LANCE_RECOVER: i32 = 140;
const SCATTER_TICKS: u16 = 90;
/// One popped bubble in this many drops a Like.
const POP_LIKE_ODDS: u32 = 5;
/// How far a ricochet looks for its next target.
const RICOCHET_RANGE: f32 = 300.0;

/// Movement wanted this tick; longer than 1 is shortened.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Input {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug)]
pub struct Weapon {
    /// Index into `WEAPONS`.
    pub def: usize,
    /// Tier 1 to 4.
    pub tier: u8,
    /// Ticks until it may attack again.
    pub cooldown: f32,
    /// Weapons can be switched off for testing.
    pub enabled: bool,
}

impl Weapon {
    pub fn new(def: usize, tier: u8) -> Weapon {
        Weapon { def, tier: tier.clamp(1, 4), cooldown: 0.0, enabled: true }
    }

    pub fn def(&self) -> &'static WeaponDef {
        &WEAPONS[self.def]
    }
}

#[derive(Clone, Debug)]
pub struct Player {
    pub pos: V2,
    pub vel: V2,
    /// Last direction moved.
    pub facing: V2,
    pub mental_health: f32,
    pub stats: Stats,
    pub invulnerable: u16,
    pub weapons: Vec<Weapon>,
    /// Hands holding on right now.
    pub grabbed: u8,
}

impl Player {
    pub fn new(weapons: Vec<Weapon>) -> Player {
        let stats = Stats::default();
        Player {
            pos: V2::new(ARENA / 2.0, ARENA / 2.0),
            vel: V2::ZERO,
            facing: V2::new(1.0, 0.0),
            mental_health: stats.max_mental_health,
            stats,
            invulnerable: 0,
            weapons,
            grabbed: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Slop {
    pub id: u32,
    pub kind: SlopKind,
    pub pos: V2,
    /// Velocity it chose this tick, for drawing.
    pub moving: V2,
    /// Knockback velocity, decaying.
    pub push: V2,
    pub radius: f32,
    pub hp: f32,
    pub max_hp: f32,
    /// Resistance to each damage type; trained by the slop later (M4).
    pub resist: [f32; 4],
    /// Countdown used by each kind's behaviour.
    pub timer: i32,
    /// Ticks left aiming (Em-Dash Lancer).
    pub charging: u16,
    pub aim: V2,
    pub marked: u16,
    pub mark_bonus: f32,
    /// Stacks of Yes-Bot agreement and ticks they last.
    pub agreed: u8,
    pub agreed_ticks: u16,
    pub last_hit: Option<u32>,
    pub flash: u8,
    /// Listicle pack and number in it.
    pub pack: u32,
    pub number: u8,
    pub fleeing: u16,
    /// Offset from you while a Six-Finger Hand holds on.
    pub grab: Option<V2>,
}

impl Slop {
    /// Damage multiplier from Yes-Bot agreement.
    pub fn agreement(&self) -> f32 {
        1.0 + AGREE_BONUS * self.agreed as f32
    }

    /// Whether its health bar shows: hit within the last three seconds (UI-03).
    pub fn shows_health(&self, tick: u32) -> bool {
        self.last_hit.is_some_and(|t| tick.saturating_sub(t) < HEALTH_BAR_TICKS) && self.hp < self.max_hp
    }
}

#[derive(Clone, Debug)]
pub struct Projectile {
    pub id: u32,
    pub pos: V2,
    pub vel: V2,
    pub radius: f32,
    pub damage: f32,
    pub kind: DamageType,
    /// Fired by slop at you.
    pub hostile: bool,
    pub pierce: u8,
    pub bounces: u8,
    /// Bonus and ticks of the mark it leaves.
    pub mark: Option<(f32, u16)>,
    pub look: Look,
    pub life: u16,
    /// Index into `BUBBLE_LINES` for speech bubbles.
    pub line: u16,
    /// Id of the slop hit last, so piercing and bouncing never hit twice in a row.
    pub last_hit: u32,
    pub knockback: f32,
}

#[derive(Clone, Debug)]
pub struct Like {
    pub id: u32,
    pub pos: V2,
    pub value: u32,
    /// Flying towards you.
    pub pulled: bool,
}

/// A slop about to appear.
#[derive(Clone, Debug)]
pub struct Warning {
    pub pos: V2,
    pub kind: SlopKind,
    pub ticks: u16,
    pub pack: u32,
    pub number: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Running,
    Cleared,
    Burnout,
}

#[derive(Clone, Copy, Debug)]
pub struct WaveConfig {
    pub number: u32,
    pub seconds: f32,
    pub spawning: bool,
}

impl WaveConfig {
    pub fn first() -> WaveConfig {
        WaveConfig { number: 1, seconds: 60.0, spawning: true }
    }
}

/// What happened in the wave, for the report.
#[derive(Clone, Debug, Default)]
pub struct Tally {
    pub kills: u32,
    pub damage_dealt: [f32; 4],
    pub damage_taken: f32,
    pub dodges: u32,
    pub bubbles_popped: u32,
    pub likes_collected: u32,
}

pub struct World {
    pub seed: u64,
    pub tick: u32,
    pub wave: WaveConfig,
    pub player: Player,
    pub slop: Vec<Slop>,
    pub projectiles: Vec<Projectile>,
    pub likes: Vec<Like>,
    pub warnings: Vec<Warning>,
    /// Currency earned this wave.
    pub clout: u32,
    pub outcome: Outcome,
    pub tally: Tally,
    next_id: u32,
    next_pack: u32,
    spawn_credit: f32,
    grid: Grid,
    events: Vec<Event>,
    rng_spawn: Rng,
    rng_combat: Rng,
    rng_ai: Rng,
}

/// Spawns per second at `progress` (0 to 1) through a wave; a Listicle pack
/// costs three.
pub fn spawn_rate(progress: f32) -> f32 {
    1.7 + 5.6 * progress
}

/// Which slop to spawn `t` seconds into the wave: new kinds join over time.
fn pick_kind(t: f32, rng: &mut Rng) -> SlopKind {
    let table = [
        (SlopKind::Listicle, 0.0, 4),
        (SlopKind::BubbleBot, 0.0, 3),
        (SlopKind::EmDashLancer, 10.0, 2),
        (SlopKind::SixFingerHand, 15.0, 2),
        (SlopKind::YesBot, 20.0, 1),
    ];
    let total: u32 = table.iter().filter(|(_, from, _)| t >= *from).map(|(_, _, w)| w).sum();
    let mut roll = rng.below(total);
    for (kind, from, weight) in table {
        if t < from {
            continue;
        }
        if roll < weight {
            return kind;
        }
        roll -= weight;
    }
    SlopKind::Listicle
}

fn clamp_to_arena(p: V2, margin: f32) -> V2 {
    V2::new(p.x.clamp(margin, ARENA - margin), p.y.clamp(margin, ARENA - margin))
}

/// The nearest living slop within `range` of `at`, other than `skip`.
fn nearest_slop(grid: &Grid, slop: &[Slop], at: V2, range: f32, skip: Option<usize>) -> Option<usize> {
    let mut best: Option<(usize, f32)> = None;
    grid.for_each_near(at, range, |j| {
        let s = &slop[j];
        if Some(j) == skip || s.hp <= 0.0 {
            return;
        }
        let d = s.pos.distance_sq(at);
        let reach = range + s.radius;
        if d <= reach * reach && best.is_none_or(|(_, bd)| d < bd) {
            best = Some((j, d));
        }
    });
    best.map(|(j, _)| j)
}

/// Applies a hit to one slop (DMG-01): Conviction, its resistance and a mark
/// all multiply the base damage.
#[allow(clippy::too_many_arguments)]
fn damage_slop(
    s: &mut Slop,
    base: f32,
    kind: DamageType,
    push: V2,
    stats: &Stats,
    tick: u32,
    tally: &mut Tally,
    events: &mut Vec<Event>,
) {
    if s.hp <= 0.0 {
        return;
    }
    let marked = if s.marked > 0 { 1.0 + s.mark_bonus } else { 1.0 };
    let amount = base * (1.0 + stats.conviction) * (1.0 - s.resist[kind as usize]) * marked;
    s.hp -= amount;
    s.flash = FLASH_TICKS;
    s.last_hit = Some(tick);
    if s.grab.is_none() {
        s.push += push;
    }
    tally.damage_dealt[kind as usize] += amount;
    events.push(Event::SlopHit { at: s.pos, amount, kind });
}

impl World {
    /// A wave with you holding all six weapons at tier I.
    pub fn new(seed: u64, wave: WaveConfig) -> World {
        let weapons = (0..WEAPONS.len()).map(|d| Weapon::new(d, 1)).collect();
        World {
            seed,
            tick: 0,
            wave,
            player: Player::new(weapons),
            slop: Vec::new(),
            projectiles: Vec::new(),
            likes: Vec::new(),
            warnings: Vec::new(),
            clout: 0,
            outcome: Outcome::Running,
            tally: Tally::default(),
            next_id: 1,
            next_pack: 1,
            spawn_credit: 0.0,
            grid: Grid::new(),
            events: Vec::new(),
            rng_spawn: Rng::new(seed, 1),
            rng_combat: Rng::new(seed, 2),
            rng_ai: Rng::new(seed, 3),
        }
    }

    /// An arena with no spawning and no time limit, for tests and the lab.
    pub fn sandbox(seed: u64) -> World {
        World::new(seed, WaveConfig { number: 1, seconds: 3600.0, spawning: false })
    }

    pub fn elapsed(&self) -> f32 {
        self.tick as f32 * DT
    }

    /// Share of the wave survived, 0 to 1.
    pub fn progress(&self) -> f32 {
        (self.elapsed() / self.wave.seconds).min(1.0)
    }

    pub fn drain_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    fn id(&mut self) -> u32 {
        self.next_id += 1;
        self.next_id
    }

    /// Places a slop right away, without warning. Returns its id.
    pub fn spawn_slop(&mut self, kind: SlopKind, at: V2) -> u32 {
        self.place(kind, at, 0, 1)
    }

    fn place(&mut self, kind: SlopKind, at: V2, pack: u32, number: u8) -> u32 {
        let def = kind.def();
        let id = self.id();
        let timer = match kind {
            SlopKind::BubbleBot => 40 + self.rng_spawn.below(110) as i32,
            SlopKind::YesBot => AGREE_INTERVAL / 2,
            _ => 0,
        };
        self.slop.push(Slop {
            id,
            kind,
            pos: clamp_to_arena(at, def.radius),
            moving: V2::ZERO,
            push: V2::ZERO,
            radius: def.radius,
            hp: def.hp,
            max_hp: def.hp,
            resist: [0.0; 4],
            timer,
            charging: 0,
            aim: V2::ZERO,
            marked: 0,
            mark_bonus: 0.0,
            agreed: 0,
            agreed_ticks: 0,
            last_hit: None,
            flash: 0,
            pack,
            number,
            fleeing: 0,
            grab: None,
        });
        id
    }

    /// Advances one tick.
    pub fn step(&mut self, input: Input) {
        if self.outcome != Outcome::Running {
            return;
        }
        self.tick += 1;
        self.move_player(input);
        if self.wave.spawning {
            self.spawn();
        }
        self.grid.rebuild(self.slop.iter().map(|s| s.pos));
        self.think();
        self.separate();
        self.touch();
        self.fire_weapons();
        self.move_projectiles();
        self.bury();
        self.collect_likes();
        self.tick_timers();
        self.check_end();
    }

    fn move_player(&mut self, input: Input) {
        let p = &mut self.player;
        let dir = V2::new(input.x, input.y).clamp_length(1.0);
        if dir.length_sq() > 0.01 {
            p.facing = dir.normalized();
        }
        let slow = (1.0 - GRAB_SLOW * p.grabbed as f32).max(MIN_GRAB_SPEED);
        let want = dir * (PLAYER_SPEED * p.stats.speed * slow);
        p.vel += (want - p.vel) * ACCELERATION;
        p.pos = clamp_to_arena(p.pos + p.vel * DT, PLAYER_RADIUS);
        p.mental_health = (p.mental_health + p.stats.self_care * SELF_CARE_RATE * DT).min(p.stats.max_mental_health);
    }

    /// Counts down warnings into slop and queues new ones (WAVE-02).
    fn spawn(&mut self) {
        let mut ready = Vec::new();
        self.warnings.retain_mut(|w| {
            w.ticks -= 1;
            if w.ticks == 0 {
                ready.push((w.kind, w.pos, w.pack, w.number));
                false
            } else {
                true
            }
        });
        for (kind, pos, pack, number) in ready {
            self.place(kind, pos, pack, number);
        }

        let t = self.elapsed();
        if t >= self.wave.seconds - SPAWN_STOP {
            return;
        }
        self.spawn_credit += spawn_rate(t / self.wave.seconds) * DT;
        while self.spawn_credit >= 1.0 && self.slop.len() + self.warnings.len() < MAX_SLOP {
            let kind = pick_kind(t, &mut self.rng_spawn);
            let at = self.spawn_point();
            if kind == SlopKind::Listicle {
                self.spawn_credit -= 3.0;
                self.next_pack += 1;
                for n in 1..=5u8 {
                    let offset = V2::from_angle(n as f32 * 1.2566) * 30.0;
                    self.warn(kind, at + offset, self.next_pack, n);
                }
            } else {
                self.spawn_credit -= 1.0;
                self.warn(kind, at, 0, 1);
            }
        }
    }

    fn warn(&mut self, kind: SlopKind, at: V2, pack: u32, number: u8) {
        let pos = clamp_to_arena(at, kind.def().radius);
        self.warnings.push(Warning { pos, kind, ticks: SPAWN_WARNING, pack, number });
        self.events.push(Event::SpawnWarning { at: pos, kind });
    }

    /// Somewhere just off-screen, inside the arena, never right next to you.
    fn spawn_point(&mut self) -> V2 {
        let player = self.player.pos;
        let mut last = player;
        for _ in 0..8 {
            let angle = self.rng_spawn.range(0.0, std::f32::consts::TAU);
            let d = self.rng_spawn.range(SPAWN_MIN, SPAWN_MAX);
            let p = clamp_to_arena(player + V2::from_angle(angle) * d, 40.0);
            if p.distance(player) >= SPAWN_CLOSEST {
                return p;
            }
            last = p;
        }
        last
    }

    /// Each slop decides where to move and whether to attack (SLOP-01).
    fn think(&mut self) {
        let player = self.player.pos;
        let mut shots = Vec::new();
        let mut agree = Vec::new();
        for i in 0..self.slop.len() {
            let (pos, kind) = (self.slop[i].pos, self.slop[i].kind);
            // Yes-Bots look around first, before anything is borrowed mutably.
            let mut friend: Option<V2> = None;
            let mut nearby: Vec<usize> = Vec::new();
            if kind == SlopKind::YesBot {
                let mut best = f32::MAX;
                let (slop, grid) = (&self.slop, &self.grid);
                grid.for_each_near(pos, YES_FOLLOW, |j| {
                    let o = &slop[j];
                    if o.kind == SlopKind::YesBot {
                        return;
                    }
                    let d = o.pos.distance_sq(pos);
                    if d < best && d < YES_FOLLOW * YES_FOLLOW {
                        best = d;
                        friend = Some(o.pos);
                    }
                    if d < AGREE_RANGE * AGREE_RANGE && nearby.len() < AGREE_TARGETS {
                        nearby.push(j);
                    }
                });
            }

            let s = &mut self.slop[i];
            let def = kind.def();
            let to = player - s.pos;
            let dist = to.length();
            let dir = to.normalized();
            let side = if s.id % 2 == 0 { 1.0 } else { -1.0 };
            let mut want = V2::ZERO;
            match kind {
                SlopKind::Listicle => {
                    want = if s.fleeing > 0 { -dir * (def.speed * 1.3) } else { dir * def.speed };
                }
                SlopKind::BubbleBot => {
                    want = if dist > BUBBLE_KEEP_FAR {
                        dir * def.speed
                    } else if dist < BUBBLE_KEEP_NEAR {
                        -dir * def.speed
                    } else {
                        dir.perp() * (side * def.speed * 0.5)
                    };
                    s.timer -= 1;
                    if s.timer <= 0 && dist < SHOOT_RANGE {
                        s.timer = BUBBLE_INTERVAL + self.rng_ai.below(40) as i32;
                        for k in -1..=1 {
                            let aim = dir.rotated(k as f32 * BUBBLE_SPREAD);
                            // Only the middle bubble talks, so a crowd stays readable.
                            shots.push((s.pos + aim * def.radius, aim * BUBBLE_SPEED, BUBBLE_DAMAGE * s.agreement(), Look::Bubble, k == 0));
                        }
                    }
                }
                SlopKind::EmDashLancer => {
                    if s.charging > 0 {
                        s.charging -= 1;
                        if s.charging == 0 {
                            shots.push((s.pos + s.aim * def.radius, s.aim * LANCE_SPEED, LANCE_DAMAGE * s.agreement(), Look::EmDash, false));
                            s.timer = LANCE_RECOVER;
                        }
                    } else if s.timer > 0 {
                        s.timer -= 1;
                        want = dir.perp() * (side * def.speed * 0.6);
                    } else if dist < LANCE_RANGE {
                        s.charging = LANCE_AIM;
                        s.aim = dir;
                        self.events.push(Event::LancerAim { from: s.pos, dir });
                    } else {
                        want = dir * def.speed;
                    }
                }
                SlopKind::YesBot => {
                    want = match friend {
                        Some(f) if f.distance(s.pos) > 70.0 => (f - s.pos).normalized() * def.speed,
                        Some(_) => V2::ZERO,
                        None if dist > 260.0 => dir * def.speed,
                        None => V2::ZERO,
                    };
                    s.timer -= 1;
                    if s.timer <= 0 {
                        s.timer = AGREE_INTERVAL;
                        if !nearby.is_empty() {
                            let line = self.rng_ai.below(AGREE_LINES.len() as u32) as u16;
                            self.events.push(Event::Agreed { at: s.pos, line });
                            agree.extend(nearby);
                        }
                    }
                }
                SlopKind::SixFingerHand => {
                    if let Some(offset) = s.grab {
                        s.pos = player + offset;
                        s.moving = V2::ZERO;
                        continue;
                    }
                    want = dir * def.speed;
                }
            }
            s.moving = want;
            s.pos = clamp_to_arena(s.pos + (want + s.push) * DT, def.radius);
            s.push *= KNOCKBACK_DECAY;
        }
        for j in agree {
            let s = &mut self.slop[j];
            s.agreed = (s.agreed + 1).min(AGREE_MAX);
            s.agreed_ticks = AGREE_TICKS;
        }
        let bubbles = self.projectiles.iter().filter(|p| p.look == Look::Bubble).count();
        for (pos, vel, damage, look, talks) in shots {
            if look == Look::Bubble && bubbles >= MAX_BUBBLES {
                continue;
            }
            let id = self.id();
            let line = if talks { self.rng_ai.below(BUBBLE_LINES.len() as u32) as u16 } else { NO_LINE };
            let (radius, life) = if look == Look::Bubble { (BUBBLE_RADIUS, BUBBLE_LIFE) } else { (LANCE_RADIUS, LANCE_LIFE) };
            self.projectiles.push(Projectile {
                id,
                pos,
                vel,
                radius,
                damage,
                kind: DamageType::Analog,
                hostile: true,
                pierce: 0,
                bounces: 0,
                mark: None,
                look,
                life,
                line,
                last_hit: 0,
                knockback: 0.0,
            });
        }
    }

    /// Pushes overlapping slop apart so crowds spread out instead of stacking.
    fn separate(&mut self) {
        let mut shove = vec![V2::ZERO; self.slop.len()];
        for (i, s) in self.slop.iter().enumerate() {
            if s.grab.is_some() {
                continue;
            }
            self.grid.for_each_near(s.pos, s.radius * 2.0 + 8.0, |j| {
                if j == i {
                    return;
                }
                let o = &self.slop[j];
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
            if s.grab.is_none() {
                s.pos = clamp_to_arena(s.pos + push.clamp_length(6.0), s.kind.def().radius);
            }
        }
    }

    /// Slop touching you: contact damage, and hands that grab (SLOP-02).
    fn touch(&mut self) {
        let p = self.player.pos;
        let mut holding = self.slop.iter().filter(|s| s.grab.is_some()).count() as u8;
        let mut contact = Vec::new();
        let mut grab_hits = 0;
        for s in &mut self.slop {
            let def = s.kind.def();
            let touching = s.pos.distance_sq(p) < (def.radius + PLAYER_RADIUS) * (def.radius + PLAYER_RADIUS);
            if s.kind == SlopKind::SixFingerHand {
                if s.grab.is_some() {
                    s.timer -= 1;
                    if s.timer <= 0 {
                        s.timer = GRAB_INTERVAL;
                        grab_hits += 1;
                    }
                } else if touching && holding < MAX_GRABS {
                    s.grab = Some((s.pos - p).clamp_length(PLAYER_RADIUS + 6.0));
                    s.timer = GRAB_INTERVAL;
                    holding += 1;
                    self.events.push(Event::Grabbed);
                }
                continue;
            }
            if touching {
                contact.push(def.contact * s.agreement());
            }
        }
        self.player.grabbed = holding;
        for dmg in contact {
            self.hurt(dmg);
        }
        for _ in 0..grab_hits {
            let amount = self.player.stats.damage_taken(GRAB_DAMAGE);
            self.player.mental_health -= amount;
            self.tally.damage_taken += amount;
            self.events.push(Event::PlayerHurt { amount });
        }
    }

    /// A hit on you: Scepticism may ignore it, Composure softens it, and you
    /// are briefly invulnerable afterwards (FAIR-03).
    fn hurt(&mut self, raw: f32) {
        if self.player.invulnerable > 0 || raw <= 0.0 {
            return;
        }
        if self.rng_combat.chance(self.player.stats.dodge_chance()) {
            self.player.invulnerable = DODGE_INVULNERABLE;
            self.tally.dodges += 1;
            self.events.push(Event::Dodged);
            return;
        }
        let amount = self.player.stats.damage_taken(raw);
        self.player.mental_health -= amount;
        self.player.invulnerable = HIT_INVULNERABLE;
        self.tally.damage_taken += amount;
        self.events.push(Event::PlayerHurt { amount });
    }

    /// Every weapon whose cooldown is up attacks the nearest slop (WPN-01).
    fn fire_weapons(&mut self) {
        let stats = self.player.stats;
        let at = self.player.pos;
        let mut melee: Vec<(usize, f32, DamageType, V2)> = Vec::new();
        for wi in 0..self.player.weapons.len() {
            let w = &mut self.player.weapons[wi];
            if !w.enabled {
                continue;
            }
            w.cooldown -= 1.0 + stats.focus;
            if w.cooldown > 0.0 {
                continue;
            }
            let def = w.def();
            let range = def.range * (1.0 + stats.reach);
            let Some(target) = nearest_slop(&self.grid, &self.slop, at, range, None) else {
                // Ready, waiting for something in range.
                w.cooldown = 0.0;
                continue;
            };
            w.cooldown += def.cooldown * TICK_RATE as f32;
            let damage = def.damage * TIER_DAMAGE[w.tier as usize - 1];
            let aim = (self.slop[target].pos - at).normalized();
            let (speed, radius, pierce, bounces, mark, look) = match def.pattern {
                Pattern::Shot { speed, radius, pierce, look } => (speed, radius, pierce, 0, None, look),
                Pattern::Mark { speed, radius, pierce, bonus, seconds } => {
                    (speed, radius, pierce, 0, Some((bonus, (seconds * TICK_RATE as f32) as u16)), Look::Pen)
                }
                Pattern::Ricochet { speed, radius, bounces } => (speed, radius, 0, bounces, None, Look::EyeRoll),
                Pattern::Arc { half_angle } | Pattern::Cone { half_angle } => {
                    let dir = aim;
                    let cos = crate::math::cos(half_angle);
                    self.grid.for_each_near(at, range + 30.0, |j| {
                        let s = &self.slop[j];
                        let d = s.pos - at;
                        let len = d.length();
                        if s.hp > 0.0 && len <= range + s.radius && (len < s.radius + PLAYER_RADIUS || d.dot(dir) >= cos * len) {
                            melee.push((j, damage, def.kind, d.normalized() * def.knockback));
                        }
                    });
                    self.events.push(Event::Swing { at, angle: dir.angle(), range, half_angle, weapon: wi as u8 });
                    continue;
                }
            };
            let id = self.next_id + 1;
            self.next_id = id;
            self.projectiles.push(Projectile {
                id,
                pos: at + aim * 20.0,
                vel: aim * speed,
                radius,
                damage,
                kind: def.kind,
                hostile: false,
                pierce,
                bounces,
                mark,
                look,
                life: (range / speed * TICK_RATE as f32 * 1.15) as u16 + 4,
                line: 0,
                last_hit: 0,
                knockback: def.knockback,
            });
            self.events.push(Event::Fired { weapon: wi as u8 });
        }
        for (j, damage, kind, push) in melee {
            damage_slop(&mut self.slop[j], damage, kind, push, &stats, self.tick, &mut self.tally, &mut self.events);
        }
    }

    fn move_projectiles(&mut self) {
        let player = self.player.pos;
        let stats = self.player.stats;
        let tick = self.tick;
        let mut hits_on_you = Vec::new();
        for i in 0..self.projectiles.len() {
            let p = &mut self.projectiles[i];
            p.pos += p.vel * DT;
            p.life = p.life.saturating_sub(1);
            if p.pos.x < -50.0 || p.pos.y < -50.0 || p.pos.x > ARENA + 50.0 || p.pos.y > ARENA + 50.0 {
                p.life = 0;
            }
            if p.life == 0 {
                continue;
            }
            if p.hostile {
                let reach = p.radius + PLAYER_RADIUS * 0.8;
                if p.pos.distance_sq(player) < reach * reach {
                    hits_on_you.push(p.damage);
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
            let Some(j) = hit else { continue };
            {
                let s = &mut self.slop[j];
                damage_slop(s, p.damage, p.kind, p.vel.normalized() * p.knockback, &stats, tick, &mut self.tally, &mut self.events);
                if let Some((bonus, ticks)) = p.mark {
                    s.marked = ticks;
                    s.mark_bonus = bonus;
                }
                p.last_hit = s.id;
            }
            if p.bounces > 0 {
                p.bounces -= 1;
                match nearest_slop(&self.grid, &self.slop, p.pos, RICOCHET_RANGE, Some(j)) {
                    Some(k) => {
                        let speed = p.vel.length();
                        p.vel = (self.slop[k].pos - p.pos).normalized() * speed;
                        p.life = p.life.max(40);
                    }
                    None => p.life = 0,
                }
            } else if p.pierce > 0 {
                p.pierce -= 1;
            } else {
                p.life = 0;
            }
        }
        self.pop_bubbles();
        for dmg in hits_on_you {
            self.hurt(dmg);
        }
        self.projectiles.retain(|p| p.life > 0);
    }

    /// Your projectiles pop speech bubbles they pass through (BUB-01).
    fn pop_bubbles(&mut self) {
        let mine: Vec<(V2, f32)> =
            self.projectiles.iter().filter(|p| !p.hostile && p.life > 0).map(|p| (p.pos, p.radius)).collect();
        if mine.is_empty() {
            return;
        }
        let mut dropped = Vec::new();
        for b in self.projectiles.iter_mut().filter(|p| p.look == Look::Bubble && p.life > 0) {
            if mine.iter().any(|(at, r)| at.distance_sq(b.pos) < (r + b.radius) * (r + b.radius)) {
                b.life = 0;
                self.tally.bubbles_popped += 1;
                self.events.push(Event::BubblePopped { at: b.pos, line: b.line });
                if self.rng_combat.below(POP_LIKE_ODDS) == 0 {
                    dropped.push(b.pos);
                }
            }
        }
        for at in dropped {
            let id = self.id();
            self.likes.push(Like { id, pos: at, value: 1, pulled: false });
        }
    }

    /// Removes dead slop, dropping their Likes (SLOP-03 for Listicles).
    fn bury(&mut self) {
        let mut scatter = Vec::new();
        let mut drops = Vec::new();
        for s in self.slop.iter().filter(|s| s.hp <= 0.0) {
            self.tally.kills += 1;
            drops.push((s.pos, s.kind.def().likes));
            self.events.push(Event::SlopDied { at: s.pos, kind: s.kind });
            if s.kind == SlopKind::Listicle && s.number == 1 {
                scatter.push((s.pack, s.pos));
            }
        }
        self.slop.retain(|s| s.hp > 0.0);
        for (at, value) in drops {
            let jitter = V2::new(self.rng_combat.range(-6.0, 6.0), self.rng_combat.range(-6.0, 6.0));
            let id = self.id();
            self.likes.push(Like { id, pos: at + jitter, value, pulled: false });
        }
        for (pack, at) in scatter {
            let mut any = false;
            for s in self.slop.iter_mut().filter(|s| s.kind == SlopKind::Listicle && s.pack == pack) {
                s.fleeing = SCATTER_TICKS;
                any = true;
            }
            if any {
                self.events.push(Event::Scattered { at });
            }
        }
    }

    /// Likes within Curiosity fly to you and become Clout (LIKE-01).
    fn collect_likes(&mut self) {
        let p = self.player.pos;
        let curiosity = self.player.stats.curiosity;
        let step = LIKE_PULL_SPEED * DT;
        let mut got = 0;
        self.likes.retain_mut(|l| {
            let d = l.pos.distance(p);
            if d < curiosity {
                l.pulled = true;
            }
            if !l.pulled {
                return true;
            }
            if d <= step + PLAYER_RADIUS {
                got += l.value;
                return false;
            }
            l.pos += (p - l.pos).normalized() * step;
            true
        });
        if got > 0 {
            self.clout += got;
            self.tally.likes_collected += got;
            self.events.push(Event::LikeCollected { value: got });
        }
    }

    fn tick_timers(&mut self) {
        self.player.invulnerable = self.player.invulnerable.saturating_sub(1);
        for s in &mut self.slop {
            s.flash = s.flash.saturating_sub(1);
            s.marked = s.marked.saturating_sub(1);
            s.fleeing = s.fleeing.saturating_sub(1);
            if s.agreed_ticks > 0 {
                s.agreed_ticks -= 1;
                if s.agreed_ticks == 0 {
                    s.agreed = 0;
                }
            }
        }
    }

    /// Burnout at 0 Mental Health; the wave is cleared when the timer runs
    /// out, and Likes still on the floor count half (WAVE-01).
    fn check_end(&mut self) {
        if self.player.mental_health <= 0.0 {
            self.player.mental_health = 0.0;
            self.outcome = Outcome::Burnout;
            self.events.push(Event::Burnout);
        } else if self.tick >= (self.wave.seconds * TICK_RATE as f32) as u32 {
            let left: u32 = self.likes.iter().map(|l| l.value).sum();
            let credited = (left as f32 * LEFTOVER_SHARE) as u32;
            self.clout += credited;
            self.tally.likes_collected += credited;
            self.likes.clear();
            self.warnings.clear();
            self.outcome = Outcome::Cleared;
            self.events.push(Event::WaveCleared);
        }
    }

    /// A fingerprint of the state, to prove two runs are identical (SIM-01).
    pub fn checksum(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut mix = |v: u32| {
            for b in v.to_le_bytes() {
                h ^= b as u64;
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        mix(self.tick);
        mix(self.clout);
        mix(self.player.pos.x.to_bits());
        mix(self.player.pos.y.to_bits());
        mix(self.player.mental_health.to_bits());
        for s in &self.slop {
            mix(s.id);
            mix(s.pos.x.to_bits());
            mix(s.pos.y.to_bits());
            mix(s.hp.to_bits());
        }
        for p in &self.projectiles {
            mix(p.id);
            mix(p.pos.x.to_bits());
            mix(p.pos.y.to_bits());
        }
        mix(self.likes.len() as u32);
        h
    }
}
