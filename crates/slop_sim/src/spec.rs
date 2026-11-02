//! What one kind of slop is in a wave: how it looks, moves and attacks, its
//! numbers and its traits. Fixed characters, generated brainrot and bosses
//! all become a `SlopSpec`, so the simulation treats them alike.

use crate::content::{Accessory, Archetype, Body, BossKind, Object, Role};

/// Trait flags (SLOP-05).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Traits(pub u32);

impl Traits {
    pub const EXPLODE: u32 = 1;
    pub const HEAL_ON_HIT: u32 = 1 << 1;
    pub const NO_KNOCKBACK: u32 = 1 << 2;
    pub const DOUBLE_LIKES: u32 = 1 << 3;
    pub const DODGE_FIRST: u32 = 1 << 4;
    pub const THORNS: u32 = 1 << 5;
    pub const SPIN_AURA: u32 = 1 << 6;
    pub const SPEED_AURA: u32 = 1 << 7;
    pub const FRONT_SHIELD: u32 = 1 << 8;
    pub const RESPAWN: u32 = 1 << 9;
    pub const SPLIT: u32 = 1 << 10;
    pub const GROWS: u32 = 1 << 11;
    pub const HEALER: u32 = 1 << 12;
    pub const BUFF_AURA: u32 = 1 << 13;
    pub const FAKE_LIKES: u32 = 1 << 14;
    pub const RATIO: u32 = 1 << 15;
    pub const POPUP: u32 = 1 << 16;
    pub const FLIES: u32 = 1 << 17;
    pub const HEAL_BOSS: u32 = 1 << 18;
    pub const HATCH: u32 = 1 << 19;
    pub const CHATBOT: u32 = 1 << 20;
    pub const BRAINROT: u32 = 1 << 21;
    pub const DOOMSCROLL: u32 = 1 << 22;
    pub const CLONE: u32 = 1 << 23;
    pub const LONG_REACH: u32 = 1 << 24;
    pub const AGREE: u32 = 1 << 25;

    pub fn has(self, flag: u32) -> bool {
        self.0 & flag != 0
    }
}

/// How a slop moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Movement {
    /// Straight at you.
    Rush,
    /// In bursts.
    Hop,
    /// Keeps between `near` and `far` from you.
    Kite,
    /// Aims, then throws an em-dash.
    Lance,
    /// Follows other slop to buff or heal them.
    Follow,
    /// Chases you and holds on.
    Grab,
    /// Rushes in a numbered pack.
    Pack,
    /// Does not move.
    Stand,
    Boss(BossKind),
}

/// What a slop fires at you.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attack {
    /// A fan of three speech bubbles.
    Fan,
    Toast,
    /// A big red arrow that drags you forward.
    Arrow,
    /// A row of posts that push you back.
    Wall,
    /// A bomb that lands where you stand after a warning.
    Bomb,
}

impl Attack {
    /// Ticks between attacks.
    pub fn interval(self) -> i32 {
        match self {
            Attack::Fan => 180,
            Attack::Toast => 150,
            Attack::Arrow => 170,
            Attack::Wall => 260,
            Attack::Bomb => 240,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlopLook {
    Archetype(Archetype),
    Brainrot { body: Body, object: Object, accessory: Accessory },
    Boss(BossKind),
}

#[derive(Clone, Debug)]
pub struct SlopSpec {
    pub name: String,
    pub look: SlopLook,
    pub role: Role,
    pub movement: Movement,
    pub attack: Option<Attack>,
    pub hp: f32,
    pub speed: f32,
    pub radius: f32,
    /// Damage when it touches you; also scales its attacks.
    pub contact: f32,
    pub likes: u32,
    pub resist: [f32; 4],
    pub traits: Traits,
    /// A viral elite: bigger, tougher, drops a Loot Box.
    pub elite: bool,
}

impl SlopSpec {
    pub fn archetype(a: Archetype) -> SlopSpec {
        let d = a.def();
        let (movement, attack, traits) = match a {
            Archetype::ChadGpt => (Movement::Kite, Some(Attack::Fan), 0),
            Archetype::Clawed => (Movement::Lance, None, 0),
            Archetype::YesBot => (Movement::Follow, None, Traits::AGREE),
            Archetype::Hallucinator => (Movement::Rush, None, Traits::FAKE_LIKES),
            Archetype::ListicleDrone => (Movement::Pack, None, 0),
            Archetype::EngagementBait => (Movement::Stand, None, Traits::BUFF_AURA | Traits::NO_KNOCKBACK),
            Archetype::GrandmaCake => (Movement::Follow, None, Traits::HEALER),
            Archetype::SixFingerHand => (Movement::Grab, None, 0),
            Archetype::ThoughtLeader => (Movement::Kite, Some(Attack::Wall), 0),
            Archetype::RecruiterBot => (Movement::Rush, None, Traits::SPLIT),
            Archetype::ClickbaitFace => (Movement::Kite, Some(Attack::Arrow), Traits::DOOMSCROLL),
            Archetype::SubscribePopup => (Movement::Rush, None, Traits::POPUP | Traits::NO_KNOCKBACK),
            Archetype::RepostBot => (Movement::Rush, None, Traits::RESPAWN),
            Archetype::KarmaFarmer => (Movement::Rush, None, Traits::GROWS | Traits::RATIO),
            Archetype::FilterInfluencer => (Movement::Rush, None, Traits::FRONT_SHIELD),
            Archetype::SurpriseEgg => (Movement::Stand, None, Traits::HATCH | Traits::NO_KNOCKBACK),
        };
        let chatbot = if a.is_chatbot() { Traits::CHATBOT } else { 0 };
        SlopSpec {
            name: d.name.to_string(),
            look: SlopLook::Archetype(a),
            role: d.role,
            movement,
            attack,
            hp: d.hp,
            speed: d.speed,
            radius: d.radius,
            contact: d.contact,
            likes: d.likes,
            resist: [0.0; 4],
            traits: Traits(traits | chatbot),
            elite: false,
        }
    }

    /// A generated brainrot from its three parts and two numeric genes (BRAIN-01).
    pub fn brainrot(name: String, body: Body, object: Object, accessory: Accessory, toughness: f32, pace: f32, resist: [f32; 4]) -> SlopSpec {
        let (mut hp, mut speed, mut radius) = body.base();
        hp *= toughness;
        speed *= pace;
        let mut contact = 6.0;
        let mut traits = Traits::BRAINROT;
        let mut role = Role::Swarmer;
        let movement = match body {
            Body::Frog => Movement::Hop,
            _ => Movement::Rush,
        };
        match body {
            Body::Crocodile => contact *= 1.6,
            Body::Pigeon => traits |= Traits::FLIES,
            Body::Elephant => {
                role = Role::Tank;
                contact *= 1.4;
            }
            Body::Cat => traits |= Traits::DODGE_FIRST,
            Body::Shrimp => contact *= 0.7,
            _ => {}
        }
        let mut attack = None;
        match object {
            Object::Airplane => attack = Some(Attack::Bomb),
            Object::Toaster => {
                attack = Some(Attack::Toast);
                role = Role::Shooter;
            }
            Object::Espresso => traits |= Traits::SPEED_AURA,
            Object::Cactus => {
                traits |= Traits::THORNS;
                contact *= 1.5;
            }
            Object::WashingMachine => traits |= Traits::SPIN_AURA,
            Object::Baguette => traits |= Traits::LONG_REACH,
            Object::TrafficCone => {
                traits |= Traits::NO_KNOCKBACK;
                hp *= 1.4;
                speed *= 0.8;
                radius *= 1.1;
            }
        }
        let mut resist = resist;
        match accessory {
            Accessory::Sunglasses => {
                // The type it already resists most gets stronger.
                let best = (0..4).max_by(|&a, &b| resist[a].total_cmp(&resist[b])).unwrap_or(0);
                resist[best] += 0.2;
            }
            Accessory::Sneakers => speed *= 1.3,
            Accessory::Crown => hp *= 1.5,
            Accessory::ChefHat => traits |= Traits::HEAL_ON_HIT,
            Accessory::Headphones => traits |= Traits::NO_KNOCKBACK,
            Accessory::GoldChain => traits |= Traits::DOUBLE_LIKES,
            Accessory::PartyHat => traits |= Traits::EXPLODE,
        }
        SlopSpec {
            name,
            look: SlopLook::Brainrot { body, object, accessory },
            role,
            movement,
            attack,
            hp,
            speed,
            radius,
            contact,
            likes: if role == Role::Tank { 3 } else { 1 },
            resist,
            traits: Traits(traits),
            elite: false,
        }
    }

    pub fn boss(kind: BossKind, hp: f32) -> SlopSpec {
        let (_, radius, speed) = kind.base();
        SlopSpec {
            name: kind.name().to_string(),
            look: SlopLook::Boss(kind),
            role: Role::Special,
            movement: Movement::Boss(kind),
            attack: None,
            hp,
            speed,
            radius,
            contact: 12.0,
            likes: 60,
            resist: [0.0; 4],
            traits: Traits(Traits::NO_KNOCKBACK),
            elite: false,
        }
    }

    /// Scaled for a later wave.
    pub fn scaled(mut self, hp: f32, damage: f32, speed: f32) -> SlopSpec {
        self.hp *= hp;
        self.contact *= damage;
        self.speed *= speed;
        self
    }

    /// A viral elite version: twice the size, three times the HP (SLOP-06).
    pub fn elite(mut self) -> SlopSpec {
        self.hp *= 3.0;
        self.radius *= 1.4;
        self.contact *= 1.5;
        self.likes = 10;
        self.elite = true;
        self.traits.0 |= Traits::NO_KNOCKBACK;
        self
    }

    pub fn is_chatbot(&self) -> bool {
        self.traits.has(Traits::CHATBOT)
    }

    pub fn is_brainrot(&self) -> bool {
        self.traits.has(Traits::BRAINROT)
    }
}
