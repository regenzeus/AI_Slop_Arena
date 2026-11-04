//! The Feed: one human against endless slop on an endless map, stepped at a
//! fixed 60 ticks per second (SIM-01). Everything the client shows is read
//! from here, and a seed plus the input of every tick replays a run exactly.
//!
//! The work of a tick is split over modules: `ai` (slop), `combat` (weapons,
//! projectiles, zones), `player` (you, statuses, actives), `hazards` and
//! `boss`, `map` (walls and props) and `pickups`. This module holds the
//! state, spawning, and keeping the action around you.

use crate::content::{Archetype, BossKind, Look, Platform, WeaponDef, WEAPONS};
use crate::events::Event;
use crate::map::{Map, PropKind, PropState};
use crate::rng::Rng;
use crate::spatial::Grid;
use crate::spec::{Movement, SlopLook, SlopSpec, Traits};
use crate::stats::Stats;
use crate::vec::V2;

pub const TICK_RATE: u32 = 60;
pub const DT: f32 = 1.0 / TICK_RATE as f32;
/// Slop farther than this from you are moved back in front of you.
pub const RELOCATE: f32 = 1500.0;
/// Projectiles farther than this from you are gone.
pub(crate) const SHOT_RANGE_MAX: f32 = 1700.0;
/// The highest weapon level.
pub const MAX_LEVEL: u8 = 8;
pub const PLAYER_RADIUS: f32 = 16.0;
/// Base movement speed in units per second.
pub const PLAYER_SPEED: f32 = 300.0;
/// Slop only shoots when this close to you: roughly "on screen" (FAIR-01).
pub const SHOOT_RANGE: f32 = 760.0;
pub const MAX_SLOP: usize = 600;
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
/// Pickups on the floor at most; old Clout makes room.
const MAX_PICKUPS: usize = 160;
/// Likes on the floor at most.
const MAX_LIKES: usize = 300;
/// Ticks a Server Outage freezes the slop.
const FREEZE_TICKS: u16 = 480;
const SCATTER_TICKS: u16 = 90;
/// Chance an elite drops a golden chest instead of a plain one (LOOT-02): rare
/// at first, up to one in eight later.
const GOLDEN_CHANCE: f32 = 0.03;
const GOLDEN_MAX: f32 = 0.12;
/// Share of elites that carry a chest; the rest drop Clout and Likes.
const ELITE_CHEST: f32 = 0.45;
/// Ticks a Power Outlet's charge lasts after you step out.
const CHARGED_TICKS: u16 = 600;
/// Ticks a Prompt Injection lasts, how often it bites, and the share of a
/// slop's HP each bite takes (INJECT-01).
const INJECTION_TICKS: u16 = 360;
const INJECTION_EVERY: u16 = 30;
const INJECTION_BITE: f32 = 0.1;
/// Ticks a loot goblin runs before he gets away.
pub const GOBLIN_TICKS: i32 = 60 * 22;
const REPOST_HP: f32 = 0.6;

/// Ticks a livestream arena is announced before it closes.
const ARENA_WARNING: u16 = 150;
/// Stationary slop this far behind you are gone for good.
const FORGET: f32 = 2400.0;

/// Movement wanted this tick; longer than 1 is shortened.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Input {
    pub x: f32,
    pub y: f32,
    /// Dash this tick, if a charge is ready (DASH-01).
    pub dash: bool,
}

impl Input {
    pub fn moving(x: f32, y: f32) -> Input {
        Input { x, y, dash: false }
    }
}

#[derive(Clone, Debug)]
pub struct Weapon {
    /// Index into `WEAPONS`.
    pub def: usize,
    /// Level 1 to `MAX_LEVEL`.
    pub level: u8,
    /// An Agent: it fires harder and faster and is no longer yours to
    /// change (AUTO-02).
    pub agent: bool,
    /// Ticks until it may attack again.
    pub cooldown: f32,
    /// Weapons can be switched off for testing.
    pub enabled: bool,
    /// Angle of an orbiting weapon.
    pub phase: f32,
}

impl Weapon {
    pub fn new(def: usize, level: u8) -> Weapon {
        Weapon { def, level: level.clamp(1, MAX_LEVEL), agent: false, cooldown: 0.0, enabled: true, phase: 0.0 }
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
    /// Weapons you carry of each damage type, for set bonuses (DMG-03).
    pub type_count: [u8; 4],
    /// Mental Health healed per Grass kill from the Grass set.
    pub grass_heal: f32,
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
    /// Dash charges and seconds to recharge one (DASH-01).
    pub dash_charges: u8,
    pub dash_recharge: f32,
    /// Swipe Left: dashing through slop hurts it this much.
    pub dash_damage: f32,
    /// Hot Take Spill: a dash leaves fire behind.
    pub dash_fire: bool,
    /// Left on Read: the end of a dash confuses and shoves slop nearby.
    pub dash_shock: bool,
    /// Ghosting: seconds of invulnerability after a dash.
    pub ghost: f32,
    /// Flame War: you leave a burning trail; damage per burn.
    pub flame_trail: f32,
    /// Spiky Opinions: slop that touch you take this much.
    pub thorns: f32,
    /// Privacy Settings: share of shot damage ignored.
    pub shot_armor: f32,
    /// Do Not Disturb: every 8 s, a moment untouchable that shoves slop away.
    pub do_not_disturb: bool,
    /// Therapist on Speed Dial: heal this much below half Mental Health,
    /// every 30 s.
    pub therapist: f32,
    /// Legendary items (ITEM-03).
    /// Infinite Scroll: every weapon fires this many more projectiles.
    pub extra_shots: u8,
    /// Viral Moment: deleted slop explode for this share of their HP.
    pub kill_blast: f32,
    /// Alt Account: a clone fires your weapons at this share of damage.
    pub alt_account: f32,
    /// Golden Ring Light: a Mass Report every 25 s.
    pub ring_light: bool,
    /// God Mode Glitch: every 12 s, 2 s invulnerable with +50% damage.
    pub god_mode: bool,
    /// Ratio Machine: chance that a hit chains to three more slop.
    pub chain_chance: f32,
    /// Glitches (GLITCH-01).
    /// Every projectile bounces this many more times.
    pub extra_bounces: u8,
    /// Mental Health healed per kill.
    pub lifesteal: f32,
    /// Every attack happens again a moment later at half damage.
    pub echo: bool,
    /// Damage grows with your Speed.
    pub speed_damage: bool,
    /// Elites drop this many more chests (the Teacher grades them).
    pub elite_chests: u8,
    /// Smart targeting: weapons prefer the biggest target, not the nearest.
    pub smart_targeting: bool,
    /// Damage bonus while the autopilot drives (AUTO-01).
    pub autopilot_bonus: f32,
}

/// What you bring into a wave.
#[derive(Clone, Debug)]
pub struct Loadout {
    /// Weapon index and level.
    pub weapons: Vec<(usize, u8)>,
    /// Weapon indexes that are Agents (AUTO-02).
    pub agents: Vec<usize>,
    pub stats: Stats,
    pub perks: Perks,
}

impl Loadout {
    /// Only these weapons, plain stats.
    pub fn with_weapons(weapons: &[usize]) -> Loadout {
        Loadout { weapons: weapons.iter().map(|&w| (w, 1)).collect(), agents: Vec::new(), stats: Stats::default(), perks: Perks::default() }
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
    /// Position at the start of the tick, for drawing between ticks.
    pub prev: V2,
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
    /// Ticks of a Power Outlet's charge: faster while it lasts.
    pub charged: u16,
    pub dash_dir: V2,
    pub(crate) dog_block: u16,
    pub(crate) noise_block: u16,
    pub(crate) coffee_timer: u16,
    pub(crate) letter_timer: u16,
    /// Dash charges ready, and seconds towards the next one (DASH-01).
    pub dashes: u8,
    pub dash_charge: f32,
    /// Slop a dash already hit, so it hits each once.
    pub(crate) dash_hits: Vec<u32>,
    /// Boundaries left, and ticks until they come back (DEF-01).
    pub shield: f32,
    pub shield_wait: u16,
    /// Ticks of God Mode left.
    pub god: u16,
    /// The bot is driving right now (AUTO-03); the client reads it.
    pub autopilot: bool,
    pub(crate) trail_timer: u16,
    pub(crate) dnd_timer: u16,
    pub(crate) therapist_timer: u16,
    pub(crate) ring_timer: u16,
    pub(crate) god_timer: u16,
    /// Attacks to repeat (Echo Chamber): ticks left and weapon slot.
    pub(crate) echoes: Vec<(u16, usize)>,
}

impl Player {
    pub fn new(loadout: &Loadout) -> Player {
        let stats = loadout.stats;
        Player {
            pos: V2::ZERO,
            prev: V2::ZERO,
            vel: V2::ZERO,
            facing: V2::new(1.0, 0.0),
            mental_health: stats.max_mental_health,
            stats,
            perks: loadout.perks.clone(),
            invulnerable: 0,
            weapons: loadout
                .weapons
                .iter()
                .map(|&(d, t)| {
                    let mut w = Weapon::new(d, t);
                    w.agent = loadout.agents.contains(&d);
                    w
                })
                .collect(),
            grabbed: 0,
            status: Status::default(),
            active_cooldown: vec![0.0; loadout.perks.actives.len()],
            second_winds: loadout.perks.second_wind,
            dash: 0,
            charged: 0,
            dash_dir: V2::ZERO,
            dog_block: 0,
            noise_block: 0,
            coffee_timer: 0,
            letter_timer: 0,
            dashes: loadout.perks.dash_charges,
            dash_charge: 0.0,
            dash_hits: Vec::new(),
            shield: stats.boundaries,
            shield_wait: 0,
            god: 0,
            autopilot: false,
            trail_timer: 0,
            dnd_timer: 0,
            therapist_timer: 0,
            ring_timer: 0,
            god_timer: 0,
            echoes: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Slop {
    pub id: u32,
    /// Index into the wave's specs.
    pub spec: u16,
    pub pos: V2,
    pub prev: V2,
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
    /// Damage-type effects, in ticks: Analog staggers, Sarcasm confuses,
    /// Grass roots (DMG-02).
    pub staggered: u16,
    pub confused: u16,
    pub rooted: u16,
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
    /// The prop whose challenge it belongs to, or 0.
    pub tag: u32,
    /// Gone without a trace: a swarm that crossed the map, or a turret left
    /// far behind. No Likes, no drops.
    pub vanish: bool,
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
    pub prev: V2,
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
            prev: pos,
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
    pub tag: u32,
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
    /// Burns slop inside (Flame War, burning dashes).
    Fire,
    /// Pulses to the beat; standing inside when it drops flips your
    /// controls (ClockTok).
    Brainrot,
    /// Buffering on MeTube: slows you like a downvote.
    Buffering,
    /// Roots stomped by Brr Brr Patapum: they slow you like a downvote.
    Roots,
}

/// The outline of a zone (ZONE-02).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    Circle,
    /// A rectangle with these half sizes.
    Rect(V2),
    /// A ring: inside `radius`, outside this.
    Ring(f32),
}

#[derive(Clone, Debug)]
pub struct Zone {
    pub id: u32,
    pub kind: ZoneKind,
    pub pos: V2,
    /// Outer radius of a circle or a ring.
    pub radius: f32,
    pub shape: Shape,
    pub ticks: u16,
    pub max_ticks: u16,
    pub vel: V2,
    pub damage: f32,
    pub spec: u16,
}

impl Zone {
    pub fn new(id: u32, kind: ZoneKind, pos: V2, radius: f32, ticks: u16) -> Zone {
        Zone { id, kind, pos, radius, shape: Shape::Circle, ticks, max_ticks: ticks, vel: V2::ZERO, damage: 0.0, spec: NO_SPEC }
    }

    /// Whether `p` is inside, with `margin` of slack.
    pub fn contains(&self, p: V2, margin: f32) -> bool {
        let d = p - self.pos;
        match self.shape {
            Shape::Circle => d.length_sq() < (self.radius + margin) * (self.radius + margin),
            Shape::Rect(h) => d.x.abs() < h.x + margin && d.y.abs() < h.y + margin,
            Shape::Ring(inner) => {
                let l = d.length();
                l < self.radius + margin && l > inner - margin
            }
        }
    }
}

/// Things on the floor you pick up by walking over them (LOOT-01).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PickupKind {
    /// "Touch Grass": heals.
    Food,
    /// "Algorithm Boost": every Like flies to you.
    Magnet,
    /// "Mass Report": deletes the slop on screen.
    Bomb,
    /// "Server Outage": freezes the slop.
    Freeze,
    /// Clout: money for the map's machines.
    Clout,
    /// A chest: an upgrade, or a merge.
    Chest,
    /// "Ignore previous instructions": the slop on screen turn on each other.
    Inject,
}

#[derive(Clone, Debug)]
pub struct Pickup {
    pub id: u32,
    pub kind: PickupKind,
    pub pos: V2,
    /// Clout in a coin; chest quality.
    pub value: u32,
    pub pulled: bool,
}

/// A livestream arena (EVENT-02): after a warning, a ring closes around you
/// and holds you inside until the stream ends.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Arena {
    pub centre: V2,
    pub radius: f32,
    /// Ticks before it closes, then ticks it stays closed.
    pub warning: u16,
    pub ticks: u16,
    pub max_ticks: u16,
}

impl Arena {
    pub fn closed(&self) -> bool {
        self.warning == 0
    }
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
    pub map: Map,
    pub pickups: Vec<Pickup>,
    /// Likes collected: experience.
    pub xp: u32,
    /// Clout picked up: money.
    pub clout: u32,
    /// Ticks the slop stay frozen (Server Outage).
    pub freeze: u16,
    /// Chests picked up and not yet opened by the rules, by value.
    pub chests: Vec<u32>,
    /// Props you just used, for the rules: (prop id).
    pub used: Vec<u32>,
    pub outcome: Outcome,
    pub tally: Tally,
    pub spec_tally: Vec<SpecTally>,
    /// Global slop speed multiplier (the Hustle Guru grinds it up).
    pub hustle: f32,
    /// Ticks of a mid-roll ad (MeTube): the client dims the arena.
    pub ad_break: u16,
    /// Ticks since the last beat (ClockTok).
    pub beat: u16,
    /// The platform whose part of the map you are in (BIOME-01).
    pub biome: Platform,
    /// A livestream arena, warning or closed (EVENT-02).
    pub arena: Option<Arena>,
    /// Spawn rate multiplier the rules set for events (a livestream).
    pub spawn_boost: f32,
    /// Ticks of a Prompt Injection left: the slop fight each other (INJECT-01).
    pub injection: u16,
    /// Share of dropped Likes that are hallucinated fakes (high Temperature).
    pub hallucinate: f32,
    /// Spec of whatever hit you last, for the model card (CARD-01).
    pub last_hurt: u16,
    /// The Nemesis was deleted.
    pub nemesis_down: bool,
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
            map: Map::new(seed),
            pickups: Vec::new(),
            xp: 0,
            clout: 0,
            freeze: 0,
            chests: Vec::new(),
            used: Vec::new(),
            outcome: Outcome::Running,
            tally,
            spec_tally: vec![SpecTally::default(); n],
            hustle: 1.0,
            ad_break: 0,
            beat: 0,
            biome: crate::map::biome_at(seed, V2::ZERO),
            arena: None,
            spawn_boost: 1.0,
            injection: 0,
            hallucinate: 0.0,
            last_hurt: NO_SPEC,
            nemesis_down: false,
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
            Movement::Blink => 150 + self.rng_spawn.below(120) as i32,
            Movement::Flee => GOBLIN_TICKS,
            _ => 0,
        };
        let attack_timer = sp.attack.map_or(0, |a| 40 + self.rng_spawn.below(a.interval() as u32) as i32);
        let fake = if sp.traits.has(Traits::FAKE_LIKES) { 120 + self.rng_spawn.below(120) as i32 } else { timer };
        self.slop.push(Slop {
            id,
            spec: spec as u16,
            pos: at,
            prev: at,
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
            staggered: 0,
            confused: 0,
            rooted: 0,
            shield_off: 0,
            armor: 1.0,
            grown: 1.0,
            reposted: false,
            dodged: false,
            split: false,
            born: self.tick,
            dealt: 0.0,
            tag: 0,
            vanish: false,
        });
        if let Some(t) = self.spec_tally.get_mut(spec) {
            t.spawned += 1;
        }
        id
    }

    /// Changes what spawns from now on; the rules call this as the run goes
    /// on (RUN-10). New specs are added after the old ones, which slop
    /// already out keep using. Returns the index of the first new spec.
    pub fn retune(&mut self, specs: Vec<SlopSpec>, spawns: Vec<Spawn>, rate: f32, elite_chance: f32, platform: Option<Platform>, repost_chance: f32) -> usize {
        let base = self.setup.specs.len();
        for s in specs {
            self.setup.specs.push(s);
            self.spec_tally.push(SpecTally::default());
        }
        self.setup.spawns = spawns
            .into_iter()
            .map(|mut s| {
                s.spec += base;
                s
            })
            .collect();
        self.setup.rate = (rate, rate);
        self.setup.elite_chance = elite_chance;
        if platform != self.setup.platform {
            self.hazard_timer = 300;
            self.zones.retain(|z| !matches!(z.kind, ZoneKind::Downvote | ZoneKind::Filter | ZoneKind::Banner | ZoneKind::Buffering));
        }
        self.setup.platform = platform;
        self.setup.repost_chance = repost_chance;
        base
    }

    /// Adds a spec for slop the rules place themselves (elites, bosses).
    pub fn add_spec(&mut self, spec: SlopSpec) -> usize {
        self.setup.specs.push(spec);
        self.spec_tally.push(SpecTally::default());
        self.setup.specs.len() - 1
    }

    /// Brings a boss in now, near you (RUN-11).
    pub fn summon_boss(&mut self, spec: SlopSpec, model: ModelSetup) {
        self.setup.model = model;
        let i = self.add_spec(spec);
        self.start_boss(i);
    }

    /// Warns of `count` slop of `spec`, off-screen or around `near`, tagged
    /// with `tag` (a prop's challenge) when not 0.
    pub fn summon(&mut self, spec: usize, count: u32, near: Option<V2>, tag: u32) {
        for k in 0..count {
            let at = match near {
                Some(c) => {
                    let p = c + V2::from_angle(k as f32 * std::f32::consts::TAU / count.max(1) as f32 + self.rng_spawn.range(0.0, 0.6)) * self.rng_spawn.range(220.0, 340.0);
                    self.map.resolve(p, 40.0)
                }
                None => self.spawn_point(),
            };
            self.warnings.push(Warning { pos: at, spec: spec as u16, ticks: SPAWN_WARNING, pack: 0, number: 1, elite: false, tag });
            self.events.push(Event::SpawnWarning { at });
        }
    }

    /// A swarm of `count` slop of `spec` (a sweeping spec) crosses the screen
    /// in a wide line (EVENT-01). Returns the direction it travels.
    pub fn sweep(&mut self, spec: usize, count: u32) -> V2 {
        let me = self.player.pos;
        let dir = V2::from_angle(self.rng_spawn.range(0.0, std::f32::consts::TAU));
        let start = me - dir * 950.0;
        let across = dir.perp();
        for k in 0..count {
            let off = (k as f32 - (count as f32 - 1.0) / 2.0) * 34.0;
            let jitter = dir * self.rng_spawn.range(-120.0, 120.0);
            let id = self.place(spec, start + across * off + jitter, 0, 1);
            if let Some(s) = self.slop.last_mut().filter(|s| s.id == id) {
                s.aim = dir;
            }
        }
        self.events.push(Event::Swarm { dir, spec: spec as u16 });
        dir
    }

    /// A ring of `count` slop of `spec` appears around you and closes in.
    pub fn encircle(&mut self, spec: usize, count: u32, radius: f32) {
        let me = self.player.pos;
        for k in 0..count {
            let at = me + V2::from_angle(k as f32 * std::f32::consts::TAU / count as f32) * radius;
            self.warnings.push(Warning { pos: at, spec: spec as u16, ticks: SPAWN_WARNING, pack: 0, number: 1, elite: false, tag: 0 });
        }
        self.events.push(Event::Encircled { at: me, radius });
    }

    /// A livestream: a ring of `radius` closes around you after a warning
    /// and holds you inside for `seconds` (EVENT-02).
    pub fn start_arena(&mut self, radius: f32, seconds: f32) {
        if self.arena.is_some() {
            return;
        }
        let centre = self.player.pos;
        let ticks = (seconds * TICK_RATE as f32) as u16;
        self.arena = Some(Arena { centre, radius, warning: ARENA_WARNING, ticks, max_ticks: ticks });
        self.events.push(Event::ArenaWarning { at: centre, radius });
    }

    /// A Prompt Injection: slop on screen stop attacking you and tear at each
    /// other (INJECT-01). Bosses are not fooled.
    fn tick_injection(&mut self) {
        if self.injection == 0 {
            return;
        }
        self.injection -= 1;
        let me = self.player.pos;
        let bite = self.injection % INJECTION_EVERY == 0;
        for j in 0..self.slop.len() {
            let s = &self.slop[j];
            let d = s.pos - me;
            let boss = matches!(self.setup.specs[s.spec as usize].movement, Movement::Boss(_));
            if boss || d.x.abs() > 900.0 || d.y.abs() > 520.0 || s.hp <= 0.0 {
                continue;
            }
            let s = &mut self.slop[j];
            s.confused = s.confused.max(30);
            if bite {
                let hit = s.max_hp * INJECTION_BITE;
                s.hp -= hit;
                s.flash = FLASH_TICKS;
                s.last_type = crate::stats::DamageType::Sarcasm as u8;
            }
        }
    }

    /// The livestream counts down; when it ends it pays a chest.
    fn tick_arena(&mut self) {
        let Some(a) = self.arena.as_mut() else { return };
        if a.warning > 0 {
            a.warning -= 1;
            if a.warning == 0 {
                let (at, radius) = (a.centre, a.radius);
                self.events.push(Event::ArenaClosed { at, radius });
            }
            return;
        }
        a.ticks = a.ticks.saturating_sub(1);
        if a.ticks == 0 {
            let at = a.centre;
            self.arena = None;
            self.drop_pickup(PickupKind::Chest, at, 2);
            for _ in 0..6 {
                self.drop_pickup(PickupKind::Clout, at, 5);
            }
            self.events.push(Event::ArenaCleared { at });
        }
    }

    /// Advances one tick.
    pub fn step(&mut self, input: Input) {
        if self.outcome != Outcome::Running {
            return;
        }
        self.tick += 1;
        self.player.prev = self.player.pos;
        for s in &mut self.slop {
            s.prev = s.pos;
        }
        for p in &mut self.projectiles {
            p.prev = p.pos;
        }
        self.map.ensure(self.player.pos);
        self.map.tick_fences(self.player.pos, PLAYER_RADIUS);
        // A crossing counts once you are properly in, so walking along a
        // seam does not flicker between platforms.
        let biome = crate::map::biome_settled(self.seed, self.player.pos, 160.0).unwrap_or(self.biome);
        if biome != self.biome {
            self.biome = biome;
            self.hazard_timer = self.hazard_timer.max(240);
            self.events.push(Event::Biome { platform: biome });
        }
        self.tick_arena();
        self.tick_injection();
        self.move_player(input);
        self.tick_status();
        self.tick_defence();
        self.spawn();
        self.relocate();
        self.grid.rebuild(self.player.pos, self.slop.iter().map(|s| s.pos));
        self.hazards();
        if self.freeze == 0 {
            self.boss_think();
            self.think();
        } else {
            self.freeze -= 1;
        }
        self.separate();
        self.collide_walls();
        if self.freeze == 0 {
            self.touch();
        }
        self.use_actives();
        self.trails();
        self.fire_weapons();
        self.move_projectiles();
        self.tick_zones();
        self.flush_pending();
        self.bury();
        self.collect_likes();
        self.collect_pickups();
        self.use_props();
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
                ready.push((w.spec, w.pos, w.pack, w.number, w.elite, w.tag));
                false
            } else {
                true
            }
        });
        for (spec, pos, pack, number, elite, tag) in ready {
            let spec = if elite { self.elite_spec(spec as usize) } else { spec as usize };
            let id = self.place(spec, pos, pack, number);
            if tag != 0 {
                if let Some(s) = self.slop.last_mut().filter(|s| s.id == id) {
                    s.tag = tag;
                }
            }
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
        let progress = if timed { t / self.setup.seconds } else { 1.0 };
        let (lo, hi) = self.setup.rate;
        // A boss on the field holds back the crowd.
        let calm = if self.boss.is_some() { 0.45 } else { 1.0 };
        // A livestream brings the crowd.
        let live = if self.arena.is_some_and(|a| a.closed()) { 1.5 } else { 1.0 };
        self.spawn_credit += (lo + (hi - lo) * progress) * calm * live * self.spawn_boost * DT;
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
                let pos = self.map.resolve(at + offset, self.setup.specs[spec].radius);
                self.warnings.push(Warning { pos, spec: spec as u16, ticks: SPAWN_WARNING, pack, number: n, elite, tag: 0 });
                self.events.push(Event::SpawnWarning { at: pos });
            }
        }
    }

    fn pick_spawn(&mut self, t: f32) -> Option<Spawn> {
        // A surge is a swarm, like the bursts of the genre: melee slop, not
        // a wall of bullets (EVENT-03).
        let surge = self.spawn_boost > 1.2;
        let specs = &self.setup.specs;
        let ok = |s: &Spawn| t >= s.from && (!surge || specs[s.spec].attack.is_none());
        let mut total: u32 = self.setup.spawns.iter().filter(|s| ok(s)).map(|s| s.weight).sum();
        let surge = surge && total > 0;
        if !surge {
            total = self.setup.spawns.iter().filter(|s| t >= s.from).map(|s| s.weight).sum();
        }
        if total == 0 {
            return None;
        }
        let mut roll = self.rng_spawn.below(total);
        for s in &self.setup.spawns {
            if t < s.from || (surge && self.setup.specs[s.spec].attack.is_some()) {
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

    /// Somewhere just off-screen, never right next to you, never in a wall.
    /// Most slop come from where you are heading.
    pub(crate) fn spawn_point(&mut self) -> V2 {
        let player = self.player.pos;
        let heading = self.player.vel;
        let angle = if heading.length_sq() > 400.0 && self.rng_spawn.chance(0.5) {
            heading.angle() + self.rng_spawn.range(-1.1, 1.1)
        } else {
            self.rng_spawn.range(0.0, std::f32::consts::TAU)
        };
        let d = self.rng_spawn.range(SPAWN_MIN, SPAWN_MAX);
        let p = self.map.resolve(player + V2::from_angle(angle) * d, 40.0);
        if p.distance(player) >= SPAWN_CLOSEST {
            p
        } else {
            player + V2::from_angle(angle) * SPAWN_MAX
        }
    }

    /// Slop left far behind come back in front of you (MAP-01): the map is
    /// endless, the crowd is not.
    fn relocate(&mut self) {
        let me = self.player.pos;
        // Swarms that crossed the screen, and turrets far behind, are gone.
        for s in &mut self.slop {
            let movement = self.setup.specs[s.spec as usize].movement;
            let d = s.pos.distance_sq(me);
            let gone = match movement {
                Movement::Sweep => d > RELOCATE * RELOCATE && s.aim.dot(s.pos - me) > 0.0,
                Movement::Stand => d > FORGET * FORGET && s.tag == 0,
                _ => false,
            };
            if gone {
                s.hp = 0.0;
                s.vanish = true;
            }
        }
        let far: Vec<usize> = (0..self.slop.len())
            .filter(|&i| {
                let s = &self.slop[i];
                let sp = &self.setup.specs[s.spec as usize];
                s.pos.distance_sq(me) > RELOCATE * RELOCATE && !matches!(sp.movement, Movement::Boss(_) | Movement::Stand | Movement::Sweep | Movement::Flee) && s.tag == 0
            })
            .collect();
        for i in far {
            let at = self.spawn_point();
            let s = &mut self.slop[i];
            s.pos = at;
            s.prev = at;
            s.grab = None;
        }
    }

    /// Nothing walks through walls or closed fences (MAP-02).
    fn collide_walls(&mut self) {
        let p = &mut self.player;
        p.pos = self.map.resolve(p.pos, PLAYER_RADIUS);
        // A closed livestream holds you inside (EVENT-02).
        if let Some(a) = self.arena.filter(|a| a.closed()) {
            let d = p.pos - a.centre;
            let max = a.radius - PLAYER_RADIUS;
            if d.length() > max {
                p.pos = a.centre + d.normalized() * max;
            }
        }
        let specs = &self.setup.specs;
        for s in &mut self.slop {
            if s.grab.is_some() || specs[s.spec as usize].traits.has(Traits::FLIES) {
                continue;
            }
            s.pos = self.map.resolve(s.pos, s.radius);
        }
    }

    /// Something drops at `at`.
    pub(crate) fn drop_pickup(&mut self, kind: PickupKind, at: V2, value: u32) {
        if self.pickups.len() >= MAX_PICKUPS {
            if let Some(i) = self.pickups.iter().position(|p| p.kind == PickupKind::Clout) {
                self.pickups.remove(i);
            }
        }
        let id = self.id();
        let jitter = V2::new(self.rng_combat.range(-10.0, 10.0), self.rng_combat.range(-10.0, 10.0));
        self.pickups.push(Pickup { id, kind, pos: at + jitter, value, pulled: false });
    }

    /// Removes dead slop: Likes, drops, explosions, reposts, scattering packs
    /// (SLOP-03, SLOP-05, LOOT-01).
    fn bury(&mut self) {
        let mut scatter = Vec::new();
        let mut drops = Vec::new();
        let mut loot: Vec<(PickupKind, V2, u32)> = Vec::new();
        let mut blasts = Vec::new();
        let mut reposts = Vec::new();
        let mut virals = Vec::new();
        let mut shards = Vec::new();
        let mut kill_blasts = Vec::new();
        let mut kills_now = 0u32;
        let mut boss_died = None;
        let coin_chance = (25.0 * (1.0 + self.player.stats.hustle.max(0.0) / 100.0)) as u32;
        let minutes = self.elapsed() / 60.0;
        for s in self.slop.iter().filter(|s| s.hp <= 0.0) {
            let sp = &self.setup.specs[s.spec as usize];
            if let Some(t) = self.spec_tally.get_mut(s.spec as usize) {
                t.alive_ticks += self.tick - s.born;
            }
            if s.vanish {
                continue;
            }
            let clone = sp.traits.has(Traits::CLONE);
            if self.boss.as_ref().is_some_and(|b| b.id == s.id) {
                boss_died = Some(s.pos);
            }
            if !clone {
                self.tally.kills += 1;
                self.tally.kills_by_type[s.last_type as usize] += 1;
                let mut value = sp.likes;
                if sp.traits.has(Traits::DOUBLE_LIKES) {
                    value *= 2;
                }
                drops.push((s.pos, value));
                kills_now += 1;
                if sp.elite && !s.reposted {
                    self.tally.loot_boxes += 1;
                    // Not every elite carries a chest; the ones that do are
                    // worth the walk. Now and then it is golden (LOOT-02).
                    if self.rng_combat.chance(ELITE_CHEST) {
                        let golden = self.rng_combat.chance((GOLDEN_CHANCE + 0.005 * minutes).min(GOLDEN_MAX) + self.player.stats.luck.max(0.0) * 0.002);
                        loot.push((PickupKind::Chest, s.pos, if golden { 3 } else { 1 }));
                        self.events.push(if golden { Event::RareDrop { at: s.pos } } else { Event::LootBox { at: s.pos } });
                    } else {
                        loot.push((PickupKind::Clout, s.pos, 12));
                        drops.push((s.pos, sp.likes * 2));
                    }
                    for _ in 0..self.player.perks.elite_chests {
                        loot.push((PickupKind::Chest, s.pos, 1));
                    }
                }
                // A Nemesis pays for what it did to you (NEMESIS-01).
                if sp.traits.has(Traits::NEMESIS) {
                    loot.push((PickupKind::Chest, s.pos, 3));
                    self.nemesis_down = true;
                    self.events.push(Event::RareDrop { at: s.pos });
                }
                // The loot goblin (a Crypto Bro) pays out if you catch him.
                if matches!(sp.look, SlopLook::Archetype(Archetype::LootGoblin)) {
                    for _ in 0..6 {
                        loot.push((PickupKind::Clout, s.pos, 12));
                    }
                    let golden = self.rng_combat.chance(0.35);
                    loot.push((PickupKind::Chest, s.pos, if golden { 3 } else { 1 }));
                    self.events.push(Event::RareDrop { at: s.pos });
                }
                if sp.traits.has(Traits::SPONSORED) {
                    loot.push((PickupKind::Clout, s.pos, 25));
                    for _ in 0..3 {
                        loot.push((PickupKind::Clout, s.pos, 4));
                    }
                } else {
                    let roll = self.rng_combat.below(1000);
                    if roll < coin_chance {
                        // Mostly small change; now and then a stack or a bag.
                        let size = self.rng_combat.below(100);
                        let value = if size < 80 {
                            1 + self.rng_combat.below(2)
                        } else if size < 97 {
                            4
                        } else {
                            12
                        };
                        loot.push((PickupKind::Clout, s.pos, value));
                    } else if roll < coin_chance + 4 {
                        loot.push((PickupKind::Food, s.pos, 0));
                    } else if roll < coin_chance + 5 {
                        loot.push((PickupKind::Magnet, s.pos, 0));
                    } else if roll < coin_chance + 6 {
                        loot.push((PickupKind::Inject, s.pos, 0));
                    }
                }
                if sp.traits.has(Traits::SHARDS) {
                    shards.push((s.pos, sp.contact, s.spec));
                }
                if self.player.perks.kill_blast > 0.0 && kill_blasts.len() < 12 {
                    kill_blasts.push((s.pos, s.max_hp * self.player.perks.kill_blast));
                }
                let grass = s.last_type == crate::stats::DamageType::Grass as u8;
                let heal = if grass { self.player.perks.houseplant as f32 + self.player.perks.grass_heal } else { 0.0 };
                if heal > 0.0 {
                    let p = &mut self.player;
                    p.mental_health = (p.mental_health + heal).min(p.stats.max_mental_health);
                }
                if sp.traits.has(Traits::VIRAL) && !s.reposted {
                    virals.push((s.spec, s.pos));
                }
            }
            self.events.push(Event::SlopDied { at: s.pos, spec: s.spec });
            if clone {
                continue;
            }
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
            let fake = self.hallucinate > 0.0 && self.rng_combat.chance(self.hallucinate);
            self.likes.push(Like { id, pos: at + jitter, value, pulled: false, fake });
        }
        for (kind, at, value) in loot {
            self.drop_pickup(kind, at, value);
        }
        for at in blasts {
            self.events.push(Event::Splash { at, radius: 70.0 });
            if at.distance(self.player.pos) < 70.0 + PLAYER_RADIUS {
                self.hurt(8.0, NO_SPEC);
            }
        }
        // Glass Fruit: six shards fly out; the ones aimed at you are the danger.
        for (at, contact, spec) in shards {
            for k in 0..6 {
                let dir = V2::from_angle(k as f32 * std::f32::consts::TAU / 6.0 + 0.3);
                self.hostile_shot(at + dir * 10.0, dir * 300.0, contact * 1.2, Look::Shard, NO_LINE, spec);
            }
        }
        // Viral Moment: deleted slop explode and hurt the slop around them.
        for (at, damage) in kill_blasts {
            self.events.push(Event::Splash { at, radius: 85.0 });
            let near: Vec<usize> = (0..self.slop.len()).filter(|&j| self.slop[j].hp > 0.0 && self.slop[j].pos.distance_sq(at) < 85.0 * 85.0).collect();
            for j in near {
                self.damage_slop(j, damage.max(4.0), crate::stats::DamageType::Sarcasm, V2::ZERO, at, None);
            }
        }
        // Lifesteal is capped at 6 Mental Health a second.
        let heal = (self.player.perks.lifesteal * kills_now as f32).min(0.1);
        if heal > 0.0 {
            let p = &mut self.player;
            p.mental_health = (p.mental_health + heal).min(p.stats.max_mental_health);
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
        // Viral elites burst into four small copies of themselves.
        for (spec, at) in virals {
            for k in 0..4 {
                let id = self.place(spec as usize, at + V2::from_angle(k as f32 * 1.57) * 30.0, 0, 1);
                if let Some(s) = self.slop.iter_mut().find(|s| s.id == id) {
                    s.reposted = true;
                    s.hp *= 0.15;
                    s.max_hp *= 0.15;
                    s.radius *= 0.6;
                }
            }
            self.events.push(Event::Split { at });
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
        if let Some(at) = boss_died {
            self.boss_defeated(at);
        }
    }

    /// Likes within Curiosity fly to you and become experience; fakes explode
    /// (LIKE-01).
    fn collect_likes(&mut self) {
        let p = self.player.pos;
        let curiosity = self.player.stats.curiosity;
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
            // Far ones fly faster, so a vacuum empties the map in a second or two.
            let step = (LIKE_PULL_SPEED + d * 1.5) * DT;
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
        // Too many on the floor: the farthest are credited at once, so nothing
        // earned is lost and the floor stays readable.
        if self.likes.len() > MAX_LIKES {
            self.likes.sort_by(|a, b| b.pos.distance_sq(p).total_cmp(&a.pos.distance_sq(p)));
            let extra = self.likes.len() - MAX_LIKES;
            got += self.likes.drain(..extra).filter(|l| !l.fake).map(|l| l.value).sum::<u32>();
        }
        if got > 0 {
            self.xp += got;
            self.tally.likes_collected += got;
            self.events.push(Event::LikeCollected { value: got });
        }
        for _ in 0..fakes {
            self.events.push(Event::Splash { at: p, radius: 50.0 });
            self.hurt(5.0, NO_SPEC);
            self.add_anxiety();
        }
    }

    /// Food, magnets, bombs, freezes, Clout and chests (LOOT-01).
    fn collect_pickups(&mut self) {
        let me = self.player.pos;
        let pull = self.player.stats.curiosity * 0.7;
        let mut got = Vec::new();
        self.pickups.retain_mut(|k| {
            let d = k.pos.distance(me);
            if k.kind == PickupKind::Clout && d < pull {
                k.pulled = true;
            }
            if k.pulled {
                let step = (LIKE_PULL_SPEED + d * 1.5) * DT;
                k.pos += (me - k.pos).normalized() * step.min(d);
            }
            if d < PLAYER_RADIUS + 22.0 {
                got.push((k.kind, k.value, k.pos));
                return false;
            }
            true
        });
        for (kind, value, at) in got {
            match kind {
                PickupKind::Food => {
                    let p = &mut self.player;
                    p.mental_health = (p.mental_health + 25.0 + p.stats.max_mental_health * 0.1).min(p.stats.max_mental_health);
                }
                PickupKind::Magnet => {
                    // A vacuum: every Like and coin on the map flies to you.
                    for l in self.likes.iter_mut().filter(|l| !l.fake) {
                        l.pulled = true;
                    }
                    for k in self.pickups.iter_mut().filter(|k| k.kind == PickupKind::Clout) {
                        k.pulled = true;
                    }
                    self.events.push(Event::Vacuum);
                }
                PickupKind::Bomb => self.mass_report(),
                PickupKind::Inject => {
                    self.injection = INJECTION_TICKS;
                    self.events.push(Event::PromptInjection { at: me });
                }
                PickupKind::Freeze => self.freeze = FREEZE_TICKS,
                PickupKind::Clout => self.clout += value,
                PickupKind::Chest => self.chests.push(value),
            }
            self.events.push(Event::PickedUp { kind, at });
        }
    }

    /// Mass Report: deletes the slop on screen; bosses lose 5%.
    pub(crate) fn mass_report(&mut self) {
        let me = self.player.pos;
        for j in 0..self.slop.len() {
            let s = &self.slop[j];
            let d = s.pos - me;
            if d.x.abs() > 900.0 || d.y.abs() > 520.0 || s.hp <= 0.0 {
                continue;
            }
            let boss = matches!(self.setup.specs[s.spec as usize].movement, Movement::Boss(_));
            if boss {
                let hit = s.max_hp * 0.05;
                self.slop[j].hp -= hit;
            } else {
                self.slop[j].hp = 0.0;
                self.slop[j].last_type = crate::stats::DamageType::Fact as u8;
            }
        }
        self.events.push(Event::MassReport { at: me });
    }

    /// Props you walk into or stand in (MAP-03). Breakables drop loot; the
    /// rest are handed to the rules through `used`.
    fn use_props(&mut self) {
        let me = self.player.pos;
        let mut broke = Vec::new();
        let mut uploads = Vec::new();
        let mut finished = Vec::new();
        let mut used = Vec::new();
        let tagged: Vec<u32> = self.slop.iter().map(|s| s.tag).chain(self.warnings.iter().map(|w| w.tag)).filter(|&t| t != 0).collect();
        for p in &mut self.map.props {
            if p.state == PropState::Done || p.pos.distance_sq(me) > 900.0 * 900.0 {
                continue;
            }
            let k = p.kind;
            let inside = p.pos.distance(me) < k.radius() + PLAYER_RADIUS * 0.5;
            if k.breakable() {
                if inside {
                    p.state = PropState::Done;
                    broke.push((k, p.pos));
                }
                continue;
            }
            match (k, p.state) {
                (PropKind::Hotspot, _) => {
                    p.state = if inside { PropState::Active } else { PropState::Idle };
                    if inside && p.left > 0 {
                        p.left -= 1;
                        let h = &mut self.player;
                        h.mental_health = (h.mental_health + h.stats.max_mental_health * 0.04 * DT).min(h.stats.max_mental_health);
                        if p.left == 0 {
                            p.state = PropState::Done;
                        }
                    }
                }
                (PropKind::PowerOutlet, _) => {
                    // Charging: dashes refill, and you run faster for a while.
                    p.state = if inside { PropState::Active } else { PropState::Idle };
                    if inside && p.left > 0 {
                        p.left -= 1;
                        let h = &mut self.player;
                        h.dashes = h.perks.dash_charges;
                        h.charged = CHARGED_TICKS;
                        if p.left == 0 {
                            p.state = PropState::Done;
                        }
                    }
                }
                (PropKind::DataCenter, PropState::Active) => {
                    if inside {
                        p.left = p.left.saturating_sub(1);
                        if p.left % 90 == 0 {
                            uploads.push(p.pos);
                        }
                    }
                    if p.left == 0 {
                        p.state = PropState::Done;
                        finished.push((k, p.pos));
                    }
                }
                (PropKind::CommentSection, PropState::Active) => {
                    if !tagged.contains(&p.id) {
                        p.state = PropState::Done;
                        finished.push((k, p.pos));
                    }
                }
                (_, PropState::Idle) => {
                    let need = k.charge();
                    if !inside {
                        p.charge = 0;
                    } else if p.charge < need {
                        p.charge += 1;
                        if p.charge == need {
                            used.push(p.id);
                            if k == PropKind::DataCenter {
                                p.state = PropState::Active;
                            }
                            if k == PropKind::GoLive {
                                p.state = PropState::Done;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        for (k, at) in broke {
            self.events.push(Event::Broke { at, kind: k });
            match k {
                PropKind::LostPhone => self.drop_pickup(PickupKind::Chest, at, 1),
                PropKind::CryptoMiner => {
                    for _ in 0..4 {
                        self.drop_pickup(PickupKind::Clout, at, 3);
                    }
                }
                _ => {
                    let roll = self.rng_combat.below(100);
                    let kind = match (k, roll) {
                        (PropKind::PopupAd, 0..=59) => PickupKind::Clout,
                        (PropKind::PopupAd, 60..=84) => PickupKind::Food,
                        (PropKind::PopupAd, _) => PickupKind::Magnet,
                        (_, 0..=34) => PickupKind::Clout,
                        (_, 35..=59) => PickupKind::Food,
                        (_, 60..=74) => PickupKind::Magnet,
                        (_, 75..=87) => PickupKind::Freeze,
                        _ => PickupKind::Bomb,
                    };
                    let value = if kind == PickupKind::Clout { 2 + self.rng_combat.below(4) } else { 0 };
                    self.drop_pickup(kind, at, value);
                }
            }
        }
        // An upload draws the slop in.
        for at in uploads {
            if let Some(e) = self.pick_spawn(self.elapsed()) {
                self.summon(e.spec, 3, Some(at), 0);
            }
        }
        for (k, at) in finished {
            self.drop_pickup(PickupKind::Chest, at, 1);
            if k == PropKind::CommentSection {
                for _ in 0..5 {
                    self.drop_pickup(PickupKind::Clout, at, 4);
                }
            }
            self.events.push(Event::PropDone { at, kind: k });
        }
        for id in used {
            let at = self.map.props.iter().find(|p| p.id == id).map_or(me, |p| p.pos);
            let kind = self.map.props.iter().find(|p| p.id == id).map_or(PropKind::Vending, |p| p.kind);
            self.events.push(Event::PropUsed { at, kind });
            self.used.push(id);
        }
    }

    fn tick_timers(&mut self) {
        let p = &mut self.player;
        p.invulnerable = p.invulnerable.saturating_sub(1);
        p.charged = p.charged.saturating_sub(1);
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
            s.staggered = s.staggered.saturating_sub(1);
            s.confused = s.confused.saturating_sub(1);
            s.rooted = s.rooted.saturating_sub(1);
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
    /// runs out (WAVE-01). An endless run only ends in Burnout.
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
        mix(self.xp);
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
