//! Procedural music (MUSIC-01): one loop per chapter, written as notes and
//! rendered at startup; Automation quantizes it. At 0% a person is playing:
//! the timing drifts, the velocity varies. At 100% the grid plays itself.
//! A file in `assets/music/` replaces a chapter's loop (ASSET-01).

use bevy::audio::{AudioSink, AudioSinkPlayback, AudioSource, PlaybackSettings, Volume};
use bevy::prelude::*;

use crate::assets::Overrides;
use crate::state::{Game, Screen};

const RATE: u32 = 22_050;
/// Loudness of the music under the sounds.
const MUSIC: f32 = 0.16;
/// Seconds a track takes to fade in or out when the loop changes.
const FADE: f32 = 1.5;
const BARS: u32 = 8;
const BEATS: u32 = 4;

pub struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Playing>().add_systems(Update, music);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wave {
    Sine,
    Square,
    Triangle,
    Noise,
}

/// One note of a loop: when, how long, how high, how loud, on what.
#[derive(Clone, Debug, PartialEq)]
pub struct Note {
    pub start: f32,
    pub secs: f32,
    pub freq: f32,
    pub gain: f32,
    pub wave: Wave,
}

/// A chapter's sound: tempo, root, scale and the feel of the lead.
struct Profile {
    bpm: f32,
    root: f32,
    scale: &'static [i32],
    bass: Wave,
    lead: Wave,
    /// Snare on two and four.
    march: bool,
    /// A held bass note instead of a bouncing one.
    drone: bool,
}

const PROFILES: [Profile; 5] = [
    // The Feed: bright, fast, pentatonic. A notification jingle that never ends.
    Profile { bpm: 128.0, root: 57.0, scale: &[0, 2, 4, 7, 9], bass: Wave::Square, lead: Wave::Sine, march: false, drone: false },
    // The Office: hold music.
    Profile { bpm: 112.0, root: 50.0, scale: &[0, 2, 3, 5, 7, 9, 10], bass: Wave::Triangle, lead: Wave::Sine, march: false, drone: false },
    // The Boardroom: slow and low.
    Profile { bpm: 100.0, root: 48.0, scale: &[0, 2, 3, 5, 7, 8, 10], bass: Wave::Square, lead: Wave::Triangle, march: false, drone: false },
    // The Campaign: a march.
    Profile { bpm: 124.0, root: 55.0, scale: &[0, 2, 4, 5, 7, 9, 11], bass: Wave::Square, lead: Wave::Square, march: true, drone: false },
    // The Resistance: a drone and a few notes in the dark.
    Profile { bpm: 88.0, root: 40.0, scale: &[0, 1, 3, 5, 7, 8, 10], bass: Wave::Sine, lead: Wave::Triangle, march: false, drone: true },
];

/// A small deterministic generator, so a loop is the same every start.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 33) as f32) / (1u64 << 31) as f32
    }

    fn below(&mut self, n: usize) -> usize {
        ((self.next() * n as f32) as usize).min(n - 1)
    }
}

fn midi(n: f32) -> f32 {
    440.0 * 2f32.powf((n - 69.0) / 12.0)
}

/// Seconds one loop lasts.
pub fn loop_secs(chapter: usize) -> f32 {
    let p = &PROFILES[chapter.min(4)];
    (BARS * BEATS) as f32 * 60.0 / p.bpm
}

/// The notes of chapter `chapter`'s loop at Automation bucket `bucket`
/// (0 to 4: under 25%, under 50%, under 75%, under 100%, automated).
pub fn score(chapter: usize, bucket: u32) -> Vec<Note> {
    let p = &PROFILES[chapter.min(4)];
    let bucket = bucket.min(4);
    // How much of a person is left in the playing.
    let human = 1.0 - bucket as f32 / 4.0;
    let beat = 60.0 / p.bpm;
    let sixteenth = beat / 4.0;
    let mut rng = Lcg(0x5EED + chapter as u64 * 97 + bucket as u64 * 7919);
    let mut notes = Vec::new();
    let jitter = |rng: &mut Lcg| (rng.next() - 0.5) * 0.06 * human;
    let velocity = |rng: &mut Lcg| 1.0 - rng.next() * 0.35 * human;
    // A chord progression over the bars: degrees of the scale.
    let degrees = [0usize, 3, 4, 0, 5, 3, 4, 1];
    for bar in 0..BARS {
        let bar_at = bar as f32 * BEATS as f32 * beat;
        let degree = degrees[bar as usize % degrees.len()] % p.scale.len();
        let chord_root = p.root + p.scale[degree] as f32;
        // Bass: a held note, or one on every beat.
        if p.drone {
            notes.push(Note { start: bar_at, secs: BEATS as f32 * beat, freq: midi(chord_root - 12.0), gain: 0.22, wave: p.bass });
        } else {
            for b in 0..BEATS {
                let at = bar_at + b as f32 * beat + jitter(&mut rng);
                let octave = if b % 2 == 1 && bucket >= 2 { 0.0 } else { -12.0 };
                notes.push(Note { start: at.max(0.0), secs: beat * 0.8, freq: midi(chord_root + octave), gain: 0.2 * velocity(&mut rng), wave: p.bass });
            }
        }
        // Drums: a kick on the beat, hats on the eighths, a snare for a march.
        for b in 0..BEATS {
            let at = bar_at + b as f32 * beat;
            notes.push(Note { start: at, secs: 0.09, freq: 60.0, gain: 0.3, wave: Wave::Sine });
            if p.march && b % 2 == 1 {
                notes.push(Note { start: at, secs: 0.08, freq: 0.0, gain: 0.18, wave: Wave::Noise });
            }
            for e in 0..2 {
                let hat = at + e as f32 * beat / 2.0 + jitter(&mut rng) * 0.5;
                notes.push(Note { start: hat.max(0.0), secs: 0.03, freq: 0.0, gain: 0.05 + 0.03 * bucket as f32, wave: Wave::Noise });
            }
        }
        // Lead: a person noodles on the eighths; the machine arpeggiates
        // every sixteenth, exactly on the grid.
        if bucket >= 4 {
            for s in 0..(BEATS * 4) {
                let k = (s as usize + bar as usize) % 4;
                let step = [0, 2, 4, 2][k];
                let n = p.root + 12.0 + p.scale[(degree + step) % p.scale.len()] as f32 + if (degree + step) >= p.scale.len() { 12.0 } else { 0.0 };
                notes.push(Note { start: bar_at + s as f32 * sixteenth, secs: sixteenth * 0.9, freq: midi(n), gain: 0.12, wave: Wave::Square });
            }
        } else {
            let density = 0.45 + 0.12 * bucket as f32;
            let mut pos = degree;
            for e in 0..(BEATS * 2) {
                if rng.next() > density {
                    continue;
                }
                let walk = [-1i32, 0, 1, 2][rng.below(4)];
                pos = (pos as i32 + walk).rem_euclid(p.scale.len() as i32) as usize;
                let n = p.root + 12.0 + p.scale[pos] as f32;
                let at = bar_at + e as f32 * beat / 2.0 + jitter(&mut rng);
                let long = rng.next() < 0.3;
                notes.push(Note { start: at.max(0.0), secs: if long { beat } else { beat * 0.45 }, freq: midi(n), gain: 0.14 * velocity(&mut rng), wave: p.lead });
            }
        }
    }
    notes
}

/// The loop as samples, `secs` long, ready to repeat.
pub fn render(notes: &[Note], secs: f32) -> Vec<f32> {
    let len = (secs * RATE as f32) as usize;
    let mut out = vec![0.0f32; len];
    let mut noise: u32 = 0x9E37_79B9;
    for n in notes {
        let first = (n.start * RATE as f32) as usize;
        let count = (n.secs * RATE as f32) as usize;
        let mut phase = 0.0f32;
        for i in 0..count {
            let t = i as f32 / count.max(1) as f32;
            phase += n.freq / RATE as f32;
            let s = match n.wave {
                Wave::Sine => (phase * std::f32::consts::TAU).sin(),
                Wave::Square => {
                    if phase.fract() < 0.5 {
                        0.5
                    } else {
                        -0.5
                    }
                }
                Wave::Triangle => 4.0 * (phase.fract() - 0.5).abs() - 1.0,
                Wave::Noise => {
                    noise ^= noise << 13;
                    noise ^= noise >> 17;
                    noise ^= noise << 5;
                    (noise as f32 / u32::MAX as f32) * 2.0 - 1.0
                }
            };
            let attack = (t * 30.0).min(1.0);
            let decay = (1.0 - t).powf(1.4);
            // The loop wraps: a note over the end lands at the start.
            let k = (first + i) % len.max(1);
            out[k] += s * n.gain * attack * decay;
        }
    }
    let peak = out.iter().fold(0.0f32, |m, s| m.max(s.abs())).max(1e-6);
    if peak > 0.9 {
        for s in &mut out {
            *s *= 0.9 / peak;
        }
    }
    out
}

/// 16-bit mono WAV bytes.
pub fn wav(samples: &[f32]) -> Vec<u8> {
    use std::io::Write;
    let data_len = (samples.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + data_len as usize);
    let mut put = |b: &[u8]| out.write_all(b).expect("writing to a Vec cannot fail");
    put(b"RIFF");
    put(&(36 + data_len).to_le_bytes());
    put(b"WAVEfmt ");
    put(&16u32.to_le_bytes());
    put(&1u16.to_le_bytes());
    put(&1u16.to_le_bytes());
    put(&RATE.to_le_bytes());
    put(&(RATE * 2).to_le_bytes());
    put(&2u16.to_le_bytes());
    put(&16u16.to_le_bytes());
    put(b"data");
    put(&data_len.to_le_bytes());
    for s in samples {
        put(&((s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16).to_le_bytes());
    }
    out
}

/// The loop playing now, by chapter and Automation bucket.
#[derive(Resource, Default)]
struct Playing {
    key: Option<(usize, u32)>,
}

/// A looping track, fading in, or out and away.
#[derive(Component)]
struct Track {
    fading: bool,
}

/// The chapter and Automation bucket the music should follow right now.
fn wanted(game: &Game) -> (usize, u32) {
    let run = game.shown_run();
    let chapter = run.map_or(0, |r| r.chapter as usize);
    let bucket = match game.screen {
        Screen::Run => run.map_or(0, |r| (r.automation / 25).min(4)),
        _ => 0,
    };
    (chapter, bucket)
}

fn music(mut commands: Commands, game: Res<Game>, overrides: Res<Overrides>, mut sources: ResMut<Assets<AudioSource>>, mut playing: ResMut<Playing>, mut tracks: Query<(Entity, &mut AudioSink, &mut Track)>, time: Res<Time>) {
    let key = wanted(&game);
    if playing.key != Some(key) {
        playing.key = Some(key);
        for (_, _, mut t) in &mut tracks {
            t.fading = true;
        }
        let bytes = overrides.music(key.0).unwrap_or_else(|| wav(&render(&score(key.0, key.1), loop_secs(key.0))));
        let handle = sources.add(AudioSource { bytes: bytes.into() });
        commands.spawn((AudioPlayer::new(handle), PlaybackSettings::LOOP.with_volume(Volume::Linear(0.0)), Track { fading: false }));
    }
    let step = time.delta_secs() / FADE * MUSIC;
    let target = if game.paused { MUSIC * 0.4 } else { MUSIC };
    for (e, mut sink, t) in &mut tracks {
        let v = sink.volume().to_linear();
        if t.fading {
            if v <= 0.005 {
                commands.entity(e).despawn();
            } else {
                sink.set_volume(Volume::Linear((v - step).max(0.0)));
            }
        } else if v < target {
            sink.set_volume(Volume::Linear((v + step).min(target)));
        } else if v > target {
            sink.set_volume(Volume::Linear((v - step).max(target)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: MUSIC-01
    #[test]
    fn every_chapter_has_its_own_loop_and_automation_quantizes_it() {
        let loops: Vec<Vec<Note>> = (0..5).map(|c| score(c, 0)).collect();
        for pair in loops.windows(2) {
            assert_ne!(pair[0], pair[1], "chapters sound different");
        }
        let grid = |chapter: usize, bucket: u32| -> bool {
            let sixteenth = 60.0 / PROFILES[chapter].bpm / 4.0;
            score(chapter, bucket).iter().filter(|n| n.freq > 100.0).all(|n| {
                let k = (n.start / sixteenth).round();
                (n.start - k * sixteenth).abs() < 1e-3
            })
        };
        assert!(grid(1, 4), "the machine plays exactly on the grid");
        assert!(!grid(1, 0), "a person drifts");
        let samples = render(&score(2, 1), loop_secs(2));
        assert_eq!(samples.len(), (loop_secs(2) * RATE as f32) as usize);
        assert!(samples.iter().any(|s| s.abs() > 0.1), "it is not silence");
        assert!(samples.iter().all(|s| s.abs() <= 0.9001), "it does not clip");
        assert_eq!(&wav(&samples[..100])[0..4], b"RIFF");
    }
}
