//! Coping mechanisms: every item, as data (ITEM-01). Items are passives with
//! levels, picked on level-ups, bought from vending machines or found in
//! chests; you carry six. Items come in four rarities, and legendary ones
//! break the rules (ITEM-03). Shrine blessings and glitches, the rare perks
//! that change a run (GLITCH-01), are here too.

use slop_sim::world::{Active, Loadout};
use slop_sim::{DamageType, Stats};

/// The twelve stats, for items and level-ups.
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
    Boundaries,
    Hustle,
}

impl StatKind {
    pub const ALL: [StatKind; 12] = [
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
        StatKind::Boundaries,
        StatKind::Hustle,
    ];

    pub fn name(self) -> &'static str {
        ["Mental Health", "Composure", "Self-Care", "Focus", "Conviction", "Reach", "Speed", "Scepticism", "Curiosity", "Luck", "Boundaries", "Hustle"][self as usize]
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
            StatKind::Boundaries => s.boundaries += v,
            StatKind::Hustle => s.hustle += v,
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
            StatKind::MentalHealth => format!("Your health. At 0 you burn out and the run is over.\nMaximum {:.0}.", s.max_mental_health),
            StatKind::Composure => format!(
                "Armour. Every hit on you is multiplied by {h:.0} / ({h:.0} + Composure).\nNow you take {:.0}% of each hit. {h:.0} Composure halves damage;\nnegative Composure makes hits hurt more.",
                s.damage_taken(100.0),
                h = COMPOSURE_HALF
            ),
            StatKind::SelfCare => format!("Heals {:.1} Mental Health per second\n({SELF_CARE_RATE} per point).", s.self_care.max(0.0) * SELF_CARE_RATE),
            StatKind::Focus => format!("Attack speed of every weapon: cooldowns run {:.2}x as fast.\n+100% Focus fires twice as often.", (1.0 + s.focus).max(0.0)),
            StatKind::Conviction => format!("Damage of every weapon is multiplied by {:.2}.", (1.0 + s.conviction).max(0.0)),
            StatKind::Reach => format!("Range of every weapon is multiplied by {:.2}:\nshots fly further; swings, auras and slams grow.", (1.0 + s.reach).max(0.0)),
            StatKind::Speed => format!("How fast you move: {:.0} units per second ({:.0}% of normal).", slop_sim::world::PLAYER_SPEED * s.speed, s.speed * 100.0),
            StatKind::Scepticism => format!("Chance to ignore a hit completely: {:.0}%.\nCapped at {:.0}%.", s.dodge_chance() * 100.0, SCEPTICISM_CAP * 100.0),
            StatKind::Curiosity => format!("Pickup radius for Likes and Clout: {:.0} units.\nInside it they fly to you.", s.curiosity.max(0.0)),
            StatKind::Luck => format!("Better rolls: great stat-ups, rare and legendary items,\nglitches on level-ups and golden chests.\nNow +{:.0}% better rolls.", s.luck.max(0.0)),
            StatKind::Boundaries => format!(
                "A shield in front of your Mental Health: {:.0} points.\nIt takes hits first, and after 3 s without a hit\nit comes back at 35% per second.",
                s.boundaries.max(0.0)
            ),
            StatKind::Hustle => format!("+{:.0} Clout every minute, and slop drop coins\n{:.0}% more often.", s.hustle.max(0.0), s.hustle.max(0.0)),
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
            StatKind::Boundaries => format!("{:.0}", s.boundaries),
            StatKind::Hustle => format!("{:.0}", s.hustle),
        }
    }
}

/// One effect of an item, a hero, a blessing or a glitch.
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
    /// Interest on unspent Clout every minute, capped at 20.
    Interest(f32),
    /// Clout every minute.
    WaveIncome(u32),
    /// Clout every minute for every minute survived so far.
    InfluencerDeal,
    /// Every minute your Clout changes by -10% to +25%.
    CryptoWallet,
    /// Machines on the map cost this much less, or more.
    ShopDiscount(f32),
    ShopPriceUp(f32),
    /// More free rerolls on level-ups.
    ExtraReroll(u32),
    FreeReroll,
    /// One more choice on every level-up.
    ExtraOffer,
    /// More experience from Likes, and more Clout from coins.
    XpBonus(f32),
    CloutBonus(f32),
    /// The slop learns this much faster.
    LearnFaster(f32),
    /// Patch notes preview the next training run.
    Preview,
    /// Level-ups offer Fact weapons more often.
    FactWeapons,
    // Dashes (DASH-01, DASH-02).
    DashCharges(u8),
    /// Seconds off (negative) or on the recharge of a dash charge.
    DashRecharge(f32),
    DashDamage(f32),
    DashFire,
    DashShock,
    Ghost(f32),
    FlameTrail(f32),
    // Defence (DEF-01).
    Thorns(f32),
    ShotArmor(f32),
    DoNotDisturb,
    Therapist(f32),
    // Legendaries (ITEM-03).
    ExtraShots(u8),
    KillBlast(f32),
    AltAccount(f32),
    RingLight,
    GodMode,
    ChainChance(f32),
    // Glitches and hero rules (GLITCH-01, HERO-02).
    ExtraBounces(u8),
    Lifesteal(f32),
    Echo,
    SpeedDamage,
    /// +1% Conviction for every 10 Clout you hold, up to this much.
    CloutDamage(f32),
    /// More slop, and more Likes and Clout from it.
    Curse(f32),
    /// Weapons of this type are never offered.
    NoType(DamageType),
    /// New weapons start at this level.
    StartLevel(u8),
    /// More or fewer weapon slots.
    WeaponSlots(i8),
    /// Elites drop one more chest.
    EliteChests,
}

/// What kind of coping mechanism an item is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemKind {
    Healthy,
    Risky,
    Hustle,
    Active,
    /// Dashes and what they leave behind.
    Movement,
    /// Shields, thorns and other ways to take less.
    Defence,
    Legendary,
}

impl ItemKind {
    pub fn name(self) -> &'static str {
        ["Healthy habit", "Risky habit", "Side hustle", "Active", "Movement", "Defence", "LEGENDARY"][self as usize]
    }
}

/// Common, rare, epic, legendary.
pub fn rarity_name(tier: u8) -> &'static str {
    ["Common", "Rare", "Epic", "Legendary"][(tier.clamp(1, 4) - 1) as usize]
}

pub struct ItemDef {
    pub name: &'static str,
    pub kind: ItemKind,
    /// Rarity, 1 to 4: common, rare, epic, legendary. Sets the frame colour,
    /// how often it is offered, and the price.
    pub tier: u8,
    pub mods: &'static [Mod],
    /// The effect in words.
    pub effect: &'static str,
    pub flavour: &'static str,
}

const fn item(name: &'static str, kind: ItemKind, tier: u8, mods: &'static [Mod], effect: &'static str, flavour: &'static str) -> ItemDef {
    ItemDef { name, kind, tier, mods, effect, flavour }
}

use ItemKind::{Active as Act, Defence, Healthy, Hustle, Legendary, Movement, Risky};
use StatKind::*;

pub const ITEMS: &[ItemDef] = &[
    item("Meditation", Healthy, 1, &[Mod::Stat(SelfCare, 2.0), Mod::Stat(Composure, 2.0)], "+2 Self-Care, +2 Composure", "Breathe in. Breathe out. Ignore the notification."),
    item("Therapy Dog", Healthy, 2, &[Mod::Stat(MentalHealth, 25.0), Mod::TherapyDog], "+25 Mental Health. With a Dog Walk, the dog blocks a hit every 10 s", "Professionally good."),
    item("Houseplant", Healthy, 1, &[Mod::Stat(SelfCare, 1.0), Mod::Houseplant], "+1 Self-Care. Heal 1 for every Grass kill", "Has never posted anything."),
    item("Journal", Healthy, 2, &[Mod::Journal], "+4% Conviction for each different damage type you carry", "Dear diary, the slop is winning."),
    item("Noise-Cancelling Headphones", Healthy, 2, &[Mod::NoiseCancel], "Blocks one speech bubble every 3 s", "Bubbles hit the headphones instead."),
    item("Ad Blocker", Healthy, 1, &[Mod::AdBlocker, Mod::Stat(StatKind::Hustle, 4.0)], "No pop-up ever covers your screen (MeTube ads, Subscribe Pop-ups). +4 Hustle", "Worth every cent."),
    item("Weighted Blanket", Healthy, 1, &[Mod::Stat(Composure, 5.0), Mod::Stat(Speed, -5.0)], "+5 Composure, -5% Speed", "Heavy, safe, sleepy."),
    item("Library Card", Healthy, 2, &[Mod::TypeBonus(DamageType::Fact, 0.15), Mod::FactWeapons], "Fact weapons +15% damage; level-ups offer Fact weapons more often", "Free knowledge. Remember that?"),
    item("Running Shoes", Healthy, 1, &[Mod::Stat(Speed, 10.0)], "+10% Speed", "Outrun the algorithm."),
    item("Good Night's Sleep", Healthy, 3, &[Mod::SecondWind], "Once per platform, survive a killing blow with 1 Mental Health", "Rare. Like in real life."),
    item("Group Chat", Healthy, 2, &[Mod::GroupChat], "+1% Conviction for every 5 slop on screen", "Your friends hate this too."),
    item("Touch-Grass Badge", Healthy, 2, &[Mod::Stat(SelfCare, 3.0), Mod::TypeBonus(DamageType::Grass, 0.10)], "+3 Self-Care, +10% Grass damage", "Official proof you went outside."),
    item("Self-Help Book", Healthy, 1, &[Mod::Stat(MentalHealth, 10.0), Mod::Stat(SelfCare, 1.0)], "+10 Mental Health, +1 Self-Care", "Chapter one: stop reading self-help books."),
    item("Coffee", Risky, 1, &[Mod::Stat(Focus, 15.0), Mod::Coffee], "+15% Focus. An Anxiety stack every 20 s", "Third cup. Totally fine."),
    item("Energy Drink", Risky, 2, &[Mod::Stat(Speed, 20.0), Mod::Stat(Focus, 10.0), Mod::Stat(MentalHealth, -15.0)], "+20% Speed, +10% Focus, -15 Mental Health", "Tastes like a deadline."),
    item("Doomscrolling", Risky, 2, &[Mod::LikeBonus(0.5), Mod::Doomscrolling], "+50% Likes. -0.5 Mental Health per second while slop is on screen", "Just one more."),
    item("Hot Take", Risky, 2, &[Mod::TypeBonus(DamageType::Sarcasm, 0.25), Mod::HotTake], "+25% Sarcasm damage. Hits on you are 10% harder", "Ratio incoming."),
    item("Dumbphone", Risky, 1, &[Mod::Stat(Scepticism, 15.0), Mod::Dumbphone, Mod::Stat(Curiosity, -40.0)], "+15% Scepticism, immune to Doomscroll. -40 Curiosity", "Snake still slaps."),
    item("Burner Account", Risky, 2, &[Mod::FreeReroll, Mod::FreeReroll, Mod::LearnFaster(0.25)], "Two more free rerolls on level-ups. The slop learns 25% faster", "Nobody will ever know."),
    item("Clickbait Thumbnail", Risky, 2, &[Mod::LikeBonus(0.3), Mod::ShopPriceUp(0.1)], "+30% Likes. Machines cost 10% more", "YOU WON'T BELIEVE THIS ITEM."),
    item("Side Hustle", Hustle, 1, &[Mod::WaveIncome(6)], "+6 Clout every minute", "Passive income, allegedly."),
    item("Savings Account", Hustle, 2, &[Mod::Interest(0.10)], "Every minute, 10% interest on your Clout, up to 20", "Boring. Works."),
    item("Referral Link", Hustle, 1, &[Mod::LikeBonus(0.1)], "+10% Likes", "Use my code."),
    item("Coupon Clipper", Hustle, 2, &[Mod::ShopDiscount(0.1)], "Machines cost 10% less", "Ten cents off."),
    item("Garage Sale", Hustle, 1, &[Mod::CloutBonus(0.3), Mod::ExtraReroll(1)], "+30% Clout from coins, +1 free reroll on level-ups", "Everything must go."),
    item("Influencer Deal", Hustle, 2, &[Mod::InfluencerDeal, Mod::Stat(Conviction, -5.0)], "Every minute: +1 Clout per minute survived. -5% Conviction", "Brought to you by nobody."),
    item("Crypto Wallet", Hustle, 2, &[Mod::CryptoWallet], "Every minute your Clout changes by -10% to +25%", "To the moon. Or not."),
    item("Log Off", Act, 2, &[Mod::Active(Active::LogOff)], "With 8 slop close: dash away, invulnerable. Every 12 s", "Bye."),
    item("Block Button", Act, 2, &[Mod::Active(Active::BlockButton)], "Deletes a bubble about to hit you, and the next one. Every 4 s", "Blocked and reported."),
    item("Digital Detox", Act, 2, &[Mod::Active(Active::DigitalDetox)], "With 2 statuses: clear them and push slop back. Every 20 s", "Phone in the drawer."),
    item("Unsubscribe", Act, 3, &[Mod::Active(Active::Unsubscribe)], "Below 30% Mental Health: all slop on screen lose 10% HP and stall. Every 45 s", "You will not be missed."),
    item("Phone a Friend", Act, 3, &[Mod::Active(Active::PhoneAFriend)], "An elite or boss near: a friend runs through it for 40. Every 25 s", "Real friends. Remember those?"),
    // Added with Boundaries (DEF-01).
    item("Healthy Boundaries", Defence, 1, &[Mod::Stat(Boundaries, 15.0)], "+15 Boundaries: a shield that takes hits first and comes back", "No is a complete sentence."),
    item("Bulletproof Hoodie", Defence, 2, &[Mod::Stat(Composure, 4.0), Mod::Stat(MentalHealth, 15.0)], "+4 Composure, +15 Mental Health", "Hood up. World off."),
    item("Privacy Settings", Defence, 2, &[Mod::ShotArmor(0.2)], "Shots hurt you 20% less", "Friends only. No, not those friends."),
    item("Spiky Opinions", Defence, 2, &[Mod::Thorns(8.0)], "Slop touching you take 8 Sarcasm damage three times a second", "Touch me and find out."),
    item("Do Not Disturb", Defence, 3, &[Mod::DoNotDisturb], "Every 8 s: shove all slop near you away, and 1 s untouchable", "Notifications silenced. Slop silenced."),
    item("Therapist on Speed Dial", Defence, 2, &[Mod::Therapist(25.0)], "Below half Mental Health: heal 25. Every 30 s", "And how does that make you feel?"),
    item("Emotional Support Water Bottle", Defence, 1, &[Mod::Stat(SelfCare, 1.0), Mod::Stat(Boundaries, 8.0)], "+1 Self-Care, +8 Boundaries", "Covered in stickers. Hydrated. Unstoppable."),
    // Dashes (DASH-02).
    item("Speedrun Shoes", Movement, 2, &[Mod::DashCharges(1), Mod::Stat(Speed, 5.0)], "+1 dash charge, +5% Speed", "Any% Mental Health."),
    item("Swipe Left", Movement, 2, &[Mod::DashDamage(25.0)], "Dashing through slop hits it for 25 Analog", "Not interested. Next."),
    item("Hot Take Spill", Movement, 2, &[Mod::DashFire], "Dashes leave a trail of burning coffee that hurts slop", "Careful, it's hot. That's the point."),
    item("Left on Read", Movement, 2, &[Mod::DashShock], "The end of every dash confuses and shoves the slop around you", "Seen 3:14 a.m."),
    item("Ghosting", Movement, 2, &[Mod::Ghost(0.5)], "0.5 s untouchable after every dash", "They'll get the message. Eventually."),
    item("Cold Brew", Movement, 1, &[Mod::DashRecharge(-0.4), Mod::Stat(Focus, 5.0)], "Dash charges come back 0.4 s faster, +5% Focus", "Steeped for 20 hours. Like your screen time."),
    item("Flame War", Risky, 3, &[Mod::FlameTrail(5.0), Mod::HotTake], "You leave a burning trail that hurts slop. Hits on you are 10% harder", "Everyone is mad. Everything is on fire."),
    // Hustle (STAT-02).
    item("Monetised Channel", Hustle, 1, &[Mod::Stat(StatKind::Hustle, 10.0)], "+10 Hustle: Clout every minute and more coins", "Like and subscribe. Please."),
    item("Affiliate Links", Hustle, 2, &[Mod::Stat(StatKind::Hustle, 15.0), Mod::Stat(Luck, 3.0)], "+15 Hustle, +3 Luck", "As an affiliate I earn from qualifying slop."),
    // Legendaries (ITEM-03): rare, and they break the rules.
    item("Infinite Scroll", Legendary, 4, &[Mod::ExtraShots(2)], "Every weapon fires 2 more projectiles", "There is no bottom. There never was."),
    item("Viral Moment", Legendary, 4, &[Mod::KillBlast(0.3)], "Deleted slop explode for 30% of their HP. Explosions set off more", "Your post is blowing up. Literally."),
    item("Alt Account", Legendary, 4, &[Mod::AltAccount(0.6)], "A second you circles nearby and fires all your weapons at 60% damage", "Same opinions, different profile picture."),
    item("Golden Ring Light", Legendary, 4, &[Mod::RingLight], "Every 25 s: a Mass Report deletes everything on screen", "You are always camera-ready now."),
    item("God Mode Glitch", Legendary, 4, &[Mod::GodMode], "Every 12 s: 2 s untouchable, +50% damage, dashes refilled", "Found it in an old forum post. Works."),
    item("Ratio Machine", Legendary, 4, &[Mod::ChainChance(0.15)], "15% of your hits jump on to 3 more slop", "Ratio + L + ratio + L + ratio."),
    item("The Algorithm's Favourite", Legendary, 4, &[Mod::XpBonus(0.6), Mod::ExtraOffer, Mod::Stat(Luck, 10.0)], "+60% experience, one more choice on every level-up, +10 Luck", "For You. Only you. Always you."),
    item("Dragon Energy Drink", Legendary, 4, &[Mod::FlameTrail(12.0), Mod::DashFire, Mod::Stat(Speed, 25.0)], "+25% Speed. You leave a trail of fire, and so do your dashes", "Not approved in any country."),
];

impl ItemDef {
    /// How many levels the item has: effects that are on or off have one,
    /// and so do legendaries.
    pub fn max_level(&self) -> u32 {
        let unique = self.kind == ItemKind::Legendary
            || self.mods.iter().any(|m| {
                matches!(
                    m,
                    Mod::Active(_) | Mod::AdBlocker | Mod::NoiseCancel | Mod::Dumbphone | Mod::TherapyDog | Mod::FactWeapons | Mod::Fluent | Mod::ChainLetter | Mod::SecondWind | Mod::GroupChat | Mod::DashFire | Mod::DashShock | Mod::DoNotDisturb
                )
            });
        if unique {
            1
        } else {
            5
        }
    }
}

/// Shrine blessings (MAP-03): pick one of three; they last the whole run.
pub const BLESSINGS: &[ItemDef] = &[
    item("Blessing of Virality", Healthy, 3, &[Mod::XpBonus(0.25)], "+25% experience from Likes", "Your posts are doing numbers."),
    item("Blessing of Engagement", Healthy, 3, &[Mod::Stat(Conviction, 20.0)], "+20% Conviction", "Everyone is reacting."),
    item("Blessing of the Algorithm", Healthy, 3, &[Mod::ExtraOffer], "One more choice on every level-up", "It finally recommends you."),
    item("Blessing of Moderation", Healthy, 3, &[Mod::Stat(Composure, 8.0), Mod::Stat(SelfCare, 2.0)], "+8 Composure, +2 Self-Care", "The mods are on your side."),
    item("Blessing of Reach", Healthy, 3, &[Mod::Stat(Reach, 25.0), Mod::Stat(Curiosity, 60.0)], "+25% Reach, +60 Curiosity", "Seen by everyone."),
    item("Blessing of Clout", Hustle, 3, &[Mod::CloutBonus(0.6), Mod::WaveIncome(5)], "+60% Clout from coins, +5 Clout every minute", "Monetised."),
    item("Blessing of Haste", Healthy, 3, &[Mod::Stat(Focus, 18.0), Mod::Stat(Speed, 8.0)], "+18% Focus, +8% Speed", "Speedrun any%."),
    item("Blessing of Thick Skin", Healthy, 3, &[Mod::Stat(MentalHealth, 60.0), Mod::Stat(Scepticism, 8.0)], "+60 Mental Health, +8% Scepticism", "Comments don't reach you."),
    item("Blessing of the Dash", Movement, 3, &[Mod::DashCharges(1), Mod::DashRecharge(-0.5)], "+1 dash charge, charges come back 0.5 s faster", "Gotta go fast."),
    item("Blessing of Boundaries", Defence, 3, &[Mod::Stat(Boundaries, 40.0)], "+40 Boundaries", "Protected. Finally."),
];

/// Glitches (GLITCH-01): rare perks that change how a run plays. One shows
/// up now and then on a level-up, and every boss leaves a pick of three.
pub const GLITCHES: &[ItemDef] = &[
    item("Glass Cannon", Risky, 4, &[Mod::Stat(Conviction, 60.0), Mod::Stat(MentalHealth, -40.0)], "+60% Conviction, -40 Mental Health", "One hit wonder. Both ways."),
    item("Ricochet Brain", Healthy, 4, &[Mod::ExtraBounces(2)], "Every projectile bounces on to 2 more slop", "Thoughts bounce around in there."),
    item("Speed Demon", Movement, 4, &[Mod::SpeedDamage, Mod::Stat(Speed, 15.0)], "+15% Speed, and your damage is multiplied by your Speed", "Gotta go fast. Gotta hit hard."),
    item("Hoarder", Hustle, 4, &[Mod::CloutDamage(0.6), Mod::Stat(StatKind::Hustle, 10.0)], "+1% Conviction for every 10 Clout you hold (up to +60%), +10 Hustle", "Never spend. Never lose."),
    item("Vampire Feed", Healthy, 4, &[Mod::Lifesteal(0.15)], "Heal 0.15 Mental Health for every slop deleted, up to 6 a second", "Your energy is my energy."),
    item("Echo Chamber", Healthy, 4, &[Mod::Echo], "Every attack happens again a moment later at half damage", "Everyone agrees with you. Twice."),
    item("Big Brain", Healthy, 4, &[Mod::Stat(Reach, 40.0), Mod::Stat(Curiosity, 80.0)], "+40% Reach, +80 Curiosity", "Galaxy brain. Literally bigger."),
    item("Cursed Feed", Risky, 4, &[Mod::Curse(0.5)], "50% more slop. 50% more Likes and Clout from it", "The algorithm noticed you. Uh oh."),
    item("Doom Dash", Movement, 4, &[Mod::DashCharges(2), Mod::DashDamage(40.0), Mod::DashRecharge(-0.6)], "+2 dash charges that come back faster; dashes hit slop for 40", "Dash first. Think never."),
    item("Overclocked", Risky, 4, &[Mod::Stat(Focus, 50.0), Mod::Stat(MentalHealth, -25.0)], "+50% Focus, -25 Mental Health", "Thermal throttling is a mindset."),
    item("Thick Skull", Defence, 4, &[Mod::Stat(Boundaries, 60.0), Mod::Stat(Speed, -10.0)], "+60 Boundaries, -10% Speed", "Nothing gets through. Including ideas."),
    item("Plot Armour", Defence, 4, &[Mod::SecondWind, Mod::SecondWind], "Twice per platform, survive a killing blow", "Main characters don't die in episode one."),
    item("Touch Grass Forever", Healthy, 4, &[Mod::Stat(SelfCare, 8.0), Mod::TypeBonus(DamageType::Grass, 0.25)], "+8 Self-Care, +25% Grass damage", "You live outside now."),
];

/// The bad half of a risky item: it counts once, whatever the item's level.
pub fn is_drawback(m: &Mod) -> bool {
    match m {
        Mod::Stat(_, v) => *v < 0.0,
        Mod::Coffee | Mod::Doomscrolling | Mod::HotTake | Mod::ShopPriceUp(_) | Mod::LearnFaster(_) => true,
        _ => false,
    }
}

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
        Mod::XpBonus(v) => p.like_bonus += v,
        Mod::Curse(v) => p.like_bonus += v,
        Mod::Active(a) => {
            if !p.actives.contains(&a) {
                p.actives.push(a);
            }
        }
        Mod::DashCharges(n) => p.dash_charges += n,
        Mod::DashRecharge(s) => p.dash_recharge += s,
        Mod::DashDamage(v) => p.dash_damage += v,
        Mod::DashFire => p.dash_fire = true,
        Mod::DashShock => p.dash_shock = true,
        Mod::Ghost(s) => p.ghost += s,
        Mod::FlameTrail(v) => p.flame_trail += v,
        Mod::Thorns(v) => p.thorns += v,
        Mod::ShotArmor(v) => p.shot_armor += v,
        Mod::DoNotDisturb => p.do_not_disturb = true,
        Mod::Therapist(v) => p.therapist += v,
        Mod::ExtraShots(n) => p.extra_shots += n,
        Mod::KillBlast(v) => p.kill_blast += v,
        Mod::AltAccount(v) => p.alt_account = p.alt_account.max(v),
        Mod::RingLight => p.ring_light = true,
        Mod::GodMode => p.god_mode = true,
        Mod::ChainChance(v) => p.chain_chance += v,
        Mod::ExtraBounces(n) => p.extra_bounces += n,
        Mod::Lifesteal(v) => p.lifesteal += v,
        Mod::Echo => p.echo = true,
        Mod::SpeedDamage => p.speed_damage = true,
        Mod::EliteChests => p.elite_chests += 1,
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: ITEM-01, ITEM-03
    #[test]
    fn items_cover_every_kind_and_describe_themselves() {
        assert!(ITEMS.len() >= 50);
        for kind in [Healthy, Risky, Hustle, Act, Movement, Defence, Legendary] {
            assert!(ITEMS.iter().filter(|i| i.kind == kind).count() >= 5, "{kind:?}");
        }
        for i in ITEMS {
            assert!(!i.mods.is_empty() && !i.effect.is_empty() && (1..=4).contains(&i.tier), "{}", i.name);
            assert_eq!(i.kind == Legendary, i.tier == 4, "{}: legendaries and only they are tier 4", i.name);
        }
        // Risky items carry a drawback in their effect text or a second mod.
        for i in ITEMS.iter().filter(|i| i.kind == Risky) {
            assert!(i.mods.len() >= 2, "{}", i.name);
        }
        assert!(GLITCHES.len() >= 10);
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
