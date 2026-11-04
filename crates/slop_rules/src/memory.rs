//! What the slop remembers about you between runs (NEMESIS-01), and the
//! Temperature you play at (TEMP-01). The rules read and write plain text;
//! the client keeps it in a file.

use slop_sim::content::{Accessory, Archetype, Body, Object};
use slop_sim::spec::{SlopLook, SlopSpec};

/// How wild the Feed runs (TEMP-01): the difficulty, in the model's own words.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Temperature {
    /// Calm and predictable: no surges.
    Low,
    #[default]
    Default,
    High,
    /// Everything at once, and some of what you see is not real.
    Unhinged,
}

impl Temperature {
    pub const ALL: [Temperature; 4] = [Temperature::Low, Temperature::Default, Temperature::High, Temperature::Unhinged];

    /// As the model's parameter: "0.2".
    pub fn label(self) -> &'static str {
        ["0.2", "0.7", "1.0", "1.5"][self as usize]
    }

    pub fn blurb(self) -> &'static str {
        [
            "Calm and predictable. Less slop, no surges. A good first run.",
            "As intended: surges of slop, a fair fight.",
            "More slop that hits harder, more Likes and Clout. Some Likes are hallucinated.",
            "Far more slop, far more loot, and a lot of what you see is not real.",
        ][self as usize]
    }

    /// Multiplier on how much slop spawns.
    pub fn spawns(self) -> f32 {
        [0.7, 1.0, 1.25, 1.6][self as usize]
    }

    /// Multiplier on the damage slop does.
    pub fn damage(self) -> f32 {
        [0.75, 1.0, 1.15, 1.3][self as usize]
    }

    /// Extra Likes and Clout, as a fraction.
    pub fn rewards(self) -> f32 {
        [-0.1, 0.0, 0.2, 0.5][self as usize]
    }

    /// Share of dropped Likes that are hallucinated.
    pub fn hallucination(self) -> f32 {
        [0.0, 0.0, 0.03, 0.08][self as usize]
    }

    /// Whether the crowd surges.
    pub fn surges(self) -> bool {
        self != Temperature::Low
    }
}

/// What the model keeps between runs.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Memory {
    /// Runs played.
    pub runs: u32,
    /// Longest run, in seconds.
    pub best_seconds: u32,
    /// The damage type you leaned on last run, as an index: it starts ahead
    /// on that one.
    pub favourite: Option<usize>,
    /// What deleted you last, by name.
    pub killer: Option<String>,
    /// What it looked like, if it can come back.
    pub killer_look: Option<SlopLook>,
    /// Times this same thing has deleted you.
    pub killer_count: u32,
    /// The best score (END-01).
    pub best_score: u32,
    /// Last run's hero and build (weapon index, level): the title demo
    /// replays it (END-01).
    pub hero: Option<usize>,
    pub build: Vec<(usize, u8)>,
}

impl Memory {
    /// The Nemesis this memory sends after you: what deleted you last, if it
    /// is something that can come back (NEMESIS-01).
    pub fn nemesis(&self) -> Option<SlopSpec> {
        let name = self.killer.as_ref()?;
        let mut spec = match self.killer_look? {
            SlopLook::Archetype(a) => SlopSpec::archetype(a),
            SlopLook::Brainrot { body, object, accessory } => SlopSpec::brainrot(name.clone(), body, object, accessory, 1.0, 1.0, [0.0; 4]),
            SlopLook::Boss(_) => return None,
        };
        spec.name = format!("Nemesis {name}");
        Some(spec)
    }

    /// As text, for the file.
    pub fn to_text(&self) -> String {
        let mut out = format!("runs={}
best={}
count={}
score={}
", self.runs, self.best_seconds, self.killer_count, self.best_score);
        if let Some(f) = self.favourite {
            out += &format!("favourite={f}\n");
        }
        if let Some(k) = &self.killer {
            out += &format!("killer={k}\n");
        }
        match self.killer_look {
            Some(SlopLook::Archetype(a)) => out += &format!("look=a:{}\n", a as usize),
            Some(SlopLook::Brainrot { body, object, accessory }) => out += &format!("look=b:{},{},{}\n", body as usize, object as usize, accessory as usize),
            _ => {}
        }
        if let Some(h) = self.hero {
            out += &format!("hero={h}\n");
        }
        if !self.build.is_empty() {
            let parts: Vec<String> = self.build.iter().map(|(d, l)| format!("{d}:{l}")).collect();
            out += &format!("build={}\n", parts.join(","));
        }
        out
    }

    /// From the file's text; anything it cannot read stays at its default.
    pub fn from_text(text: &str) -> Memory {
        let mut m = Memory::default();
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else { continue };
            match key {
                "runs" => m.runs = value.parse().unwrap_or(0),
                "best" => m.best_seconds = value.parse().unwrap_or(0),
                "count" => m.killer_count = value.parse().unwrap_or(0),
                "score" => m.best_score = value.parse().unwrap_or(0),
                "favourite" => m.favourite = value.parse().ok().filter(|f| *f < 4),
                "killer" => m.killer = Some(value.to_string()),
                "look" => m.killer_look = parse_look(value),
                "hero" => m.hero = value.parse().ok().filter(|h| *h < crate::Hero::ALL.len()),
                "build" => m.build = value.split(',').filter_map(|p| p.split_once(':')).filter_map(|(d, l)| Some((d.parse().ok()?, l.parse().ok()?))).collect(),
                _ => {}
            }
        }
        m
    }
}

fn parse_look(value: &str) -> Option<SlopLook> {
    let (kind, rest) = value.split_once(':')?;
    match kind {
        "a" => Some(SlopLook::Archetype(*Archetype::ALL.get(rest.parse::<usize>().ok()?)?)),
        "b" => {
            let mut parts = rest.split(',').map(|p| p.parse::<usize>().ok());
            let body: Body = *Body::ALL.get(parts.next()??)?;
            let object: Object = *Object::ALL.get(parts.next()??)?;
            let accessory: Accessory = *Accessory::ALL.get(parts.next()??)?;
            Some(SlopLook::Brainrot { body, object, accessory })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: NEMESIS-01
    #[test]
    fn memory_survives_the_file_and_nonsense_is_ignored() {
        let m = Memory {
            runs: 4,
            best_seconds: 700,
            favourite: Some(2),
            killer: Some("Glass Fruit".into()),
            killer_look: Some(SlopLook::Archetype(Archetype::GlassFruit)),
            killer_count: 2,
            best_score: 1234,
            hero: Some(3),
            build: vec![(2, 4), (7, 1)],
        };
        assert_eq!(Memory::from_text(&m.to_text()), m);
        assert_eq!(Memory::from_text("hero=99\nbuild=1:2,x:y,3:4").build, vec![(1, 2), (3, 4)]);
        let rot = Memory { killer: Some("X".into()), killer_look: Some(SlopLook::Brainrot { body: Body::Frog, object: Object::Cactus, accessory: Accessory::Crown }), ..Memory::default() };
        assert_eq!(Memory::from_text(&rot.to_text()), rot);
        assert_eq!(Memory::from_text("garbage\nrice=3\nfavourite=9\nlook=a:999"), Memory::default());
        assert_eq!(m.nemesis().unwrap().name, "Nemesis Glass Fruit");
        assert!(Memory::default().nemesis().is_none());
    }

    // REQ: TEMP-01
    #[test]
    fn hotter_means_more_slop_and_more_loot() {
        for pair in Temperature::ALL.windows(2) {
            assert!(pair[1].spawns() > pair[0].spawns() && pair[1].rewards() > pair[0].rewards());
        }
        assert!(!Temperature::Low.surges() && Temperature::Default.surges());
    }
}
