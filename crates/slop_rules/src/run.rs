//! A run (RUN-01 to RUN-12): one endless trip through the Feed. The world is
//! never rebuilt; the rules watch it, retune who spawns as time goes by,
//! bring in each platform's boss, and pause it for your choices.
//!
//! ```text
//! Playing -> LevelUp (one per level gained) -> Playing
//!         -> Chest (a merge, or upgrades)    -> Playing
//!         -> Vending / Shrine (a machine)    -> Playing
//!         -> Over on Burnout (a victory if The Model fell first)
//! ```

use slop_sim::content::{merge_of, weapon_index, BossKind, Pattern, Platform, WEAPONS};
use slop_sim::map::{PropKind, PropState};
use slop_sim::rng::Rng;
use slop_sim::world::{Loadout, Outcome, Tally, Weapon, WaveSetup, World, MAX_LEVEL, TICK_RATE};
use slop_sim::{DamageType, Input, Stats};

use crate::heroes::Hero;
use crate::items::{self, item_index, tier_price, Mod, StatKind, BLESSINGS, ITEMS};
use crate::levels::{self, StatUp};
use crate::training::{PatchNotes, Training};
use crate::waves::{self, Affix};

pub const WEAPON_SLOTS: usize = 6;
pub const ITEM_SLOTS: usize = 6;
const CHOICES: usize = 3;
/// Seconds between retunes of the spawn table, retraining, and elites.
const RETUNE_EVERY: f32 = 20.0;
const RETRAIN_EVERY: f32 = 120.0;
const ELITE_EVERY: f32 = 50.0;
const FREE_REROLLS: u32 = 1;
const COMMENT_ELITES: u32 = 6;
/// Drawbacks never take your maximum Mental Health below this.
const MIN_MENTAL_HEALTH: f32 = 30.0;

/// The Creator Fund pays for every minute survived, in Clout.
pub fn payout(minute: u32) -> u32 {
    2 + minute / 3
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuleError {
    WrongPhase,
    NotEnoughClout,
    NoSuchOffer,
}

impl std::fmt::Display for RuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            RuleError::WrongPhase => "Not now.",
            RuleError::NotEnoughClout => "Not enough Clout.",
            RuleError::NoSuchOffer => "No such offer.",
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
}

/// What a chest (or a gacha pull) gave you; it is applied already.
#[derive(Clone, Debug, PartialEq)]
pub struct Reward {
    pub title: &'static str,
    /// Two weapons and what they merged into.
    pub merged: Option<(usize, usize, usize)>,
    pub upgrades: Vec<Choice>,
    pub clout: u32,
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
    Over { victory: bool },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnedWeapon {
    pub def: usize,
    pub level: u8,
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
    trained: Tally,
}

impl Run {
    pub fn new(seed: u64, hero: Hero) -> Run {
        let mut rng = Rng::new(seed, 7);
        let first = Platform::ALL[rng.below(Platform::ALL.len() as u32) as usize];
        Run::on(seed, hero, first, rng)
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
        let mut rest: Vec<Platform> = Platform::ALL.iter().copied().filter(|p| *p != first).collect();
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
            weapons: vec![OwnedWeapon { def: weapon, level: 1 }],
            items: vec![(item, 1)],
            blessings: Vec::new(),
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
            next_elite: ELITE_EVERY,
            next_minute: 60.0,
            trained: Tally::default(),
        };
        let mut l = Loadout { weapons: vec![], stats: run.base, perks: Default::default() };
        for m in def.mods {
            items::apply(*m, &mut l, 0);
        }
        run.base = l.stats;
        let mut setup = WaveSetup::sandbox();
        setup.specs.clear();
        setup.seconds = 0.0;
        setup.platform = Some(first);
        run.world = Some(World::new(seed, setup, &run.loadout()));
        run.retune();
        run
    }

    /// A run `minutes` in, with a few levelled weapons and plenty of Mental
    /// Health: the bots play it behind the title screen.
    pub fn showcase(seed: u64, hero: Hero, minutes: f32) -> Run {
        let mut run = Run::new(seed, hero);
        while run.weapons.len() < 4 {
            let def = run.rng.below(WEAPONS.len() as u32) as usize;
            if !WEAPONS[def].evolved && run.weapons.iter().all(|w| w.def != def) {
                run.weapons.push(OwnedWeapon { def, level: 4 });
            }
        }
        run.weapons[0].level = 6;
        run.base.max_mental_health += 400.0;
        run.base.conviction += 1.5;
        run.offset = minutes * 60.0;
        run.next_retrain += run.offset;
        run.next_elite += run.offset;
        run.next_minute += run.offset;
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
        let mut l = Loadout { weapons: self.weapons.iter().map(|w| (w.def, w.level)).collect(), stats: self.base, perks: Default::default() };
        let types = DamageType::ALL.iter().filter(|t| self.weapons.iter().any(|w| WEAPONS[w.def].kind == **t)).count();
        // The hero's stats are in `base` already; its other effects are not.
        let hero = self.hero.def().mods.iter().copied().filter(|m| !matches!(m, Mod::Stat(..)));
        for m in self.item_mods().into_iter().chain(hero) {
            items::apply(m, &mut l, types);
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
        let Some(w) = self.world.as_mut() else { return };
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
                nw
            })
            .collect();
        w.tally.weapon_damage = vec![0.0; w.player.weapons.len()];
        let grow = l.stats.max_mental_health - w.player.stats.max_mental_health;
        w.player.stats = l.stats;
        if grow > 0.0 {
            w.player.mental_health += grow;
        }
        w.player.mental_health = w.player.mental_health.min(l.stats.max_mental_health);
        w.player.active_cooldown.resize(l.perks.actives.len(), 0.0);
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
    fn retune(&mut self) {
        let stage = waves::stage(self.minutes());
        let t = waves::table(stage, self.platform, &self.training);
        let Some(w) = self.world.as_mut() else { return };
        let base = w.retune(t.specs, t.spawns, t.rate, t.elite_chance, Some(self.platform), t.repost_chance);
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
        if !matches!(self.phase, Phase::Playing) {
            return false;
        }
        let Some(world) = self.world.as_mut() else { return false };
        world.step(input);
        if world.outcome != Outcome::Running {
            self.sync();
            self.phase = Phase::Over { victory: self.model_beaten };
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
        if coins > 0 {
            let bonus = self.sum(|m| if let Mod::CloutBonus(v) = m { *v } else { 0.0 });
            self.clout += (coins as f32 * (1.0 + bonus)).round() as u32;
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
        if t >= self.next_elite {
            self.next_elite += ELITE_EVERY;
            self.elite(1, None, 0);
        }
        self.schedule_boss(t);
        for id in used {
            self.use_prop(id);
        }
        for value in chests {
            if matches!(self.phase, Phase::Playing) {
                self.open_chest(value, if value >= 2 { "Boss chest" } else { "Chest" });
            } else if let Some(w) = self.world.as_mut() {
                w.chests.push(value);
            }
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
            income += (self.clout as f32 * swing) as i32;
        }
        income += payout(minute) as i32;
        self.clout = (self.clout as i32 + income).max(0) as u32;
    }

    /// An elite with an affix, off-screen or around `near` (ELITE-01).
    fn elite(&mut self, count: u32, near: Option<slop_sim::V2>, tag: u32) {
        let stage = waves::stage(self.minutes());
        let t = waves::table(stage, self.platform, &self.training);
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
        let model_next = self.stage == 6;
        let due = if model_next { waves::MODEL_AFTER } else { waves::BOSS_AFTER };
        if t - self.platform_since < due {
            return;
        }
        let kind = if model_next { BossKind::TheModel } else { self.platform.boss() };
        let damage = self.damage_by_weapon();
        let (spec, model) = waves::boss(kind, self.minutes(), &self.training, &damage);
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
        let next = self.platforms[self.stage % self.platforms.len()];
        if self.stage != 6 {
            self.platform = next;
            self.toasts.push(format!("Scrolling on to {}: {}", next.name(), next.hazard()));
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
                let upgrades = self.upgrades(if jackpot { 3 } else { 1 });
                self.phase = Phase::Chest(Reward { title: if jackpot { "JACKPOT" } else { "Gacha pull" }, merged: None, upgrades, clout: 0 });
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
        let fact = self.has(|m| matches!(m, Mod::FactWeapons));
        if !items_only {
            for w in &self.weapons {
                if w.level < MAX_LEVEL && !WEAPONS[w.def].evolved {
                    let partner = merge_of(w.def).is_some_and(|(o, _)| self.weapons.iter().any(|x| x.def == o));
                    out[0].push((Choice::WeaponLevel(w.def), if partner { 2.0 } else { 1.0 }));
                }
            }
            if self.weapons.len() < WEAPON_SLOTS {
                for (d, def) in WEAPONS.iter().enumerate() {
                    if def.evolved || self.weapons.iter().any(|w| w.def == d) {
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
                    out[3].push((Choice::NewItem(i), 1.0));
                }
            }
        }
        out
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
                out.push(if out.contains(&Choice::Heal) { Choice::Clout(10 + 2 * self.minutes() as u32) } else { Choice::Heal });
            }
        }
        out
    }

    fn start_level_up(&mut self) {
        let n = CHOICES + self.count(|m| matches!(m, Mod::ExtraOffer)) as usize;
        let choices = self.pick_choices(n, false);
        self.rerolls = FREE_REROLLS + self.count(|m| matches!(m, Mod::FreeReroll)) + self.sum(|m| if let Mod::ExtraReroll(n) = m { *n as f32 } else { 0.0 }) as u32;
        self.paid_rerolls = 0;
        self.phase = Phase::LevelUp(choices);
    }

    /// Applies a choice to your build.
    fn apply(&mut self, c: Choice) {
        match c {
            Choice::NewWeapon(d) => {
                if self.weapons.len() < WEAPON_SLOTS && self.weapons.iter().all(|w| w.def != d) {
                    self.weapons.push(OwnedWeapon { def: d, level: 1 });
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
    fn upgrades(&mut self, n: usize) -> Vec<Choice> {
        let mut out = Vec::new();
        for _ in 0..n {
            let cands = self.candidates(false);
            let mut pool: Vec<Choice> = cands[0].iter().chain(cands[2].iter()).map(|c| c.0).collect();
            if pool.is_empty() {
                pool = cands[1].iter().chain(cands[3].iter()).map(|c| c.0).collect();
            }
            let c = if pool.is_empty() { Choice::Clout(15) } else { pool[self.rng.below(pool.len() as u32) as usize] };
            self.apply(c);
            out.push(c);
        }
        out
    }

    /// A merge that is ready: two weapons at max level that make a third.
    pub fn ready_merge(&self) -> Option<(usize, usize, usize)> {
        self.weapons.iter().filter(|w| w.level >= MAX_LEVEL).find_map(|w| {
            let (other, into) = merge_of(w.def)?;
            self.weapons.iter().any(|x| x.def == other && x.level >= MAX_LEVEL).then_some((w.def, other, into))
        })
    }

    /// Opens a chest: a merge if one is ready, otherwise upgrades (CHEST-01).
    fn open_chest(&mut self, value: u32, title: &'static str) {
        let merged = self.ready_merge();
        let mut upgrades = Vec::new();
        if let Some((a, b, into)) = merged {
            let slot = self.weapons.iter().position(|w| w.def == a).unwrap_or(0);
            self.weapons[slot] = OwnedWeapon { def: into, level: MAX_LEVEL };
            self.weapons.retain(|w| w.def != b);
            self.sync();
        } else {
            let n = if value >= 2 {
                3
            } else if self.rng.chance(0.2) {
                2
            } else {
                1
            };
            upgrades = self.upgrades(n);
        }
        let clout = 5 + self.rng.below(10) + 10 * value;
        self.clout += clout;
        self.phase = Phase::Chest(Reward { title: if merged.is_some() { "MERGE!" } else { title }, merged, upgrades, clout });
    }

    /// Closes a chest, a vending machine or a shrine and plays on.
    pub fn close(&mut self) -> Result<(), RuleError> {
        match self.phase {
            Phase::Chest(_) | Phase::Vending { .. } | Phase::Shrine { .. } => {
                self.phase = Phase::Playing;
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
        let due = if self.stage == 6 { waves::MODEL_AFTER } else { waves::BOSS_AFTER };
        Some((due - (self.seconds() - self.platform_since)).max(0.0))
    }

    /// The boss coming next.
    pub fn next_boss(&self) -> BossKind {
        if self.stage == 6 {
            BossKind::TheModel
        } else {
            self.platform.boss()
        }
    }
}

#[cfg(test)]
impl Run {
    /// Draws a fresh level-up without spending anything.
    pub fn reroll_free_for_test(&mut self) {
        self.rerolls = 1;
        let _ = self.reroll();
    }
}
