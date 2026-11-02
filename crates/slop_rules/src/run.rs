//! A run: 20 waves, and between them level-ups, patch notes and the shop
//! (RUN-01 to RUN-05).
//!
//! ```text
//! Wave -> LevelUp (one per level gained) -> PatchNotes -> Shop -> Wave ...
//!   \-> Over { victory: false } on Burnout; Over { victory: true } after wave 20
//! ```

use slop_sim::content::{weapon_index, Pattern, Platform, WEAPONS};
use slop_sim::rng::Rng;
use slop_sim::world::{Loadout, Outcome, World};
use slop_sim::{DamageType, Input, Stats};

use crate::heroes::Hero;
use crate::items::{self, item_index, Mod, ITEMS};
use crate::levels::{self, StatUp};
use crate::shop::{self, Goods, Note, Offer, Shop};
use crate::training::{PatchNotes, Training};
use crate::waves::{self, WAVES};

pub const WEAPON_SLOTS: usize = 6;
const OFFERS: usize = 5;
const WEAPON_SHARE: f32 = 0.4;
/// Interest on unspent Clout after a wave, and its cap.
const INTEREST: (f32, u32) = (0.05, 10);
const SAVINGS_CAP: u32 = 20;
const SELL_REFUND: f32 = 0.5;
const MAX_LOOT_BOXES: u32 = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuleError {
    WrongPhase,
    NotEnoughClout,
    NoWeaponSlot,
    MaxTier,
    NoSuchOffer,
    Sold,
}

impl std::fmt::Display for RuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            RuleError::WrongPhase => "Not now.",
            RuleError::NotEnoughClout => "Not enough Clout.",
            RuleError::NoWeaponSlot => "All six weapon slots are full. Sell one first.",
            RuleError::MaxTier => "That weapon is already tier IV.",
            RuleError::NoSuchOffer => "No such offer.",
            RuleError::Sold => "Already sold.",
        })
    }
}

#[derive(Clone, Debug)]
pub enum Phase {
    Wave,
    /// Pick one of these; more level-ups may be queued.
    LevelUp(Vec<StatUp>),
    PatchNotes(PatchNotes),
    Shop,
    Over { victory: bool },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnedWeapon {
    pub def: usize,
    pub tier: u8,
    /// Clout paid in total, for refunds.
    pub paid: u32,
}

/// What a finished wave earned, for the report.
#[derive(Clone, Debug, Default)]
pub struct WaveReport {
    pub wave: u32,
    pub cleared: bool,
    pub kills: u32,
    pub likes: u32,
    pub interest: u32,
    pub income: i32,
    pub levels: u32,
    pub damage_taken: f32,
    pub damage_by_type: [f32; 4],
    pub loot_boxes: u32,
}

pub struct Run {
    pub seed: u64,
    pub hero: Hero,
    pub platform: Platform,
    /// The wave being played or coming next, 1 to 20.
    pub wave: u32,
    pub clout: u32,
    pub xp: u32,
    pub level: u32,
    pub weapons: Vec<OwnedWeapon>,
    /// Item index and count.
    pub items: Vec<(usize, u32)>,
    /// Stats from your hero and level-ups; items are added by `loadout`.
    pub base: Stats,
    pub pending_levels: u32,
    pub shop: Shop,
    pub training: Training,
    pub phase: Phase,
    pub world: Option<World>,
    pub reports: Vec<WaveReport>,
    /// Damage each weapon has dealt over the run.
    pub weapon_damage: Vec<f32>,
    pub loot_boxes: u32,
    /// The last wave ended below 30% Mental Health.
    low_health: bool,
    /// Patch notes waiting while level-ups are chosen.
    parked_notes: Option<Phase>,
    genomes: Vec<(usize, usize)>,
    rng: Rng,
}

impl Run {
    pub fn new(seed: u64, hero: Hero) -> Run {
        let mut rng = Rng::new(seed, 7);
        let platform = Platform::ALL[rng.below(Platform::ALL.len() as u32) as usize];
        Run::on(seed, hero, platform, rng)
    }

    /// A run on a chosen platform.
    pub fn on_platform(seed: u64, hero: Hero, platform: Platform) -> Run {
        Run::on(seed, hero, platform, Rng::new(seed, 7))
    }

    fn on(seed: u64, hero: Hero, platform: Platform, mut rng: Rng) -> Run {
        let def = hero.def();
        let weapon = weapon_index(def.weapon).expect("hero weapons exist");
        let item = item_index(def.item).expect("hero items exist");
        let training = Training::new(&mut rng);
        let mut run = Run {
            seed,
            hero,
            platform,
            wave: 1,
            clout: 0,
            xp: 0,
            level: 0,
            weapons: vec![OwnedWeapon { def: weapon, tier: 1, paid: 20 }],
            items: vec![(item, 1)],
            base: Stats::default(),
            pending_levels: 0,
            shop: Shop::default(),
            training,
            phase: Phase::Wave,
            world: None,
            reports: Vec::new(),
            weapon_damage: vec![0.0; WEAPONS.len()],
            loot_boxes: 0,
            low_health: false,
            parked_notes: None,
            genomes: Vec::new(),
            rng,
        };
        let mut l = Loadout { weapons: vec![], stats: run.base, perks: Default::default() };
        for m in def.mods {
            items::apply(*m, &mut l, 0);
        }
        run.base = l.stats;
        run.start_wave();
        run
    }

    /// Every modifier from the hero and the items, with counts.
    fn mods(&self) -> Vec<Mod> {
        let mut out: Vec<Mod> = self.hero.def().mods.to_vec();
        for &(i, n) in &self.items {
            for _ in 0..n {
                out.extend_from_slice(ITEMS[i].mods);
            }
        }
        out
    }

    fn has(&self, f: impl Fn(&Mod) -> bool) -> bool {
        self.mods().iter().any(f)
    }

    fn sum(&self, f: impl Fn(&Mod) -> f32) -> f32 {
        self.mods().iter().map(f).sum()
    }

    /// What you bring into the next wave: weapons, stats and perks (RUN-02).
    pub fn loadout(&self) -> Loadout {
        let mut l = Loadout { weapons: self.weapons.iter().map(|w| (w.def, w.tier)).collect(), stats: self.base, perks: Default::default() };
        let types = DamageType::ALL.iter().filter(|t| self.weapons.iter().any(|w| WEAPONS[w.def].kind == **t)).count();
        for &(i, n) in &self.items {
            for _ in 0..n {
                for m in ITEMS[i].mods {
                    items::apply(*m, &mut l, types);
                }
            }
        }
        for m in self.hero.def().mods {
            match m {
                Mod::Stat(..) => {}
                other => items::apply(*other, &mut l, types),
            }
        }
        l
    }

    /// The stats you would enter a wave with, for the stats panel.
    pub fn stats(&self) -> Stats {
        self.loadout().stats
    }

    fn start_wave(&mut self) {
        let built = waves::build(self.wave, self.platform, &self.training, &self.weapon_damage);
        self.genomes = built.genomes;
        let seed = self.seed.wrapping_mul(31).wrapping_add(self.wave as u64);
        self.world = Some(World::new(seed, built.setup, &self.loadout()));
        self.phase = Phase::Wave;
    }

    /// Advances the wave one tick; returns true when it just ended.
    pub fn step(&mut self, input: Input) -> bool {
        let Some(world) = self.world.as_mut() else { return false };
        if !matches!(self.phase, Phase::Wave) {
            return false;
        }
        world.step(input);
        if world.outcome == Outcome::Running {
            return false;
        }
        self.finish_wave();
        true
    }

    /// Books the wave's results and moves on (RUN-03).
    fn finish_wave(&mut self) {
        let world = self.world.as_ref().expect("a wave was running");
        let cleared = world.outcome == Outcome::Cleared;
        let tally = world.tally.clone();
        for (slot, dmg) in tally.weapon_damage.iter().enumerate() {
            if let Some(w) = self.weapons.get(slot) {
                self.weapon_damage[w.def] += dmg;
            }
        }
        let mut report = WaveReport {
            wave: self.wave,
            cleared,
            kills: tally.kills,
            likes: world.clout,
            damage_taken: tally.damage_taken,
            damage_by_type: tally.damage_dealt,
            loot_boxes: tally.loot_boxes,
            ..WaveReport::default()
        };
        if !cleared {
            self.reports.push(report);
            self.phase = Phase::Over { victory: false };
            return;
        }
        let health = world.player.mental_health / world.player.stats.max_mental_health;
        self.low_health = health < 0.3;
        let earned = world.clout;
        let genome_tallies: Vec<_> = self.genomes.iter().map(|&(spec, g)| (g, world.spec_tally[spec])).collect();

        // Clout: Likes, interest and side hustles.
        self.clout += earned;
        let (rate, cap) = if self.has(|m| matches!(m, Mod::Interest(_))) { (0.10, SAVINGS_CAP) } else { INTEREST };
        report.interest = ((self.clout as f32 * rate) as u32).min(cap);
        self.clout += report.interest;
        let mut income: i32 = self.sum(|m| if let Mod::WaveIncome(n) = m { *n as f32 } else { 0.0 }) as i32;
        let deals = self.mods().iter().filter(|m| matches!(m, Mod::InfluencerDeal)).count() as i32;
        income += deals * self.wave as i32;
        for _ in 0..self.mods().iter().filter(|m| matches!(m, Mod::CryptoWallet)).count() {
            let swing = self.rng.range(-0.10, 0.25);
            income += (self.clout as f32 * swing) as i32;
        }
        self.clout = (self.clout as i32 + income).max(0) as u32;
        report.income = income;

        // Levels: Clout earned is XP.
        self.xp += earned;
        while self.xp >= levels::xp_for(self.level + 1) {
            self.xp -= levels::xp_for(self.level + 1);
            self.level += 1;
            self.pending_levels += 1;
            report.levels += 1;
        }
        self.loot_boxes = (self.loot_boxes + tally.loot_boxes).min(MAX_LOOT_BOXES);
        self.reports.push(report);

        if self.wave >= WAVES {
            self.phase = Phase::Over { victory: true };
            return;
        }
        self.training.learn_rate = 1.0 + self.sum(|m| if let Mod::LearnFaster(v) = m { *v } else { 0.0 });
        let preview = self.has(|m| matches!(m, Mod::Preview));
        let notes = self.training.train(self.wave, &tally, &genome_tallies, &mut self.rng, preview);
        self.phase = Phase::PatchNotes(notes);
        self.next_level_up();
    }

    /// Shows the next queued level-up, if any, before the patch notes.
    fn next_level_up(&mut self) {
        if self.pending_levels > 0 {
            let luck = self.stats().luck;
            let offers = levels::offers(&mut self.rng, luck);
            let notes = std::mem::replace(&mut self.phase, Phase::Wave);
            self.parked_notes = Some(notes);
            self.phase = Phase::LevelUp(offers);
        }
    }

    pub fn choose_level(&mut self, i: usize) -> Result<(), RuleError> {
        let Phase::LevelUp(offers) = &self.phase else { return Err(RuleError::WrongPhase) };
        let up = *offers.get(i).ok_or(RuleError::NoSuchOffer)?;
        up.stat.apply(&mut self.base, up.amount);
        self.pending_levels -= 1;
        self.phase = self.parked_notes.take().unwrap_or(Phase::Shop);
        if self.pending_levels > 0 {
            self.next_level_up();
        }
        Ok(())
    }

    /// Done reading the patch notes: the shop opens (RUN-04).
    pub fn close_patch_notes(&mut self) -> Result<(), RuleError> {
        if !matches!(self.phase, Phase::PatchNotes(_)) {
            return Err(RuleError::WrongPhase);
        }
        self.open_shop();
        Ok(())
    }

    fn discount(&self) -> (f32, f32) {
        (self.sum(|m| if let Mod::ShopDiscount(v) = m { *v } else { 0.0 }), self.sum(|m| if let Mod::ShopPriceUp(v) = m { *v } else { 0.0 }))
    }

    fn open_shop(&mut self) {
        self.wave += 1;
        self.shop.rerolls = 0;
        self.shop.free_rerolls = self.mods().iter().filter(|m| matches!(m, Mod::FreeReroll)).count() as u32;
        let kept: Vec<Offer> = self.shop.offers.iter().filter(|o| o.locked && !o.sold).copied().collect();
        self.shop.offers = kept;
        for _ in 0..self.loot_boxes {
            let item = self.rng.below(ITEMS.len() as u32) as usize;
            self.shop.offers.insert(0, Offer { goods: Goods::Item { item }, price: 0, locked: false, sold: false, note: Some(Note::LootBox) });
        }
        self.loot_boxes = 0;
        if self.low_health {
            let book = item_index("Self-Help Book").expect("exists");
            let (d, up) = self.discount();
            let price = shop::price(Goods::Item { item: book }, self.wave, d, up) / 2;
            self.shop.offers.push(Offer { goods: Goods::Item { item: book }, price, locked: false, sold: false, note: Some(Note::Mercy) });
        }
        self.fill_shop();
        self.phase = Phase::Shop;
    }

    fn offer_count(&self) -> usize {
        OFFERS + self.mods().iter().filter(|m| matches!(m, Mod::ExtraOffer)).count()
    }

    /// Fills the shop up with fresh offers (SHOP-01).
    fn fill_shop(&mut self) {
        let luck = self.stats().luck;
        let (d, up) = self.discount();
        let owned: Vec<DamageType> = self.weapons.iter().map(|w| WEAPONS[w.def].kind).collect();
        let fact = self.has(|m| matches!(m, Mod::FactWeapons));
        let mut fresh = Vec::new();
        let regular = self.shop.offers.iter().filter(|o| o.note.is_none()).count();
        let want = self.offer_count().saturating_sub(regular);
        let mut has_match = self.shop.offers.iter().any(|o| matches!(o.goods, Goods::Weapon { def, .. } if owned.contains(&WEAPONS[def].kind)));
        let mut has_fact = self.shop.offers.iter().any(|o| matches!(o.goods, Goods::Weapon { def, .. } if WEAPONS[def].kind == DamageType::Fact));
        for k in 0..want {
            let last = k + 1 == want;
            let need_fact = last && fact && !has_fact;
            let need_match = last && !has_match && !owned.is_empty();
            let weapon = self.rng.chance(WEAPON_SHARE) || need_match || need_fact;
            let goods = if weapon {
                let allowed = |d: usize| !WEAPONS[d].rare || self.wave >= 6;
                let wanted = |d: usize| {
                    if need_fact {
                        WEAPONS[d].kind == DamageType::Fact
                    } else if need_match {
                        owned.contains(&WEAPONS[d].kind)
                    } else {
                        true
                    }
                };
                let mut pool: Vec<usize> = (0..WEAPONS.len()).filter(|&d| allowed(d) && wanted(d)).collect();
                if pool.is_empty() {
                    pool = (0..WEAPONS.len()).filter(|&d| allowed(d)).collect();
                }
                let def = pool[self.rng.below(pool.len() as u32) as usize];
                has_match |= owned.contains(&WEAPONS[def].kind);
                has_fact |= WEAPONS[def].kind == DamageType::Fact;
                Goods::Weapon { def, tier: shop::roll_tier(&mut self.rng, self.wave, luck) }
            } else {
                let tier = shop::roll_tier(&mut self.rng, self.wave, luck);
                let pool: Vec<usize> = (0..ITEMS.len()).filter(|&i| ITEMS[i].tier <= tier.max(1) + 1 && ITEMS[i].name != "Self-Help Book").collect();
                Goods::Item { item: pool[self.rng.below(pool.len() as u32) as usize] }
            };
            let price = shop::price(goods, self.wave, d, up);
            fresh.push(Offer { goods, price, locked: false, sold: false, note: None });
        }
        self.shop.offers.extend(fresh);
    }

    /// Buys offer `i` (SHOP-01). Weapons you own are upgraded a tier.
    pub fn buy(&mut self, i: usize) -> Result<(), RuleError> {
        if !matches!(self.phase, Phase::Shop) {
            return Err(RuleError::WrongPhase);
        }
        let offer = *self.shop.offers.get(i).ok_or(RuleError::NoSuchOffer)?;
        if offer.sold {
            return Err(RuleError::Sold);
        }
        if offer.price > self.clout {
            return Err(RuleError::NotEnoughClout);
        }
        match offer.goods {
            Goods::Weapon { def, tier } => {
                if let Some(w) = self.weapons.iter_mut().find(|w| w.def == def) {
                    if w.tier >= 4 {
                        return Err(RuleError::MaxTier);
                    }
                    w.tier = (w.tier + 1).max(tier).min(4);
                    w.paid += offer.price;
                } else if self.weapons.len() >= WEAPON_SLOTS {
                    return Err(RuleError::NoWeaponSlot);
                } else {
                    self.weapons.push(OwnedWeapon { def, tier, paid: offer.price });
                }
            }
            Goods::Item { item } => match self.items.iter_mut().find(|(it, _)| *it == item) {
                Some((_, n)) => *n += 1,
                None => self.items.push((item, 1)),
            },
        }
        self.clout -= offer.price;
        let o = &mut self.shop.offers[i];
        o.sold = true;
        o.locked = false;
        Ok(())
    }

    /// Whether buying offer `i` would upgrade a weapon you own (UI: the ribbon).
    pub fn upgrades(&self, i: usize) -> Option<u8> {
        match self.shop.offers.get(i)?.goods {
            Goods::Weapon { def, .. } => self.weapons.iter().find(|w| w.def == def).map(|w| w.tier),
            Goods::Item { .. } => None,
        }
    }

    pub fn reroll(&mut self) -> Result<(), RuleError> {
        if !matches!(self.phase, Phase::Shop) {
            return Err(RuleError::WrongPhase);
        }
        let cost = self.shop.reroll_cost();
        if cost > self.clout {
            return Err(RuleError::NotEnoughClout);
        }
        self.clout -= cost;
        if self.shop.free_rerolls > 0 {
            self.shop.free_rerolls -= 1;
        } else {
            self.shop.rerolls += 1;
        }
        self.shop.offers.retain(|o| o.locked && !o.sold);
        self.fill_shop();
        Ok(())
    }

    /// Locked offers stay through rerolls and into the next shop (SHOP-04).
    pub fn toggle_lock(&mut self, i: usize) -> Result<(), RuleError> {
        let o = self.shop.offers.get_mut(i).ok_or(RuleError::NoSuchOffer)?;
        if o.sold || o.note.is_some() {
            return Err(RuleError::Sold);
        }
        o.locked = !o.locked;
        Ok(())
    }

    /// The refund for selling weapon slot `slot`.
    pub fn sell_value(&self, slot: usize) -> u32 {
        let share = if self.has(|m| matches!(m, Mod::SellRefund(_))) { 0.75 } else { SELL_REFUND };
        self.weapons.get(slot).map_or(0, |w| (w.paid as f32 * share) as u32)
    }

    pub fn sell(&mut self, slot: usize) -> Result<(), RuleError> {
        if !matches!(self.phase, Phase::Shop) {
            return Err(RuleError::WrongPhase);
        }
        if slot >= self.weapons.len() {
            return Err(RuleError::NoSuchOffer);
        }
        self.clout += self.sell_value(slot);
        self.weapons.remove(slot);
        Ok(())
    }

    /// Leaves the shop and starts the next wave (RUN-01).
    pub fn next_wave(&mut self) -> Result<(), RuleError> {
        if !matches!(self.phase, Phase::Shop) {
            return Err(RuleError::WrongPhase);
        }
        self.start_wave();
        Ok(())
    }

    /// True once the run is won or lost.
    pub fn is_over(&self) -> bool {
        matches!(self.phase, Phase::Over { .. })
    }

    /// Kills by damage type over the run, for the end screen.
    pub fn kills_by_type(&self) -> [u32; 4] {
        self.training.kills
    }

    /// Weapon slots holding an orbit weapon (for the Therapy Dog note).
    pub fn has_dog(&self) -> bool {
        self.weapons.iter().any(|w| matches!(WEAPONS[w.def].pattern, Pattern::Orbit { .. }))
    }
}
