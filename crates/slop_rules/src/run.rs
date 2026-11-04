//! A run (RUN-01 to RUN-12): one endless trip through the Feed. The world is
//! never rebuilt; the rules watch it, retune who spawns as time goes by,
//! bring in each platform's boss, and pause it for your choices.
//!
//! ```text
//! Playing -> LevelUp (one per level gained) -> Playing
//!         -> Chest (a merge, or upgrades)    -> Glitch after a boss's chest
//!         -> Vending / Shrine (a machine)    -> Playing
//!         -> Over on Burnout (a victory if The Model fell first)
//! ```
//!
//! Besides the steady crowd, the director throws events: swarms crossing
//! the screen, rings of slop closing in, and livestreams that lock you in
//! (EVENT-01, EVENT-02).

use slop_sim::content::{merge_of, weapon_index, Archetype, BossKind, Pattern, Platform, WEAPONS};
use slop_sim::map::{PropKind, PropState};
use slop_sim::rng::Rng;
use slop_sim::spec::SlopSpec;
use slop_sim::world::{Loadout, Outcome, Tally, Weapon, WaveSetup, World, MAX_LEVEL, TICK_RATE};
use slop_sim::{DamageType, Input, Stats, V2};

use crate::heroes::Hero;
use crate::items::{self, item_index, tier_price, ItemKind, Mod, StatKind, BLESSINGS, GLITCHES, ITEMS};
use crate::chapters::Chapter;
use crate::levels::{self, StatUp};
use crate::memory::{Memory, Temperature};
use crate::training::{PatchNotes, Training};
use crate::waves::{self, Affix};

/// Weapon slots before hero rules.
pub const WEAPON_SLOTS: usize = 6;
pub const ITEM_SLOTS: usize = 6;
const CHOICES: usize = 3;
/// Seconds between retunes of the spawn table, retraining, and elites.
const RETUNE_EVERY: f32 = 20.0;
const RETRAIN_EVERY: f32 = 120.0;
const ELITE_EVERY: f32 = 30.0;
const FREE_REROLLS: u32 = 1;
const COMMENT_ELITES: u32 = 6;
/// Drawbacks never take your maximum Mental Health below this.
const MIN_MENTAL_HEALTH: f32 = 30.0;
/// Everyone starts with two dash charges that come back in 3 s (DASH-01).
const BASE_DASHES: u8 = 2;
const BASE_DASH_RECHARGE: f32 = 3.0;
const MIN_DASH_RECHARGE: f32 = 0.8;
/// Seconds between director events (EVENT-01, EVENT-02): the first one, and
/// the gap after.
const SWARM_FIRST: f32 = 75.0;
const SWARM_EVERY: (f32, f32) = (70.0, 110.0);
const RING_FIRST: f32 = 200.0;
const RING_EVERY: (f32, f32) = (140.0, 200.0);
const LIVE_FIRST: f32 = 540.0;
const LIVE_EVERY: (f32, f32) = (260.0, 360.0);
/// When the Nemesis arrives, and how far ahead the slop start on your
/// favourite damage type (NEMESIS-01).
const NEMESIS_AT: f32 = 150.0;
/// How long the bot plays your character at the end (END-01).
const TAKEOVER_SECONDS: f32 = 20.0;
const REMEMBERED: f32 = 0.09;
const GOBLIN_FIRST: f32 = 150.0;
const GOBLIN_EVERY: (f32, f32) = (150.0, 220.0);
const LIVE_RADIUS: f32 = 430.0;
const LIVE_SECONDS: f32 = 28.0;
/// A glitch card turns up on a level-up this often, plus Luck (GLITCH-01).
const GLITCH_CHANCE: f32 = 0.015;

/// How often an item of a rarity is offered: legendaries are rare, and Luck
/// makes them less so (ITEM-03).
pub fn rarity_weight(tier: u8, luck: f32) -> f32 {
    match tier {
        1 => 1.0,
        2 => 0.75,
        3 => 0.4 + luck * 0.01,
        _ => 0.04 + luck * 0.004,
    }
}

/// The Creator Fund pays for every minute survived, in Clout.
pub fn payout(minute: u32) -> u32 {
    2 + minute / 3
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuleError {
    WrongPhase,
    NotEnoughClout,
    NoSuchOffer,
    /// Automation took the choice away (AUTO-03).
    Automated,
}

impl std::fmt::Display for RuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            RuleError::WrongPhase => "Not now.",
            RuleError::NotEnoughClout => "Not enough Clout.",
            RuleError::NoSuchOffer => "No such offer.",
            RuleError::Automated => "The model has already considered the alternatives.",
        })
    }
}

/// One pick on a level-up (LEVEL-02). Every one on offer is valid.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Choice {
    NewWeapon(usize),
    /// A level for the weapon with this index.
    WeaponLevel(usize),
    NewItem(usize),
    ItemLevel(usize),
    Stat(StatUp),
    /// Nothing else left: heal.
    Heal,
    /// Nothing else left: Clout.
    Clout(u32),
    /// A rare perk that changes the run (GLITCH-01), by index into `GLITCHES`.
    Glitch(usize),
    /// The easy option (AUTO-01).
    Delegate(Delegate),
}

impl Choice {
    /// How rare the choice is, 1 to 4, for frames and fanfare (ITEM-03).
    pub fn rarity(self) -> u8 {
        match self {
            Choice::NewItem(i) | Choice::ItemLevel(i) => ITEMS[i].tier,
            Choice::Glitch(_) => 4,
            Choice::Delegate(_) => 3,
            Choice::Stat(up) => up.tier,
            Choice::NewWeapon(d) | Choice::WeaponLevel(d) if WEAPONS[d].rare => 3,
            _ => 1,
        }
    }
}

/// What a chest (or a gacha pull) gave you; it is applied already.
#[derive(Clone, Debug, PartialEq)]
pub struct Reward {
    pub title: &'static str,
    /// Two weapons and what they merged into.
    pub merged: Option<(usize, usize, usize)>,
    pub upgrades: Vec<Choice>,
    pub clout: u32,
    /// The rarest thing inside, 1 to 4: how loud the opening is (LOOT-02).
    pub best: u8,
}

#[derive(Clone, Debug)]
pub enum Phase {
    Playing,
    /// Pick one of these; more level-ups may be queued.
    LevelUp(Vec<Choice>),
    Chest(Reward),
    /// A vending machine: items and their prices.
    Vending { prop: u32, offers: Vec<(Choice, u32)> },
    /// A shrine: pick one blessing.
    Shrine { prop: u32, offers: Vec<usize> },
    /// After a boss: pick one of three glitches (GLITCH-01).
    Glitch(Vec<usize>),
    /// A recycle bin: pick one of your items to throw away for Clout (MAP-05).
    Recycle { prop: u32, items: Vec<usize> },
    /// Between chapters: the next chapter's card (CH-01).
    ChapterCard(Chapter),
    /// The takeover (END-01): the bot plays your character for a while, and
    /// then it is over.
    Takeover { left: f32 },
    Over { victory: bool },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnedWeapon {
    pub def: usize,
    pub level: u8,
    /// Handed to the AI: it levels itself and is no longer yours to change
    /// (AUTO-02).
    pub agent: bool,
}

/// How one chapter went (CH-01): how long, and what deleted you with its
/// lesson (LESSON-01).
#[derive(Clone, Debug, PartialEq)]
pub struct ChapterResult {
    pub chapter: Chapter,
    pub seconds: f32,
    pub kills: u32,
    pub killer: Option<(String, String)>,
}

/// The easy options (AUTO-01): always the strongest card on offer, always
/// at the cost of Automation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Delegate {
    /// This weapon becomes an Agent.
    Agent(usize),
    /// Level-ups pick themselves after a moment; auto-picks bring a bonus.
    AutoPick,
    /// The game dashes for you; dashes recharge faster.
    AssistDash,
    /// Weapons prefer the biggest target; +20% damage.
    SmartTargeting,
    /// For 10 s of every minute the bot drives, and you hit harder while it does.
    AutopilotTrial,
    /// Every weapon becomes an Agent; +40% damage.
    FullAutomation,
}

impl Delegate {
    pub fn cost(self) -> u32 {
        match self {
            Delegate::Agent(_) | Delegate::SmartTargeting => 8,
            Delegate::AutoPick | Delegate::AssistDash => 10,
            Delegate::AutopilotTrial => 15,
            Delegate::FullAutomation => 25,
        }
    }

    pub fn name(self) -> String {
        match self {
            Delegate::Agent(d) => format!("Agent: {}", WEAPONS[d].name),
            Delegate::AutoPick => "Auto-pick".into(),
            Delegate::AssistDash => "Assisted dash".into(),
            Delegate::SmartTargeting => "Smart targeting".into(),
            Delegate::AutopilotTrial => "Autopilot (trial)".into(),
            Delegate::FullAutomation => "Full automation".into(),
        }
    }

    pub fn effect(self) -> &'static str {
        match self {
            Delegate::Agent(_) => "x1.6 damage, x0.8 cooldown, and it levels itself every 2 minutes. You can no longer level, merge or recycle it.",
            Delegate::AutoPick => "Level-ups pick themselves after 3 s, and every auto-pick brings a free stat-up. You can still pick first.",
            Delegate::AssistDash => "The game dashes for you whenever it would. Dashes recharge 40% faster.",
            Delegate::SmartTargeting => "Every weapon prefers the biggest target in range. +20% damage. Targets are no longer the nearest.",
            Delegate::AutopilotTrial => "For 10 s of every minute the model drives. It dodges better than you. +15% damage while it does.",
            Delegate::FullAutomation => "Every weapon becomes an Agent. +40% damage.",
        }
    }
}

pub struct Run {
    pub seed: u64,
    pub hero: Hero,
    /// The platforms in the order the Feed visits them.
    pub platforms: Vec<Platform>,
    /// Platforms passed so far; 6 is The Model.
    pub stage: usize,
    pub platform: Platform,
    /// When the current platform started, in seconds of the run.
    pub platform_since: f32,
    pub bosses_beaten: u32,
    pub model_beaten: bool,
    /// A boss of this kind is out.
    pub boss_out: Option<BossKind>,
    pub clout: u32,
    pub xp: u32,
    pub level: u32,
    pub weapons: Vec<OwnedWeapon>,
    /// Item index and level.
    pub items: Vec<(usize, u32)>,
    /// Shrine blessings, as indexes into `BLESSINGS`.
    pub blessings: Vec<usize>,
    /// Glitches taken, as indexes into `GLITCHES`.
    pub glitches: Vec<usize>,
    /// The part of the map you are in (BIOME-01); the spawns follow it.
    pub biome: Platform,
    /// Stats from your hero and level-ups; items are added by `loadout`.
    pub base: Stats,
    pub pending_levels: u32,
    /// Free rerolls left on this level-up, and paid ones so far.
    pub rerolls: u32,
    pub paid_rerolls: u32,
    pub training: Training,
    pub phase: Phase,
    pub world: Option<World>,
    /// Damage each weapon has dealt over the run, by weapon index.
    pub weapon_damage: Vec<f32>,
    /// What the slop learned last.
    pub notes: Option<PatchNotes>,
    /// One-line news for the client to show and clear.
    pub toasts: Vec<String>,
    /// Seconds added to the clock (the title screen's demo starts late).
    pub offset: f32,
    genomes: Vec<(usize, usize)>,
    rng: Rng,
    xp_seen: u32,
    clout_seen: u32,
    next_retune: f32,
    next_retrain: f32,
    next_elite: f32,
    next_minute: f32,
    next_swarm: f32,
    next_ring: f32,
    next_live: f32,
    /// A boss fell: its chest is followed by a pick of glitches.
    pending_glitch: bool,
    /// Clout held, in tens, when the loadout was last synced (Hoarder).
    clout_tens: u32,
    /// A merge was announced as ready.
    merge_told: bool,
    /// Spawn multiplier from a Cursed Feed; the surge multiplies it.
    spawn_curse: f32,
    next_goblin: f32,
    /// When the Nemesis comes, and what it is, if the slop remember one.
    next_nemesis: f32,
    nemesis: Option<SlopSpec>,
    /// You deleted your Nemesis this run.
    pub nemesis_beaten: bool,
    /// How wild this run is (TEMP-01).
    pub temperature: Temperature,
    /// What the slop remembered when it started (NEMESIS-01).
    pub memory: Memory,
    /// The chapter you are in and when it began (CH-01).
    pub chapter: Chapter,
    pub chapter_since: f32,
    /// What happened in each chapter so far.
    pub chapters: Vec<ChapterResult>,
    /// Automation, 0 to 100, never down (AUTO-01).
    pub automation: u32,
    pub delegated: Vec<Delegate>,
    /// Unplugged: Automation does not rise this chapter (AUTO-04).
    pub unplugged: bool,
    /// Thresholds already announced, as bits.
    told: u8,
    /// The bot's pick on this level-up, when Automation suggests (AUTO-03).
    pub suggested: Option<usize>,
    /// Agents that defected in the Resistance (AUTO-02).
    pub defected: Vec<usize>,
    chapter_bosses: u32,
    /// The final score (END-01).
    pub score: u32,
    trained: Tally,
}

impl Run {
    /// A run that starts in the part of the map where you stand: that
    /// platform's boss comes first.
    pub fn new(seed: u64, hero: Hero) -> Run {
        let first = slop_sim::map::biome_at(seed, V2::ZERO, Chapter::Feed.def().biomes);
        Run::on(seed, hero, first, Rng::new(seed, 7))
    }

    /// A run at a Temperature, against slop that remember you (TEMP-01,
    /// NEMESIS-01): it starts ahead on your favourite damage type, and what
    /// deleted you last comes back.
    pub fn with(seed: u64, hero: Hero, temperature: Temperature, memory: Memory) -> Run {
        let mut run = Run::new(seed, hero);
        run.temperature = temperature;
        if let Some(fav) = memory.favourite {
            for (k, r) in run.training.model.iter_mut().enumerate() {
                *r += if k == fav { REMEMBERED } else { -REMEMBERED / 3.0 };
            }
        }
        run.nemesis = memory.nemesis();
        run.memory = memory;
        if let Some(w) = run.world.as_mut() {
            w.hallucinate = temperature.hallucination();
        }
        run.sync();
        run.retune();
        run
    }

    /// A run that starts on a chosen platform.
    pub fn on_platform(seed: u64, hero: Hero, platform: Platform) -> Run {
        Run::on(seed, hero, platform, Rng::new(seed, 7))
    }

    fn on(seed: u64, hero: Hero, first: Platform, mut rng: Rng) -> Run {
        let def = hero.def();
        let weapon = weapon_index(def.weapon).expect("hero weapons exist");
        let item = item_index(def.item).expect("hero items exist");
        let training = Training::new(&mut rng);
        let mut rest: Vec<Platform> = Chapter::Feed.def().biomes.iter().copied().filter(|p| *p != first).collect();
        for i in (1..rest.len()).rev() {
            let j = rng.below(i as u32 + 1) as usize;
            rest.swap(i, j);
        }
        let mut platforms = vec![first];
        platforms.extend(rest);
        let mut run = Run {
            seed,
            hero,
            platforms,
            stage: 0,
            platform: first,
            platform_since: 0.0,
            bosses_beaten: 0,
            model_beaten: false,
            boss_out: None,
            clout: 0,
            xp: 0,
            level: 1,
            weapons: vec![OwnedWeapon { def: weapon, level: 1, agent: false }],
            items: vec![(item, 1)],
            blessings: Vec::new(),
            glitches: Vec::new(),
            biome: first,
            base: Stats::default(),
            pending_levels: 0,
            rerolls: 0,
            paid_rerolls: 0,
            training,
            phase: Phase::Playing,
            world: None,
            weapon_damage: vec![0.0; WEAPONS.len()],
            notes: None,
            toasts: Vec::new(),
            offset: 0.0,
            genomes: Vec::new(),
            rng,
            xp_seen: 0,
            clout_seen: 0,
            next_retune: 0.0,
            next_retrain: RETRAIN_EVERY,
            next_elite: 90.0,
            next_minute: 60.0,
            next_swarm: SWARM_FIRST,
            next_ring: RING_FIRST,
            next_live: LIVE_FIRST,
            pending_glitch: false,
            clout_tens: 0,
            merge_told: false,
            spawn_curse: 1.0,
            next_goblin: GOBLIN_FIRST,
            next_nemesis: NEMESIS_AT,
            nemesis: None,
            nemesis_beaten: false,
            temperature: Temperature::Default,
            memory: Memory::default(),
            chapter: Chapter::Feed,
            chapter_since: 0.0,
            chapters: Vec::new(),
            automation: 0,
            delegated: Vec::new(),
            unplugged: false,
            told: 0,
            suggested: None,
            defected: Vec::new(),
            chapter_bosses: 0,
            score: 0,
            trained: Tally::default(),
        };
        let mut l = Loadout { weapons: vec![], agents: Vec::new(), stats: run.base, perks: Default::default() };
        for m in def.mods {
            items::apply(*m, &mut l, 0);
        }
        run.base = l.stats;
        let mut setup = WaveSetup::sandbox();
        setup.specs.clear();
        setup.seconds = 0.0;
        setup.platform = Some(first);
        setup.biomes = Chapter::Feed.def().biomes.to_vec();
        run.world = Some(World::new(seed, setup, &run.loadout()));
        run.retune();
        run
    }

    /// A run `minutes` in, with a few levelled weapons and plenty of Mental
    /// Health: the bots play it behind the title screen.
    pub fn showcase(seed: u64, hero: Hero, minutes: f32) -> Run {
        Run::showcase_build(seed, hero, minutes, &[])
    }

    /// A showcase of `build` (weapon index and level): the title replays
    /// your last run's build (END-01). An empty build is made up.
    pub fn showcase_build(seed: u64, hero: Hero, minutes: f32, build: &[(usize, u8)]) -> Run {
        let mut run = Run::new(seed, hero);
        for &(def, level) in build.iter().filter(|(d, _)| *d < WEAPONS.len()).take(WEAPON_SLOTS) {
            match run.weapons.iter_mut().find(|w| w.def == def) {
                Some(w) => w.level = level.clamp(1, MAX_LEVEL),
                None => run.weapons.push(OwnedWeapon { def, level: level.clamp(1, MAX_LEVEL), agent: false }),
            }
        }
        while run.weapons.len() < 4 {
            let def = run.rng.below(WEAPONS.len() as u32) as usize;
            if !WEAPONS[def].evolved && run.weapons.iter().all(|w| w.def != def) {
                run.weapons.push(OwnedWeapon { def, level: 4, agent: false });
            }
        }
        if build.is_empty() {
            run.weapons[0].level = 6;
        }
        run.base.max_mental_health += 400.0;
        run.base.conviction += 1.5;
        run.offset = minutes * 60.0;
        run.next_retrain += run.offset;
        run.next_elite += run.offset;
        run.next_minute += run.offset;
        run.next_swarm += run.offset;
        run.next_ring += run.offset;
        run.next_live += run.offset;
        run.next_goblin += run.offset;
        run.platform_since = run.offset;
        run.sync();
        if let Some(w) = run.world.as_mut() {
            w.player.mental_health = w.player.stats.max_mental_health;
        }
        run.retune();
        run
    }

    /// Seconds into the run, as the director counts them.
    pub fn seconds(&self) -> f32 {
        self.world.as_ref().map_or(0.0, |w| w.tick as f32 / TICK_RATE as f32) + self.offset
    }

    pub fn minutes(&self) -> f32 {
        self.seconds() / 60.0
    }

    /// Every modifier from the items (once per level) and blessings.
    /// An item's drawback counts once; only its upside grows with its level
    /// (ITEM-01).
    fn item_mods(&self) -> Vec<Mod> {
        let mut out: Vec<Mod> = Vec::new();
        for &(i, n) in &self.items {
            out.extend_from_slice(ITEMS[i].mods);
            for _ in 1..n {
                out.extend(ITEMS[i].mods.iter().copied().filter(|m| !items::is_drawback(m)));
            }
        }
        for &b in &self.blessings {
            out.extend_from_slice(BLESSINGS[b].mods);
        }
        for &g in &self.glitches {
            out.extend_from_slice(GLITCHES[g].mods);
        }
        out
    }

    /// Every modifier from the hero, the items and blessings.
    fn mods(&self) -> Vec<Mod> {
        let mut out: Vec<Mod> = self.hero.def().mods.to_vec();
        out.extend(self.item_mods());
        out
    }

    fn has(&self, f: impl Fn(&Mod) -> bool) -> bool {
        self.mods().iter().any(f)
    }

    fn sum(&self, f: impl Fn(&Mod) -> f32) -> f32 {
        self.mods().iter().map(f).sum()
    }

    fn count(&self, f: impl Fn(&Mod) -> bool) -> u32 {
        self.mods().iter().filter(|m| f(m)).count() as u32
    }

    /// Your weapons, stats and perks, as the world needs them (RUN-02).
    pub fn loadout(&self) -> Loadout {
        let mut l = Loadout { weapons: self.weapons.iter().map(|w| (w.def, w.level)).collect(), agents: Vec::new(), stats: self.base, perks: Default::default() };
        l.perks.dash_charges = BASE_DASHES;
        l.perks.dash_recharge = BASE_DASH_RECHARGE;
        let types = DamageType::ALL.iter().filter(|t| self.weapons.iter().any(|w| WEAPONS[w.def].kind == **t)).count();
        // The hero's stats are in `base` already; its other effects are not.
        let hero = self.hero.def().mods.iter().copied().filter(|m| !matches!(m, Mod::Stat(..)));
        for m in self.item_mods().into_iter().chain(hero) {
            items::apply(m, &mut l, types);
        }
        // What you delegated (AUTO-01, AUTO-02).
        l.agents = self.weapons.iter().filter(|w| w.agent).map(|w| w.def).collect();
        for d in &self.delegated {
            match d {
                Delegate::SmartTargeting => {
                    l.perks.smart_targeting = true;
                    l.stats.conviction += 0.2;
                }
                Delegate::AssistDash => l.perks.dash_recharge *= 0.6,
                Delegate::AutopilotTrial => l.perks.autopilot_bonus += 0.15,
                Delegate::FullAutomation => l.stats.conviction += 0.4,
                _ => {}
            }
        }
        l.perks.dash_recharge = l.perks.dash_recharge.max(MIN_DASH_RECHARGE);
        l.perks.like_bonus += self.temperature.rewards();
        // Clout is power (Influencer, Hoarder): +1% per 10 Clout held, capped.
        let cap = self.sum(|m| if let Mod::CloutDamage(v) = m { *v } else { 0.0 });
        if cap > 0.0 {
            l.stats.conviction += ((self.clout / 10) as f32 * 0.01).min(cap);
        }
        // Set bonuses (DMG-03): two weapons of a type add 10% to it.
        for t in DamageType::ALL {
            let n = self.weapons.iter().filter(|w| WEAPONS[w.def].kind == t).count() as u8;
            l.perks.type_count[t as usize] = n;
            if n >= 2 {
                l.perks.type_bonus[t as usize] += 0.10;
            }
            if n >= 4 && t == DamageType::Grass {
                l.perks.grass_heal += 1.0;
            }
        }
        l.stats.max_mental_health = l.stats.max_mental_health.max(MIN_MENTAL_HEALTH);
        l
    }

    /// The stats you have now, for the stats panel.
    pub fn stats(&self) -> Stats {
        self.loadout().stats
    }

    /// Hands your loadout to the world without resetting it.
    fn sync(&mut self) {
        let l = self.loadout();
        // A Cursed Feed brings more slop (GLITCH-01).
        let curse: f32 = self.mods().iter().map(|m| if let Mod::Curse(v) = m { *v } else { 0.0 }).sum();
        self.spawn_curse = 1.0 + curse;
        self.clout_tens = self.clout / 10;
        let Some(w) = self.world.as_mut() else { return };
        w.spawn_boost = 1.0 + curse;
        for (slot, dmg) in w.tally.weapon_damage.iter().enumerate() {
            if let Some(pw) = w.player.weapons.get(slot) {
                self.weapon_damage[pw.def] += dmg;
            }
        }
        let old = std::mem::take(&mut w.player.weapons);
        w.player.weapons = l
            .weapons
            .iter()
            .map(|&(d, lv)| {
                let mut nw = old.iter().find(|o| o.def == d).cloned().unwrap_or_else(|| Weapon::new(d, lv));
                nw.level = lv.clamp(1, MAX_LEVEL);
                nw.agent = l.agents.contains(&d);
                nw
            })
            .collect();
        w.tally.weapon_damage = vec![0.0; w.player.weapons.len()];
        let grow = l.stats.max_mental_health - w.player.stats.max_mental_health;
        let shield_grow = l.stats.boundaries - w.player.stats.boundaries;
        w.player.stats = l.stats;
        if grow > 0.0 {
            w.player.mental_health += grow;
        }
        if shield_grow > 0.0 {
            w.player.shield += shield_grow;
        }
        w.player.mental_health = w.player.mental_health.min(l.stats.max_mental_health);
        w.player.active_cooldown.resize(l.perks.actives.len(), 0.0);
        if l.perks.dash_charges > w.player.perks.dash_charges {
            w.player.dashes += l.perks.dash_charges - w.player.perks.dash_charges;
        }
        w.player.dashes = w.player.dashes.min(l.perks.dash_charges);
        if l.perks.second_wind > w.player.perks.second_wind {
            w.player.second_winds += l.perks.second_wind - w.player.perks.second_wind;
        }
        w.player.perks = l.perks;
    }

    /// Damage by weapon index, including what the world has not booked yet.
    pub fn damage_by_weapon(&self) -> Vec<f32> {
        let mut d = self.weapon_damage.clone();
        if let Some(w) = &self.world {
            for (slot, dmg) in w.tally.weapon_damage.iter().enumerate() {
                if let Some(pw) = w.player.weapons.get(slot) {
                    d[pw.def] += dmg;
                }
            }
        }
        d
    }

    /// Hands the director's spawn table for now to the world (RUN-10).
    /// The table follows the part of the map you are in (BIOME-01).
    fn retune(&mut self) {
        let stage = self.capability();
        let mut t = waves::table(stage, self.biome, &self.training);
        for s in &mut t.specs {
            s.contact *= self.temperature.damage();
        }
        let Some(w) = self.world.as_mut() else { return };
        let base = w.retune(t.specs, t.spawns, t.rate, t.elite_chance, Some(self.biome), t.repost_chance);
        self.genomes = t.genomes.into_iter().map(|(spec, g)| (base + spec, g)).collect();
        self.next_retune = self.seconds() + RETUNE_EVERY;
    }

    /// The slop retrains on what killed it since last time (EVO-01).
    fn retrain(&mut self) {
        let Some(w) = self.world.as_ref() else { return };
        let mut delta = w.tally.clone();
        for k in 0..4 {
            delta.kills_by_type[k] -= self.trained.kills_by_type[k];
            delta.damage_dealt[k] -= self.trained.damage_dealt[k];
        }
        delta.damage_taken -= self.trained.damage_taken;
        let genome_tallies: Vec<_> = self.genomes.iter().filter_map(|&(spec, g)| w.spec_tally.get(spec).map(|t| (g, *t))).collect();
        self.trained = w.tally.clone();
        self.training.learn_rate = 1.0 + self.sum(|m| if let Mod::LearnFaster(v) = m { *v } else { 0.0 });
        let preview = self.has(|m| matches!(m, Mod::Preview));
        let version = (self.minutes() / 2.0) as u32 + 1;
        let notes = self.training.train(version, &delta, &genome_tallies, &mut self.rng, preview);
        if let Some(line) = notes.lines.first() {
            self.toasts.push(format!("{} patch notes: {}", notes.version, line.text));
        }
        self.notes = Some(notes);
        self.retune();
    }

    /// Advances the Feed one tick; returns true when the phase changed.
    pub fn step(&mut self, input: Input) -> bool {
        if let Phase::Takeover { left } = self.phase {
            return self.step_takeover(left);
        }
        if !matches!(self.phase, Phase::Playing) {
            return false;
        }
        // Automation drives, or helps, when you let it (AUTO-01, AUTO-03).
        let auto = self.autopilot_now();
        let input = self.steer_input(input, auto);
        let Some(world) = self.world.as_mut() else { return false };
        world.player.autopilot = auto;
        world.step(input);
        if world.outcome != Outcome::Running {
            self.sync();
            self.chapter_over();
            return true;
        }
        // Experience and Clout picked up this tick.
        let gained = world.xp - self.xp_seen;
        self.xp_seen = world.xp;
        let coins = world.clout - self.clout_seen;
        self.clout_seen = world.clout;
        let chests: Vec<u32> = std::mem::take(&mut world.chests);
        let used: Vec<u32> = std::mem::take(&mut world.used);
        let boss_gone = self.boss_out.is_some() && world.boss.is_none();
        let biome = world.biome;
        if coins > 0 {
            let bonus = self.sum(|m| if let Mod::CloutBonus(v) | Mod::Curse(v) = m { *v } else { 0.0 });
            self.clout += (coins as f32 * (1.0 + bonus + self.temperature.rewards()).max(0.1)).round() as u32;
        }
        // The floor and the HUD say where you are; no announcement.
        if biome != self.biome {
            self.biome = biome;
            self.retune();
        }
        // Hoarders get stronger as their Clout grows.
        if self.clout / 10 != self.clout_tens && self.has(|m| matches!(m, Mod::CloutDamage(_))) {
            self.sync();
        }
        self.xp += gained;
        while self.xp >= levels::xp_for(self.level + 1) {
            self.xp -= levels::xp_for(self.level + 1);
            self.level += 1;
            self.pending_levels += 1;
        }

        let t = self.seconds();
        if boss_gone {
            self.boss_defeated();
        }
        if t >= self.next_minute {
            self.next_minute += 60.0;
            self.minute();
        }
        if t >= self.next_retrain {
            self.next_retrain += RETRAIN_EVERY;
            self.retrain();
        } else if t >= self.next_retune {
            self.retune();
        }
        // A merge is ready: say so, and send an elite with a chest soon.
        if self.ready_merge().is_some() {
            if !self.merge_told {
                self.merge_told = true;
                self.toasts.push("MERGE READY: the next chest merges them.".into());
                self.next_elite = self.next_elite.min(t + 8.0);
            }
        } else {
            self.merge_told = false;
        }
        if t >= self.next_elite {
            self.next_elite += ELITE_EVERY;
            // More elites as the Feed goes on (ELITE-02).
            let n = 1 + (self.minutes() / 8.0).min(2.0) as u32;
            self.elite(n, None, 0);
        }
        // The crowd surges and ebbs (EVENT-03).
        if let Some(w) = self.world.as_mut() {
            let surge = if self.temperature.surges() { waves::surge(t) } else { 1.0 };
            w.spawn_boost = self.spawn_curse * self.temperature.spawns() * surge;
            self.nemesis_beaten |= w.nemesis_down;
        }
        self.director(t);
        self.schedule_boss(t);
        for id in used {
            self.use_prop(id);
        }
        for value in chests {
            if matches!(self.phase, Phase::Playing) {
                let title = match value {
                    3.. => "Golden chest",
                    2 => "Big chest",
                    _ => "Chest",
                };
                self.open_chest(value, title);
            } else if let Some(w) = self.world.as_mut() {
                w.chests.push(value);
            }
        }
        if self.automation >= 100 && matches!(self.phase, Phase::Playing) {
            self.start_takeover();
            return true;
        }
        if self.pending_levels > 0 && matches!(self.phase, Phase::Playing) {
            self.start_level_up();
        }
        !matches!(self.phase, Phase::Playing)
    }

    /// Side hustles pay every minute (ITEM-02).
    fn minute(&mut self) {
        let minute = self.minutes() as u32;
        let mut income = self.sum(|m| if let Mod::WaveIncome(n) = m { *n as f32 } else { 0.0 }) as i32;
        income += (self.count(|m| matches!(m, Mod::InfluencerDeal)) * minute) as i32;
        let rate = self.sum(|m| if let Mod::Interest(v) = m { *v } else { 0.0 });
        income += ((self.clout as f32 * rate) as i32).min(20);
        for _ in 0..self.count(|m| matches!(m, Mod::CryptoWallet)) {
            let swing = self.rng.range(-0.10, 0.25);
            // It swings on at most 200 Clout, so it cannot snowball.
            income += (self.clout.min(200) as f32 * swing) as i32;
        }
        // Agents level themselves every other minute (AUTO-02).
        if minute % 2 == 0 && self.weapons.iter().any(|w| w.agent) {
            for w in &mut self.weapons {
                if w.agent {
                    w.level = (w.level + 1).min(MAX_LEVEL);
                }
            }
            self.sync();
        }
        income += payout(minute) as i32;
        // Hustle pays every minute (STAT-02).
        income += self.stats().hustle.max(0.0) as i32;
        self.clout = (self.clout as i32 + income).max(0) as u32;
    }

    /// The director's events (EVENT-01, EVENT-02): swarms crossing the
    /// screen, rings closing in, and livestreams that lock you in.
    fn director(&mut self, t: f32) {
        let minutes = self.minutes();
        let stage = self.capability();
        let calm = self.boss_out.is_some() || self.world.as_ref().is_none_or(|w| w.arena.is_some());
        // During a boss or a livestream, events wait a little.
        if calm {
            self.next_swarm = self.next_swarm.max(t + 15.0);
            self.next_ring = self.next_ring.max(t + 15.0);
            self.next_live = self.next_live.max(t + 15.0);
        }
        if t >= self.next_swarm {
            self.next_swarm = t + self.rng.range(SWARM_EVERY.0, SWARM_EVERY.1);
            {
                // Seen, not announced: the swarm is its own warning.
                let mut kinds = vec![Archetype::TrampolineBunny, Archetype::ListicleDrone];
                if minutes >= 4.0 {
                    kinds.push(Archetype::SixFingerHand);
                    kinds.push(Archetype::GlassFruit);
                }
                if minutes >= 7.0 {
                    kinds.push(Archetype::Gusello);
                }
                let kind = kinds[self.rng.below(kinds.len() as u32) as usize];
                let (hp, dmg, speed) = waves::scaling(stage, self.biome);
                let mut spec = SlopSpec::archetype(kind).scaled(hp * 0.6, dmg, speed).sweeping();
                spec.resist = self.training.model;
                let count = (36.0 + 3.0 * minutes).min(90.0) as u32;
                if let Some(w) = self.world.as_mut() {
                    let i = w.add_spec(spec);
                    w.sweep(i, count);
                }
            }
        }
        if t >= self.next_ring {
            self.next_ring = t + self.rng.range(RING_EVERY.0, RING_EVERY.1);
            {
                let (hp, dmg, speed) = waves::scaling(stage, self.biome);
                // A networking event: Recruiter Bots ring you and close in.
                let mut spec = SlopSpec::archetype(Archetype::RecruiterBot).scaled(hp * 0.8, dmg, speed);
                spec.name = "Networking Event".into();
                spec.speed *= 0.6;
                spec.resist = self.training.model;
                let count = (12.0 + minutes * 0.8).min(36.0) as u32;
                if let Some(w) = self.world.as_mut() {
                    let i = w.add_spec(spec);
                    w.encircle(i, count, 560.0);
                }
            }
        }
        if t >= self.next_live {
            self.next_live = t + self.rng.range(LIVE_EVERY.0, LIVE_EVERY.1);
            self.go_live();
        }
        // What deleted you last time comes back with a name (NEMESIS-01).
        if t >= self.next_nemesis {
            if let Some(base) = self.nemesis.take() {
                let (hp, dmg, speed) = waves::scaling(stage, self.biome);
                let mut spec = base.scaled(hp * 2.0, dmg * self.temperature.damage(), speed).elite();
                spec.traits.0 |= slop_sim::spec::Traits::NEMESIS;
                if let Some(w) = self.world.as_mut() {
                    let i = w.add_spec(spec);
                    let near = w.player.pos;
                    w.summon(i, 1, Some(near), 0);
                }
            }
        }
        // The loot goblin: a Crypto Bro runs past with a bag of Clout.
        if t >= self.next_goblin {
            self.next_goblin = t + self.rng.range(GOBLIN_EVERY.0, GOBLIN_EVERY.1);
            let (hp, _, _) = waves::scaling(stage, self.biome);
            let spec = SlopSpec::archetype(Archetype::LootGoblin).scaled(hp, 1.0, 1.0);
            if let Some(w) = self.world.as_mut() {
                let i = w.add_spec(spec);
                let near = w.player.pos;
                w.summon(i, 1, Some(near), 0);
            }
        }
    }

    /// A livestream: locked in with a growing crowd, then a big chest.
    fn go_live(&mut self) {
        if let Some(w) = self.world.as_mut() {
            w.start_arena(LIVE_RADIUS, LIVE_SECONDS);
        }
    }

    /// An elite with an affix, off-screen or around `near` (ELITE-01).
    fn elite(&mut self, count: u32, near: Option<slop_sim::V2>, tag: u32) {
        let stage = self.capability();
        let t = waves::table(stage, self.biome, &self.training);
        let singles: Vec<&slop_sim::SlopSpec> = t.spawns.iter().filter(|s| s.group == 1).map(|s| &t.specs[s.spec]).collect();
        if singles.is_empty() {
            return;
        }
        for _ in 0..count {
            let base = singles[self.rng.below(singles.len() as u32) as usize];
            let affix = Affix::ALL[self.rng.below(Affix::ALL.len() as u32) as usize];
            let spec = waves::elite(base, affix);
            if let Some(w) = self.world.as_mut() {
                let i = w.add_spec(spec);
                w.summon(i, 1, near, tag);
            }
        }
    }

    /// Each platform's boss comes after a while; The Model after the sixth
    /// (RUN-11).
    fn schedule_boss(&mut self, t: f32) {
        if self.boss_out.is_some() {
            return;
        }
        if t - self.platform_since < self.boss_due() {
            return;
        }
        let kind = self.next_boss();
        self.chapter_bosses += 1;
        let damage = self.damage_by_weapon();
        let (mut spec, model) = waves::boss(kind, self.hard_minutes(), &self.training, &damage);
        spec.contact *= self.temperature.damage();
        if let Some(w) = self.world.as_mut() {
            w.summon_boss(spec, model);
        }
        self.boss_out = Some(kind);
    }

    fn boss_defeated(&mut self) {
        let kind = self.boss_out.take();
        self.bosses_beaten += 1;
        if kind == Some(BossKind::TheModel) {
            self.model_beaten = true;
            self.toasts.push("THE MODEL COLLAPSED. You beat the Feed. The Feed does not care: it keeps going.".into());
        }
        self.stage += 1;
        // Its chest is followed by a pick of glitches (GLITCH-01).
        self.pending_glitch = true;
        let next = self.platforms[self.stage % self.platforms.len()];
        if self.stage != 6 {
            self.platform = next;
            self.toasts.push(format!("Boss deleted. Next up: {} from {}, in 4:30.", next.boss().name(), next.name()));
        } else {
            self.toasts.push("Every platform cleared. The Model is training on your run...".into());
        }
        self.platform_since = self.seconds();
        if let Some(w) = self.world.as_mut() {
            w.player.second_winds = w.player.perks.second_wind;
        }
        self.retune();
    }

    /// Something you stood at on the map (MAP-03).
    fn use_prop(&mut self, id: u32) {
        let Some((kind, pos)) = self.world.as_ref().and_then(|w| w.map.props.iter().find(|p| p.id == id).map(|p| (p.kind, p.pos))) else { return };
        let minutes = self.minutes();
        match kind {
            PropKind::Vending if matches!(self.phase, Phase::Playing) => {
                let mut offers = Vec::new();
                for c in self.pick_choices(3, true) {
                    let item = match c {
                        Choice::NewItem(i) | Choice::ItemLevel(i) => i,
                        _ => continue,
                    };
                    let price = self.price(tier_price(ITEMS[item].tier) as f32 * (0.6 + minutes / 15.0));
                    offers.push((c, price));
                }
                if offers.is_empty() {
                    self.toasts.push("Sold out: your items are all maxed.".into());
                } else {
                    self.phase = Phase::Vending { prop: id, offers };
                }
            }
            PropKind::Gacha => {
                let price = self.gacha_price(id);
                if self.clout < price {
                    self.toasts.push(format!("The gacha wants {price} Clout. You have {}.", self.clout));
                    return;
                }
                self.clout -= price;
                if let Some(p) = self.world.as_mut().and_then(|w| w.map.props.iter_mut().find(|p| p.id == id)) {
                    p.price += 1;
                }
                let jackpot = self.rng.chance(0.12);
                let upgrades = self.upgrades(if jackpot { 3 } else { 1 }, false);
                let best = upgrades.iter().map(|c| c.rarity()).max().unwrap_or(1);
                self.phase = Phase::Chest(Reward { title: if jackpot { "JACKPOT" } else { "Gacha pull" }, merged: None, upgrades, clout: 0, best });
                self.sync();
            }
            PropKind::Shrine => {
                let mut pool: Vec<usize> = (0..BLESSINGS.len()).filter(|b| !self.blessings.contains(b)).collect();
                let mut offers = Vec::new();
                while offers.len() < 3 && !pool.is_empty() {
                    offers.push(pool.swap_remove(self.rng.below(pool.len() as u32) as usize));
                }
                if offers.is_empty() {
                    self.done(id);
                } else {
                    self.phase = Phase::Shrine { prop: id, offers };
                }
            }
            PropKind::CommentSection => {
                self.toasts.push("The comment section is open. Survive the replies.".into());
                if let Some(p) = self.world.as_mut().and_then(|w| w.map.props.iter_mut().find(|p| p.id == id)) {
                    p.state = PropState::Active;
                }
                self.elite(COMMENT_ELITES, Some(pos), id);
            }
            PropKind::DataCenter => self.toasts.push("Uploading... stay inside the data center. The slop wants it too.".into()),
            PropKind::GoLive => self.go_live(),
            PropKind::RecycleBin if matches!(self.phase, Phase::Playing) => {
                let items: Vec<usize> = self.items.iter().map(|(i, _)| *i).collect();
                if !items.is_empty() {
                    self.phase = Phase::Recycle { prop: id, items };
                }
            }
            // The fine-tuning station sells weapon levels, like a vending machine.
            PropKind::FineTuning if matches!(self.phase, Phase::Playing) => {
                let mut offers = Vec::new();
                for w in self.weapons.clone() {
                    if w.level < MAX_LEVEL && !WEAPONS[w.def].evolved {
                        let price = self.price(20.0 + 6.0 * w.level as f32 + 1.5 * minutes);
                        offers.push((Choice::WeaponLevel(w.def), price));
                    }
                }
                if !offers.is_empty() {
                    self.phase = Phase::Vending { prop: id, offers };
                }
            }
            // The data broker buys 15 maximum Mental Health for a rare item.
            PropKind::DataBroker if matches!(self.phase, Phase::Playing) => {
                self.base.max_mental_health -= 15.0;
                self.done(id);
                let upgrades = self.rare_item();
                let best = upgrades.iter().map(|c| c.rarity()).max().unwrap_or(1);
                self.phase = Phase::Chest(Reward { title: "Data sold", merged: None, upgrades, clout: 0, best });
                self.sync();
            }
            _ => {}
        }
    }

    fn done(&mut self, prop: u32) {
        if let Some(p) = self.world.as_mut().and_then(|w| w.map.props.iter_mut().find(|p| p.id == prop)) {
            p.state = PropState::Done;
        }
    }

    /// A price after the machines' discounts.
    fn price(&self, base: f32) -> u32 {
        let d = self.sum(|m| if let Mod::ShopDiscount(v) = m { *v } else { 0.0 });
        let up = self.sum(|m| if let Mod::ShopPriceUp(v) = m { *v } else { 0.0 });
        (base * (1.0 - d + up)).round().max(1.0) as u32
    }

    /// What a gacha machine wants now; it gets dearer every pull.
    pub fn gacha_price(&self, prop: u32) -> u32 {
        let pulls = self.world.as_ref().and_then(|w| w.map.props.iter().find(|p| p.id == prop)).map_or(0, |p| p.price);
        self.price(15.0 + 2.0 * self.minutes() + 15.0 * pulls as f32)
    }

    // ---- Choices -------------------------------------------------------

    fn owned_item(&self, item: usize) -> Option<u32> {
        self.items.iter().find(|(i, _)| *i == item).map(|(_, n)| *n)
    }

    /// Everything that could be offered now, by category: weapon levels,
    /// new weapons, item levels, new items, stat-ups (LEVEL-02).
    fn candidates(&self, items_only: bool) -> [Vec<(Choice, f32)>; 5] {
        let mut out: [Vec<(Choice, f32)>; 5] = Default::default();
        let mods = self.mods();
        let fact = mods.iter().any(|m| matches!(m, Mod::FactWeapons));
        let banned: Vec<DamageType> = mods.iter().filter_map(|m| if let Mod::NoType(t) = m { Some(*t) } else { None }).collect();
        let luck = self.stats().luck.max(0.0);
        if !items_only {
            for w in &self.weapons {
                if w.level < MAX_LEVEL && !WEAPONS[w.def].evolved && !w.agent {
                    let partner = merge_of(w.def).is_some_and(|(o, _)| self.weapons.iter().any(|x| x.def == o));
                    out[0].push((Choice::WeaponLevel(w.def), if partner { 2.0 } else { 1.0 }));
                }
            }
            if self.weapons.len() < self.weapon_slots() {
                for (d, def) in WEAPONS.iter().enumerate() {
                    if def.evolved || self.weapons.iter().any(|w| w.def == d) || banned.contains(&def.kind) {
                        continue;
                    }
                    let partner = merge_of(d).is_some_and(|(o, _)| self.weapons.iter().any(|x| x.def == o));
                    let mut wt = if partner { 2.5 } else { 1.0 };
                    if fact && def.kind == DamageType::Fact {
                        wt *= 3.0;
                    }
                    out[1].push((Choice::NewWeapon(d), wt));
                }
            }
        }
        for &(i, n) in &self.items {
            if n < ITEMS[i].max_level() {
                out[2].push((Choice::ItemLevel(i), 1.0));
            }
        }
        if self.items.len() < ITEM_SLOTS {
            for (i, it) in ITEMS.iter().enumerate() {
                if self.owned_item(i).is_none() && it.name != "Self-Help Book" {
                    out[3].push((Choice::NewItem(i), rarity_weight(it.tier, luck)));
                }
            }
        }
        out
    }

    /// Weapon slots, after the hero's rules.
    pub fn weapon_slots(&self) -> usize {
        let extra: i32 = self.hero.def().mods.iter().map(|m| if let Mod::WeaponSlots(n) = m { *n as i32 } else { 0 }).sum();
        (WEAPON_SLOTS as i32 + extra).clamp(1, 8) as usize
    }

    /// `n` different valid choices (LEVEL-02). Categories are drawn first,
    /// so a long list of new weapons does not crowd out your own levels.
    fn pick_choices(&mut self, n: usize, items_only: bool) -> Vec<Choice> {
        let mut cands = self.candidates(items_only);
        let luck = self.stats().luck;
        let weights = [38.0, 24.0, 16.0, 12.0, if items_only { 0.0 } else { 10.0 }];
        let mut out: Vec<Choice> = Vec::new();
        let mut stats_used: Vec<StatKind> = Vec::new();
        let mut guard = 0;
        while out.len() < n && guard < 40 {
            guard += 1;
            let avail: Vec<usize> = (0..5).filter(|&c| weights[c] > 0.0 && (c == 4 || !cands[c].is_empty())).collect();
            if avail.is_empty() {
                break;
            }
            let total: f32 = avail.iter().map(|&c| weights[c]).sum();
            let mut roll = self.rng.range(0.0, total);
            let mut cat = avail[avail.len() - 1];
            for &c in &avail {
                if roll < weights[c] {
                    cat = c;
                    break;
                }
                roll -= weights[c];
            }
            if cat == 4 {
                if let Some(up) = levels::stat_up(&mut self.rng, luck, &stats_used) {
                    stats_used.push(up.stat);
                    out.push(Choice::Stat(up));
                }
                if stats_used.len() >= StatKind::ALL.len() {
                    break;
                }
                continue;
            }
            let list = &mut cands[cat];
            let total: f32 = list.iter().map(|c| c.1).sum();
            let mut roll = self.rng.range(0.0, total);
            let mut k = list.len() - 1;
            for (j, c) in list.iter().enumerate() {
                if roll < c.1 {
                    k = j;
                    break;
                }
                roll -= c.1;
            }
            out.push(list.swap_remove(k).0);
        }
        if !items_only {
            while out.len() < n {
                out.push(if out.contains(&Choice::Heal) { self.limit_break() } else { Choice::Heal });
            }
            // Now and then a glitch takes the last card (GLITCH-01).
            if self.rng.chance(GLITCH_CHANCE + luck.max(0.0) * 0.003) {
                if let Some(g) = self.glitch_offers(1).first() {
                    if let Some(last) = out.last_mut() {
                        *last = Choice::Glitch(*g);
                    }
                }
            }
            // And now and then the easy option (AUTO-01).
            if let Some(d) = self.offer_delegate() {
                if let Some(last) = out.iter_mut().rev().find(|c| !matches!(c, Choice::Glitch(_))) {
                    *last = Choice::Delegate(d);
                }
            }
        }
        out
    }

    /// `n` different glitches you do not have yet.
    pub(crate) fn glitch_offers(&mut self, n: usize) -> Vec<usize> {
        // Unplug (AUTO-04) is only worth offering once there is something to unplug.
        let mut pool: Vec<usize> = (0..GLITCHES.len()).filter(|g| !self.glitches.contains(g)).filter(|&g| !GLITCHES[g].mods.contains(&Mod::Unplug) || (self.automation >= 25 && !self.unplugged)).collect();
        let mut out = Vec::new();
        while out.len() < n && !pool.is_empty() {
            out.push(pool.swap_remove(self.rng.below(pool.len() as u32) as usize));
        }
        out
    }

    /// An epic or legendary item you do not have, or a level of one you do.
    fn rare_item(&mut self) -> Vec<Choice> {
        let room = self.items.len() < ITEM_SLOTS;
        let pool: Vec<usize> = (0..ITEMS.len())
            .filter(|&i| ITEMS[i].tier >= 3)
            .filter(|&i| match self.owned_item(i) {
                Some(n) => n < ITEMS[i].max_level(),
                None => room,
            })
            .collect();
        let c = if pool.is_empty() { self.limit_break() } else {
            let i = pool[self.rng.below(pool.len() as u32) as usize];
            if self.owned_item(i).is_some() { Choice::ItemLevel(i) } else { Choice::NewItem(i) }
        };
        self.apply(c);
        vec![c]
    }

    /// Throws item `i` of a recycle bin's offer away for Clout (MAP-05).
    pub fn recycle(&mut self, i: usize) -> Result<(), RuleError> {
        let Phase::Recycle { prop, items } = &self.phase else { return Err(RuleError::WrongPhase) };
        let item = *items.get(i).ok_or(RuleError::NoSuchOffer)?;
        let prop = *prop;
        let level = self.owned_item(item).unwrap_or(1);
        self.items.retain(|(it, _)| *it != item);
        self.clout += tier_price(ITEMS[item].tier) * level / 2 + 5;
        self.sync();
        self.done(prop);
        self.phase = Phase::Playing;
        if self.pending_levels > 0 {
            self.start_level_up();
        }
        Ok(())
    }

    /// A good stat-up for a build with nothing left to level (CHEST-02).
    fn limit_break(&mut self) -> Choice {
        // A plain stat-up: no Luck, so a maxed build creeps up rather than
        // snowballing.
        match levels::stat_up(&mut self.rng, -100.0, &[]) {
            Some(mut up) => {
                up.amount *= 0.5;
                Choice::Stat(up)
            }
            None => Choice::Clout(15),
        }
    }

    /// Takes glitch offer `i` after a boss (GLITCH-01).
    pub fn take_glitch(&mut self, i: usize) -> Result<(), RuleError> {
        let Phase::Glitch(offers) = &self.phase else { return Err(RuleError::WrongPhase) };
        let g = *offers.get(i).ok_or(RuleError::NoSuchOffer)?;
        self.phase = Phase::Playing;
        self.apply(Choice::Glitch(g));
        if self.pending_levels > 0 {
            self.start_level_up();
        }
        Ok(())
    }

    fn start_level_up(&mut self) {
        let n = CHOICES + self.count(|m| matches!(m, Mod::ExtraOffer)) as usize;
        let choices = self.pick_choices(n, false);
        self.rerolls = FREE_REROLLS + self.count(|m| matches!(m, Mod::FreeReroll)) + self.sum(|m| if let Mod::ExtraReroll(n) = m { *n as f32 } else { 0.0 }) as u32;
        self.paid_rerolls = 0;
        // From 25 Automation the model suggests; from 50 it picks (AUTO-03).
        self.suggested = (self.automation >= 25).then(|| crate::bot::best(self, &choices));
        self.phase = Phase::LevelUp(choices);
    }

    /// Applies a choice to your build.
    fn apply(&mut self, c: Choice) {
        match c {
            Choice::NewWeapon(d) => {
                if self.weapons.len() < self.weapon_slots() && self.weapons.iter().all(|w| w.def != d) {
                    let start = self.hero.def().mods.iter().map(|m| if let Mod::StartLevel(n) = m { *n } else { 1 }).max().unwrap_or(1);
                    self.weapons.push(OwnedWeapon { def: d, level: start.clamp(1, MAX_LEVEL), agent: false });
                }
            }
            Choice::WeaponLevel(d) => {
                if let Some(w) = self.weapons.iter_mut().find(|w| w.def == d) {
                    w.level = (w.level + 1).min(MAX_LEVEL);
                }
            }
            Choice::NewItem(i) | Choice::ItemLevel(i) => match self.items.iter().position(|(it, _)| *it == i) {
                Some(k) => self.items[k].1 = (self.items[k].1 + 1).min(ITEMS[i].max_level()),
                None if self.items.len() < ITEM_SLOTS => self.items.push((i, 1)),
                None => {}
            },
            Choice::Stat(up) => up.stat.apply(&mut self.base, up.amount),
            Choice::Heal => {
                if let Some(w) = self.world.as_mut() {
                    w.player.mental_health = w.player.stats.max_mental_health;
                }
            }
            Choice::Clout(n) => self.clout += n,
            Choice::Glitch(g) => {
                if !self.glitches.contains(&g) {
                    self.glitches.push(g);
                }
                if GLITCHES[g].mods.contains(&Mod::Unplug) {
                    self.unplug();
                }
            }
            Choice::Delegate(d) => self.delegate(d),
        }
        self.sync();
    }

    /// The level a choice brings a weapon or item to, and from.
    pub fn levels_of(&self, c: Choice) -> Option<(u32, u32)> {
        match c {
            Choice::NewWeapon(_) | Choice::NewItem(_) => Some((0, 1)),
            Choice::WeaponLevel(d) => self.weapons.iter().find(|w| w.def == d).map(|w| (w.level as u32, w.level as u32 + 1)),
            Choice::ItemLevel(i) => self.owned_item(i).map(|n| (n, n + 1)),
            _ => None,
        }
    }

    /// Picks level-up choice `i` (LEVEL-02).
    pub fn choose(&mut self, i: usize) -> Result<(), RuleError> {
        let Phase::LevelUp(choices) = &self.phase else { return Err(RuleError::WrongPhase) };
        let c = *choices.get(i).ok_or(RuleError::NoSuchOffer)?;
        self.pending_levels -= 1;
        self.phase = Phase::Playing;
        self.apply(c);
        if self.pending_levels > 0 {
            self.start_level_up();
        }
        Ok(())
    }

    /// What the next reroll of a level-up costs: free ones first.
    pub fn reroll_cost(&self) -> u32 {
        if self.rerolls > 0 {
            0
        } else {
            4 + 4 * self.paid_rerolls
        }
    }

    pub fn reroll(&mut self) -> Result<(), RuleError> {
        if !matches!(self.phase, Phase::LevelUp(_)) {
            return Err(RuleError::WrongPhase);
        }
        if self.automation >= 25 {
            return Err(RuleError::Automated);
        }
        let cost = self.reroll_cost();
        if cost > self.clout {
            return Err(RuleError::NotEnoughClout);
        }
        self.clout -= cost;
        if self.rerolls > 0 {
            self.rerolls -= 1;
        } else {
            self.paid_rerolls += 1;
        }
        let n = CHOICES + self.count(|m| matches!(m, Mod::ExtraOffer)) as usize;
        self.phase = Phase::LevelUp(self.pick_choices(n, false));
        Ok(())
    }

    /// `n` random upgrades of what you have (or new things if nothing can
    /// level), applied (CHEST-01).
    fn upgrades(&mut self, n: usize, golden: bool) -> Vec<Choice> {
        let mut out = Vec::new();
        // A golden chest always holds a legendary, if there is room (LOOT-02).
        if golden && self.items.len() < ITEM_SLOTS {
            let pool: Vec<usize> = (0..ITEMS.len()).filter(|&i| ITEMS[i].kind == ItemKind::Legendary && self.owned_item(i).is_none()).collect();
            if !pool.is_empty() {
                let c = Choice::NewItem(pool[self.rng.below(pool.len() as u32) as usize]);
                self.apply(c);
                out.push(c);
            }
        }
        while out.len() < n {
            let cands = self.candidates(false);
            let mut pool: Vec<Choice> = cands[0].iter().chain(cands[2].iter()).map(|c| c.0).collect();
            if pool.is_empty() {
                pool = cands[1].iter().chain(cands[3].iter()).map(|c| c.0).collect();
            }
            // Everything maxed: a small stat-up instead.
            let c = if pool.is_empty() { self.limit_break() } else { pool[self.rng.below(pool.len() as u32) as usize] };
            self.apply(c);
            out.push(c);
        }
        out
    }

    /// A merge that is ready: two weapons at max level that make a third.
    pub fn ready_merge(&self) -> Option<(usize, usize, usize)> {
        self.weapons.iter().filter(|w| w.level >= MAX_LEVEL && !w.agent).find_map(|w| {
            let (other, into) = merge_of(w.def)?;
            self.weapons.iter().any(|x| x.def == other && x.level >= MAX_LEVEL && !x.agent).then_some((w.def, other, into))
        })
    }

    /// Opens a chest: a merge if one is ready, otherwise upgrades (CHEST-01).
    fn open_chest(&mut self, value: u32, title: &'static str) {
        let merged = self.ready_merge();
        // Nothing left to upgrade: the chest pays out without stopping the
        // game (CHEST-02).
        let cands = self.candidates(false);
        if merged.is_none() && cands[..4].iter().all(|c| c.is_empty()) {
            self.clout += 20 + 15 * value;
            if let Some(w) = self.world.as_mut() {
                let p = &mut w.player;
                p.mental_health = (p.mental_health + p.stats.max_mental_health * 0.3).min(p.stats.max_mental_health);
            }
            return;
        }
        let mut upgrades = Vec::new();
        if let Some((a, b, into)) = merged {
            let slot = self.weapons.iter().position(|w| w.def == a).unwrap_or(0);
            self.weapons[slot] = OwnedWeapon { def: into, level: MAX_LEVEL, agent: false };
            self.weapons.retain(|w| w.def != b);
            self.sync();
        } else {
            // One, three or five upgrades, like the treasure of the genre;
            // bigger chests roll better (LOOT-02).
            let luck = self.stats().luck.max(0.0) * 0.004;
            let roll = self.rng.f32() - luck;
            let n = match value {
                3.. => 5,
                2 if roll < 0.3 => 5,
                2 => 3,
                _ if roll < 0.02 => 5,
                _ if roll < 0.16 => 3,
                _ => 1,
            };
            upgrades = self.upgrades(n, value >= 3);
        }
        let clout = 5 + self.rng.below(10) + 12 * value;
        self.clout += clout;
        let best = upgrades.iter().map(|c| c.rarity()).max().unwrap_or(if merged.is_some() { 4 } else { 1 });
        let title = if merged.is_some() {
            "MERGE!"
        } else if upgrades.len() >= 5 && value < 3 {
            "JACKPOT"
        } else {
            title
        };
        self.phase = Phase::Chest(Reward { title, merged, upgrades, clout, best });
    }

    /// Closes a chest, a vending machine or a shrine and plays on. A boss's
    /// chest is followed by a pick of glitches (GLITCH-01).
    pub fn close(&mut self) -> Result<(), RuleError> {
        match self.phase {
            Phase::Chest(_) | Phase::Vending { .. } | Phase::Shrine { .. } | Phase::Recycle { .. } => {
                let chest = matches!(self.phase, Phase::Chest(_));
                self.phase = Phase::Playing;
                if self.pending_glitch && chest {
                    self.pending_glitch = false;
                    let offers = self.glitch_offers(3);
                    if !offers.is_empty() {
                        self.phase = Phase::Glitch(offers);
                        return Ok(());
                    }
                }
                if self.pending_levels > 0 {
                    self.start_level_up();
                }
                Ok(())
            }
            _ => Err(RuleError::WrongPhase),
        }
    }

    /// Buys offer `i` of a vending machine; the machine is then empty.
    pub fn buy(&mut self, i: usize) -> Result<(), RuleError> {
        let Phase::Vending { prop, offers } = &self.phase else { return Err(RuleError::WrongPhase) };
        let (c, price) = *offers.get(i).ok_or(RuleError::NoSuchOffer)?;
        let prop = *prop;
        if price > self.clout {
            return Err(RuleError::NotEnoughClout);
        }
        self.clout -= price;
        self.apply(c);
        self.done(prop);
        self.close()
    }

    /// Takes shrine blessing `i`.
    pub fn bless(&mut self, i: usize) -> Result<(), RuleError> {
        let Phase::Shrine { prop, offers } = &self.phase else { return Err(RuleError::WrongPhase) };
        let b = *offers.get(i).ok_or(RuleError::NoSuchOffer)?;
        let prop = *prop;
        self.blessings.push(b);
        self.sync();
        self.done(prop);
        self.close()
    }

    /// What deleted you, if it was something with a name.
    fn killer(&self) -> Option<(String, slop_sim::spec::SlopLook)> {
        let w = self.world.as_ref()?;
        let sp = w.setup.specs.get(w.last_hurt as usize)?;
        let name = sp.name.strip_prefix("Nemesis ").unwrap_or(&sp.name);
        Some((name.to_string(), sp.look))
    }

    /// What the slop will remember of this run (NEMESIS-01).
    pub fn remembered(&self) -> Memory {
        let mut m = self.memory.clone();
        m.runs += 1;
        m.best_seconds = m.best_seconds.max(self.seconds() as u32);
        m.best_score = m.best_score.max(self.score);
        let kills = self.kills_by_type();
        m.favourite = (0..4).max_by_key(|&k| kills[k]).filter(|&k| kills[k] > 0);
        if let Some((name, look)) = self.killer() {
            m.killer_count = if m.killer.as_deref() == Some(name.as_str()) { m.killer_count + 1 } else { 1 };
            m.killer = Some(name);
            m.killer_look = Some(look);
        }
        // The title demo replays this build (END-01).
        m.hero = Some(self.hero as usize);
        m.build = self.weapons.iter().map(|w| (w.def, w.level)).collect();
        m
    }

    /// Slop deleted over the whole run.
    pub fn total_kills(&self) -> u32 {
        let done: u32 = self.chapters.iter().map(|c| c.kills).sum();
        if self.chapters.last().is_some_and(|c| c.chapter == self.chapter) { done } else { done + self.kills() }
    }

    /// The end screen: a fake model card written from your real run (CARD-01).
    pub fn model_card(&self) -> Vec<String> {
        let s = self.seconds() as u32;
        let mut card = vec![format!("Human-{}.0 ({}). Temperature {}. Status: deprecated.", self.memory.runs + 1, self.hero.def().name, self.temperature.label())];
        card.push(format!("Trained for {}:{:02} on {} slop over {} of 5 chapters.", s / 60, s % 60, self.total_kills(), self.chapters.len().max(1)));
        for r in &self.chapters {
            let m = r.seconds as u32;
            let word = r.chapter.def().death.to_lowercase();
            match &r.killer {
                Some((name, lesson)) => card.push(format!("{}: {}:{:02}, {word} by {name}. {lesson}", r.chapter.def().name, m / 60, m % 60)),
                None => card.push(format!("{}: {}:{:02}, {word}.", r.chapter.def().name, m / 60, m % 60)),
            }
        }
        card.push(format!("Automation: {}%. Humanity retained: {}%.", self.automation, self.humanity()));
        let dmg = self.damage_by_weapon();
        let total: f32 = dmg.iter().sum();
        let top = (0..dmg.len()).max_by(|&a, &b| dmg[a].total_cmp(&dmg[b]));
        let share = top.filter(|_| total > 0.0).map_or(0.0, |d| dmg[d] / total * 100.0);
        if let Some(d) = top.filter(|_| total > 0.0) {
            card.push(format!("Known bias: {} did {share:.0}% of your damage.", WEAPONS[d].name));
        }
        let m = self.training.model;
        let high = (0..4).max_by(|&a, &b| m[a].total_cmp(&m[b])).unwrap_or(0);
        let low = (0..4).min_by(|&a, &b| m[a].total_cmp(&m[b])).unwrap_or(0);
        if m[high] > 0.02 {
            card.push(format!("What I learned: I resist {} by {:.0}%. {} still hurts. Please do not tell anyone.", DamageType::ALL[high].name(), m[high] * 100.0, DamageType::ALL[low].name()));
        }
        match self.killer() {
            Some((name, _)) if self.memory.killer.as_deref() == Some(name.as_str()) => card.push(format!("Limitations: deleted by {name}, {} times now. It has noticed.", self.memory.killer_count + 1)),
            Some((name, _)) => card.push(format!("Limitations: deleted by {name}. It will remember this.")),
            None => card.push("Limitations: deleted by something you did not read the tooltip for.".into()),
        }
        if self.nemesis_beaten {
            card.push("Nemesis: deleted. It is in therapy now. So are you.".into());
        }
        if self.model_beaten {
            card.push("Benchmark: beat The Model. The Model disputes the methodology.".into());
        }
        card.push(if share >= 55.0 { "Recommendation: have you tried a second weapon?".into() } else { "Recommendation: touch grass.".into() });
        if self.automation >= 100 {
            card.push("Status: fully automated. Thank you for your training data.".into());
        }
        let best = if self.score > self.memory.best_score && self.memory.best_score > 0 { " A new best." } else { "" };
        card.push(format!("Score: {}.{best} Confidence: 99% you will do this again.", self.score));
        card
    }

    /// True once the run is over.
    pub fn is_over(&self) -> bool {
        matches!(self.phase, Phase::Over { .. })
    }

    /// Kills by damage type over the run, for the end screen.
    pub fn kills_by_type(&self) -> [u32; 4] {
        self.world.as_ref().map_or([0; 4], |w| w.tally.kills_by_type)
    }

    pub fn kills(&self) -> u32 {
        self.world.as_ref().map_or(0, |w| w.tally.kills)
    }

    /// Weapon slots holding an orbit weapon (for the Therapy Dog note).
    pub fn has_dog(&self) -> bool {
        self.weapons.iter().any(|w| matches!(WEAPONS[w.def].pattern, Pattern::Orbit { .. }))
    }

    /// Seconds until the current platform's boss (or The Model) arrives;
    /// None while a boss is out.
    pub fn boss_in(&self) -> Option<f32> {
        if self.boss_out.is_some() {
            return None;
        }
        Some((self.boss_due() - (self.seconds() - self.platform_since)).max(0.0))
    }

    /// Seconds after the last boss before the next (CH-02): the Feed cycles
    /// its platforms; the other chapters have one boss, back again and again.
    fn boss_due(&self) -> f32 {
        match self.chapter {
            Chapter::Feed if self.stage == 6 => waves::MODEL_AFTER,
            Chapter::Feed => waves::BOSS_AFTER,
            c if self.chapter_bosses == 0 => c.def().boss_at,
            _ => waves::BOSS_AFTER + 60.0,
        }
    }

    /// The boss coming next.
    pub fn next_boss(&self) -> BossKind {
        match self.chapter {
            Chapter::Feed if self.stage == 6 => BossKind::TheModel,
            Chapter::Feed => self.platform.boss(),
            c => c.def().boss,
        }
    }
}

/// Chapters, Automation and the takeover (CH-01, AUTO-01 to AUTO-04, END-01).
impl Run {
    /// Minutes into the chapter.
    pub fn chapter_minutes(&self) -> f32 {
        ((self.seconds() - self.chapter_since) / 60.0).max(0.0)
    }

    /// How strong the AI is right now (CAP-01).
    pub fn capability(&self) -> f32 {
        waves::capability(self.chapter_minutes(), self.chapter.def().head_start, self.automation)
    }

    /// Minutes as the boss scaling counts them: capability, not the clock.
    fn hard_minutes(&self) -> f32 {
        (self.capability() - 1.0) / 0.62
    }

    /// The version on the HUD (CAP-01).
    pub fn version(&self) -> String {
        waves::version(self.chapter.def().ai, self.chapter.number(), self.chapter_minutes() + self.chapter.def().head_start)
    }

    /// Humanity retained, 0 to 100.
    pub fn humanity(&self) -> u32 {
        100 - self.automation.min(100)
    }

    /// Whether the bot is driving right now (AUTO-03).
    pub fn autopilot_now(&self) -> bool {
        if matches!(self.phase, Phase::Takeover { .. }) {
            return true;
        }
        let t = self.seconds() - self.chapter_since;
        if self.automation >= 75 {
            let window = (12.0 + 3.0 * self.delegated.len() as f32).min(30.0);
            return t % 45.0 < window;
        }
        self.delegated.contains(&Delegate::AutopilotTrial) && t % 60.0 < 10.0
    }

    /// Seconds after which a level-up picks itself, when it does (AUTO-03).
    pub fn auto_pick_after(&self) -> Option<f32> {
        if self.automation >= 50 {
            Some(2.0)
        } else if self.delegated.contains(&Delegate::AutoPick) {
            Some(3.0)
        } else {
            None
        }
    }

    /// Picks the suggested card, as Auto-pick does, with its free stat-up.
    pub fn auto_choose(&mut self) -> Result<(), RuleError> {
        let i = self.suggested.unwrap_or(0);
        self.choose(i)?;
        if self.delegated.contains(&Delegate::AutoPick) {
            let c = self.limit_break();
            self.apply(c);
        }
        Ok(())
    }

    /// Your input, with the bot's help where you asked for it.
    fn steer_input(&self, input: Input, auto: bool) -> Input {
        let assist = self.automation >= 50 || self.delegated.contains(&Delegate::AssistDash);
        if !auto && !assist {
            return input;
        }
        let Some(w) = self.world.as_ref() else { return input };
        let bot = slop_sim::bot::steer(w);
        let mut out = if auto { bot } else { input };
        if assist {
            out.dash |= bot.dash;
        }
        out
    }

    /// A Delegate card for this level-up, now and then (AUTO-01).
    fn offer_delegate(&mut self) -> Option<Delegate> {
        if self.unplugged || self.automation >= 100 {
            return None;
        }
        let chance = [0.25, 0.35, 0.45, 0.55, 0.6][self.chapter as usize];
        if !self.rng.chance(chance) {
            return None;
        }
        let mut pool = Vec::new();
        for w in &self.weapons {
            if !w.agent && !WEAPONS[w.def].evolved {
                pool.push(Delegate::Agent(w.def));
            }
        }
        for d in [Delegate::AutoPick, Delegate::AssistDash, Delegate::SmartTargeting, Delegate::AutopilotTrial] {
            if !self.delegated.contains(&d) {
                pool.push(d);
            }
        }
        if self.weapons.iter().any(|w| !w.agent) && self.automation >= 30 {
            pool.push(Delegate::FullAutomation);
        }
        (!pool.is_empty()).then(|| pool[self.rng.below(pool.len() as u32) as usize])
    }

    fn delegate(&mut self, d: Delegate) {
        match d {
            Delegate::Agent(def) => {
                if let Some(w) = self.weapons.iter_mut().find(|w| w.def == def) {
                    w.agent = true;
                }
            }
            Delegate::FullAutomation => {
                for w in &mut self.weapons {
                    w.agent = true;
                }
                self.delegated.push(d);
            }
            _ => {
                if !self.delegated.contains(&d) {
                    self.delegated.push(d);
                }
            }
        }
        self.automate(d.cost());
    }

    /// Raises Automation and says, once, what each threshold takes (AUTO-03).
    fn automate(&mut self, n: u32) {
        if self.unplugged {
            return;
        }
        self.automation = (self.automation + n).min(100);
        let lines = [
            (1u8, 25, "Suggestions enabled. The model has already considered the alternatives, so rerolls are gone."),
            (2, 50, "Assist enabled. Level-ups pick themselves after 2 s. Your dash is handled."),
            (4, 75, "Autopilot enabled. For part of every minute, the model drives. Relax."),
        ];
        for (bit, at, line) in lines {
            if self.automation >= at && self.told & bit == 0 {
                self.told |= bit;
                self.toasts.push(line.into());
            }
        }
    }

    /// Unplug (AUTO-04): Automation stops for the chapter, the agents go,
    /// and so does half your resource.
    pub fn unplug(&mut self) {
        self.unplugged = true;
        self.delegated.clear();
        for w in &mut self.weapons {
            w.agent = false;
        }
        self.base.max_mental_health *= 0.5;
        self.sync();
    }

    /// What deleted you, with its lesson (LESSON-01).
    fn cause_of_death(&self) -> Option<(String, String)> {
        let w = self.world.as_ref()?;
        let sp = w.setup.specs.get(w.last_hurt as usize)?;
        let name = sp.name.strip_prefix("Nemesis ").unwrap_or(&sp.name).to_string();
        let lesson = match sp.look {
            slop_sim::spec::SlopLook::Archetype(a) => a.def().lesson().to_string(),
            slop_sim::spec::SlopLook::Boss(k) => k.puzzle().to_string(),
            slop_sim::spec::SlopLook::Brainrot { .. } => "Brainrot is made of parts. Read them: each one is a trick.".into(),
        };
        Some((name, lesson))
    }

    /// Your resource hit zero: the next chapter, or the end (CH-01).
    fn chapter_over(&mut self) {
        let seconds = self.seconds() - self.chapter_since;
        let killer = self.cause_of_death();
        let kills = self.kills();
        self.chapters.push(ChapterResult { chapter: self.chapter, seconds, kills, killer });
        match self.chapter.next() {
            Some(next) if self.automation < 100 => self.phase = Phase::ChapterCard(next),
            _ => self.start_takeover(),
        }
    }

    /// Continues from a chapter card into the next chapter.
    pub fn next_chapter(&mut self) -> Result<(), RuleError> {
        let Phase::ChapterCard(c) = self.phase else { return Err(RuleError::WrongPhase) };
        self.start_chapter(c);
        Ok(())
    }

    /// Starts chapter `c`: a new map and a fresh build, carrying Automation,
    /// the agents, the glitches and what the model learned (CH-01).
    fn start_chapter(&mut self, c: Chapter) {
        let def = c.def();
        let hero = self.hero.def();
        // The clock carries on across the new world.
        self.offset = self.seconds();
        let t = self.offset;
        self.chapter = c;
        self.chapter_since = t;
        self.chapter_bosses = 0;
        self.unplugged = false;
        // Agents come with you; in the Resistance they defect.
        let mut agents: Vec<OwnedWeapon> = self.weapons.iter().filter(|w| w.agent).copied().collect();
        if c == Chapter::Resistance {
            self.defected = agents.iter().map(|w| w.def).collect();
            agents.clear();
        }
        let weapon = weapon_index(def.weapon).expect("chapter weapons exist");
        self.weapons = vec![OwnedWeapon { def: weapon, level: 1, agent: false }];
        for a in agents {
            if a.def == weapon {
                self.weapons[0] = a;
            } else {
                self.weapons.push(a);
            }
        }
        self.items = vec![(item_index(hero.item).expect("hero items exist"), 1)];
        self.blessings.clear();
        self.level = 1;
        self.xp = 0;
        self.pending_levels = 0;
        self.xp_seen = 0;
        self.clout_seen = 0;
        self.base = hero_base(self.hero);
        self.stage = 0;
        self.boss_out = None;
        self.platform_since = t;
        self.pending_glitch = false;
        self.merge_told = false;
        self.trained = Tally::default();
        // A new map.
        let seed = self.seed.wrapping_add(c as u64 * 7919);
        let mut rng = Rng::new(seed, 7);
        let first = slop_sim::map::biome_at(seed, V2::ZERO, def.biomes);
        let mut rest: Vec<Platform> = def.biomes.iter().copied().filter(|p| *p != first).collect();
        for i in (1..rest.len()).rev() {
            let j = rng.below(i as u32 + 1) as usize;
            rest.swap(i, j);
        }
        self.platforms = vec![first];
        self.platforms.extend(rest);
        self.platform = first;
        self.biome = first;
        let mut setup = WaveSetup::sandbox();
        setup.specs.clear();
        setup.seconds = 0.0;
        setup.platform = Some(first);
        setup.biomes = def.biomes.to_vec();
        // In the Resistance your agents fire your own weapons back (CH-02).
        for (k, d) in self.defected.iter().take(3).enumerate() {
            setup.model.copies[k] = Some(waves::weapon_look(*d));
        }
        let mut w = World::new(seed, setup, &self.loadout());
        w.hallucinate = self.temperature.hallucination();
        self.world = Some(w);
        self.next_retune = t;
        self.next_retrain = t + RETRAIN_EVERY;
        self.next_elite = t + 90.0;
        self.next_minute = t + 60.0;
        self.next_swarm = t + SWARM_FIRST;
        self.next_ring = t + RING_FIRST;
        self.next_live = t + LIVE_FIRST;
        self.next_goblin = t + GOBLIN_FIRST;
        self.next_nemesis = t + NEMESIS_AT;
        self.phase = Phase::Playing;
        self.sync();
        self.retune();
    }

    /// Your role has been automated (END-01).
    fn start_takeover(&mut self) {
        // Automation reached 100 mid-chapter: that chapter ends here too.
        if self.chapters.last().map(|c| c.chapter) != Some(self.chapter) {
            let seconds = self.seconds() - self.chapter_since;
            let kills = self.kills();
            self.chapters.push(ChapterResult { chapter: self.chapter, seconds, kills, killer: Some(("Automation".into(), "Every Delegate card was the strongest choice. That was the point.".into())) });
        }
        if let Some(w) = self.world.as_mut() {
            w.outcome = Outcome::Running;
            let p = &mut w.player;
            p.mental_health = p.stats.max_mental_health;
            p.invulnerable = 90;
            p.autopilot = true;
        }
        self.phase = Phase::Takeover { left: TAKEOVER_SECONDS };
        self.toasts.push("Your role has been automated.".into());
    }

    /// The bot plays your character; the Feed goes on; then it is over.
    fn step_takeover(&mut self, left: f32) -> bool {
        let Some(world) = self.world.as_mut() else { return false };
        let input = slop_sim::bot::steer(world);
        world.player.autopilot = true;
        world.step(input);
        if world.outcome != Outcome::Running {
            // Even the bot can lose. It does not mind.
            world.outcome = Outcome::Running;
            let p = &mut world.player;
            p.mental_health = p.stats.max_mental_health;
            p.invulnerable = 60;
        }
        let left = left - 1.0 / TICK_RATE as f32;
        if left <= 0.0 {
            self.finish();
            return true;
        }
        self.phase = Phase::Takeover { left };
        false
    }

    fn finish(&mut self) {
        let seconds: f32 = self.chapters.iter().map(|c| c.seconds).sum::<f32>().max(self.seconds());
        self.score = (seconds * (1.0 + self.humanity() as f32 / 100.0)) as u32;
        self.phase = Phase::Over { victory: false };
    }
}

/// A hero's stats before any level-up.
fn hero_base(hero: Hero) -> Stats {
    let mut l = Loadout { weapons: vec![], agents: Vec::new(), stats: Stats::default(), perks: Default::default() };
    for m in hero.def().mods {
        items::apply(*m, &mut l, 0);
    }
    l.stats
}

#[cfg(test)]
impl Run {
    /// Draws a fresh level-up without spending anything.
    pub fn reroll_free_for_test(&mut self) {
        self.rerolls = 1;
        let _ = self.reroll();
    }
}
