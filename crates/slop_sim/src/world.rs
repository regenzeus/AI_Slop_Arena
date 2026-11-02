//! The arena: one human against a wave of slop, stepped at a fixed 60 ticks
//! per second (SIM-01). Everything the client shows is read from here, and a
//! seed plus the input of every tick replays a wave exactly.
//!
//! The work of a tick is split over modules: `ai` (slop), `combat` (weapons,
//! projectiles, zones), `player` (you, statuses, actives), `hazards` and
//! `boss`. This module holds the state, spawning and the end of a wave.

use crate::content::{Archetype, BossKind, Look, Platform, WeaponDef, WEAPONS};
use crate::events::Event;
use crate::rng::Rng;
use crate::spatial::Grid;
use crate::spec::{Movement, SlopLook, SlopSpec, Traits};
use crate::stats::Stats;
use crate::vec::V2;

pub const TICK_RATE: u32 = 60;
pub const DT: f32 = 1.0 / TICK_RATE as f32;
/// Side of the square arena in world units; the client shows 1600 x 900 of it.
pub const ARENA: f32 = 2400.0;
pub const PLAYER_RADIUS: f32 = 16.0;
/// Base movement speed in units per second.
pub const PLAYER_SPEED: f32 = 300.0;
/// Slop only shoots when this close to you: roughly "on screen" (FAIR-01).
pub const SHOOT_RANGE: f32 = 760.0;
pub const MAX_SLOP: usize = 400;
pub const MAX_BUBBLES: usize = 300;
/// Ticks a spawn is telegraphed before the slop appears (FAIR-02).
pub const SPAWN_WARNING: u16 = 60;
/// Ticks a health bar stays after the last hit.
pub const HEALTH_BAR_TICKS: u32 = 180;
/// Ticks a slop flashes white after a hit.
pub const FLASH_TICKS: u8 = 4;
pub const BUBBLE_RADIUS: f32 = 13.0;
/// Ticks an em-dash is aimed before it is thrown (FAIR-02).
pub const LANCE_AIM: u16 = 48;
/// Ticks a bomb's landing spot is marked before it explodes (FAIR-02).
pub const BOMB_WARNING: u16 = 60;
/// Line index of a bubble that says nothing ("...").
pub const NO_LINE: u16 = u16::MAX;
/// Spec index used by projectiles nobody owns.
pub const NO_SPEC: u16 = u16::MAX;
const SPAWN_MIN: f32 = 700.0;
const SPAWN_MAX: f32 = 920.0;
const SPAWN_CLOSEST: f32 = 480.0;
/// No new spawns in the last seconds of a timed wave.
const SPAWN_STOP: f32 = 2.0;
const LIKE_PULL_SPEED: f32 = 700.0;
/// Share of uncollected Likes credited when a wave is cleared.
const LEFTOVER_SHARE: f32 = 0.5;
const SCATTER_TICKS: u16 = 90;
const REPOST_HP: f32 = 0.6;

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
    /// Angle of an orbiting weapon.
    pub phase: f32,
}

impl Weapon {
    pub fn new(def: usize, tier: u8) -> Weapon {
        Weapon { def, tier: tier.clamp(1, 4), cooldown: 0.0, enabled: true, phase: 0.0 }
    }

    pub fn def(&self) -> &'static WeaponDef {
        &WEAPONS[self.def]
    }
}

/// Items that fire on their own when their trigger is met (ACT-01).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Active {
    /// Dash away when surrounded.
    LogOff,
    /// Delete a bubble about to hit you.
    BlockButton,
    /// Clear your statuses and push slop back.
    DigitalDetox,
    /// Low on Mental Health: every slop on screen loses HP and slows.
    Unsubscribe,
    /// A friend runs through an elite or boss.
    PhoneAFriend,
}

impl Active {
    pub fn cooldown(self) -> f32 {
        match self {
            Active::LogOff => 12.0,
            Active::BlockButton => 4.0,
            Active::DigitalDetox => 20.0,
            Active::Unsubscribe => 45.0,
            Active::PhoneAFriend => 25.0,
        }
    }
}

/// What your items and hero do inside a wave, beyond plain stats.
#[derive(Clone, Debug, Default)]
pub struct Perks {
    /// Extra damage per damage type, as fractions.
    pub type_bonus: [f32; 4],
    /// Extra damage against generated brainrot (Artist).
    pub brainrot_bonus: f32,
    /// Extra mark bonus from the Red Pen (Teacher).
    pub mark_bonus: f32,
    /// Extra Likes per drop, as a fraction.
    pub like_bonus: f32,
    /// Coffee: an Anxiety stack every 20 s per cup.
    pub coffee: u8,
    /// Doomscrolling: Mental Health drains while slop is on screen.
    pub doomscrolling: u8,
    /// Group Chat: +1% damage per 5 slop on screen, per chat.
    pub group_chat: u8,
    /// Noise-Cancelling Headphones: block a bubble every 3 s.
    pub noise_cancel: bool,
    pub ad_blocker: bool,
    /// Good Night's Sleep: survive killing blows, per wave.
    pub second_wind: u8,
    /// Houseplant: heal per Grass kill.
    pub houseplant: u8,
    /// Therapy Dog with a Dog Walk: block a hit every 10 s.
    pub therapy_dog: bool,
    /// Hot Take: slop near you hits 10% harder.
    pub hot_take: bool,
    /// Dumbphone: never Doomscrolled.
    pub dumbphone: bool,
    /// Boomer: a chain letter every 10 s.
    pub chain_letter: bool,
    /// Zoomer: immune to Brainrot, pop-ups close twice as fast.
    pub fluent: bool,
    pub actives: Vec<Active>,
}

/// What you bring into a wave.
#[derive(Clone, Debug)]
pub struct Loadout {
    /// Weapon index and tier.
    pub weapons: Vec<(usize, u8)>,
    pub stats: Stats,
    pub perks: Perks,
}

impl Loadout {
    /// Only these weapons, plain stats.
    pub fn with_weapons(weapons: &[usize]) -> Loadout {
        Loadout { weapons: weapons.iter().map(|&w| (w, 1)).collect(), stats: Stats::default(), perks: Perks::default() }
    }
}

/// Status effects the slop puts on you (STATUS-01), in ticks left.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Status {
    pub anxiety: u8,
    pub anxiety_ticks: u16,
    /// Dragged forward.
    pub doomscroll: u16,
    /// A fake dialog covers part of the screen.
    pub popup: u16,
    /// Your damage is 20% lower.
    pub ratio: u16,
    /// Controls are reversed.
    pub brainrot: u16,
    /// Brainrot is about to hit: telegraphed above you.
    pub brainrot_warning: u16,
}

impl Status {
    pub fn count(&self) -> usize {
        [self.anxiety > 0, self.doomscroll > 0, self.popup > 0, self.ratio > 0, self.brainrot > 0].iter().filter(|b| **b).count()
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
    pub perks: Perks,
    pub invulnerable: u16,
    pub weapons: Vec<Weapon>,
    /// Hands holding on right now.
    pub grabbed: u8,
    pub status: Status,
    /// Seconds until each active may fire again.
    pub active_cooldown: Vec<f32>,
    pub second_winds: u8,
    /// Ticks of a Log Off dash left, and its direction.
    pub dash: u16,
    pub dash_dir: V2,
    pub(crate) dog_block: u16,
    pub(crate) noise_block: u16,
    pub(crate) coffee_timer: u16,
    pub(crate) letter_timer: u16,
}

impl Player {
    pub fn new(loadout: &Loadout) -> Player {
        let stats = loadout.stats;
        Player {
            pos: V2::new(ARENA / 2.0, ARENA / 2.0),
            vel: V2::ZERO,
            facing: V2::new(1.0, 0.0),
            mental_health: stats.max_mental_health,
            stats,
            perks: loadout.perks.clone(),
            invulnerable: 0,
            weapons: loadout.weapons.iter().map(|&(d, t)| Weapon::new(d, t)).collect(),
            grabbed: 0,
            status: Status::default(),
            active_cooldown: vec![0.0; loadout.perks.actives.len()],
            second_winds: loadout.perks.second_wind,
            dash: 0,
            dash_dir: V2::ZERO,
            dog_block: 0,
            noise_block: 0,
            coffee_timer: 0,
            letter_timer: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Slop {
    pub id: u32,
    /// Index into the wave's specs.
    pub spec: u16,
    pub pos: V2,
    /// Velocity it chose this tick, for drawing.
    pub moving: V2,
    /// Knockback velocity, decaying.
    pub push: V2,
    pub radius: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub resist: [f32; 4],
    /// Countdowns used by each kind's behaviour and attack.
    pub timer: i32,
    pub attack_timer: i32,
    /// Ticks left aiming (Clawed).
    pub charging: u16,
    pub aim: V2,
    pub marked: u16,
    pub mark_bonus: f32,
    /// Stacks of Yes-Bot agreement and ticks they last.
    pub agreed: u8,
    pub agreed_ticks: u16,
    pub last_hit: Option<u32>,
    /// Damage type of the last hit, which counts for the killing blow.
    pub last_type: u8,
    pub flash: u8,
    /// Listicle pack and number in it.
    pub pack: u32,
    pub number: u8,
    pub fleeing: u16,
    /// Offset from you while a Six-Finger Hand holds on.
    pub grab: Option<V2>,
    pub stunned: u16,
    /// Ticks its beauty filter is switched off by a fact-check.
    pub shield_off: u16,
    /// Damage taken is multiplied by this (bosses in invulnerable phases).
    pub armor: f32,
    /// Size growth of a Karma Farmer.
    pub grown: f32,
    pub reposted: bool,
    pub dodged: bool,
    pub split: bool,
    pub born: u32,
    /// Mental Health it took from you, for training.
    pub dealt: f32,
}

impl Slop {
    /// Damage multiplier from Yes-Bot agreement.
    pub fn agreement(&self) -> f32 {
        1.0 + 0.1 * self.agreed as f32
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
    pub kind: crate::stats::DamageType,
    /// Fired by slop at you.
    pub hostile: bool,
    pub pierce: u8,
    pub bounces: u8,
    /// Bonus and ticks of the mark it leaves.
    pub mark: Option<(f32, u16)>,
    pub look: Look,
    pub life: u16,
    /// Index into the line table of its look, or `NO_LINE`.
    pub line: u16,
    /// Id of the slop hit last, so piercing and bouncing never hit twice in a row.
    pub last_hit: u32,
    pub knockback: f32,
    /// Splash radius where it lands (lobbed); it hits nothing on the way.
    pub splash: f32,
    /// Comes back to you (Frisbee).
    pub boomerang: bool,
    /// Ticks of stun it gives chatbots.
    pub stun: u16,
    /// Spec of the slop that fired it, for training.
    pub spec: u16,
    /// Pushes you back when it hits (posts).
    pub shove: f32,
    /// Drags you forward when it hits (arrows).
    pub doomscroll: bool,
    /// Your weapon slot that fired it, for the report; `u8::MAX` for none.
    pub weapon: u8,
}

impl Projectile {
    /// A projectile with every optional behaviour switched off.
    #[allow(clippy::too_many_arguments)]
    pub fn new(id: u32, pos: V2, vel: V2, radius: f32, damage: f32, kind: crate::stats::DamageType, hostile: bool, look: Look, life: u16) -> Projectile {
        Projectile {
            id,
            pos,
            vel,
            radius,
            damage,
            kind,
            hostile,
            pierce: 0,
            bounces: 0,
            mark: None,
            look,
            life,
            line: NO_LINE,
            last_hit: 0,
            knockback: 0.0,
            splash: 0.0,
            boomerang: false,
            stun: 0,
            spec: NO_SPEC,
            shove: 0.0,
            doomscroll: false,
            weapon: u8::MAX,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Like {
    pub id: u32,
    pub pos: V2,
    pub value: u32,
    /// Flying towards you.
    pub pulled: bool,
    /// A Hallucinator's fake: explodes into Anxiety.
    pub fake: bool,
}

/// A slop about to appear.
#[derive(Clone, Debug)]
pub struct Warning {
    pub pos: V2,
    pub spec: u16,
    pub ticks: u16,
    pub pack: u32,
    pub number: u8,
    pub elite: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ZoneKind {
    /// Hurts slop inside and heals you (Seed Bomb).
    Seed { heal: f32 },
    /// Explodes when its time runs out.
    Bomb,
    /// Slows you inside (Thredd).
    Downvote,
    /// A band across the arena that pushes you (LinkedOut).
    Banner,
    /// Slop inside take less damage (InstaSlop).
    Filter,
}

#[derive(Clone, Debug)]
pub struct Zone {
    pub id: u32,
    pub kind: ZoneKind,
    pub pos: V2,
    pub radius: f32,
    pub ticks: u16,
    pub max_ticks: u16,
    pub vel: V2,
    pub damage: f32,
    pub spec: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Running,
    Cleared,
    Burnout,
}

/// One entry of a wave's spawn table.
#[derive(Clone, Copy, Debug)]
pub struct Spawn {
    /// Index into `WaveSetup::specs`.
    pub spec: usize,
    pub weight: u32,
    /// Seconds into the wave before it can appear.
    pub from: f32,
    /// How many appear together (a pack or a swarm).
    pub group: u8,
    /// Spawn credits it costs: its base HP over 20, for the whole group.
    pub cost: f32,
}

/// The Model's view of your run, for the final boss.
#[derive(Clone, Copy, Debug, Default)]
pub struct ModelSetup {
    /// 0 to 1: how evenly your kills spread over damage types (BOSS-07).
    pub collapse: f32,
    /// Looks of your three most-used weapons, which it copies.
    pub copies: [Option<Look>; 3],
    /// Parts of the Supreme Brainrot: the brainrot that did best against you.
    pub parts: Option<(crate::content::Body, crate::content::Object, crate::content::Accessory)>,
}

/// Everything a wave is made of; built by the rules between waves.
#[derive(Clone, Debug)]
pub struct WaveSetup {
    pub number: u32,
    /// Seconds to survive; 0 means it lasts until the boss dies.
    pub seconds: f32,
    pub platform: Option<Platform>,
    pub specs: Vec<SlopSpec>,
    pub spawns: Vec<Spawn>,
    /// Spawn credits per second at the start and the end of the wave.
    pub rate: (f32, f32),
    pub elite_chance: f32,
    /// Spec of the boss, placed at the start.
    pub boss: Option<usize>,
    /// Share of dying slop that come back as reposts (Thredd).
    pub repost_chance: f32,
    pub model: ModelSetup,
}

impl WaveSetup {
    /// Every archetype as spec `a as usize`, nothing spawning, no time limit.
    pub fn sandbox() -> WaveSetup {
        WaveSetup {
            number: 1,
            seconds: 3600.0,
            platform: None,
            specs: Archetype::ALL.iter().map(|&a| SlopSpec::archetype(a)).collect(),
            spawns: Vec::new(),
            rate: (0.0, 0.0),
            elite_chance: 0.0,
            boss: None,
            repost_chance: 0.0,
            model: ModelSetup::default(),
        }
    }

    /// The milestone 1 test wave: chatbots and hands for 60 seconds.
    pub fn first() -> WaveSetup {
        let mut w = WaveSetup::sandbox();
        w.seconds = 60.0;
        w.rate = (1.2, 4.0);
        w.spawns = vec![
            Spawn { spec: Archetype::ListicleDrone as usize, weight: 4, from: 0.0, group: 5, cost: 2.75 },
            Spawn { spec: Archetype::ChadGpt as usize, weight: 3, from: 0.0, group: 1, cost: 1.3 },
            Spawn { spec: Archetype::Clawed as usize, weight: 2, from: 10.0, group: 1, cost: 1.7 },
            Spawn { spec: Archetype::SixFingerHand as usize, weight: 2, from: 15.0, group: 1, cost: 0.7 },
            Spawn { spec: Archetype::YesBot as usize, weight: 1, from: 20.0, group: 1, cost: 1.2 },
        ];
        w
    }

    pub fn is_boss_wave(&self) -> bool {
        self.boss.is_some()
    }
}

/// How one kind of slop did in the wave, for training (EVO-01).
#[derive(Clone, Copy, Debug, Default)]
pub struct SpecTally {
    pub spawned: u32,
    /// Mental Health it took from you.
    pub damage: f32,
    /// Ticks its members stayed alive.
    pub alive_ticks: u32,
}

/// What happened in the wave, for the report and for training.
#[derive(Clone, Debug, Default)]
pub struct Tally {
    pub kills: u32,
    pub kills_by_type: [u32; 4],
    pub damage_dealt: [f32; 4],
    /// Damage dealt per weapon slot.
    pub weapon_damage: Vec<f32>,
    pub damage_taken: f32,
    pub dodges: u32,
    pub bubbles_popped: u32,
    pub likes_collected: u32,
    pub loot_boxes: u32,
    /// Sum and count of your distance to the nearest slop, sampled each second.
    pub distance_sum: f32,
    pub distance_samples: u32,
}

/// The boss fight in progress (BOSS-01).
#[derive(Clone, Debug)]
pub struct BossState {
    pub id: u32,
    pub kind: BossKind,
    pub phase: u8,
    pub timer: i32,
    pub timer2: i32,
    pub regenerated: bool,
    /// Spec of the slop it summons.
    pub minion: usize,
    /// Ticks of an unskippable ad (Content Farm) or of thinking (The Model).
    pub shielded: u16,
}

pub struct World {
    pub seed: u64,
    pub tick: u32,
    pub setup: WaveSetup,
    pub player: Player,
    pub slop: Vec<Slop>,
    pub projectiles: Vec<Projectile>,
    pub likes: Vec<Like>,
    pub warnings: Vec<Warning>,
    pub zones: Vec<Zone>,
    pub boss: Option<BossState>,
    /// Currency earned this wave.
    pub clout: u32,
    pub outcome: Outcome,
    pub tally: Tally,
    pub spec_tally: Vec<SpecTally>,
    /// Global slop speed multiplier (the Hustle Guru grinds it up).
    pub hustle: f32,
    /// Ticks of a mid-roll ad (MeTube): the client dims the arena.
    pub ad_break: u16,
    /// Ticks since the last beat (ClockTok).
    pub beat: u16,
    pub(crate) next_id: u32,
    pub(crate) next_pack: u32,
    pub(crate) spawn_credit: f32,
    pub(crate) hazard_timer: i32,
    /// Slop to place after this tick's damage: (spec, position, HP or 0 for full, pack, number).
    pub(crate) pending: Vec<(usize, V2, f32, u32, u8)>,
    /// Kills counted the last time Karma Farmers grew.
    pub(crate) karma_seen: u32,
    /// Slop on screen this tick (Group Chat).
    pub(crate) crowd: u32,
    pub(crate) grid: Grid,
    pub(crate) events: Vec<Event>,
    pub(crate) rng_spawn: Rng,
    pub(crate) rng_combat: Rng,
    pub(crate) rng_ai: Rng,
}

pub(crate) fn clamp_to_arena(p: V2, margin: f32) -> V2 {
    V2::new(p.x.clamp(margin, ARENA - margin), p.y.clamp(margin, ARENA - margin))
}

/// The nearest living slop within `range` of `at`, other than `skip`.
pub(crate) fn nearest_slop(grid: &Grid, slop: &[Slop], at: V2, range: f32, skip: Option<usize>) -> Option<usize> {
    let mut best: Option<(usize, f32)> = None;
    grid.for_each_near(at, range + 40.0, |j| {
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

impl World {
    pub fn new(seed: u64, setup: WaveSetup, loadout: &Loadout) -> World {
        let n = setup.specs.len();
        let tally = Tally { weapon_damage: vec![0.0; loadout.weapons.len()], ..Tally::default() };
        let mut w = World {
            seed,
            tick: 0,
            player: Player::new(loadout),
            slop: Vec::new(),
            projectiles: Vec::new(),
            likes: Vec::new(),
            warnings: Vec::new(),
            zones: Vec::new(),
            boss: None,
            clout: 0,
            outcome: Outcome::Running,
            tally,
            spec_tally: vec![SpecTally::default(); n],
            hustle: 1.0,
            ad_break: 0,
            beat: 0,
            next_id: 1,
            next_pack: 1,
            spawn_credit: 0.0,
            hazard_timer: 240,
            pending: Vec::new(),
            karma_seen: 0,
            crowd: 0,
            grid: Grid::new(),
            events: Vec::new(),
            rng_spawn: Rng::new(seed, 1),
            rng_combat: Rng::new(seed, 2),
            rng_ai: Rng::new(seed, 3),
            setup,
        };
        if let Some(b) = w.setup.boss {
            w.start_boss(b);
        }
        w
    }

    /// An arena with no spawning and no time limit, holding the given weapons.
    pub fn sandbox(seed: u64, weapons: &[usize]) -> World {
        World::new(seed, WaveSetup::sandbox(), &Loadout::with_weapons(weapons))
    }

    pub fn elapsed(&self) -> f32 {
        self.tick as f32 * DT
    }

    /// Share of the wave survived, 0 to 1; boss waves follow the boss's health.
    pub fn progress(&self) -> f32 {
        if self.setup.seconds <= 0.0 {
            return self.boss_health().map_or(1.0, |h| 1.0 - h);
        }
        (self.elapsed() / self.setup.seconds).min(1.0)
    }

    /// The boss's health as a share of its maximum, while it lives.
    pub fn boss_health(&self) -> Option<f32> {
        let b = self.boss.as_ref()?;
        self.slop.iter().find(|s| s.id == b.id).map(|s| (s.hp / s.max_hp).clamp(0.0, 1.0))
    }

    pub fn spec(&self, s: &Slop) -> &SlopSpec {
        &self.setup.specs[s.spec as usize]
    }

    pub fn drain_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    pub(crate) fn id(&mut self) -> u32 {
        self.next_id += 1;
        self.next_id
    }

    /// Places a slop of an archetype right away (it must be in the specs, as
    /// in the sandbox). Returns its id.
    pub fn spawn_archetype(&mut self, a: Archetype, at: V2) -> u32 {
        let spec = self
            .setup
            .specs
            .iter()
            .position(|s| s.look == SlopLook::Archetype(a))
            .unwrap_or_else(|| {
                self.setup.specs.push(SlopSpec::archetype(a));
                self.spec_tally.push(SpecTally::default());
                self.setup.specs.len() - 1
            });
        self.place(spec, at, 0, 1)
    }

    /// Places a slop of spec `spec` right away. Returns its id.
    pub fn place(&mut self, spec: usize, at: V2, pack: u32, number: u8) -> u32 {
        let id = self.id();
        let sp = &self.setup.specs[spec];
        let timer = match sp.movement {
            Movement::Follow => 90,
            Movement::Stand if sp.traits.has(Traits::HATCH) => 180,
            _ => 0,
        };
        let attack_timer = sp.attack.map_or(0, |a| 40 + self.rng_spawn.below(a.interval() as u32) as i32);
        let fake = if sp.traits.has(Traits::FAKE_LIKES) { 120 + self.rng_spawn.below(120) as i32 } else { timer };
        self.slop.push(Slop {
            id,
            spec: spec as u16,
            pos: clamp_to_arena(at, sp.radius),
            moving: V2::ZERO,
            push: V2::ZERO,
            radius: sp.radius,
            hp: sp.hp,
            max_hp: sp.hp,
            resist: sp.resist,
            timer: fake,
            attack_timer,
            charging: 0,
            aim: V2::ZERO,
            marked: 0,
            mark_bonus: 0.0,
            agreed: 0,
            agreed_ticks: 0,
            last_hit: None,
            last_type: 0,
            flash: 0,
            pack,
            number,
            fleeing: 0,
            grab: None,
            stunned: 0,
            shield_off: 0,
            armor: 1.0,
            grown: 1.0,
            reposted: false,
            dodged: false,
            split: false,
            born: self.tick,
            dealt: 0.0,
        });
        if let Some(t) = self.spec_tally.get_mut(spec) {
            t.spawned += 1;
        }
        id
    }

    /// Advances one tick.
    pub fn step(&mut self, input: Input) {
        if self.outcome != Outcome::Running {
            return;
        }
        self.tick += 1;
        self.move_player(input);
        self.tick_status();
        self.spawn();
        self.grid.rebuild(self.slop.iter().map(|s| s.pos));
        self.hazards();
        self.boss_think();
        self.think();
        self.separate();
        self.touch();
        self.use_actives();
        self.fire_weapons();
        self.move_projectiles();
        self.tick_zones();
        self.flush_pending();
        self.bury();
        self.collect_likes();
        self.tick_timers();
        self.check_end();
    }

    /// Counts down warnings into slop and queues new ones from the spawn
    /// budget (WAVE-02).
    fn spawn(&mut self) {
        let mut ready = Vec::new();
        self.warnings.retain_mut(|w| {
            w.ticks -= 1;
            if w.ticks == 0 {
                ready.push((w.spec, w.pos, w.pack, w.number, w.elite));
                false
            } else {
                true
            }
        });
        for (spec, pos, pack, number, elite) in ready {
            let spec = if elite { self.elite_spec(spec as usize) } else { spec as usize };
            self.place(spec, pos, pack, number);
            let sp = &self.setup.specs[spec];
            if sp.is_brainrot() || sp.elite {
                self.events.push(Event::Named { at: pos, spec: spec as u16 });
            }
        }

        let t = self.elapsed();
        let timed = self.setup.seconds > 0.0;
        if self.setup.spawns.is_empty() || (timed && t >= self.setup.seconds - SPAWN_STOP) {
            return;
        }
        let progress = if timed { t / self.setup.seconds } else { (t / 120.0).min(1.0) };
        let (lo, hi) = self.setup.rate;
        self.spawn_credit += (lo + (hi - lo) * progress) * DT;
        while self.spawn_credit >= 1.0 && self.slop.len() + self.warnings.len() < MAX_SLOP {
            let Some(entry) = self.pick_spawn(t) else { break };
            self.spawn_credit -= entry.cost.max(0.2);
            let at = self.spawn_point();
            let elite = entry.group == 1 && self.setup.elite_chance > 0.0 && self.rng_spawn.chance(self.setup.elite_chance);
            let pack = if entry.group > 1 {
                self.next_pack += 1;
                self.next_pack
            } else {
                0
            };
            for n in 1..=entry.group {
                let offset = if entry.group > 1 { V2::from_angle(n as f32 * 1.2566) * 30.0 } else { V2::ZERO };
                let spec = entry.spec;
                let pos = clamp_to_arena(at + offset, self.setup.specs[spec].radius);
                self.warnings.push(Warning { pos, spec: spec as u16, ticks: SPAWN_WARNING, pack, number: n, elite });
                self.events.push(Event::SpawnWarning { at: pos });
            }
        }
    }

    fn pick_spawn(&mut self, t: f32) -> Option<Spawn> {
        let total: u32 = self.setup.spawns.iter().filter(|s| t >= s.from).map(|s| s.weight).sum();
        if total == 0 {
            return None;
        }
        let mut roll = self.rng_spawn.below(total);
        for s in &self.setup.spawns {
            if t < s.from {
                continue;
            }
            if roll < s.weight {
                return Some(*s);
            }
            roll -= s.weight;
        }
        None
    }

    /// Places slop queued during the tick: splits, hatchlings, reposts of bosses.
    fn flush_pending(&mut self) {
        for (spec, at, hp, pack, number) in std::mem::take(&mut self.pending) {
            if self.slop.len() >= MAX_SLOP {
                break;
            }
            let id = self.place(spec, at, pack, number);
            if hp > 0.0 {
                if let Some(s) = self.slop.iter_mut().find(|s| s.id == id) {
                    s.hp = hp;
                    s.max_hp = hp;
                }
            }
        }
    }

    /// The index of a spec with this name, adding it to the wave if new.
    pub(crate) fn ensure_spec(&mut self, spec: SlopSpec) -> usize {
        if let Some(i) = self.setup.specs.iter().position(|s| s.name == spec.name && s.elite == spec.elite) {
            return i;
        }
        self.setup.specs.push(spec);
        self.spec_tally.push(SpecTally::default());
        self.setup.specs.len() - 1
    }

    /// The elite version of a spec, added to the wave once.
    fn elite_spec(&mut self, spec: usize) -> usize {
        let elite = self.setup.specs[spec].clone().elite();
        if let Some(i) = self.setup.specs.iter().position(|s| s.elite && s.name == elite.name) {
            return i;
        }
        self.setup.specs.push(elite);
        self.spec_tally.push(SpecTally::default());
        self.setup.specs.len() - 1
    }

    /// Somewhere just off-screen, inside the arena, never right next to you.
    pub(crate) fn spawn_point(&mut self) -> V2 {
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

    /// Removes dead slop: Likes, explosions, reposts, scattering packs
    /// (SLOP-03, SLOP-05).
    fn bury(&mut self) {
        let mut scatter = Vec::new();
        let mut drops = Vec::new();
        let mut blasts = Vec::new();
        let mut reposts = Vec::new();
        let mut boss_died = false;
        for s in self.slop.iter().filter(|s| s.hp <= 0.0) {
            let sp = &self.setup.specs[s.spec as usize];
            let clone = sp.traits.has(Traits::CLONE);
            if let Some(t) = self.spec_tally.get_mut(s.spec as usize) {
                t.alive_ticks += self.tick - s.born;
            }
            if self.boss.as_ref().is_some_and(|b| b.id == s.id) {
                boss_died = true;
            }
            if !clone {
                self.tally.kills += 1;
                self.tally.kills_by_type[s.last_type as usize] += 1;
                let mut value = sp.likes;
                if sp.traits.has(Traits::DOUBLE_LIKES) {
                    value *= 2;
                }
                drops.push((s.pos, value));
                if sp.elite {
                    self.tally.loot_boxes += 1;
                    self.events.push(Event::LootBox { at: s.pos });
                }
                if s.last_type == crate::stats::DamageType::Grass as u8 && self.player.perks.houseplant > 0 {
                    let p = &mut self.player;
                    p.mental_health = (p.mental_health + p.perks.houseplant as f32).min(p.stats.max_mental_health);
                }
            }
            self.events.push(Event::SlopDied { at: s.pos, spec: s.spec });
            if sp.traits.has(Traits::EXPLODE) {
                blasts.push(s.pos);
            }
            if matches!(sp.look, SlopLook::Archetype(Archetype::ListicleDrone)) && s.number == 1 && s.pack > 0 {
                scatter.push((s.pack, s.pos));
            }
            let boss = matches!(sp.movement, Movement::Boss(_));
            if !s.reposted && !clone && !boss && (sp.traits.has(Traits::RESPAWN) || self.rng_combat.chance(self.setup.repost_chance)) {
                reposts.push((s.spec, s.pos));
            }
        }
        self.slop.retain(|s| s.hp > 0.0);
        for (at, value) in drops {
            let jitter = V2::new(self.rng_combat.range(-6.0, 6.0), self.rng_combat.range(-6.0, 6.0));
            let bonus = self.player.perks.like_bonus;
            let value = (value as f32 * (1.0 + bonus)).round().max(1.0) as u32;
            let id = self.id();
            self.likes.push(Like { id, pos: at + jitter, value, pulled: false, fake: false });
        }
        for at in blasts {
            self.events.push(Event::Splash { at, radius: 70.0 });
            if at.distance(self.player.pos) < 70.0 + PLAYER_RADIUS {
                self.hurt(8.0, NO_SPEC);
            }
        }
        for (spec, at) in reposts {
            let id = self.place(spec as usize, at, 0, 1);
            if let Some(s) = self.slop.iter_mut().find(|s| s.id == id) {
                s.reposted = true;
                s.hp *= REPOST_HP;
                s.max_hp *= REPOST_HP;
            }
            self.events.push(Event::Reposted { at });
        }
        for (pack, at) in scatter {
            let mut any = false;
            for s in self.slop.iter_mut().filter(|s| s.pack == pack) {
                s.fleeing = SCATTER_TICKS;
                any = true;
            }
            if any {
                self.events.push(Event::Scattered { at });
            }
        }
        if boss_died {
            self.boss_defeated();
        }
    }

    /// Likes within Curiosity fly to you and become Clout; fakes explode
    /// (LIKE-01).
    fn collect_likes(&mut self) {
        let p = self.player.pos;
        let curiosity = self.player.stats.curiosity;
        let step = LIKE_PULL_SPEED * DT;
        let mut got = 0;
        let mut fakes = 0;
        self.likes.retain_mut(|l| {
            let d = l.pos.distance(p);
            if d < curiosity {
                l.pulled = true;
            }
            if !l.pulled {
                return true;
            }
            if d <= step + PLAYER_RADIUS {
                if l.fake {
                    fakes += 1;
                } else {
                    got += l.value;
                }
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
        for _ in 0..fakes {
            self.events.push(Event::Splash { at: p, radius: 50.0 });
            self.hurt(5.0, NO_SPEC);
            self.add_anxiety();
        }
    }

    fn tick_timers(&mut self) {
        let p = &mut self.player;
        p.invulnerable = p.invulnerable.saturating_sub(1);
        p.dog_block = p.dog_block.saturating_sub(1);
        p.noise_block = p.noise_block.saturating_sub(1);
        for c in &mut p.active_cooldown {
            *c = (*c - DT).max(0.0);
        }
        self.ad_break = self.ad_break.saturating_sub(1);
        for s in &mut self.slop {
            s.flash = s.flash.saturating_sub(1);
            s.marked = s.marked.saturating_sub(1);
            s.fleeing = s.fleeing.saturating_sub(1);
            s.stunned = s.stunned.saturating_sub(1);
            s.shield_off = s.shield_off.saturating_sub(1);
            if s.agreed_ticks > 0 {
                s.agreed_ticks -= 1;
                if s.agreed_ticks == 0 {
                    s.agreed = 0;
                }
            }
        }
        if self.tick % TICK_RATE == 0 {
            let me = self.player.pos;
            if let Some(d) = self.slop.iter().map(|s| s.pos.distance(me)).min_by(|a, b| a.total_cmp(b)) {
                self.tally.distance_sum += d;
                self.tally.distance_samples += 1;
            }
        }
    }

    /// Burnout at 0 Mental Health; a timed wave is cleared when its time
    /// runs out, and Likes still on the floor count half (WAVE-01).
    fn check_end(&mut self) {
        if self.player.mental_health <= 0.0 {
            self.player.mental_health = 0.0;
            self.outcome = Outcome::Burnout;
            self.events.push(Event::Burnout);
        } else if self.setup.seconds > 0.0 && self.tick >= (self.setup.seconds * TICK_RATE as f32) as u32 {
            self.clear();
        }
    }

    pub(crate) fn clear(&mut self) {
        let left: u32 = self.likes.iter().filter(|l| !l.fake).map(|l| l.value).sum();
        let credited = (left as f32 * LEFTOVER_SHARE) as u32;
        self.clout += credited;
        self.tally.likes_collected += credited;
        for s in &self.slop {
            if let Some(t) = self.spec_tally.get_mut(s.spec as usize) {
                t.alive_ticks += self.tick - s.born;
            }
        }
        self.likes.clear();
        self.warnings.clear();
        self.outcome = Outcome::Cleared;
        self.events.push(Event::WaveCleared);
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
