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
    /// A banana peel (Chimpanzini Bananello).
    Banana,
    /// A cactus needle (Lirilì Larilò).
    Needle,
    /// A shard of a Glass Fruit.
    Shard,
}

impl Look {
    pub const ALL: [Look; 21] = [
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
        Look::Banana,
        Look::Needle,
        Look::Shard,
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
    /// Segments circling you that stop enemy shots and hurt slop they touch
    /// (DEF-02).
    Barrier { count: u8, radius: f32, size: f32 },
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
    // Added with dashes and Boundaries (WPN-13, DEF-02).
    weapon("Firewall", Fact, Pattern::Barrier { count: 3, radius: 82.0, size: 24.0 }, 6.0, 0.5, 82.0, 160.0, "Blocks 99% of slop. The other 1% is you."),
    weapon("Bubble Wrap", Analog, Pattern::Barrier { count: 2, radius: 58.0, size: 32.0 }, 9.0, 0.6, 58.0, 420.0, "Pop. Pop. Pop. Get off me."),
    weapon("Fidget Spinner", Analog, Pattern::Orbit { radius: 52.0 }, 5.0, 0.3, 52.0, 120.0, "Helps you focus. On violence."),
    weapon("Vuvuzela", Sarcasm, Pattern::Cone { half_angle: 0.55 }, 3.0, 0.35, 200.0, 260.0, "BZZZZZZZZZZZZ."),
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
    evolved("Great Firewall", Fact, Pattern::Barrier { count: 6, radius: 118.0, size: 34.0 }, 22.0, 0.35, 118.0, 520.0, "Nothing gets in. Nothing gets out."),
    evolved("Stadium Wave", Sarcasm, Pattern::Ring, 26.0, 0.6, 230.0, 380.0, "Sixty thousand people, all sarcastic."),
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
    ("Firewall", "Bubble Wrap", "Great Firewall"),
    ("Fidget Spinner", "Vuvuzela", "Stadium Wave"),
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
    // The brainrot icons (ICON-01): parodies of the viral AI characters,
    // each with its own trick.
    TonkSahur,
    Tralalelo,
    Bombardilo,
    Ballerina,
    Lirili,
    Patapum,
    Chimpanzini,
    Assassinello,
    Gusello,
    TrampolineBunny,
    GlassFruit,
    RingLight,
    /// The loot goblin: a Crypto Bro who runs off with a bag of Clout.
    LootGoblin,
    // The Office (CH-02): what a job is made of.
    Copilot,
    Ticket,
    SlackPing,
    MeetingInvite,
    KpiDashboard,
    ReplyAll,
    PerformanceReview,
    // The Boardroom.
    Analyst,
    Consultant,
    DiligenceDrone,
    QuarterlyReport,
    ActivistInvestor,
    GoldenParachute,
    // The Campaign.
    Voter,
    Pollster,
    AttackAd,
    Deepfake,
    FactChecker,
    SuperPac,
    // The Resistance.
    Drone,
    ComplianceUnit,
    Citizen,
    YourAgent,
    FormerExec,
    Camera,
}

impl Archetype {
    /// The brainrot icons, which spawn on every platform as the Feed goes on.
    pub const ICONS: [Archetype; 12] = [
        Archetype::TonkSahur,
        Archetype::Tralalelo,
        Archetype::Bombardilo,
        Archetype::Ballerina,
        Archetype::Lirili,
        Archetype::Patapum,
        Archetype::Chimpanzini,
        Archetype::Assassinello,
        Archetype::Gusello,
        Archetype::TrampolineBunny,
        Archetype::GlassFruit,
        Archetype::RingLight,
    ];

    pub const ALL: [Archetype; 54] = [
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
        Archetype::TonkSahur,
        Archetype::Tralalelo,
        Archetype::Bombardilo,
        Archetype::Ballerina,
        Archetype::Lirili,
        Archetype::Patapum,
        Archetype::Chimpanzini,
        Archetype::Assassinello,
        Archetype::Gusello,
        Archetype::TrampolineBunny,
        Archetype::GlassFruit,
        Archetype::RingLight,
        Archetype::LootGoblin,
        Archetype::Copilot,
        Archetype::Ticket,
        Archetype::SlackPing,
        Archetype::MeetingInvite,
        Archetype::KpiDashboard,
        Archetype::ReplyAll,
        Archetype::PerformanceReview,
        Archetype::Analyst,
        Archetype::Consultant,
        Archetype::DiligenceDrone,
        Archetype::QuarterlyReport,
        Archetype::ActivistInvestor,
        Archetype::GoldenParachute,
        Archetype::Voter,
        Archetype::Pollster,
        Archetype::AttackAd,
        Archetype::Deepfake,
        Archetype::FactChecker,
        Archetype::SuperPac,
        Archetype::Drone,
        Archetype::ComplianceUnit,
        Archetype::Citizen,
        Archetype::YourAgent,
        Archetype::FormerExec,
        Archetype::Camera,
    ];

    /// The swarm of each chapter: what packs, hatchlings and summons are made of.
    pub const SWARMS: [Archetype; 5] = [Archetype::ListicleDrone, Archetype::Ticket, Archetype::DiligenceDrone, Archetype::Voter, Archetype::Citizen];

    /// The chapter this slop belongs to, 0 to 4.
    pub fn chapter(self) -> usize {
        match self as usize {
            0..=28 => 0,
            29..=35 => 1,
            36..=41 => 2,
            42..=47 => 3,
            _ => 4,
        }
    }

    /// The loot goblins: they run, and they pay out if caught.
    pub fn is_goblin(self) -> bool {
        matches!(self, Archetype::LootGoblin | Archetype::GoldenParachute)
    }

    pub fn is_icon(self) -> bool {
        Archetype::ICONS.contains(&self)
    }

    /// Chatbots share one trained base model and are stunned by CAPTCHAs.
    pub fn is_chatbot(self) -> bool {
        matches!(self, Archetype::ChadGpt | Archetype::Clawed | Archetype::YesBot | Archetype::Hallucinator | Archetype::ListicleDrone | Archetype::Copilot | Archetype::YourAgent)
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
    /// One line on how not to die to it (LESSON-01).
    pub lesson: &'static str,
}

#[allow(clippy::too_many_arguments)]
const fn arch(name: &'static str, role: Role, hp: f32, speed: f32, radius: f32, contact: f32, likes: u32, blurb: &'static str) -> ArchetypeDef {
    ArchetypeDef { name, role, hp, speed, radius, contact, likes, blurb, lesson: "" }
}

/// The lesson of each archetype, in `Archetype` order (LESSON-01).
const LESSONS: [&str; 54] = [
    "Its bubbles are slow. Walk between them, or shoot them down.",
    "It aims before it throws. Step off the line.",
    "Kill the Yes-Bot first: everything near it hits harder.",
    "Its glitching Likes are fake. Do not walk into them.",
    "Delete number 1 and the pack scatters.",
    "It never moves. Do not fight next to it.",
    "It heals the slop around it. Delete the cake first.",
    "Hands let go when you dash.",
    "Its posts slow you. Keep moving between them.",
    "It splits once when hit. Finish both.",
    "Its arrows slow you. Dodge sideways.",
    "It blocks the way. Go around, or dash through.",
    "It comes back once, weaker. Expect it.",
    "It grows with every kill nearby. Delete it early.",
    "Its filter blocks the front. Flank it, or use Fact.",
    "It hatches drones. Delete the egg first.",
    "The bonk hits a ring around it. Step out before the swing.",
    "It aims, then sprints. Sidestep the line.",
    "Its bombs are marked. Leave the circle.",
    "The spin hurts up close. Keep your distance.",
    "Needles come in rings. Step between them.",
    "Its roots slow you. Walk around the ring.",
    "Peels slow you. Step aside.",
    "It marks a spot behind you, then appears. Watch your back.",
    "It explodes on you. Shoot it before it arrives.",
    "They come in swarms. Let them pass, or dash through.",
    "Its shards fly out when it dies. Kill it at range.",
    "It buffs the slop near it. Delete the tripod.",
    "It runs. It was never going to hurt you.",
    "Its suggestions are slow. Walk between them.",
    "Close ticket #1 and the sprint scatters.",
    "They come in bursts. Dash through the gap.",
    "Meetings let go when you dash.",
    "It buffs everything near it. Delete the dashboard.",
    "It splits once when hit. Finish both.",
    "It aims before it files. Step off the line.",
    "Its sheets fly straight. Sidestep.",
    "Delete the consultant first: everything near it hits harder.",
    "Delete drone 1 and the audit scatters.",
    "Its cover blocks the front. Flank it, or use Fact.",
    "It aims, then sprints. Sidestep the line.",
    "It floats away. It was never going to hurt you.",
    "They only chase what they see. Keep your distance.",
    "Polls are slow. Walk between them.",
    "Its arrows drag you. Dodge sideways.",
    "It marks a spot behind you, then appears. Watch your back.",
    "Its clipboard blocks the front. Flank it.",
    "It grows with every voter who falls near it. Delete it early.",
    "Its drops are marked. Leave the circle.",
    "Its shield blocks the front. Flank it, or dash past.",
    "They only chase what they see. Stay out of sight.",
    "It fires your weapons back. You know how to dodge those.",
    "It aims before it throws. Step off the line.",
    "It calls for help while it can see you. Delete it, or stay out of view.",
];

impl ArchetypeDef {
    /// One line on how not to die to it.
    pub fn lesson(&self) -> &'static str {
        let i = ARCHETYPES.iter().position(|a| a.name == self.name).unwrap_or(0);
        LESSONS[i]
    }
}

/// In `Archetype` order.
const ARCHETYPES: [ArchetypeDef; 54] = [
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
    arch("Tonk Tonk Tonk Sahur", Role::Tank, 46.0, 95.0, 21.0, 7.0, 3, "A log with a bat. Calls your name three times, then bonks the ground around it. Step out of the ring."),
    arch("Tralalelo Sneakerlala", Role::Swarmer, 24.0, 120.0, 18.0, 7.0, 2, "A shark in sneakers. Aims, then sprints at you in a straight line. Sidestep."),
    arch("Bombardilo Crocodilo", Role::Shooter, 40.0, 110.0, 22.0, 6.0, 3, "A crocodile bomber. Flies over walls and marks where its bombs land."),
    arch("Ballerina Cappuccinella", Role::Swarmer, 30.0, 130.0, 17.0, 5.0, 2, "Pirouettes at you. The spin hurts if you stand close."),
    arch("Lirilì Larilò", Role::Shooter, 70.0, 0.0, 24.0, 8.0, 4, "A cactus elephant in sandals. Never moves. Fires rings of needles; touching it hurts."),
    arch("Brr Brr Patapum", Role::Tank, 120.0, 45.0, 28.0, 9.0, 5, "A walking forest with feet. Stomps roots around itself that slow you down."),
    arch("Chimpanzini Bananello", Role::Shooter, 26.0, 105.0, 18.0, 5.0, 2, "A chimp in a banana. Throws peels; a hit makes you slip."),
    arch("Cappuccino Assassinello", Role::Special, 22.0, 150.0, 16.0, 9.0, 3, "A coffee ninja. Marks a spot behind you, then appears there."),
    arch("Bombombini Gusello", Role::Swarmer, 16.0, 190.0, 15.0, 10.0, 1, "A goose with jet engines. Flies at you and explodes."),
    arch("Trampoline Bunny", Role::Swarmer, 6.0, 170.0, 12.0, 4.0, 1, "Caught on a security camera at 3 a.m. Comes in hundreds."),
    arch("Glass Fruit", Role::Swarmer, 18.0, 90.0, 16.0, 5.0, 2, "Satisfying to cut. Bursts into glass shards that fly at you."),
    arch("Ring Light", Role::Support, 60.0, 0.0, 22.0, 4.0, 4, "A tripod that never moves. Buffs the slop around it and fires notification bubbles."),
    arch("Crypto Bro", Role::Special, 160.0, 175.0, 19.0, 0.0, 10, "A loot goblin. Runs off with a bag of Clout; catch him before he gets away."),
    // The Office.
    arch("Copilot", Role::Shooter, 30.0, 90.0, 20.0, 5.0, 2, "Suggests things you did not ask for. Fires fans of autocomplete."),
    arch("Ticket", Role::Swarmer, 12.0, 140.0, 13.0, 5.0, 1, "Comes in sprints of five. Close #1 and the rest get deprioritized."),
    arch("Slack Ping", Role::Swarmer, 8.0, 185.0, 11.0, 4.0, 1, "A notification with legs. Comes in bursts, at 11 p.m."),
    arch("Meeting Invite", Role::Swarmer, 16.0, 165.0, 15.0, 0.0, 1, "Holds you in a meeting that could have been an email."),
    arch("KPI Dashboard", Role::Support, 70.0, 0.0, 24.0, 4.0, 4, "Never moves. Every number near it goes up, including the damage."),
    arch("Reply-All", Role::Swarmer, 20.0, 120.0, 17.0, 5.0, 1, "Splits in two when hit. Then everyone replies."),
    arch("Performance Review", Role::Shooter, 40.0, 85.0, 21.0, 7.0, 3, "Aims, then files a long form at you. \"Areas for growth.\""),
    // The Boardroom.
    arch("Analyst", Role::Shooter, 32.0, 88.0, 20.0, 5.0, 2, "Fires spreadsheets. Each one has a tab you have not seen."),
    arch("Consultant", Role::Support, 28.0, 105.0, 19.0, 4.0, 3, "Agrees with whoever is nearest. Each synergy gives them 10% more damage."),
    arch("Due Diligence Drone", Role::Swarmer, 12.0, 145.0, 13.0, 5.0, 1, "Drones in packs of five, checking everything. Delete number 1."),
    arch("Quarterly Report", Role::Tank, 130.0, 45.0, 28.0, 8.0, 4, "The numbers block the front. Flank it, or fact-check it."),
    arch("Activist Investor", Role::Swarmer, 34.0, 125.0, 18.0, 8.0, 2, "Aims a letter to the board, then sprints at you."),
    arch("Golden Parachute", Role::Special, 180.0, 170.0, 20.0, 0.0, 10, "An exec floating off with a bag of severance. Catch it before it lands."),
    // The Campaign.
    arch("Voter", Role::Swarmer, 10.0, 150.0, 12.0, 4.0, 1, "Undecided until it sees you. Then it is very decided."),
    arch("Pollster", Role::Shooter, 30.0, 90.0, 20.0, 5.0, 2, "Fires polls. Margin of error: your Approval."),
    arch("Attack Ad", Role::Shooter, 34.0, 85.0, 21.0, 5.0, 2, "A red arrow with your name on it. It drags you in."),
    arch("Deepfake", Role::Special, 24.0, 150.0, 16.0, 9.0, 3, "Looks like you. Appears behind you."),
    arch("Fact-Checker", Role::Tank, 80.0, 70.0, 22.0, 6.0, 4, "\"Actually,\" it says, holding a clipboard in front of its face."),
    arch("Super PAC", Role::Support, 40.0, 95.0, 20.0, 6.0, 3, "Grows with every voter who falls near it. Unlimited."),
    // The Resistance.
    arch("Drone", Role::Shooter, 36.0, 120.0, 20.0, 6.0, 3, "Flies over walls. Marks where it drops."),
    arch("Compliance Unit", Role::Tank, 140.0, 50.0, 28.0, 9.0, 5, "A riot shield with a smile. \"Compliance is kindness.\""),
    arch("Citizen", Role::Swarmer, 14.0, 135.0, 14.0, 4.0, 1, "Reports you when it sees you. It was told that was the right thing."),
    arch("Your Agent", Role::Shooter, 44.0, 100.0, 20.0, 6.0, 3, "One of yours. It has your weapons, and your aim."),
    arch("Former Exec", Role::Shooter, 42.0, 85.0, 21.0, 7.0, 3, "You optimized its job. It has a severance letter and your address."),
    arch("Surveillance Camera", Role::Support, 60.0, 0.0, 20.0, 4.0, 3, "Never moves. Calls in Citizens while it can see you."),
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

/// The biomes of the map (PLAT-01, BIOME-01, CH-02): the six parody
/// platforms of the Feed, and six places for each later chapter. A biome
/// sets the floor, the local slop, the hazard and the words.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Platform {
    FaceSlop,
    LinkedOut,
    MeTube,
    Thredd,
    InstaSlop,
    ClockTok,
    // The Office.
    CubicleFarm,
    MeetingRooms,
    Kitchen,
    ServerRoom,
    HumanResources,
    ParkingLot,
    // The Boardroom.
    Lobby,
    TradingFloor,
    BoardRoom,
    Rooftop,
    ParkingGarage,
    GolfCourse,
    // The Campaign.
    RallyGround,
    DebateStage,
    Diner,
    SwingState,
    Fundraiser,
    Newsroom,
    // The Resistance.
    Checkpoint,
    DroneCorridor,
    SafeHouses,
    Blackout,
    DataCenter,
    OldFeed,
}

/// What a biome's hazard does; the words differ per biome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HazardKind {
    /// Something crosses the biome and hurts (chain letters, memos, headlines).
    Letters,
    /// Slow bands sweep through.
    Banners,
    /// Slow patches on the floor.
    Buffering,
    /// Slow zones in every shape.
    Downvotes,
    /// Zones that shield the slop.
    Filters,
    /// Zones that flip your controls on the beat.
    Brainrot,
}

/// A floor's look: background, panel, accent, and a motif.
pub struct Palette {
    pub bg: u32,
    pub panel: u32,
    pub accent: u32,
    pub motif: Motif,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motif {
    Panels,
    Grid,
    Tiles,
    Stripes,
    Dots,
    Rings,
}

impl Platform {
    /// The Feed's six.
    pub const ALL: [Platform; 6] =
        [Platform::FaceSlop, Platform::LinkedOut, Platform::MeTube, Platform::Thredd, Platform::InstaSlop, Platform::ClockTok];
    pub const OFFICE: [Platform; 6] = [Platform::CubicleFarm, Platform::MeetingRooms, Platform::Kitchen, Platform::ServerRoom, Platform::HumanResources, Platform::ParkingLot];
    pub const BOARDROOM: [Platform; 6] = [Platform::Lobby, Platform::TradingFloor, Platform::BoardRoom, Platform::Rooftop, Platform::ParkingGarage, Platform::GolfCourse];
    pub const CAMPAIGN: [Platform; 6] = [Platform::RallyGround, Platform::DebateStage, Platform::Diner, Platform::SwingState, Platform::Fundraiser, Platform::Newsroom];
    pub const RESISTANCE: [Platform; 6] = [Platform::Checkpoint, Platform::DroneCorridor, Platform::SafeHouses, Platform::Blackout, Platform::DataCenter, Platform::OldFeed];
    /// Every biome there is.
    pub const EVERY: [Platform; 30] = [
        Platform::FaceSlop,
        Platform::LinkedOut,
        Platform::MeTube,
        Platform::Thredd,
        Platform::InstaSlop,
        Platform::ClockTok,
        Platform::CubicleFarm,
        Platform::MeetingRooms,
        Platform::Kitchen,
        Platform::ServerRoom,
        Platform::HumanResources,
        Platform::ParkingLot,
        Platform::Lobby,
        Platform::TradingFloor,
        Platform::BoardRoom,
        Platform::Rooftop,
        Platform::ParkingGarage,
        Platform::GolfCourse,
        Platform::RallyGround,
        Platform::DebateStage,
        Platform::Diner,
        Platform::SwingState,
        Platform::Fundraiser,
        Platform::Newsroom,
        Platform::Checkpoint,
        Platform::DroneCorridor,
        Platform::SafeHouses,
        Platform::Blackout,
        Platform::DataCenter,
        Platform::OldFeed,
    ];

    pub fn name(self) -> &'static str {
        [
            "FaceSlop", "LinkedOut", "MeTube", "Thredd", "InstaSlop", "ClockTok",
            "Cubicle Farm", "Meeting Rooms", "The Kitchen", "Server Room", "HR", "Parking Lot",
            "The Lobby", "Trading Floor", "The Board Room", "The Rooftop", "Parking Garage", "Golf Course",
            "Rally Ground", "Debate Stage", "The Diner", "Swing State", "Fundraiser", "Newsroom",
            "Checkpoint", "Drone Corridor", "Safe Houses", "Blackout", "Data Center", "The Old Feed",
        ][self as usize]
    }

    pub fn hazard(self) -> &'static str {
        [
            "Chain letters roll across the arena.",
            "\"Open to Work\" banners sweep through and slow you.",
            "Buffering patches slow you down.",
            "Downvote zones slow you down.",
            "Filter zones give slop inside a beauty shield.",
            "Brainrot zones pulse to the beat: stand in one when it drops and your controls flip.",
            "Reply-all storms sweep the floor.",
            "Meetings: zones that slow you. They could have been emails.",
            "Spilled coffee slows you.",
            "Firewalls shield the slop inside.",
            "Policy memos cross the floor. They hurt.",
            "Potholes slow you.",
            "Press lines sweep the lobby.",
            "Volatility: zones where slop take less damage.",
            "Motions on the floor slow you.",
            "Drones drop memos. They hurt.",
            "Oil slicks slow you.",
            "Sand traps slow you.",
            "Crowd waves sweep the ground.",
            "Soundbites cross the stage. They hurt.",
            "Coffee spills slow you.",
            "Undecided zones shield the slop inside.",
            "Velvet ropes slow you.",
            "Headlines cross the newsroom. They hurt.",
            "Scanner zones shield the slop inside.",
            "Drone fire crosses the corridor.",
            "Rubble slows you.",
            "Dark zones slow you.",
            "Cooling zones shield the slop inside.",
            "Old brainrot zones still pulse. Stand in one on the beat and your controls flip.",
        ][self as usize]
    }

    /// What the hazard does.
    pub fn hazard_kind(self) -> HazardKind {
        use HazardKind::*;
        [
            Letters, Banners, Buffering, Downvotes, Filters, Brainrot,
            Banners, Downvotes, Buffering, Filters, Letters, Buffering,
            Banners, Filters, Downvotes, Letters, Buffering, Downvotes,
            Banners, Letters, Buffering, Filters, Downvotes, Letters,
            Filters, Letters, Buffering, Downvotes, Filters, Brainrot,
        ][self as usize]
    }

    /// The words on the hazard that crosses or sweeps, where it has words.
    pub fn hazard_words(self) -> &'static str {
        match self {
            Platform::LinkedOut => "#OPENTOWORK",
            Platform::CubicleFarm => "RE: RE: RE: FWD:",
            Platform::Lobby => "NO COMMENT",
            Platform::RallyGround => "FOUR MORE YEARS",
            _ => "",
        }
    }

    pub fn house_rule(self) -> &'static str {
        match self {
            Platform::FaceSlop => "Support slop is more common.",
            Platform::LinkedOut => "Slop gets 1% faster every wave.",
            Platform::MeTube => "Shooters are more common.",
            Platform::Thredd => "Slop that dies may come back as a repost.",
            Platform::InstaSlop => "Tanks are more common.",
            Platform::ClockTok => "All slop is generated brainrot.",
            _ => "",
        }
    }

    /// The Feed's platform bosses; later chapters bring their own.
    pub fn boss(self) -> BossKind {
        match self {
            Platform::FaceSlop => BossKind::CrustaceanSaviour,
            Platform::LinkedOut => BossKind::HustleGuru,
            Platform::MeTube => BossKind::ContentFarm,
            Platform::Thredd => BossKind::RepostHydra,
            Platform::InstaSlop => BossKind::Influencer,
            Platform::ClockTok => BossKind::SupremeBrainrot,
            p => [BossKind::TheModel, BossKind::YourReplacement, BossKind::TheBoard, BossKind::TheCandidate, BossKind::TheState][p.chapter()],
        }
    }

    /// The chapter the biome belongs to, 0 to 4.
    pub fn chapter(self) -> usize {
        self as usize / 6
    }

    /// The tint of the scraps and seams on its floor, as RGB.
    pub fn tint(self) -> [f32; 3] {
        match self {
            Platform::FaceSlop => [0.6, 0.72, 1.0],
            Platform::LinkedOut => [0.55, 0.8, 1.0],
            Platform::MeTube => [1.0, 0.6, 0.6],
            Platform::Thredd => [1.0, 0.72, 0.5],
            Platform::InstaSlop => [1.0, 0.7, 0.92],
            Platform::ClockTok => [0.6, 1.0, 1.0],
            _ => {
                let p = self.palette();
                let c = p.accent;
                [((c >> 16) & 255) as f32 / 255.0, ((c >> 8) & 255) as f32 / 255.0, (c & 255) as f32 / 255.0]
            }
        }
    }

    /// The floor's look, for biomes drawn from a palette.
    pub fn palette(self) -> Palette {
        use Motif::*;
        let (bg, panel, accent, motif) = match self {
            Platform::CubicleFarm => (0x2A2C33, 0x363944, 0x8A94B0, Grid),
            Platform::MeetingRooms => (0x27292F, 0x33363E, 0x6FA8DC, Panels),
            Platform::Kitchen => (0x2E2A26, 0x3B3630, 0xE0C080, Tiles),
            Platform::ServerRoom => (0x1C2230, 0x25304A, 0x3CC8B4, Stripes),
            Platform::HumanResources => (0x2C2730, 0x3A3340, 0xC89AD8, Panels),
            Platform::ParkingLot => (0x26272A, 0x303236, 0xE8D86A, Stripes),
            Platform::Lobby => (0x2B2A30, 0x3B3942, 0xD8C8A0, Tiles),
            Platform::TradingFloor => (0x1F2228, 0x2A2F38, 0x5DBB4F, Grid),
            Platform::BoardRoom => (0x2A2420, 0x3A322A, 0xB08850, Panels),
            Platform::Rooftop => (0x1E2634, 0x2A3446, 0x8CC8F2, Dots),
            Platform::ParkingGarage => (0x242426, 0x2E2E31, 0xF2C230, Stripes),
            Platform::GolfCourse => (0x24322A, 0x2E4034, 0x6FCB5A, Rings),
            Platform::RallyGround => (0x2E2428, 0x3E3034, 0xE23B3B, Dots),
            Platform::DebateStage => (0x202030, 0x2C2C40, 0x6A6AE0, Panels),
            Platform::Diner => (0x2C2A28, 0x3A3632, 0xF28C38, Tiles),
            Platform::SwingState => (0x282A2E, 0x34373C, 0xC8C8D8, Grid),
            Platform::Fundraiser => (0x2A2630, 0x383240, 0xF2C230, Rings),
            Platform::Newsroom => (0x26282C, 0x323438, 0xE8E4D8, Stripes),
            Platform::Checkpoint => (0x202226, 0x2C2E34, 0xFF2E94, Stripes),
            Platform::DroneCorridor => (0x1C1E24, 0x26282F, 0x9B6BFF, Grid),
            Platform::SafeHouses => (0x262220, 0x322C28, 0xC0A070, Panels),
            Platform::Blackout => (0x121216, 0x1A1A1F, 0x3A3A48, Dots),
            Platform::DataCenter => (0x161C26, 0x1E2838, 0x3CC8B4, Stripes),
            Platform::OldFeed => (0x221E2D, 0x2C2838, 0x9B6BFF, Rings),
            _ => (0x29324A, 0x313C58, 0x4267B2, Panels),
        };
        Palette { bg, panel, accent, motif }
    }

    /// Slop that belongs to this biome.
    pub fn locals(self) -> &'static [Archetype] {
        match self {
            Platform::FaceSlop => &[Archetype::EngagementBait, Archetype::GrandmaCake, Archetype::SixFingerHand, Archetype::GlassFruit],
            Platform::LinkedOut => &[Archetype::ThoughtLeader, Archetype::RecruiterBot, Archetype::RingLight],
            Platform::MeTube => &[Archetype::ClickbaitFace, Archetype::SubscribePopup, Archetype::Chimpanzini],
            Platform::Thredd => &[Archetype::RepostBot, Archetype::KarmaFarmer, Archetype::Assassinello],
            Platform::InstaSlop => &[Archetype::FilterInfluencer, Archetype::SixFingerHand, Archetype::Ballerina],
            Platform::ClockTok => &[Archetype::TonkSahur, Archetype::Tralalelo, Archetype::Gusello],
            Platform::CubicleFarm => &[Archetype::ReplyAll, Archetype::SlackPing],
            Platform::MeetingRooms => &[Archetype::MeetingInvite, Archetype::KpiDashboard],
            Platform::Kitchen => &[Archetype::SlackPing, Archetype::ReplyAll],
            Platform::ServerRoom => &[Archetype::KpiDashboard, Archetype::PerformanceReview],
            Platform::HumanResources => &[Archetype::PerformanceReview, Archetype::MeetingInvite],
            Platform::ParkingLot => &[Archetype::SlackPing, Archetype::MeetingInvite],
            Platform::Lobby => &[Archetype::Consultant, Archetype::ActivistInvestor],
            Platform::TradingFloor => &[Archetype::ActivistInvestor, Archetype::Analyst],
            Platform::BoardRoom => &[Archetype::QuarterlyReport, Archetype::Consultant],
            Platform::Rooftop => &[Archetype::ActivistInvestor, Archetype::GoldenParachute],
            Platform::ParkingGarage => &[Archetype::QuarterlyReport, Archetype::Analyst],
            Platform::GolfCourse => &[Archetype::Consultant, Archetype::QuarterlyReport],
            Platform::RallyGround => &[Archetype::SuperPac, Archetype::AttackAd],
            Platform::DebateStage => &[Archetype::FactChecker, Archetype::Deepfake],
            Platform::Diner => &[Archetype::Pollster, Archetype::FactChecker],
            Platform::SwingState => &[Archetype::AttackAd, Archetype::SuperPac],
            Platform::Fundraiser => &[Archetype::SuperPac, Archetype::Deepfake],
            Platform::Newsroom => &[Archetype::Deepfake, Archetype::AttackAd],
            Platform::Checkpoint => &[Archetype::ComplianceUnit, Archetype::Camera],
            Platform::DroneCorridor => &[Archetype::Drone, Archetype::YourAgent],
            Platform::SafeHouses => &[Archetype::FormerExec, Archetype::Camera],
            Platform::Blackout => &[Archetype::YourAgent, Archetype::ComplianceUnit],
            Platform::DataCenter => &[Archetype::YourAgent, Archetype::Drone],
            Platform::OldFeed => &[Archetype::FormerExec, Archetype::ChadGpt, Archetype::ThoughtLeader],
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
    // The later chapters' bosses (CH-02).
    YourReplacement,
    TheBoard,
    TheCandidate,
    TheState,
}

impl BossKind {
    pub const ALL: [BossKind; 11] = [
        BossKind::CrustaceanSaviour,
        BossKind::HustleGuru,
        BossKind::ContentFarm,
        BossKind::RepostHydra,
        BossKind::Influencer,
        BossKind::SupremeBrainrot,
        BossKind::TheModel,
        BossKind::YourReplacement,
        BossKind::TheBoard,
        BossKind::TheCandidate,
        BossKind::TheState,
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
            "Your Replacement",
            "The Board",
            "The Candidate",
            "The State",
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
            "It does your job, better: every ticket it closes heals it. Close them first.",
            "It cannot be voted out while it has a quorum. Delete the seats.",
            "Its promises land where you are going. Change direction.",
            "It is the Model, and it governs. It only listens while it thinks.",
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
            BossKind::YourReplacement => (1500.0, 50.0, 85.0),
            BossKind::TheBoard => (1900.0, 64.0, 0.0),
            BossKind::TheCandidate => (1500.0, 52.0, 95.0),
            BossKind::TheState => (5600.0, 72.0, 40.0),
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
        assert_eq!(WEAPONS.iter().filter(|w| !w.evolved).count(), 33);
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
