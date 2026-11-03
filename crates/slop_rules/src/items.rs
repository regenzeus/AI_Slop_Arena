//! Coping mechanisms: every item, as data (ITEM-01). Items stack without limit;
//! their drawbacks and their price are what make each purchase a choice.

use slop_sim::world::{Active, Loadout};
use slop_sim::{DamageType, Stats};

/// The ten stats, for items and level-ups.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StatKind {
    MentalHealth,
    Composure,
    SelfCare,
    Focus,
    Conviction,
    Reach,
    Speed,
    Scepticism,
    Curiosity,
    Luck,
}

impl StatKind {
    pub const ALL: [StatKind; 10] = [
        StatKind::MentalHealth,
        StatKind::Composure,
        StatKind::SelfCare,
        StatKind::Focus,
        StatKind::Conviction,
        StatKind::Reach,
        StatKind::Speed,
        StatKind::Scepticism,
        StatKind::Curiosity,
        StatKind::Luck,
    ];

    pub fn name(self) -> &'static str {
        ["Mental Health", "Composure", "Self-Care", "Focus", "Conviction", "Reach", "Speed", "Scepticism", "Curiosity", "Luck"][self as usize]
    }

    /// Whether the stat is shown as a percentage.
    pub fn percent(self) -> bool {
        matches!(self, StatKind::Focus | StatKind::Conviction | StatKind::Reach | StatKind::Speed | StatKind::Scepticism)
    }

    /// Adds `v` to the stat. Percent stats take whole percents.
    pub fn apply(self, s: &mut Stats, v: f32) {
        match self {
            StatKind::MentalHealth => s.max_mental_health += v,
            StatKind::Composure => s.composure += v,
            StatKind::SelfCare => s.self_care += v,
            StatKind::Focus => s.focus += v / 100.0,
            StatKind::Conviction => s.conviction += v / 100.0,
            StatKind::Reach => s.reach += v / 100.0,
            StatKind::Speed => s.speed += v / 100.0,
            StatKind::Scepticism => s.scepticism += v / 100.0,
            StatKind::Curiosity => s.curiosity += v,
            StatKind::Luck => s.luck += v,
        }
    }

    /// "+12% Focus" or "+3 Composure".
    pub fn describe(self, v: f32) -> String {
        let sign = if v >= 0.0 { "+" } else { "-" };
        if self.percent() {
            format!("{sign}{:.0}% {}", v.abs(), self.name())
        } else {
            format!("{sign}{:.0} {}", v.abs(), self.name())
        }
    }

    /// What the stat does, with its current effect worked out (the tooltip).
    pub fn explain(self, s: &Stats) -> String {
        use slop_sim::stats::{COMPOSURE_HALF, SCEPTICISM_CAP, SELF_CARE_RATE};
        match self {
            StatKind::MentalHealth => format!("Your health. At 0 you burn out and the run is over.\nMaximum {:.0}. Every wave starts with it full.", s.max_mental_health),
            StatKind::Composure => format!(
                "Armour. Every hit on you is multiplied by {h:.0} / ({h:.0} + Composure).\nNow you take {:.0}% of each hit. {h:.0} Composure halves damage;\nnegative Composure makes hits hurt more.",
                s.damage_taken(100.0),
                h = COMPOSURE_HALF
            ),
            StatKind::SelfCare => format!("Heals {:.1} Mental Health per second during waves\n({SELF_CARE_RATE} per point).", s.self_care.max(0.0) * SELF_CARE_RATE),
            StatKind::Focus => format!("Attack speed of every weapon: cooldowns run {:.2}x as fast.\n+100% Focus fires twice as often.", (1.0 + s.focus).max(0.0)),
            StatKind::Conviction => format!("Damage of every weapon is multiplied by {:.2}.", (1.0 + s.conviction).max(0.0)),
            StatKind::Reach => format!("Range of every weapon is multiplied by {:.2}:\nshots fly further; swings, auras and slams grow.", (1.0 + s.reach).max(0.0)),
            StatKind::Speed => format!("How fast you move: {:.0} units per second ({:.0}% of normal).", slop_sim::world::PLAYER_SPEED * s.speed, s.speed * 100.0),
            StatKind::Scepticism => format!("Chance to ignore a hit completely: {:.0}%.\nCapped at {:.0}%.", s.dodge_chance() * 100.0, SCEPTICISM_CAP * 100.0),
            StatKind::Curiosity => format!("Pickup radius for Clout coins: {:.0} units.\nCoins inside it fly to you.", s.curiosity.max(0.0)),
            StatKind::Luck => format!(
                "Each point moves 1% of the shop's odds up a tier\nand makes good and great level-ups likelier.\nNow +{:.0}% better rolls.",
                s.luck.max(0.0)
            ),
        }
    }

    /// The stat's current value, as shown on the stats panel.
    pub fn read(self, s: &Stats) -> String {
        match self {
            StatKind::MentalHealth => format!("{:.0}", s.max_mental_health),
            StatKind::Composure => format!("{:.0}", s.composure),
            StatKind::SelfCare => format!("{:.0}", s.self_care),
            StatKind::Focus => format!("{:+.0}%", s.focus * 100.0),
            StatKind::Conviction => format!("{:+.0}%", s.conviction * 100.0),
            StatKind::Reach => format!("{:+.0}%", s.reach * 100.0),
            StatKind::Speed => format!("{:.0}%", s.speed * 100.0),
            StatKind::Scepticism => format!("{:.0}%", s.scepticism * 100.0),
            StatKind::Curiosity => format!("{:.0}", s.curiosity),
            StatKind::Luck => format!("{:.0}", s.luck),
        }
    }
}

/// One effect of an item or a hero.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mod {
    Stat(StatKind, f32),
    TypeBonus(DamageType, f32),
    LikeBonus(f32),
    BrainrotBonus(f32),
    MarkBonus(f32),
    /// +4% Conviction for each different damage type among your weapons.
    Journal,
    Coffee,
    Doomscrolling,
    GroupChat,
    NoiseCancel,
    AdBlocker,
    SecondWind,
    Houseplant,
    TherapyDog,
    HotTake,
    Dumbphone,
    ChainLetter,
    Fluent,
    Active(Active),
    /// Interest on unspent Clout rises to this share, capped at 20.
    Interest(f32),
    /// Clout at the end of every wave.
    WaveIncome(u32),
    /// Clout per wave for every wave survived so far.
    InfluencerDeal,
    /// Clout changes by -10% to +25% after each wave.
    CryptoWallet,
    ShopDiscount(f32),
    ShopPriceUp(f32),
    /// Selling refunds this share.
    SellRefund(f32),
    FreeReroll,
    ExtraOffer,
    /// The slop learns this much faster.
    LearnFaster(f32),
    /// Patch notes preview the next training run.
    Preview,
    /// The shop always offers a Fact weapon.
    FactWeapons,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemKind {
    Healthy,
    Risky,
    Hustle,
    Active,
}

impl ItemKind {
    pub fn name(self) -> &'static str {
        ["Healthy habit", "Risky habit", "Side hustle", "Active"][self as usize]
    }
}

pub struct ItemDef {
    pub name: &'static str,
    pub kind: ItemKind,
    /// 1 to 4: sets the frame colour and the price.
    pub tier: u8,
    pub mods: &'static [Mod],
    /// The effect in words.
    pub effect: &'static str,
    pub flavour: &'static str,
}

const fn item(name: &'static str, kind: ItemKind, tier: u8, mods: &'static [Mod], effect: &'static str, flavour: &'static str) -> ItemDef {
    ItemDef { name, kind, tier, mods, effect, flavour }
}

use ItemKind::{Active as Act, Healthy, Hustle, Risky};
use StatKind::*;

pub const ITEMS: &[ItemDef] = &[
    item("Meditation", Healthy, 1, &[Mod::Stat(SelfCare, 2.0), Mod::Stat(Composure, 2.0)], "+2 Self-Care, +2 Composure", "Breathe in. Breathe out. Ignore the notification."),
    item("Therapy Dog", Healthy, 2, &[Mod::Stat(MentalHealth, 25.0), Mod::TherapyDog], "+25 Mental Health. With a Dog Walk, the dog blocks a hit every 10 s", "Professionally good."),
    item("Houseplant", Healthy, 1, &[Mod::Stat(SelfCare, 1.0), Mod::Houseplant], "+1 Self-Care. Heal 1 for every Grass kill", "Has never posted anything."),
    item("Journal", Healthy, 2, &[Mod::Journal], "+4% Conviction for each different damage type you carry", "Dear diary, the slop is winning."),
    item("Noise-Cancelling Headphones", Healthy, 2, &[Mod::NoiseCancel], "Blocks one speech bubble every 3 s", "Bubbles hit the headphones instead."),
    item("Ad Blocker", Healthy, 1, &[Mod::AdBlocker], "Pop-ups never appear", "Worth every cent."),
    item("Weighted Blanket", Healthy, 1, &[Mod::Stat(Composure, 5.0), Mod::Stat(Speed, -5.0)], "+5 Composure, -5% Speed", "Heavy, safe, sleepy."),
    item("Library Card", Healthy, 2, &[Mod::TypeBonus(DamageType::Fact, 0.15), Mod::FactWeapons], "Fact weapons +15% damage; the shop always offers a Fact weapon", "Free knowledge. Remember that?"),
    item("Running Shoes", Healthy, 1, &[Mod::Stat(Speed, 10.0)], "+10% Speed", "Outrun the algorithm."),
    item("Good Night's Sleep", Healthy, 3, &[Mod::SecondWind], "Once per wave, survive a killing blow with 1 Mental Health", "Rare. Like in real life."),
    item("Group Chat", Healthy, 2, &[Mod::GroupChat], "+1% Conviction for every 5 slop on screen", "Your friends hate this too."),
    item("Touch-Grass Badge", Healthy, 2, &[Mod::Stat(SelfCare, 3.0), Mod::TypeBonus(DamageType::Grass, 0.10)], "+3 Self-Care, +10% Grass damage", "Official proof you went outside."),
    item("Self-Help Book", Healthy, 1, &[Mod::Stat(MentalHealth, 10.0), Mod::Stat(SelfCare, 1.0)], "+10 Mental Health, +1 Self-Care", "Chapter one: stop reading self-help books."),
    item("Coffee", Risky, 1, &[Mod::Stat(Focus, 15.0), Mod::Coffee], "+15% Focus. An Anxiety stack every 20 s", "Third cup. Totally fine."),
    item("Energy Drink", Risky, 2, &[Mod::Stat(Speed, 20.0), Mod::Stat(Focus, 10.0), Mod::Stat(MentalHealth, -15.0)], "+20% Speed, +10% Focus, -15 Mental Health", "Tastes like a deadline."),
    item("Doomscrolling", Risky, 2, &[Mod::LikeBonus(0.5), Mod::Doomscrolling], "+50% Likes. -1 Mental Health per second while slop is on screen", "Just one more."),
    item("Hot Take", Risky, 2, &[Mod::TypeBonus(DamageType::Sarcasm, 0.25), Mod::HotTake], "+25% Sarcasm damage. Hits on you are 10% harder", "Ratio incoming."),
    item("Dumbphone", Risky, 1, &[Mod::Stat(Scepticism, 15.0), Mod::Dumbphone, Mod::Stat(Curiosity, -40.0)], "+15% Scepticism, immune to Doomscroll. -40 Curiosity", "Snake still slaps."),
    item("Burner Account", Risky, 2, &[Mod::FreeReroll, Mod::LearnFaster(0.25)], "One free reroll per shop. The slop learns 25% faster", "Nobody will ever know."),
    item("Clickbait Thumbnail", Risky, 2, &[Mod::LikeBonus(0.3), Mod::ShopPriceUp(0.1)], "+30% Likes. Shop prices +10%", "YOU WON'T BELIEVE THIS ITEM."),
    item("Side Hustle", Hustle, 1, &[Mod::WaveIncome(8)], "+8 Clout at the end of every wave", "Passive income, allegedly."),
    item("Savings Account", Hustle, 2, &[Mod::Interest(0.10)], "Interest on unspent Clout rises to 10%, up to 20 per wave", "Boring. Works."),
    item("Referral Link", Hustle, 1, &[Mod::LikeBonus(0.1)], "+10% Likes", "Use my code."),
    item("Coupon Clipper", Hustle, 2, &[Mod::ShopDiscount(0.08)], "Shop prices -8%", "Ten cents off."),
    item("Garage Sale", Hustle, 1, &[Mod::SellRefund(0.75)], "Selling a weapon refunds 75% instead of 50%", "Everything must go."),
    item("Influencer Deal", Hustle, 2, &[Mod::InfluencerDeal, Mod::Stat(Conviction, -5.0)], "+1 Clout per wave for every wave survived. -5% Conviction", "Brought to you by nobody."),
    item("Crypto Wallet", Hustle, 2, &[Mod::CryptoWallet], "After each wave your Clout changes by -10% to +25%", "To the moon. Or not."),
    item("Log Off", Act, 2, &[Mod::Active(Active::LogOff)], "With 8 slop close: dash away, invulnerable. Every 12 s", "Bye."),
    item("Block Button", Act, 2, &[Mod::Active(Active::BlockButton)], "Deletes a bubble about to hit you, and the next one. Every 4 s", "Blocked and reported."),
    item("Digital Detox", Act, 2, &[Mod::Active(Active::DigitalDetox)], "With 2 statuses: clear them and push slop back. Every 20 s", "Phone in the drawer."),
    item("Unsubscribe", Act, 3, &[Mod::Active(Active::Unsubscribe)], "Below 30% Mental Health: all slop on screen lose 10% HP and stall. Every 45 s", "You will not be missed."),
    item("Phone a Friend", Act, 3, &[Mod::Active(Active::PhoneAFriend)], "An elite or boss near: a friend runs through it for 40. Every 25 s", "Real friends. Remember those?"),
];

pub fn item_index(name: &str) -> Option<usize> {
    ITEMS.iter().position(|i| i.name == name)
}

/// Base price of an item tier; weapons cost a third more.
pub fn tier_price(tier: u8) -> u32 {
    [12, 25, 50, 100][(tier.clamp(1, 4) - 1) as usize]
}

/// Folds one modifier into a loadout. Rules-only modifiers are ignored here.
pub fn apply(m: Mod, l: &mut Loadout, weapon_types: usize) {
    let p = &mut l.perks;
    match m {
        Mod::Stat(k, v) => k.apply(&mut l.stats, v),
        Mod::TypeBonus(t, v) => p.type_bonus[t as usize] += v,
        Mod::LikeBonus(v) => p.like_bonus += v,
        Mod::BrainrotBonus(v) => p.brainrot_bonus += v,
        Mod::MarkBonus(v) => p.mark_bonus += v,
        Mod::Journal => l.stats.conviction += 0.04 * weapon_types as f32,
        Mod::Coffee => p.coffee += 1,
        Mod::Doomscrolling => p.doomscrolling += 1,
        Mod::GroupChat => p.group_chat += 1,
        Mod::NoiseCancel => p.noise_cancel = true,
        Mod::AdBlocker => p.ad_blocker = true,
        Mod::SecondWind => p.second_wind += 1,
        Mod::Houseplant => p.houseplant += 1,
        Mod::TherapyDog => p.therapy_dog = true,
        Mod::HotTake => p.hot_take = true,
        Mod::Dumbphone => p.dumbphone = true,
        Mod::ChainLetter => p.chain_letter = true,
        Mod::Fluent => p.fluent = true,
        Mod::Active(a) => {
            if !p.actives.contains(&a) {
                p.actives.push(a);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: ITEM-01
    #[test]
    fn items_cover_every_kind_and_describe_themselves() {
        assert!(ITEMS.len() >= 30);
        for kind in [Healthy, Risky, Hustle, Act] {
            assert!(ITEMS.iter().filter(|i| i.kind == kind).count() >= 5, "{kind:?}");
        }
        for i in ITEMS {
            assert!(!i.mods.is_empty() && !i.effect.is_empty() && (1..=4).contains(&i.tier), "{}", i.name);
        }
        // Risky items carry a drawback in their effect text or a second mod.
        for i in ITEMS.iter().filter(|i| i.kind == Risky) {
            assert!(i.mods.len() >= 2, "{}", i.name);
        }
    }

    // REQ: UI-13
    #[test]
    fn every_stat_explains_its_current_effect() {
        let mut s = Stats { composure: 15.0, ..Stats::default() };
        assert!(StatKind::Composure.explain(&s).contains("50%"), "15 Composure halves damage");
        s.focus = 1.0;
        assert!(StatKind::Focus.explain(&s).contains("2.00x"));
        for k in StatKind::ALL {
            assert!(k.explain(&s).len() > 30, "{k:?}");
        }
    }

    #[test]
    fn stats_apply_in_their_units() {
        let mut s = Stats::default();
        StatKind::Focus.apply(&mut s, 15.0);
        StatKind::MentalHealth.apply(&mut s, 25.0);
        assert!((s.focus - 0.15).abs() < 1e-6 && s.max_mental_health == 125.0);
        assert_eq!(StatKind::Focus.describe(15.0), "+15% Focus");
        assert_eq!(StatKind::Composure.describe(-3.0), "-3 Composure");
    }
}
