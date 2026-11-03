//! The eight heroes (HERO-01, HERO-02): the same human with different
//! baggage. Each starts with a weapon and an item, and each bends the rules
//! of a run a little. The hero screen lists every property exactly.

use crate::items::{Mod, StatKind};
use slop_sim::DamageType;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Hero {
    Boomer,
    Zoomer,
    Teacher,
    Programmer,
    Artist,
    Influencer,
    Intern,
    Moderator,
}

pub struct HeroDef {
    pub name: &'static str,
    pub weapon: &'static str,
    pub item: &'static str,
    pub mods: &'static [Mod],
    /// One line per property, as the hero screen shows it: "+" is good,
    /// "-" is bad, anything else is a rule.
    pub properties: &'static [&'static str],
    pub tagline: &'static str,
}

impl Hero {
    pub const ALL: [Hero; 8] = [Hero::Boomer, Hero::Zoomer, Hero::Teacher, Hero::Programmer, Hero::Artist, Hero::Influencer, Hero::Intern, Hero::Moderator];

    pub fn def(self) -> &'static HeroDef {
        &HEROES[self as usize]
    }
}

use StatKind::*;

const HEROES: [HeroDef; 8] = [
    HeroDef {
        name: "Boomer",
        weapon: "Newspaper Roll",
        item: "Weighted Blanket",
        mods: &[
            Mod::Stat(MentalHealth, 30.0),
            Mod::Stat(Composure, 3.0),
            Mod::Stat(Speed, -10.0),
            Mod::ChainLetter,
            Mod::TypeBonus(DamageType::Analog, 0.2),
            Mod::NoType(DamageType::Sarcasm),
            Mod::WaveIncome(3),
        ],
        properties: &[
            "+30 Mental Health, +3 Composure",
            "+20% Analog damage",
            "+3 Clout every minute (pension)",
            "Every 10 s a chain letter bounces through 5 slop",
            "Doesn't get memes: Sarcasm weapons are never offered",
            "-10% Speed",
        ],
        tagline: "Forwarded you this. Read to the end.",
    },
    HeroDef {
        name: "Zoomer",
        weapon: "Eye-Roll",
        item: "Energy Drink",
        mods: &[Mod::Stat(Speed, 15.0), Mod::Stat(MentalHealth, -20.0), Mod::Fluent, Mod::DashCharges(1), Mod::DashRecharge(-0.6), Mod::Stat(Focus, 15.0), Mod::Stat(Reach, -20.0)],
        properties: &[
            "+15% Speed, +15% Focus",
            "3 dash charges, and they come back 0.6 s faster",
            "Fluent: immune to Brainrot; pop-ups close twice as fast",
            "Short attention span: -20% Reach",
            "-20 Mental Health",
        ],
        tagline: "It's giving main character.",
    },
    HeroDef {
        name: "Teacher",
        weapon: "Red Pen",
        item: "Library Card",
        mods: &[Mod::TypeBonus(DamageType::Fact, 0.10), Mod::MarkBonus(0.15), Mod::EliteChests, Mod::WeaponSlots(-1)],
        properties: &[
            "+10% Fact damage",
            "See me after class: marked slop takes 15% more damage",
            "Grading: elites drop an extra chest",
            "Strict: only 5 weapon slots",
        ],
        tagline: "This will be on the test.",
    },
    HeroDef {
        name: "Programmer",
        weapon: "Mechanical Keyboard",
        item: "Coffee",
        mods: &[Mod::Stat(Focus, 5.0), Mod::FreeReroll, Mod::Preview, Mod::StartLevel(2), Mod::Stat(MentalHealth, -10.0), Mod::Stat(Curiosity, 40.0)],
        properties: &[
            "+5% Focus, +40 Curiosity",
            "Copy-paste: new weapons start at level 2",
            "One more free reroll on every level-up",
            "Reads the changelog: patch notes preview the next update",
            "-10 Mental Health (works from home)",
        ],
        tagline: "It works on my machine.",
    },
    HeroDef {
        name: "Artist",
        weapon: "Slingshot",
        item: "Journal",
        mods: &[Mod::Stat(Conviction, 10.0), Mod::BrainrotBonus(0.4), Mod::ExtraOffer, Mod::CloutBonus(-0.25)],
        properties: &[
            "+10% Conviction",
            "Hates AI art: +40% damage against brainrot",
            "One more choice on every level-up",
            "Starving artist: -25% Clout from coins",
        ],
        tagline: "I can draw hands, actually.",
    },
    HeroDef {
        name: "Influencer",
        weapon: "Side-Eye",
        item: "Clickbait Thumbnail",
        mods: &[Mod::CloutDamage(0.6), Mod::Stat(StatKind::Hustle, 25.0), Mod::Stat(MentalHealth, -15.0), Mod::Stat(Luck, 5.0)],
        properties: &[
            "Clout is power: +1% Conviction for every 10 Clout held (up to +60%)",
            "+25 Hustle: Clout every minute, and more coins",
            "+5 Luck",
            "-15 Mental Health",
        ],
        tagline: "Link in bio.",
    },
    HeroDef {
        name: "Intern",
        weapon: "Stapler",
        item: "Running Shoes",
        mods: &[Mod::XpBonus(0.3), Mod::ExtraOffer, Mod::Stat(Conviction, -10.0), Mod::ShopPriceUp(0.25)],
        properties: &[
            "Eager: +30% experience",
            "One more choice on every level-up",
            "-10% Conviction",
            "Unpaid: machines cost 25% more",
        ],
        tagline: "Happy to help! Is this paid?",
    },
    HeroDef {
        name: "Moderator",
        weapon: "Ban Hammer",
        item: "Block Button",
        mods: &[Mod::DashDamage(30.0), Mod::DashShock, Mod::Stat(Boundaries, 25.0), Mod::Stat(MentalHealth, -20.0), Mod::Stat(Speed, -5.0)],
        properties: &[
            "Power trip: dashes hit slop for 30 and end in a shockwave",
            "+25 Boundaries",
            "-20 Mental Health",
            "-5% Speed (unpaid volunteer)",
        ],
        tagline: "Locked. Removed. Banned.",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: HERO-01, HERO-02
    #[test]
    fn heroes_start_with_real_weapons_and_items_and_list_their_rules() {
        for h in Hero::ALL {
            let d = h.def();
            assert!(slop_sim::content::weapon_index(d.weapon).is_some(), "{}", d.weapon);
            assert!(crate::items::item_index(d.item).is_some(), "{}", d.item);
            assert!(d.properties.len() >= 3, "{}", d.name);
            // Every hero bends at least one rule beyond plain stats.
            assert!(d.mods.iter().any(|m| !matches!(m, Mod::Stat(..))), "{}", d.name);
        }
    }
}
