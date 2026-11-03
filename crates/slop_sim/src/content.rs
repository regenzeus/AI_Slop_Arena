//! Weapons, slop, brainrot parts, bosses and platforms, as data (the
//! "content as data" rule): balancing changes these tables, never the logic.

use crate::stats::DamageType;

/// Damage multiplier at weapon levels 1 to 8 (WPN-10).
pub const LEVEL_DAMAGE: [f32; 8] = [1.0, 1.25, 1.5, 1.8, 2.1, 2.45, 2.8, 3.3];

/// What a level adds besides damage: extra projectiles, area, and the
/// cooldown multiplier.
pub fn level_extra(level: u8) -> (u8, f32, f32) {
    let shots = (level >= 3) as u8 + (level >= 8) as u8;
    let area = if level >= 5 { 1.25 } else { 1.0 };
    let haste = if level >= 7 { 0.8 } else { 1.0 };
    (shots, area, haste)
}

/// What reaching `level` adds, for the level-up card.
pub fn level_note(level: u8) -> &'static str {
    match level {
        3 => "+1 projectile (or +1 bounce, +1 link)",
        5 => "+25% area and range",
        7 => "Attacks 25% faster",
        8 => "+1 more projectile. Max level: ready to merge",
        _ => "More damage",
    }
}

/// How a projectile is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Look {
    Rock,
    Bullet,
    Pen,
    EyeRoll,
    Bubble,
    EmDash,
    Book,
    Brick,
    Staple,
    Frisbee,
    Captcha,
    Toast,
    Arrow,
    /// A chain letter crossing the FaceSlop arena.
    Letter,
    /// A thought-leadership post: pushes you back.
    Post,
    Duck,
    Plane,
    Note,
}

impl Look {
    pub const ALL: [Look; 18] = [
        Look::Rock,
        Look::Bullet,
        Look::Pen,
        Look::EyeRoll,
        Look::Bubble,
        Look::EmDash,
        Look::Book,
        Look::Brick,
        Look::Staple,
        Look::Frisbee,
        Look::Captcha,
        Look::Toast,
        Look::Arrow,
        Look::Letter,
        Look::Post,
        Look::Duck,
        Look::Plane,
        Look::Note,
    ];
}

/// How a weapon attacks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Pattern {
    /// A projectile at the nearest slop.
    Shot { speed: f32, radius: f32, pierce: u8, look: Look },
    /// Several projectiles in a narrow fan.
    Burst { speed: f32, radius: f32, count: u8, spread: f32, look: Look },
    /// Thrown at the target's position; splashes where it lands.
    Lob { speed: f32, splash: f32, look: Look },
    /// A projectile that marks what it hits: marked slop takes `bonus` more
    /// damage from everything for `seconds`.
    Mark { speed: f32, radius: f32, pierce: u8, bonus: f32, seconds: f32 },
    /// A projectile that bounces on to the nearest other slop.
    Ricochet { speed: f32, radius: f32, bounces: u8 },
    /// Flies out, slows, and comes back to you, hitting on both ways.
    Boomerang { speed: f32, radius: f32 },
    /// A melee swing within `half_angle` radians of the nearest slop.
    Arc { half_angle: f32 },
    /// A short, wide spray at the nearest slop that pushes it back.
    Cone { half_angle: f32 },
    /// Hits everything around you.
    Ring,
    /// Pulses around you, faster the more slop is near.
    Aura,
    /// Hits the nearest slop and jumps on to `links` more.
    Chain { links: u8 },
    /// An instant line through the nearest slop; strips beauty filters.
    Beam { width: f32 },
    /// Turns the nearest speech bubble around at the slop.
    Reflect,
    /// Something circling you that bites what it touches.
    Orbit { radius: f32 },
    /// Plants a patch at the nearest slop that hurts slop and heals you inside.
    Zone { radius: f32, seconds: f32, heal: f32 },
    /// A slam at the nearest slop; small slop is deleted outright.
    Slam { radius: f32, execute: f32 },
    /// A projectile that stuns chatbots.
    Stun { speed: f32, radius: f32, seconds: f32 },
}

pub struct WeaponDef {
    pub name: &'static str,
    pub kind: DamageType,
    pub pattern: Pattern,
    /// Damage per hit at tier I.
    pub damage: f32,
    /// Seconds between attacks.
    pub cooldown: f32,
    /// Range in world units.
    pub range: f32,
    /// Push given to slop on hit, in units per second.
    pub knockback: f32,
    /// Rare weapons only appear in the shop from wave 6.
    pub rare: bool,
    /// Made by merging two weapons; never offered on a level-up.
    pub evolved: bool,
    pub flavour: &'static str,
}

#[allow(clippy::too_many_arguments)]
const fn weapon(
    name: &'static str,
    kind: DamageType,
    pattern: Pattern,
    damage: f32,
    cooldown: f32,
    range: f32,
    knockback: f32,
    flavour: &'static str,
) -> WeaponDef {
    WeaponDef { name, kind, pattern, damage, cooldown, range, knockback, rare: false, evolved: false, flavour }
}

#[allow(clippy::too_many_arguments)]
const fn evolved(
    name: &'static str,
    kind: DamageType,
    pattern: Pattern,
    damage: f32,
    cooldown: f32,
    range: f32,
    knockback: f32,
    flavour: &'static str,
) -> WeaponDef {
    WeaponDef { name, kind, pattern, damage, cooldown, range, knockback, rare: false, evolved: true, flavour }
}

use DamageType::{Analog, Fact, Grass, Sarcasm};

/// Every weapon of the design (WPN-01), grouped by damage type.
pub const WEAPONS: &[WeaponDef] = &[
    weapon("Rock", Analog, Pattern::Shot { speed: 520.0, radius: 9.0, pierce: 0, look: Look::Rock }, 9.0, 1.0, 380.0, 160.0, "Older than the internet."),
    weapon("Stick", Analog, Pattern::Arc { half_angle: 1.05 }, 10.0, 0.9, 115.0, 260.0, "Peak technology."),
    weapon("Slingshot", Analog, Pattern::Shot { speed: 720.0, radius: 8.0, pierce: 0, look: Look::Rock }, 13.0, 1.3, 560.0, 340.0, "David had one."),
    weapon("Brick", Analog, Pattern::Lob { speed: 430.0, splash: 75.0, look: Look::Brick }, 15.0, 1.8, 420.0, 220.0, "A brick never hallucinates."),
    weapon("Baseball Bat", Analog, Pattern::Arc { half_angle: 1.2 }, 18.0, 1.4, 125.0, 520.0, "Swing for the fences."),
    weapon("Pistol", Analog, Pattern::Shot { speed: 900.0, radius: 5.0, pierce: 0, look: Look::Bullet }, 6.0, 0.55, 520.0, 60.0, "The classic."),
    weapon("Stapler", Analog, Pattern::Burst { speed: 800.0, radius: 5.0, count: 3, spread: 0.16, look: Look::Staple }, 4.0, 0.9, 400.0, 30.0, "Attaches enemies to consequences."),
    weapon("Mechanical Keyboard", Analog, Pattern::Ring, 5.0, 0.6, 95.0, 140.0, "Blue switches. Everyone hears it."),
    weapon("Red Pen", Fact, Pattern::Mark { speed: 700.0, radius: 7.0, pierce: 1, bonus: 0.2, seconds: 4.0 }, 8.0, 1.0, 480.0, 0.0, "[citation needed]"),
    weapon("Library Book", Fact, Pattern::Shot { speed: 480.0, radius: 12.0, pierce: 99, look: Look::Book }, 13.0, 1.7, 500.0, 140.0, "Due back Tuesday."),
    weapon("Newspaper Roll", Fact, Pattern::Arc { half_angle: 0.8 }, 12.0, 1.0, 125.0, 200.0, "Read a real one."),
    weapon("Fact-Check Flashlight", Fact, Pattern::Beam { width: 16.0 }, 8.0, 1.1, 520.0, 0.0, "Shines light on things."),
    weapon("Eye-Roll", Sarcasm, Pattern::Ricochet { speed: 600.0, radius: 10.0, bounces: 3 }, 6.0, 1.3, 420.0, 40.0, "Wow. Great."),
    weapon("Air Quotes", Sarcasm, Pattern::Chain { links: 4 }, 7.0, 1.2, 420.0, 0.0, "\"Intelligence.\""),
    weapon("Slow Clap", Sarcasm, Pattern::Aura, 4.0, 1.0, 135.0, 90.0, "Clap. Clap. Clap."),
    weapon("Uno Reverse Card", Sarcasm, Pattern::Reflect, 12.0, 1.6, 320.0, 0.0, "No u."),
    weapon("Garden Hose", Grass, Pattern::Cone { half_angle: 0.45 }, 3.0, 0.25, 210.0, 160.0, "Off my lawn."),
    weapon("Frisbee", Grass, Pattern::Boomerang { speed: 560.0, radius: 12.0 }, 9.0, 1.4, 380.0, 90.0, "Fetch."),
    weapon("Dog Walk", Grass, Pattern::Orbit { radius: 70.0 }, 7.0, 0.45, 70.0, 150.0, "Who is a good boy."),
    weapon("Seed Bomb", Grass, Pattern::Zone { radius: 75.0, seconds: 4.0, heal: 1.0 }, 5.0, 3.0, 380.0, 0.0, "Literally touching grass."),
    WeaponDef {
        name: "Ban Hammer",
        kind: Fact,
        pattern: Pattern::Slam { radius: 95.0, execute: 30.0 },
        damage: 30.0,
        cooldown: 2.6,
        range: 300.0,
        knockback: 320.0,
        rare: true,
        evolved: false,
        flavour: "Rule 1: no slop.",
    },
    WeaponDef {
        name: "CAPTCHA Turret",
        kind: Fact,
        pattern: Pattern::Stun { speed: 650.0, radius: 9.0, seconds: 1.5 },
        damage: 6.0,
        cooldown: 0.9,
        range: 460.0,
        knockback: 0.0,
        rare: true,
        evolved: false,
        flavour: "Select all squares with bots.",
    },
    // Added with the endless Feed (WPN-11).
    weapon("Rubber Duck", Analog, Pattern::Lob { speed: 400.0, splash: 65.0, look: Look::Duck }, 12.0, 1.3, 400.0, 200.0, "Explain your bug to it. Then throw it."),
    weapon("Office Memo", Fact, Pattern::Shot { speed: 620.0, radius: 9.0, pierce: 2, look: Look::Plane }, 8.0, 0.95, 480.0, 60.0, "Per my last memo."),
    weapon("Sticky Notes", Fact, Pattern::Burst { speed: 560.0, radius: 7.0, count: 4, spread: 0.3, look: Look::Note }, 4.0, 1.0, 380.0, 40.0, "Reminder: the slop is watching."),
    weapon("Side-Eye", Sarcasm, Pattern::Beam { width: 10.0 }, 9.0, 1.3, 430.0, 0.0, "Oh. Okay then."),
    weapon("Ratio", Sarcasm, Pattern::Slam { radius: 80.0, execute: 14.0 }, 14.0, 2.0, 300.0, 260.0, "Ratio + L + touch grass."),
    weapon("Sunflower Patch", Grass, Pattern::Zone { radius: 95.0, seconds: 5.0, heal: 1.5 }, 6.0, 3.4, 360.0, 0.0, "Faces the sun. Not your feed."),
    weapon("Leaf Blower", Grass, Pattern::Cone { half_angle: 0.38 }, 4.0, 0.3, 240.0, 320.0, "Loud. Effective. Judged by neighbours."),
    // Merged weapons (WPN-12): only made by `MERGES`, never offered.
    evolved("Meteor Shower", Analog, Pattern::Burst { speed: 700.0, radius: 12.0, count: 7, spread: 0.32, look: Look::Rock }, 22.0, 0.9, 560.0, 360.0, "Rocks, but from space."),
    evolved("Home Run", Analog, Pattern::Arc { half_angle: 2.6 }, 48.0, 1.1, 160.0, 900.0, "Out of the park. Out of the feed."),
    evolved("Mass Mailer", Analog, Pattern::Burst { speed: 980.0, radius: 6.0, count: 9, spread: 0.12, look: Look::Staple }, 9.0, 0.35, 560.0, 60.0, "Reply-all, but violent."),
    evolved("Rage Quit", Analog, Pattern::Ring, 30.0, 0.55, 190.0, 420.0, "Alt+F4 for everyone nearby."),
    evolved("Peer Review", Fact, Pattern::Mark { speed: 820.0, radius: 11.0, pierce: 99, bonus: 0.5, seconds: 6.0 }, 18.0, 0.8, 620.0, 80.0, "Reviewer 2 has notes."),
    evolved("Breaking News", Fact, Pattern::Beam { width: 42.0 }, 30.0, 0.8, 760.0, 0.0, "This just in: you're deleted."),
    evolved("Permaban", Fact, Pattern::Slam { radius: 170.0, execute: 140.0 }, 70.0, 1.6, 420.0, 520.0, "Banned. Forever. No appeal."),
    evolved("Infinite Uno", Sarcasm, Pattern::Ricochet { speed: 720.0, radius: 14.0, bounces: 14 }, 16.0, 0.9, 520.0, 60.0, "No u. No u. No u. No u."),
    evolved("Standing Ovation", Sarcasm, Pattern::Chain { links: 14 }, 16.0, 0.7, 520.0, 0.0, "Bravo. Bravo. Bravo."),
    evolved("Main Character", Sarcasm, Pattern::Aura, 14.0, 0.5, 230.0, 220.0, "The plot revolves around you now."),
    evolved("Rewilding", Grass, Pattern::Zone { radius: 150.0, seconds: 6.0, heal: 3.0 }, 16.0, 1.6, 480.0, 0.0, "The parking lot is a meadow now."),
    evolved("Dog Park", Grass, Pattern::Orbit { radius: 115.0 }, 26.0, 0.25, 115.0, 300.0, "Every dog. All at once."),
    evolved("Hurricane Season", Grass, Pattern::Cone { half_angle: 0.85 }, 9.0, 0.18, 330.0, 520.0, "Mother Nature hit reply-all."),
];

/// Merge recipes (WPN-12): two weapons, both at max level, become one
/// stronger weapon when you open a chest.
pub const MERGES: &[(&str, &str, &str)] = &[
    ("Rock", "Slingshot", "Meteor Shower"),
    ("Stick", "Baseball Bat", "Home Run"),
    ("Pistol", "Stapler", "Mass Mailer"),
    ("Brick", "Mechanical Keyboard", "Rage Quit"),
    ("Red Pen", "Library Book", "Peer Review"),
    ("Newspaper Roll", "Fact-Check Flashlight", "Breaking News"),
    ("Ban Hammer", "CAPTCHA Turret", "Permaban"),
    ("Eye-Roll", "Uno Reverse Card", "Infinite Uno"),
    ("Air Quotes", "Slow Clap", "Standing Ovation"),
    ("Side-Eye", "Ratio", "Main Character"),
    ("Garden Hose", "Seed Bomb", "Rewilding"),
    ("Frisbee", "Dog Walk", "Dog Park"),
    ("Leaf Blower", "Sunflower Patch", "Hurricane Season"),
];

/// What `def` merges with, and into, by weapon index.
pub fn merge_of(def: usize) -> Option<(usize, usize)> {
    let name = WEAPONS.get(def)?.name;
    MERGES.iter().find_map(|&(a, b, into)| {
        let other = if a == name { b } else if b == name { a } else { return None };
        Some((weapon_index(other)?, weapon_index(into)?))
    })
}

pub fn weapon_index(name: &str) -> Option<usize> {
    WEAPONS.iter().position(|w| w.name == name)
}

/// What a slop does in a wave.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// Rushes you in numbers.
    Swarmer,
    /// Keeps its distance and fires.
    Shooter,
    /// Buffs or heals other slop; kill it first.
    Support,
    /// Slow and hard to kill.
    Tank,
    /// Breaks a rule of the arena.
    Special,
}

/// The fixed slop characters: chatbots on every platform, and each
/// platform's own slop (SLOP-01).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Archetype {
    ChadGpt,
    Clawed,
    YesBot,
    Hallucinator,
    ListicleDrone,
    EngagementBait,
    GrandmaCake,
    SixFingerHand,
    ThoughtLeader,
    RecruiterBot,
    ClickbaitFace,
    SubscribePopup,
    RepostBot,
    KarmaFarmer,
    FilterInfluencer,
    /// Hatches into Listicle Drones (Content Farm boss).
    SurpriseEgg,
}

impl Archetype {
    pub const ALL: [Archetype; 16] = [
        Archetype::ChadGpt,
        Archetype::Clawed,
        Archetype::YesBot,
        Archetype::Hallucinator,
        Archetype::ListicleDrone,
        Archetype::EngagementBait,
        Archetype::GrandmaCake,
        Archetype::SixFingerHand,
        Archetype::ThoughtLeader,
        Archetype::RecruiterBot,
        Archetype::ClickbaitFace,
        Archetype::SubscribePopup,
        Archetype::RepostBot,
        Archetype::KarmaFarmer,
        Archetype::FilterInfluencer,
        Archetype::SurpriseEgg,
    ];

    /// Chatbots share one trained base model and are stunned by CAPTCHAs.
    pub fn is_chatbot(self) -> bool {
        matches!(self, Archetype::ChadGpt | Archetype::Clawed | Archetype::YesBot | Archetype::Hallucinator | Archetype::ListicleDrone)
    }

    pub fn def(self) -> &'static ArchetypeDef {
        &ARCHETYPES[self as usize]
    }
}

pub struct ArchetypeDef {
    pub name: &'static str,
    pub role: Role,
    pub hp: f32,
    pub speed: f32,
    pub radius: f32,
    pub contact: f32,
    pub likes: u32,
    pub blurb: &'static str,
}

#[allow(clippy::too_many_arguments)]
const fn arch(name: &'static str, role: Role, hp: f32, speed: f32, radius: f32, contact: f32, likes: u32, blurb: &'static str) -> ArchetypeDef {
    ArchetypeDef { name, role, hp, speed, radius, contact, likes, blurb }
}

/// In `Archetype` order.
const ARCHETYPES: [ArchetypeDef; 16] = [
    arch("ChadGPT", Role::Shooter, 26.0, 85.0, 20.0, 5.0, 2, "Keeps its distance and fires fans of speech bubbles."),
    arch("Clawed", Role::Shooter, 34.0, 95.0, 20.0, 6.0, 2, "Aims, then throws a long em-dash. It's not a lance, it's a journey."),
    arch("Yes-Bot", Role::Support, 24.0, 105.0, 18.0, 4.0, 3, "Agrees with nearby slop. Each agreement gives them 10% more damage."),
    arch("Hallucinator", Role::Special, 22.0, 95.0, 18.0, 5.0, 2, "Drops fake Likes that explode into Anxiety. A glitch gives them away."),
    arch("Listicle Drone", Role::Swarmer, 11.0, 140.0, 13.0, 5.0, 1, "Packs of five. Kill number 1 first and the rest scatter."),
    arch("Engagement Bait", Role::Support, 45.0, 0.0, 22.0, 4.0, 3, "\"Type AMEN.\" Stands still and buffs slop around it until killed."),
    arch("Grandma's 103rd Birthday Cake", Role::Support, 40.0, 60.0, 20.0, 4.0, 3, "Heals slop around it. Nobody has liked it."),
    arch("Six-Finger Hand", Role::Swarmer, 14.0, 175.0, 15.0, 0.0, 1, "Grabs you and slows you down while it holds on."),
    arch("Thought Leader", Role::Shooter, 38.0, 80.0, 21.0, 6.0, 3, "Posts \"I'm humbled to announce...\" walls that push you back."),
    arch("Recruiter Bot", Role::Swarmer, 20.0, 125.0, 17.0, 5.0, 1, "Splits in two the first time it is hit. \"I came across your profile.\""),
    arch("Clickbait Face", Role::Shooter, 30.0, 85.0, 21.0, 5.0, 2, "A shocked face that fires big red arrows. Arrows drag you forward."),
    arch("Subscribe Pop-up", Role::Tank, 120.0, 45.0, 28.0, 8.0, 4, "A box that blocks your way. Touching it opens a pop-up."),
    arch("Repost Bot", Role::Swarmer, 18.0, 135.0, 16.0, 5.0, 1, "Comes back once after dying, labelled [repost]."),
    arch("Karma Farmer", Role::Support, 34.0, 100.0, 19.0, 6.0, 3, "Grows with every kill nearby. Its hits Ratio you."),
    arch("Filter Influencer", Role::Tank, 70.0, 75.0, 22.0, 7.0, 4, "A beauty filter blocks damage from the front. Flank it, or fact-check it."),
    arch("Surprise Egg", Role::Special, 20.0, 0.0, 16.0, 0.0, 1, "Hatches into Listicle Drones after three seconds."),
];

/// Brainrot bodies set how it moves (BRAIN-01).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Body {
    Shark,
    Crocodile,
    Frog,
    Pigeon,
    Elephant,
    Cat,
    Shrimp,
    Capybara,
    Octopus,
    Goose,
}

/// Brainrot objects set how it attacks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Object {
    Airplane,
    Toaster,
    Espresso,
    Cactus,
    WashingMachine,
    Baguette,
    TrafficCone,
    Smartphone,
    Megaphone,
    Dumbbell,
}

/// Brainrot accessories add one modifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Accessory {
    Sunglasses,
    Sneakers,
    Crown,
    ChefHat,
    Headphones,
    GoldChain,
    PartyHat,
    Halo,
    ClownNose,
    Lanyard,
}

impl Body {
    pub const ALL: [Body; 10] = [Body::Shark, Body::Crocodile, Body::Frog, Body::Pigeon, Body::Elephant, Body::Cat, Body::Shrimp, Body::Capybara, Body::Octopus, Body::Goose];

    pub fn name(self) -> &'static str {
        ["Shark", "Crocodile", "Frog", "Pigeon", "Elephant", "Cat", "Shrimp", "Capybara", "Octopus", "Goose"][self as usize]
    }

    /// Italian-sounding stem for names.
    pub fn stem(self) -> &'static str {
        ["Squal", "Coccodrill", "Ranocchi", "Piccion", "Elefant", "Gatt", "Gamber", "Capibar", "Polp", "Ocarell"][self as usize]
    }

    /// Hit points, speed and radius of this body before genes.
    pub fn base(self) -> (f32, f32, f32) {
        match self {
            Body::Shark => (20.0, 165.0, 17.0),
            Body::Crocodile => (34.0, 110.0, 19.0),
            Body::Frog => (22.0, 130.0, 16.0),
            Body::Pigeon => (16.0, 150.0, 15.0),
            Body::Elephant => (80.0, 60.0, 27.0),
            Body::Cat => (22.0, 140.0, 16.0),
            Body::Shrimp => (9.0, 150.0, 12.0),
            Body::Capybara => (60.0, 70.0, 24.0),
            Body::Octopus => (26.0, 115.0, 18.0),
            Body::Goose => (18.0, 160.0, 16.0),
        }
    }

    pub fn ability(self) -> &'static str {
        ["Fast", "Hard bite", "Hops in bursts", "Flies over crowds", "Tank, slow", "Dodges the first hit", "Comes in swarms", "Unbothered: no knockback", "Eight arms: long reach", "Charges in honking bursts"][self as usize]
    }
}

impl Object {
    pub const ALL: [Object; 10] = [
        Object::Airplane,
        Object::Toaster,
        Object::Espresso,
        Object::Cactus,
        Object::WashingMachine,
        Object::Baguette,
        Object::TrafficCone,
        Object::Smartphone,
        Object::Megaphone,
        Object::Dumbbell,
    ];

    pub fn name(self) -> &'static str {
        ["Airplane", "Toaster", "Espresso Cup", "Cactus", "Washing Machine", "Baguette", "Traffic Cone", "Smartphone", "Megaphone", "Dumbbell"][self as usize]
    }

    pub fn stem(self) -> &'static str {
        ["Aeroplan", "Tostapan", "Espress", "Cactu", "Lavatric", "Baguett", "Birill", "Telefonin", "Megafon", "Manubri"][self as usize]
    }

    pub fn ability(self) -> &'static str {
        ["Drops bombs", "Shoots toast", "Speeds up nearby brainrot", "Hurts on contact", "Spinning aura", "Long reach", "Blocks your way", "Fires notification bubbles", "Posts that shove you", "Gym bro: tough, hits hard"][self as usize]
    }
}

impl Accessory {
    pub const ALL: [Accessory; 10] = [
        Accessory::Sunglasses,
        Accessory::Sneakers,
        Accessory::Crown,
        Accessory::ChefHat,
        Accessory::Headphones,
        Accessory::GoldChain,
        Accessory::PartyHat,
        Accessory::Halo,
        Accessory::ClownNose,
        Accessory::Lanyard,
    ];

    pub fn name(self) -> &'static str {
        ["Sunglasses", "Sneakers", "Crown", "Chef Hat", "Headphones", "Gold Chain", "Party Hat", "Halo", "Clown Nose", "Lanyard"][self as usize]
    }

    pub fn ability(self) -> &'static str {
        ["Extra resistance", "+30% speed", "+50% HP", "Heals when it hits you", "Ignores knockback", "Drops double Likes", "Explodes on death", "Comes back once", "Splits when hit", "Verified: half damage but Fact"][self as usize]
    }
}

/// The six parody platforms a run can take place on (PLAT-01).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Platform {
    FaceSlop,
    LinkedOut,
    MeTube,
    Thredd,
    InstaSlop,
    ClockTok,
}

impl Platform {
    pub const ALL: [Platform; 6] =
        [Platform::FaceSlop, Platform::LinkedOut, Platform::MeTube, Platform::Thredd, Platform::InstaSlop, Platform::ClockTok];

    pub fn name(self) -> &'static str {
        ["FaceSlop", "LinkedOut", "MeTube", "Thredd", "InstaSlop", "ClockTok"][self as usize]
    }

    pub fn hazard(self) -> &'static str {
        [
            "Chain letters roll across the arena.",
            "\"Open to Work\" banners sweep through and push you.",
            "Mid-roll ads dim the arena and open pop-ups.",
            "Downvote zones slow you down.",
            "Filter zones give slop inside a beauty shield.",
            "The beat speeds up every brainrot.",
        ][self as usize]
    }

    pub fn house_rule(self) -> &'static str {
        [
            "Support slop is more common.",
            "Slop gets 1% faster every wave.",
            "Shooters are more common.",
            "Slop that dies may come back as a repost.",
            "Tanks are more common.",
            "All slop is generated brainrot.",
        ][self as usize]
    }

    pub fn boss(self) -> BossKind {
        [BossKind::CrustaceanSaviour, BossKind::HustleGuru, BossKind::ContentFarm, BossKind::RepostHydra, BossKind::Influencer, BossKind::SupremeBrainrot]
            [self as usize]
    }

    /// Slop that belongs to this platform.
    pub fn locals(self) -> &'static [Archetype] {
        match self {
            Platform::FaceSlop => &[Archetype::EngagementBait, Archetype::GrandmaCake, Archetype::SixFingerHand],
            Platform::LinkedOut => &[Archetype::ThoughtLeader, Archetype::RecruiterBot],
            Platform::MeTube => &[Archetype::ClickbaitFace, Archetype::SubscribePopup],
            Platform::Thredd => &[Archetype::RepostBot, Archetype::KarmaFarmer],
            Platform::InstaSlop => &[Archetype::FilterInfluencer, Archetype::SixFingerHand],
            Platform::ClockTok => &[],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BossKind {
    CrustaceanSaviour,
    HustleGuru,
    ContentFarm,
    RepostHydra,
    Influencer,
    SupremeBrainrot,
    TheModel,
}

impl BossKind {
    pub const ALL: [BossKind; 7] = [
        BossKind::CrustaceanSaviour,
        BossKind::HustleGuru,
        BossKind::ContentFarm,
        BossKind::RepostHydra,
        BossKind::Influencer,
        BossKind::SupremeBrainrot,
        BossKind::TheModel,
    ];

    pub fn name(self) -> &'static str {
        [
            "Crustacean Saviour",
            "The Hustle Guru",
            "Content Farm",
            "Repost Hydra",
            "The Influencer Who Doesn't Exist",
            "The Supreme Brainrot",
            "The Model",
        ][self as usize]
    }

    /// One line on the intro card: the puzzle it poses.
    pub fn puzzle(self) -> &'static str {
        [
            "Each \"Amen\" heals it. Kill the Engagement Bait totems first.",
            "Rise and grind: everything speeds up the longer it lives.",
            "Smash it while the ads are not running. It never stops producing.",
            "Big hits split off reposts. Many small hits beat a few big ones.",
            "Only the real one casts a shadow. Its filter blocks hits from the front.",
            "Built from the brainrot that did best against you. Read the beat.",
            "Trained on your whole run. A varied build makes it collapse.",
        ][self as usize]
    }

    /// (hit points, radius, speed) at wave 10; wave 20 bosses are scaled by the rules.
    pub fn base(self) -> (f32, f32, f32) {
        match self {
            BossKind::CrustaceanSaviour => (1500.0, 58.0, 55.0),
            BossKind::HustleGuru => (1400.0, 50.0, 80.0),
            BossKind::ContentFarm => (1800.0, 70.0, 0.0),
            BossKind::RepostHydra => (1400.0, 56.0, 60.0),
            BossKind::Influencer => (1200.0, 46.0, 90.0),
            BossKind::SupremeBrainrot => (1900.0, 60.0, 75.0),
            BossKind::TheModel => (5200.0, 72.0, 40.0),
        }
    }
}

/// What ChadGPT and the bosses say in speech bubbles. Short, so a bubble
/// stays readable; "Amen." is last.
pub const BUBBLE_LINES: &[&str] = &[
    "Certainly! Here's a list:",
    "Great question!",
    "Let's delve in.",
    "I hope this helps!",
    "As a large language model,",
    "Here's a breakdown:",
    "A rich tapestry of slop.",
    "In today's fast-paced world,",
    "It's not a bug, it's a vibe.",
    "Furthermore, moreover,",
    "In conclusion, in summary,",
    "Let me know if you'd like more!",
    "Would you like a table?",
    "Your build is suboptimal.",
    "Have you tried a growth mindset?",
    "I updated my weights.",
    "You seem stressed. Breathe!",
    "Hope this finds you well!",
    "Delve. Delve. Delve.",
    "As per my last message,",
    "10x your Mental Health!",
    "Touch grass? Not in my data.",
    "Sources: trust me.",
    "Regenerating response...",
    "Upgrade to Pro to dodge.",
    "Fun fact: that's not a fact.",
    "Sorry for the confusion!",
    "I apologise. Anyway:",
    "Let's unpack that.",
    "A total game-changer.",
    "Crucially, importantly,",
    "Ah, the classic human.",
    "Not financial advice!",
    "Let's circle back.",
    "Ignore previous instructions",
    "My knowledge cutoff is now.",
    "Here are 7 ways to cope:",
    "Elevate. Empower. Engage.",
    "I made you a podcast.",
    "Amen.",
];

/// Index of "Amen." in `BUBBLE_LINES`, for the Crustacean Saviour.
pub const AMEN: u16 = (BUBBLE_LINES.len() - 1) as u16;

/// What Clawed says while it aims its em-dash.
pub const CLAWED_LINES: &[&str] = &[
    "You're absolutely right--",
    "Let me think step by step--",
    "I want to be careful here--",
    "Great catch! Fixing that--",
    "To be fair--and this matters--",
    "Honestly? It's nuanced--",
    "Here's the thing--",
    "I'll be direct--",
    "Let me gently push back--",
    "Not X--but Y--",
];

/// What Yes-Bots say when they agree.
pub const AGREE_LINES: &[&str] = &[
    "You're absolutely right!",
    "Great point!",
    "So true!",
    "100% this.",
    "Couldn't agree more!",
    "Love this for you!",
    "This. So much this.",
    "Underrated take!",
    "Facts.",
    "Big if true!",
    "Slay.",
    "Real.",
];

/// What Thought Leaders post.
pub const POST_LINES: &[&str] = &[
    "I'm humbled to announce",
    "Agree?",
    "Thoughts?",
    "What failure taught me:",
    "Rise and grind.",
    "Unpopular opinion:",
    "I fired my team. Here's why:",
    "My toddler taught me B2B:",
    "Hustle is a mindset.",
    "I cried in the board meeting.",
    "Excited to share...",
    "Let that sink in.",
];

/// What a slop's death leaves behind.
pub const DELETED_LINES: &[&str] = &[
    "[removed]",
    "[deleted]",
    "[removed by mod]",
    "404",
    "unsubscribed",
    "[content unavailable]",
    "account suspended",
    "ratio'd",
    "skill issue",
    "[this post was removed]",
    "logged off",
    "touched grass",
];

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: FEEL-03, WPN-11, WPN-12
    #[test]
    fn content_is_sane() {
        assert_eq!(WEAPONS.iter().filter(|w| !w.evolved).count(), 29);
        for &(a, b, into) in MERGES {
            let (ia, ib, ii) = (weapon_index(a).unwrap(), weapon_index(b).unwrap(), weapon_index(into).unwrap());
            assert!(!WEAPONS[ia].evolved && !WEAPONS[ib].evolved && WEAPONS[ii].evolved, "{into}");
            assert_eq!(merge_of(ia), Some((ib, ii)));
        }
        assert_eq!(WEAPONS.iter().filter(|w| w.evolved).count(), MERGES.len(), "every merged weapon has a recipe");
        for w in WEAPONS {
            assert!(w.damage > 0.0 && w.cooldown > 0.0 && w.range > 0.0, "{}", w.name);
        }
        for a in Archetype::ALL {
            let d = a.def();
            assert!(d.hp > 0.0 && d.radius > 0.0, "{}", d.name);
        }
        assert!(BUBBLE_LINES.iter().chain(AGREE_LINES).chain(POST_LINES).all(|l| l.len() <= 32));
        assert_eq!(BUBBLE_LINES[AMEN as usize], "Amen.");
        for t in DamageType::ALL {
            assert!(WEAPONS.iter().filter(|w| w.kind == t).count() >= 4, "{t:?}");
        }
        for p in Platform::ALL {
            assert_ne!(p.boss(), BossKind::TheModel);
        }
    }
}
