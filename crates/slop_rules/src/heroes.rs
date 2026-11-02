//! The five heroes: the same human with a different starting loadout and a
//! quirk (HERO-01). All are unlocked from the start.

use crate::items::{Mod, StatKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Hero {
    Boomer,
    Zoomer,
    Teacher,
    Programmer,
    Artist,
}

pub struct HeroDef {
    pub name: &'static str,
    pub weapon: &'static str,
    pub item: &'static str,
    pub mods: &'static [Mod],
    pub stats: &'static str,
    pub quirk: &'static str,
}

impl Hero {
    pub const ALL: [Hero; 5] = [Hero::Boomer, Hero::Zoomer, Hero::Teacher, Hero::Programmer, Hero::Artist];

    pub fn def(self) -> &'static HeroDef {
        &HEROES[self as usize]
    }
}

const HEROES: [HeroDef; 5] = [
    HeroDef {
        name: "Boomer",
        weapon: "Newspaper Roll",
        item: "Weighted Blanket",
        mods: &[Mod::Stat(StatKind::MentalHealth, 30.0), Mod::Stat(StatKind::Composure, 3.0), Mod::Stat(StatKind::Speed, -10.0), Mod::ChainLetter],
        stats: "+30 Mental Health, +3 Composure, -10% Speed",
        quirk: "Shares everything: every 10 s a chain letter bounces through 5 slop.",
    },
    HeroDef {
        name: "Zoomer",
        weapon: "Eye-Roll",
        item: "Energy Drink",
        mods: &[Mod::Stat(StatKind::Speed, 15.0), Mod::Stat(StatKind::MentalHealth, -20.0), Mod::Fluent],
        stats: "+15% Speed, -20 Mental Health",
        quirk: "Fluent: immune to Brainrot. Pop-ups close twice as fast.",
    },
    HeroDef {
        name: "Teacher",
        weapon: "Red Pen",
        item: "Library Card",
        mods: &[Mod::TypeBonus(slop_sim::DamageType::Fact, 0.10), Mod::MarkBonus(0.15)],
        stats: "+10% Fact damage",
        quirk: "See me after class: marked slop takes 15% more damage.",
    },
    HeroDef {
        name: "Programmer",
        weapon: "Mechanical Keyboard",
        item: "Coffee",
        mods: &[Mod::Stat(StatKind::Focus, 5.0), Mod::FreeReroll, Mod::Preview],
        stats: "+5% Focus",
        quirk: "Reads the changelog: patch notes preview the next update. One free reroll per shop.",
    },
    HeroDef {
        name: "Artist",
        weapon: "Slingshot",
        item: "Journal",
        mods: &[Mod::Stat(StatKind::Conviction, 10.0), Mod::BrainrotBonus(0.25), Mod::ExtraOffer],
        stats: "+10% Conviction",
        quirk: "Hates AI art: +25% damage against brainrot. The shop shows one extra offer.",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: HERO-01
    #[test]
    fn heroes_start_with_real_weapons_and_items() {
        for h in Hero::ALL {
            let d = h.def();
            assert!(slop_sim::content::weapon_index(d.weapon).is_some(), "{}", d.weapon);
            assert!(crate::items::item_index(d.item).is_some(), "{}", d.item);
        }
    }
}
