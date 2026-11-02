//! Placeholder sounds, synthesised at startup (FEEL-02): no audio files ship.
//! Each sound is rate-limited so a crowded wave does not turn into noise.

use std::collections::HashMap;
use std::io::Write;

use bevy::audio::{AudioSource, PlaybackSettings, Volume};
use bevy::prelude::*;
use slop_sim::Event;

use crate::play::Simulation;
use crate::state::Game;

const RATE: u32 = 22_050;
const MASTER: f32 = 0.35;

pub struct AudioFxPlugin;

impl Plugin for AudioFxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(Update, play.after(Simulation));
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Sound {
    Hit,
    Death,
    Pop,
    Hurt,
    Like,
    Swing,
    Shoot,
    Agree,
    Grab,
    Cleared,
    Burnout,
}

#[derive(Resource)]
struct Sounds {
    clips: HashMap<Sound, Handle<AudioSource>>,
}

#[derive(Clone, Copy)]
enum Wave {
    Sine,
    Square,
    Noise,
}

/// One synthesised note: a pitch sweep with a quick attack and a decay.
fn note(samples: &mut Vec<f32>, start: f32, wave: Wave, f0: f32, f1: f32, secs: f32, gain: f32) {
    let n = (secs * RATE as f32) as usize;
    let first = (start * RATE as f32) as usize;
    if samples.len() < first + n {
        samples.resize(first + n, 0.0);
    }
    let mut phase = 0.0f32;
    let mut noise: u32 = 0x1234_5678;
    for i in 0..n {
        let t = i as f32 / n as f32;
        let f = f0 + (f1 - f0) * t;
        phase += f / RATE as f32;
        let s = match wave {
            Wave::Sine => (phase * std::f32::consts::TAU).sin(),
            Wave::Square => {
                if phase.fract() < 0.5 {
                    0.6
                } else {
                    -0.6
                }
            }
            Wave::Noise => {
                noise ^= noise << 13;
                noise ^= noise >> 17;
                noise ^= noise << 5;
                (noise as f32 / u32::MAX as f32) * 2.0 - 1.0
            }
        };
        let attack = (t * 40.0).min(1.0);
        let decay = (1.0 - t).powf(1.6);
        samples[first + i] += s * gain * attack * decay;
    }
}

/// 16-bit mono WAV bytes.
fn wav(samples: &[f32]) -> Vec<u8> {
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

fn synth(sound: Sound) -> Vec<f32> {
    let mut s = Vec::new();
    match sound {
        Sound::Hit => {
            note(&mut s, 0.0, Wave::Noise, 0.0, 0.0, 0.04, 0.35);
            note(&mut s, 0.0, Wave::Sine, 260.0, 120.0, 0.06, 0.5);
        }
        Sound::Death => {
            note(&mut s, 0.0, Wave::Noise, 0.0, 0.0, 0.09, 0.3);
            note(&mut s, 0.0, Wave::Square, 520.0, 140.0, 0.12, 0.25);
        }
        Sound::Pop => note(&mut s, 0.0, Wave::Sine, 1300.0, 420.0, 0.07, 0.5),
        Sound::Hurt => {
            note(&mut s, 0.0, Wave::Square, 190.0, 80.0, 0.18, 0.45);
            note(&mut s, 0.0, Wave::Noise, 0.0, 0.0, 0.06, 0.3);
        }
        Sound::Like => {
            note(&mut s, 0.0, Wave::Sine, 880.0, 880.0, 0.06, 0.3);
            note(&mut s, 0.05, Wave::Sine, 1320.0, 1320.0, 0.08, 0.3);
        }
        Sound::Swing => note(&mut s, 0.0, Wave::Noise, 0.0, 0.0, 0.08, 0.18),
        Sound::Shoot => note(&mut s, 0.0, Wave::Square, 700.0, 300.0, 0.035, 0.12),
        Sound::Agree => {
            note(&mut s, 0.0, Wave::Sine, 660.0, 660.0, 0.07, 0.25);
            note(&mut s, 0.07, Wave::Sine, 880.0, 880.0, 0.09, 0.25);
        }
        Sound::Grab => note(&mut s, 0.0, Wave::Square, 130.0, 110.0, 0.14, 0.35),
        Sound::Cleared => {
            for (i, f) in [523.0, 659.0, 784.0, 1047.0].iter().enumerate() {
                note(&mut s, i as f32 * 0.09, Wave::Sine, *f, *f, 0.18, 0.35);
            }
        }
        Sound::Burnout => note(&mut s, 0.0, Wave::Square, 420.0, 70.0, 0.7, 0.4),
    }
    s
}

fn setup(mut commands: Commands, mut sources: ResMut<Assets<AudioSource>>) {
    let all = [
        Sound::Hit,
        Sound::Death,
        Sound::Pop,
        Sound::Hurt,
        Sound::Like,
        Sound::Swing,
        Sound::Shoot,
        Sound::Agree,
        Sound::Grab,
        Sound::Cleared,
        Sound::Burnout,
    ];
    let clips = all.iter().map(|&s| (s, sources.add(AudioSource { bytes: wav(&synth(s)).into() }))).collect();
    commands.insert_resource(Sounds { clips });
}

/// Seconds between two plays of the same sound.
fn gap(sound: Sound) -> f32 {
    match sound {
        Sound::Hit | Sound::Shoot | Sound::Swing => 0.06,
        Sound::Death | Sound::Like | Sound::Pop => 0.05,
        _ => 0.12,
    }
}

fn play(mut commands: Commands, game: Res<Game>, sounds: Res<Sounds>, time: Res<Time>, mut last: Local<HashMap<Sound, f32>>) {
    let now = time.elapsed_secs();
    for e in &game.events {
        let sound = match e {
            Event::SlopHit { .. } => Sound::Hit,
            Event::SlopDied { .. } => Sound::Death,
            Event::BubblePopped { .. } => Sound::Pop,
            Event::PlayerHurt { .. } => Sound::Hurt,
            Event::LikeCollected { .. } => Sound::Like,
            Event::Swing { .. } => Sound::Swing,
            Event::Fired { .. } => Sound::Shoot,
            Event::Agreed { .. } => Sound::Agree,
            Event::Grabbed => Sound::Grab,
            Event::WaveCleared => Sound::Cleared,
            Event::Burnout => Sound::Burnout,
            _ => continue,
        };
        if last.get(&sound).is_some_and(|t| now - t < gap(sound)) {
            continue;
        }
        last.insert(sound, now);
        commands.spawn((AudioPlayer::new(sounds.clips[&sound].clone()), PlaybackSettings::DESPAWN.with_volume(Volume::Linear(MASTER))));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: FEEL-02
    #[test]
    fn sounds_are_valid_wav() {
        let bytes = wav(&synth(Sound::Pop));
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        let len = u32::from_le_bytes(bytes[40..44].try_into().unwrap()) as usize;
        assert_eq!(bytes.len(), 44 + len);
        assert!(synth(Sound::Cleared).iter().any(|s| s.abs() > 0.1));
    }
}
