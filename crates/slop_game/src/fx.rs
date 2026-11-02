//! Effects that make a wave readable and satisfying (UI-04, FEEL-01): damage
//! numbers in the colour of their type, slop being deleted, bubbles popping
//! into letters, swings and sprays, spawn and aim warnings, screen shake.
//! Effects only read simulation events; they never change the wave.

use bevy::prelude::*;
use slop_sim::content::{Pattern, AGREE_LINES, BUBBLE_LINES};
use slop_sim::world::{LANCE_AIM, SPAWN_WARNING, TICK_RATE};
use slop_sim::{Event, SlopKind, V2};

use crate::art::Art;
use crate::play::Simulation;
use crate::state::Game;
use crate::theme;
use crate::views::{Shake, Z_FX, Z_TEXT};

/// Caps keep a crowded wave from drowning in effects.
const MAX_NUMBERS: usize = 70;
const MAX_PARTICLES: usize = 260;
const AIM_LENGTH: f32 = 560.0;

pub struct FxPlugin;

impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn_fx, animate).chain().after(Simulation));
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Number,
    Particle,
    Text,
    /// Fades in and out with a pulse.
    Warning,
    Sector,
}

#[derive(Component)]
struct Fx {
    kind: Kind,
    life: f32,
    max: f32,
    vel: Vec2,
    base_alpha: f32,
}

impl Fx {
    fn new(kind: Kind, life: f32, vel: Vec2) -> Fx {
        Fx { kind, life, max: life, vel, base_alpha: 1.0 }
    }
}

fn v(p: V2) -> Vec2 {
    Vec2::new(p.x, p.y)
}

/// A cheap deterministic scatter for effect directions.
fn spread(n: u32) -> Vec2 {
    let a = n as f32 * 2.399_963;
    let r = 0.6 + 0.4 * ((n * 7919) % 100) as f32 / 100.0;
    Vec2::new(a.cos(), a.sin()) * r
}

fn slop_color(kind: SlopKind) -> Color {
    match kind {
        SlopKind::BubbleBot => Color::srgb(0.56, 0.89, 0.94),
        SlopKind::EmDashLancer => Color::srgb(0.61, 0.42, 1.0),
        SlopKind::Listicle => Color::srgb(1.0, 0.88, 0.40),
        SlopKind::YesBot => Color::srgb(0.48, 0.88, 0.48),
        SlopKind::SixFingerHand => Color::srgb(0.94, 0.79, 0.63),
    }
}

fn text(commands: &mut Commands, words: impl Into<String>, at: Vec2, size: f32, color: Color, fx: Fx) {
    commands.spawn((
        Text2d::new(words),
        TextFont { font_size: size, ..default() },
        TextColor(color),
        Transform::from_xyz(at.x, at.y, Z_TEXT),
        fx,
    ));
}

fn particle(commands: &mut Commands, art: &Art, at: Vec2, size: f32, color: Color, fx: Fx) {
    commands.spawn((
        Sprite { image: art.dot.clone(), custom_size: Some(Vec2::splat(size)), color, ..default() },
        Transform::from_xyz(at.x, at.y, Z_FX),
        fx,
    ));
}

fn spawn_fx(
    mut commands: Commands,
    game: Res<Game>,
    art: Res<Art>,
    mut shake: ResMut<Shake>,
    existing: Query<&Fx>,
    mut counter: Local<u32>,
) {
    if game.events.is_empty() {
        return;
    }
    let mut numbers = existing.iter().filter(|f| f.kind == Kind::Number).count();
    let mut particles = existing.iter().filter(|f| f.kind == Kind::Particle).count();
    let world = &game.world;
    let me = v(world.player.pos);
    for e in &game.events {
        *counter = counter.wrapping_add(1);
        let n = *counter;
        match *e {
            Event::SlopHit { at, amount, kind } => {
                if numbers >= MAX_NUMBERS {
                    continue;
                }
                numbers += 1;
                let words = if amount < 1.0 { format!("{amount:.1}") } else { format!("{amount:.0}") };
                let at = v(at) + spread(n) * 14.0 + Vec2::Y * 18.0;
                text(&mut commands, words, at, 20.0, theme::type_color(kind), Fx::new(Kind::Number, 0.6, Vec2::new(0.0, 70.0)));
            }
            Event::SlopDied { at, kind } => {
                let at = v(at);
                for k in 0..7 {
                    if particles >= MAX_PARTICLES {
                        break;
                    }
                    particles += 1;
                    let dir = spread(n * 7 + k);
                    particle(&mut commands, &art, at, 7.0, slop_color(kind), Fx::new(Kind::Particle, 0.45, dir * 220.0));
                }
                text(&mut commands, "[removed]", at + Vec2::Y * 10.0, 13.0, theme::QUIET, Fx::new(Kind::Text, 0.7, Vec2::Y * 30.0));
            }
            Event::SpawnWarning { at, .. } => {
                commands.spawn((
                    Sprite { image: art.typing.clone(), custom_size: Some(Vec2::new(44.0, 28.0)), ..default() },
                    Transform::from_xyz(at.x, at.y, Z_FX),
                    Fx::new(Kind::Warning, SPAWN_WARNING as f32 / TICK_RATE as f32, Vec2::ZERO),
                ));
            }
            Event::PlayerHurt { amount } => {
                shake.trauma = (shake.trauma + 0.3 + amount * 0.02).min(1.0);
                text(&mut commands, format!("-{:.0}", amount.max(1.0)), me + Vec2::new(20.0, 34.0), 22.0, theme::DANGER, Fx::new(Kind::Text, 0.7, Vec2::Y * 50.0));
            }
            Event::Dodged => {
                text(&mut commands, "Nope.", me + Vec2::new(0.0, 40.0), 18.0, theme::QUIET, Fx::new(Kind::Text, 0.8, Vec2::Y * 40.0));
            }
            Event::Swing { at, angle, range, weapon, .. } => {
                let spray = world
                    .player
                    .weapons
                    .get(weapon as usize)
                    .is_some_and(|w| matches!(w.def().pattern, Pattern::Cone { .. }));
                let image = if spray { art.spray.clone() } else { art.swing.clone() };
                let mut fx = Fx::new(Kind::Sector, if spray { 0.14 } else { 0.16 }, Vec2::ZERO);
                fx.base_alpha = if spray { 0.55 } else { 0.7 };
                commands.spawn((
                    Sprite { image, custom_size: Some(Vec2::splat(range * 2.0)), ..default() },
                    Transform::from_xyz(at.x, at.y, Z_FX - 0.5).with_rotation(Quat::from_rotation_z(angle)),
                    fx,
                ));
                if spray && particles < MAX_PARTICLES {
                    for k in 0..3 {
                        particles += 1;
                        let dir = Vec2::from_angle(angle + (k as f32 - 1.0) * 0.25);
                        particle(&mut commands, &art, v(at) + dir * 20.0, 6.0, Color::srgb(0.45, 0.7, 1.0), Fx::new(Kind::Particle, 0.35, dir * range * 2.6));
                    }
                }
            }
            Event::BubblePopped { at, line } => {
                let words = BUBBLE_LINES.get(line as usize).copied().unwrap_or("pop");
                for (k, ch) in words.chars().filter(|c| c.is_alphanumeric()).step_by(2).take(6).enumerate() {
                    let dir = spread(n * 11 + k as u32);
                    text(&mut commands, ch.to_string(), v(at), 15.0, theme::INK, Fx::new(Kind::Text, 0.5, dir * 160.0));
                }
            }
            Event::Agreed { at, line } => {
                let words = AGREE_LINES.get(line as usize).copied().unwrap_or("Agreed!");
                text(&mut commands, words, v(at) + Vec2::Y * 40.0, 16.0, theme::AGREE, Fx::new(Kind::Text, 1.4, Vec2::Y * 25.0));
            }
            Event::Scattered { at } => {
                text(&mut commands, "Listicle scattered!", v(at) + Vec2::Y * 30.0, 16.0, theme::INK, Fx::new(Kind::Text, 1.0, Vec2::Y * 30.0));
            }
            Event::Grabbed => {
                text(&mut commands, "Grabbed!", me + Vec2::new(0.0, 44.0), 18.0, theme::DANGER, Fx::new(Kind::Text, 0.8, Vec2::Y * 30.0));
            }
            Event::LancerAim { from, dir } => {
                let (from, dir) = (v(from), v(dir));
                let centre = from + dir * (AIM_LENGTH / 2.0);
                let mut fx = Fx::new(Kind::Warning, LANCE_AIM as f32 / TICK_RATE as f32, Vec2::ZERO);
                fx.base_alpha = 0.55;
                commands.spawn((
                    Sprite { image: art.dot.clone(), custom_size: Some(Vec2::new(AIM_LENGTH, 3.0)), color: theme::DANGER, ..default() },
                    Transform::from_xyz(centre.x, centre.y, Z_FX).with_rotation(Quat::from_rotation_z(dir.to_angle())),
                    fx,
                ));
            }
            Event::WaveCleared => {
                // Every slop left is deleted at once.
                for (k, s) in world.slop.iter().enumerate() {
                    let at = v(s.pos);
                    for j in 0..3 {
                        let dir = spread(n * 3 + k as u32 * 5 + j);
                        particle(&mut commands, &art, at, 7.0, slop_color(s.kind), Fx::new(Kind::Particle, 0.6, dir * 200.0));
                    }
                }
            }
            Event::Burnout => shake.trauma = 1.0,
            Event::Fired { .. } | Event::LikeCollected { .. } => {}
        }
    }
}

#[allow(clippy::type_complexity)]
fn animate(
    mut commands: Commands,
    time: Res<Time>,
    mut fx: Query<(Entity, &mut Fx, &mut Transform, Option<&mut Sprite>, Option<&mut TextColor>)>,
) {
    let dt = time.delta_secs();
    for (e, mut f, mut tf, sprite, color) in &mut fx {
        f.life -= dt;
        if f.life <= 0.0 {
            commands.entity(e).despawn();
            continue;
        }
        let left = f.life / f.max;
        tf.translation += (f.vel * dt).extend(0.0);
        f.vel *= 1.0 - 3.0 * dt;
        let a = match f.kind {
            Kind::Warning => f.base_alpha * (0.6 + 0.4 * (time.elapsed_secs() * 14.0).sin()),
            Kind::Number => {
                tf.scale = Vec3::splat(1.0 + 0.5 * (left - 0.75).max(0.0) * 4.0);
                f.base_alpha * left.sqrt()
            }
            _ => f.base_alpha * left,
        };
        if let Some(mut s) = sprite {
            s.color.set_alpha(a);
        }
        if let Some(mut c) = color {
            c.0.set_alpha(a);
        }
    }
}
