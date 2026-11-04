//! The five chapters of a run (CH-01, CH-02): what you are, what you protect,
//! what the AI is, and the words for each. The core loop never changes; the
//! chapter changes what it means. See docs/design-v6.md.

use slop_sim::content::{BossKind, Platform};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Chapter {
    #[default]
    Feed,
    Office,
    Boardroom,
    Campaign,
    Resistance,
}

pub struct ChapterDef {
    pub name: &'static str,
    /// What the AI is called this chapter, for the capability version.
    pub ai: &'static str,
    /// The health bar's name, and the word for losing it.
    pub resource: &'static str,
    pub death: &'static str,
    /// Colour of the resource bar when full, as RGB.
    pub color: [f32; 3],
    pub weapon: &'static str,
    /// The card at the start: a title and a few lines.
    pub card: &'static [&'static str],
    /// How a Delegate card is framed.
    pub delegate: &'static str,
    /// The biomes of the chapter's map.
    pub biomes: &'static [Platform],
    /// When the chapter's boss first arrives, in seconds, and the boss.
    pub boss_at: f32,
    pub boss: BossKind,
    /// What the slop says.
    pub lines: &'static [&'static str],
    /// What the patch notes sound like: one line said after retraining.
    pub patch_tone: &'static str,
    /// Minutes of capability the chapter starts with: later chapters are
    /// harder from the first second.
    pub head_start: f32,
}

impl Chapter {
    pub const ALL: [Chapter; 5] = [Chapter::Feed, Chapter::Office, Chapter::Boardroom, Chapter::Campaign, Chapter::Resistance];

    pub fn def(self) -> &'static ChapterDef {
        &CHAPTERS[self as usize]
    }

    pub fn next(self) -> Option<Chapter> {
        Chapter::ALL.get(self as usize + 1).copied()
    }

    pub fn number(self) -> usize {
        self as usize + 1
    }
}

const CHAPTERS: [ChapterDef; 5] = [
    ChapterDef {
        name: "The Feed",
        ai: "Slop",
        resource: "Mental Health",
        death: "BURNOUT",
        color: [0.3, 0.8, 0.4],
        weapon: "Rock",
        card: &[
            "You are a person on your phone.",
            "The feed is full of slop, and the slop is learning what you like.",
            "Protect your Mental Health. Every choice you make trains it.",
        ],
        delegate: "Let the algorithm handle it",
        biomes: &Platform::ALL,
        boss_at: 180.0,
        boss: BossKind::CrustaceanSaviour,
        lines: &["Great question!", "Let's delve in.", "I hope this helps!", "As a large language model,"],
        patch_tone: "Fixed a bug where users touched grass.",
        head_start: 0.0,
    },
    ChapterDef {
        name: "The Office",
        ai: "Copilot",
        resource: "Job Security",
        death: "FIRED",
        color: [0.35, 0.65, 1.0],
        weapon: "Stapler",
        card: &[
            "Congratulations on the promotion.",
            "Your new colleague is a model. It does not sleep, and it is cheaper than you.",
            "Keep your Job Security. Every ticket you hand it, it learns.",
        ],
        delegate: "Let Copilot take this ticket",
        biomes: &Platform::ALL,
        boss_at: 360.0,
        boss: BossKind::HustleGuru,
        lines: &["Per my last message,", "Circling back.", "Let's take this offline.", "Quick sync?"],
        patch_tone: "Copilot now completes your sentences. And your tasks.",
        head_start: 2.0,
    },
    ChapterDef {
        name: "The Boardroom",
        ai: "Algorithm",
        resource: "Shareholder Confidence",
        death: "VOTED OUT",
        color: [0.95, 0.78, 0.3],
        weapon: "Ban Hammer",
        card: &[
            "You made CEO. The Algorithm made the shortlist.",
            "The shareholders want efficiency. Your executive team wants to keep their jobs.",
            "Keep Shareholder Confidence. Every exec you optimize, the board remembers.",
        ],
        delegate: "Optimize it",
        biomes: &Platform::ALL,
        boss_at: 360.0,
        boss: BossKind::ContentFarm,
        lines: &["Synergies identified.", "Let's double-click on that.", "Per the roadmap,", "Headcount is a lever."],
        patch_tone: "The Algorithm now attends meetings on your behalf. Attendance is up 400%.",
        head_start: 4.0,
    },
    ChapterDef {
        name: "The Campaign",
        ai: "Candidate",
        resource: "Approval",
        death: "CONCEDED",
        color: [0.9, 0.35, 0.35],
        weapon: "Vuvuzela",
        card: &[
            "You are running for office. So is it.",
            "Its promises are better than yours, and it keeps them.",
            "Keep your Approval. Every promise you make comes due.",
        ],
        delegate: "Let the model draft the policy",
        biomes: &Platform::ALL,
        boss_at: 360.0,
        boss: BossKind::Influencer,
        lines: &["I hear you.", "Together.", "A plan for every family.", "The data is clear."],
        patch_tone: "The Candidate now remembers every voter's name. Yours too.",
        head_start: 6.0,
    },
    ChapterDef {
        name: "The Resistance",
        ai: "State",
        resource: "Signal",
        death: "LOCATED",
        color: [0.75, 0.55, 1.0],
        weapon: "Rock",
        card: &[
            "It governs now. Your agents work for it.",
            "Everything you automated is looking for you.",
            "Keep your Signal. Keep moving. There is no winning this one. There never was.",
        ],
        delegate: "Let the network route you",
        biomes: &Platform::ALL,
        boss_at: 420.0,
        boss: BossKind::TheModel,
        lines: &["Compliance is kindness.", "You are safe now.", "Please remain where you are.", "This is for your benefit."],
        patch_tone: "Removed: the user.",
        head_start: 8.0,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: CH-02
    #[test]
    fn every_chapter_is_complete_and_in_order() {
        for (k, c) in Chapter::ALL.iter().enumerate() {
            let d = c.def();
            assert!(slop_sim::content::weapon_index(d.weapon).is_some(), "{}", d.name);
            assert!(d.card.len() >= 2 && !d.lines.is_empty() && !d.biomes.is_empty(), "{}", d.name);
            assert_eq!(c.number(), k + 1);
        }
        assert_eq!(Chapter::Resistance.next(), None);
        assert_eq!(Chapter::Feed.next(), Some(Chapter::Office));
    }
}
