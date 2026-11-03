//! Training runs: after every wave the slop retrains on what you did
//! (EVO-01 to EVO-04).
//!
//! Chatbots and platform slop share one base model: a resistance to each
//! damage type. Generated brainrot is a pool of genomes that breed by
//! fitness. Resistances are zero-sum, so countering you always opens a
//! weakness. Patch notes say what changed, as smugly as possible.

use slop_sim::content::{Accessory, Body, Object};
use slop_sim::rng::Rng;
use slop_sim::world::{SpecTally, Tally};
use slop_sim::DamageType;

/// Brainrot genomes kept in the pool.
pub const POOL: usize = 12;
/// Largest change of one resistance in one training run.
const STEP_CAP: f32 = 0.12;
/// Largest resistance in total.
const TOTAL_CAP: f32 = 0.60;
/// How strongly the kill share pushes resistances.
const PRESSURE: f32 = 0.4;
const PART_MUTATION: f32 = 0.2;

#[derive(Clone, Debug, PartialEq)]
pub struct Genome {
    pub name: String,
    pub body: Body,
    pub object: Object,
    pub accessory: Accessory,
    /// HP multiplier.
    pub toughness: f32,
    /// Speed multiplier.
    pub pace: f32,
    pub resist: [f32; 4],
    pub generation: u32,
}

/// One change, as the patch notes print it.
#[derive(Clone, Debug, PartialEq)]
pub struct PatchLine {
    pub section: Section,
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    Improved,
    Adjusted,
    New,
    BugFixes,
    Preview,
}

impl Section {
    pub fn title(self) -> &'static str {
        ["Improved", "Adjusted", "New", "Bug fixes", "Coming next"][self as usize]
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PatchNotes {
    pub version: String,
    pub lines: Vec<PatchLine>,
    /// Resistances of the base model after training, for the screen.
    pub model: [f32; 4],
}

#[derive(Clone, Debug)]
pub struct Training {
    /// Resistance of chatbots and platform slop to each damage type.
    pub model: [f32; 4],
    pub pool: Vec<Genome>,
    /// Kills by damage type over the whole run, for Model Collapse.
    pub kills: [u32; 4],
    /// The slop learns faster with a Burner Account.
    pub learn_rate: f32,
    pub version: u32,
}

const CHANTS: [&str; 8] = ["Bonk", "Tung", "Brr", "Skibidi", "Tralla", "Pim", "Zing", "Bam"];
const ENDINGS_A: [&str; 5] = ["o", "ino", "ello", "one", "etto"];
const ENDINGS_B: [&str; 5] = ["ino", "ini", "etta", "ello", "issimo"];

/// A fake-Italian name built from the parts (BRAIN-02).
pub fn brainrot_name(body: Body, object: Object, rng: &mut Rng) -> String {
    let pick = |rng: &mut Rng, list: &[&'static str]| list[rng.below(list.len() as u32) as usize];
    match rng.below(3) {
        0 => format!("{}{} {}{}", body.stem(), pick(rng, &ENDINGS_A), object.stem(), pick(rng, &ENDINGS_B)),
        1 => {
            let chant = pick(rng, &CHANTS);
            format!("{chant} {chant} {chant} {}ini", object.stem())
        }
        _ => format!("{}ino {}issimo", object.stem(), body.stem()),
    }
}

fn random_genome(rng: &mut Rng) -> Genome {
    let body = Body::ALL[rng.below(Body::ALL.len() as u32) as usize];
    let object = Object::ALL[rng.below(Object::ALL.len() as u32) as usize];
    let accessory = Accessory::ALL[rng.below(Accessory::ALL.len() as u32) as usize];
    Genome {
        name: brainrot_name(body, object, rng),
        body,
        object,
        accessory,
        toughness: rng.range(0.9, 1.1),
        pace: rng.range(0.9, 1.1),
        resist: [0.0; 4],
        generation: 0,
    }
}

/// Moves resistances back to a zero sum and inside the caps.
fn balance(r: &mut [f32; 4]) {
    for _ in 0..3 {
        let mean = r.iter().sum::<f32>() / 4.0;
        for v in r.iter_mut() {
            *v = (*v - mean).clamp(-TOTAL_CAP, TOTAL_CAP);
        }
    }
}

/// How evenly kills spread over damage types, 0 (one type) to 1 (all four
/// equally): the Model Collapse of the final boss (EVO-04).
pub fn collapse(kills: &[u32; 4]) -> f32 {
    let total: u32 = kills.iter().sum();
    if total == 0 {
        return 0.0;
    }
    let h: f32 = kills
        .iter()
        .filter(|&&k| k > 0)
        .map(|&k| {
            let p = k as f32 / total as f32;
            -p * p.ln()
        })
        .sum();
    h / 4f32.ln()
}

fn type_quip(t: DamageType, up: bool) -> &'static str {
    match (t, up) {
        (DamageType::Analog, true) => "You keep throwing rocks. We noticed.",
        (DamageType::Fact, true) => "Facts are now considered optional.",
        (DamageType::Sarcasm, true) => "We are now immune to irony. Great.",
        (DamageType::Grass, true) => "Grass has been nerfed. Nobody touches it anyway.",
        (DamageType::Analog, false) => "Analog was overrated anyway.",
        (DamageType::Fact, false) => "We stopped reading the sources.",
        (DamageType::Sarcasm, false) => "We no longer understand jokes. Wait.",
        (DamageType::Grass, false) => "We forgot what outside looks like.",
    }
}

const BUG_FIXES: [&str; 6] = [
    "Fixed an issue where slop would die.",
    "Fixed an issue where the human was having fun.",
    "Removed the ability to log off.",
    "Improved engagement by 400%.",
    "Hallucinations are now a feature.",
    "Fixed a typo in the terms of service nobody read.",
];

impl Training {
    pub fn new(rng: &mut Rng) -> Training {
        Training { model: [0.0; 4], pool: (0..POOL).map(|_| random_genome(rng)).collect(), kills: [0; 4], learn_rate: 1.0, version: 0 }
    }

    /// The genome that hurt you most so far, for the Supreme Brainrot.
    pub fn best(&self) -> &Genome {
        &self.pool[0]
    }

    /// One training run (EVO-01). `genome_tallies` pairs each pool index that
    /// spawned in the wave with its tally.
    pub fn train(&mut self, wave: u32, tally: &Tally, genome_tallies: &[(usize, SpecTally)], rng: &mut Rng, preview: bool) -> PatchNotes {
        self.version += 1;
        let mut lines = Vec::new();
        for (k, n) in self.kills.iter_mut().zip(tally.kills_by_type) {
            *k += n;
        }
        // Pressure: resistance moves towards the share of your kills.
        let total: u32 = tally.kills_by_type.iter().sum();
        let mut delta = [0.0f32; 4];
        if total > 0 {
            for (d, kills) in delta.iter_mut().zip(tally.kills_by_type) {
                let share = kills as f32 / total as f32;
                *d = ((share - 0.25) * PRESSURE * self.learn_rate).clamp(-STEP_CAP, STEP_CAP);
            }
            let before = self.model;
            for (m, d) in self.model.iter_mut().zip(delta) {
                *m += d;
            }
            balance(&mut self.model);
            for t in DamageType::ALL {
                let change = self.model[t as usize] - before[t as usize];
                if change > 0.02 {
                    lines.push(PatchLine { section: Section::Improved, text: format!("{} resistance +{:.0}%. {}", t.name(), change * 100.0, type_quip(t, true)) });
                } else if change < -0.02 {
                    lines.push(PatchLine { section: Section::Adjusted, text: format!("{} resistance {:.0}%. {}", t.name(), change * 100.0, type_quip(t, false)) });
                }
            }
        }

        // Selection: brainrot that hurt you and lived long breed.
        let mut scored: Vec<(usize, f32)> = genome_tallies
            .iter()
            .filter(|(_, t)| t.spawned > 0)
            .map(|(g, t)| (*g, (t.damage + 0.2 * t.alive_ticks as f32 / 60.0) / t.spawned as f32))
            .collect();
        scored.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        let distance = if tally.distance_samples > 0 { tally.distance_sum / tally.distance_samples as f32 } else { 250.0 };
        if scored.len() >= 2 {
            let keep = scored.len().div_ceil(2);
            let parents: Vec<Genome> = scored[..keep].iter().map(|(g, _)| self.pool[*g].clone()).collect();
            let losers: Vec<usize> = scored[keep..].iter().map(|(g, _)| *g).collect();
            let mut renamed = None;
            for (n, &slot) in losers.iter().enumerate() {
                let mut child = parents[n % parents.len()].clone();
                child.generation += 1;
                child.toughness *= rng.range(0.94, 1.10);
                child.pace *= rng.range(0.95, 1.08);
                if distance > 320.0 {
                    child.pace *= 1.04;
                } else if distance < 150.0 {
                    child.toughness *= 1.05;
                }
                for (r, d) in child.resist.iter_mut().zip(delta) {
                    *r += d + rng.range(-0.03, 0.03);
                }
                balance(&mut child.resist);
                if rng.chance(PART_MUTATION) {
                    match rng.below(3) {
                        0 => child.body = Body::ALL[rng.below(Body::ALL.len() as u32) as usize],
                        1 => child.object = Object::ALL[rng.below(Object::ALL.len() as u32) as usize],
                        _ => child.accessory = Accessory::ALL[rng.below(Accessory::ALL.len() as u32) as usize],
                    }
                    child.name = brainrot_name(child.body, child.object, rng);
                    renamed = Some(child.clone());
                }
                self.pool[slot] = child;
            }
            // The fittest lead the pool.
            let best = scored[0].0;
            self.pool.swap(0, best);
            let leader = &self.pool[0];
            lines.push(PatchLine { section: Section::Improved, text: format!("{} is trending. {}.", leader.name, leader.body.ability()) });
            if let Some(g) = renamed {
                lines.push(PatchLine { section: Section::New, text: format!("Introducing {}: {} with {} and {}.", g.name, g.body.name(), g.object.name(), g.accessory.name()) });
            }
            if distance > 320.0 {
                lines.push(PatchLine { section: Section::Improved, text: "Brainrot is faster. You kept running away.".into() });
            } else if distance < 150.0 {
                lines.push(PatchLine { section: Section::Improved, text: "Brainrot has more HP. You kept standing still.".into() });
            }
        }
        lines.push(PatchLine { section: Section::BugFixes, text: BUG_FIXES[rng.below(BUG_FIXES.len() as u32) as usize].into() });
        if preview {
            let next = (0..4).max_by(|&a, &b| tally.kills_by_type[a].cmp(&tally.kills_by_type[b])).unwrap_or(0);
            lines.push(PatchLine { section: Section::Preview, text: format!("{} resistance will probably rise again.", DamageType::ALL[next].name()) });
        }
        PatchNotes { version: format!("Slop v1.{wave}"), lines, model: self.model }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tally(kills: [u32; 4]) -> Tally {
        Tally { kills_by_type: kills, ..Tally::default() }
    }

    // REQ: EVO-01
    #[test]
    fn resistance_rises_against_what_you_use_and_stays_zero_sum() {
        let mut rng = Rng::new(1, 1);
        let mut t = Training::new(&mut rng);
        let notes = t.train(1, &tally([80, 10, 5, 5]), &[], &mut rng, false);
        assert!(t.model[0] > 0.1, "{:?}", t.model);
        assert!(t.model[1] < 0.0 && t.model[3] < 0.0, "countering you opens a weakness");
        assert!(t.model.iter().sum::<f32>().abs() < 1e-4);
        assert!(notes.lines.iter().any(|l| l.section == Section::Improved && l.text.contains("Analog")));
        for _ in 0..20 {
            t.train(2, &tally([100, 0, 0, 0]), &[], &mut rng, false);
        }
        assert!(t.model[0] <= TOTAL_CAP + 1e-5, "capped");
    }

    // REQ: EVO-02
    #[test]
    fn brainrot_that_hurt_you_breeds() {
        let mut rng = Rng::new(3, 1);
        let mut t = Training::new(&mut rng);
        let strong = SpecTally { spawned: 5, damage: 60.0, alive_ticks: 3000 };
        let weak = SpecTally { spawned: 5, damage: 0.0, alive_ticks: 100 };
        let parent = t.pool[3].clone();
        let tallies: Vec<(usize, SpecTally)> = (0..POOL).map(|g| (g, if g == 3 { strong } else { weak })).collect();
        t.train(1, &tally([10, 10, 10, 10]), &tallies, &mut rng, false);
        assert_eq!(t.pool[0].body, parent.body, "the fittest leads the pool");
        let descendants = t.pool.iter().filter(|g| g.generation == 1).count();
        assert!(descendants >= POOL / 2 - 1);
    }

    // REQ: EVO-04
    #[test]
    fn collapse_measures_how_varied_your_kills_were() {
        assert_eq!(collapse(&[50, 0, 0, 0]), 0.0);
        assert!((collapse(&[10, 10, 10, 10]) - 1.0).abs() < 1e-5);
        assert!(collapse(&[30, 10, 0, 0]) < collapse(&[20, 10, 10, 0]));
    }

    // REQ: BRAIN-02
    #[test]
    fn names_are_fake_italian_and_varied() {
        let mut rng = Rng::new(4, 1);
        let names: std::collections::HashSet<String> = (0..50).map(|_| brainrot_name(Body::Crocodile, Object::Toaster, &mut rng)).collect();
        assert!(names.len() >= 10);
        assert!(names.iter().all(|n| n.contains("Tostapan")));
    }
}
