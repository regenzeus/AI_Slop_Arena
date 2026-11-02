//! Effects that make a wave readable and satisfying (UI-04, FEEL-01): damage
//! numbers in the colour of their type, slop being deleted, brainrot shouting
//! its name, bubbles popping into letters, swings, beams, chains and splashes,
//! spawn and aim warnings, status words and screen shake. Effects only read
//! simulation events; they never change the wave.

use bevy::prelude::*;
use slop_sim::content::{Archetype, Pattern, AGREE_LINES, BUBBLE_LINES};
use slop_sim::events::StatusKind;
use slop_sim::spec::SlopLook;
use slop_sim::world::{Active, LANCE_AIM, SPAWN_WARNING, TICK_RATE};
use slop_sim::{Event, V2};

use crate::art::Art;
use crate::play::Simulation;
use crate::state::Game;
use crate::theme;
use crate::views::{Shake, Z_FX, Z_TEXT};

/// Caps keep a crowded wave from drowning in effects.
const MAX_NUMBERS: usize = 70;
const MAX_PARTICLES: usize = 260;
const MAX_WORDS: usize = 30;
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
    /// A swing, spray or line that fades fast.
    Flash,
    /// A ring that grows as it fades.
    Ring,
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

fn look_color(look: SlopLook) -> Color {
    match look {
        SlopLook::Archetype(Archetype::ChadGpt) => Color::srgb(0.24, 0.78, 0.71),
        SlopLook::Archetype(Archetype::Clawed) => Color::srgb(0.91, 0.51, 0.35),
        SlopLook::Archetype(Archetype::YesBot) => Color::srgb(0.48, 0.88, 0.48),
        SlopLook::Archetype(Archetype::Hallucinator) => Color::srgb(0.61, 0.42, 1.0),
        SlopLook::Archetype(_) => Color::srgb(0.85, 0.85, 0.9),
        SlopLook::Brainrot { .. } => Color::srgb(1.0, 0.7, 0.45),
        SlopLook::Boss(_) => Color::srgb(1.0, 0.85, 0.3),
    }
}

fn text(commands: &mut Commands, words: impl Into<String>, at: Vec2, size: f32, color: Color, fx: Fx) {
    commands.spawn((Text2d::new(words), TextFont { font_size: size, ..default() }, TextColor(color), Transform::from_xyz(at.x, at.y, Z_TEXT), fx));
}

fn particle(commands: &mut Commands, art: &Art, at: Vec2, size: f32, color: Color, fx: Fx) {
    commands.spawn((Sprite { image: art.dot.clone(), custom_size: Some(Vec2::splat(size)), color, ..default() }, Transform::from_xyz(at.x, at.y, Z_FX), fx));
}

/// A thin line between two points that fades fast.
fn line(commands: &mut Commands, art: &Art, from: Vec2, to: Vec2, width: f32, color: Color, life: f32) {
    let d = to - from;
    let mid = (from + to) / 2.0;
    commands.spawn((
        Sprite { image: art.dot.clone(), custom_size: Some(Vec2::new(d.length(), width)), color, ..default() },
        Transform::from_xyz(mid.x, mid.y, Z_FX).with_rotation(Quat::from_rotation_z(d.to_angle())),
        Fx::new(Kind::Flash, life, Vec2::ZERO),
    ));
}

/// A ring that expands to `radius` as it fades.
fn ring(commands: &mut Commands, art: &Art, at: Vec2, radius: f32, color: Color, life: f32) {
    let fx = Fx::new(Kind::Ring, life, Vec2::ZERO);
    commands.spawn((Sprite { image: art.ring.clone(), custom_size: Some(Vec2::splat(radius * 2.0)), color, ..default() }, Transform::from_xyz(at.x, at.y, Z_FX), fx));
}

fn status_words(kind: StatusKind) -> (&'static str, Color) {
    match kind {
        StatusKind::Anxiety => ("Anxiety!", theme::DANGER),
        StatusKind::Doomscroll => ("Doomscrolling...", theme::DANGER),
        StatusKind::Popup => ("Pop-up!", theme::DANGER),
        StatusKind::Ratio => ("Ratio'd", theme::DANGER),
        StatusKind::Brainrot => ("BRAINROT", Color::srgb(0.62, 0.36, 1.0)),
        StatusKind::BrainrotWarning => ("Brainrot incoming...", Color::srgb(0.62, 0.36, 1.0)),
    }
}

fn active_words(a: Active) -> &'static str {
    match a {
        Active::LogOff => "Logged off!",
        Active::BlockButton => "Blocked.",
        Active::DigitalDetox => "Digital detox!",
        Active::Unsubscribe => "UNSUBSCRIBE",
        Active::PhoneAFriend => "Phone a friend!",
    }
}

#[allow(clippy::too_many_lines)]
fn spawn_fx(mut commands: Commands, game: Res<Game>, art: Res<Art>, mut shake: ResMut<Shake>, existing: Query<&Fx>, mut counter: Local<u32>) {
    if game.events.is_empty() {
        return;
    }
    let Some(world) = game.world() else { return };
    let mut numbers = existing.iter().filter(|f| f.kind == Kind::Number).count();
    let mut particles = existing.iter().filter(|f| f.kind == Kind::Particle).count();
    let mut words = existing.iter().filter(|f| f.kind == Kind::Text).count();
    let me = v(world.player.pos);
    let mut say = |commands: &mut Commands, w: String, at: Vec2, size: f32, color: Color, life: f32| {
        if words < MAX_WORDS {
            words += 1;
            text(commands, w, at, size, color, Fx::new(Kind::Text, life, Vec2::Y * 32.0));
        }
    };
    for e in &game.events {
        *counter = counter.wrapping_add(1);
        let n = *counter;
        match *e {
            Event::SlopHit { at, amount, kind } => {
                if numbers >= MAX_NUMBERS {
                    continue;
                }
                numbers += 1;
                let w = if amount < 1.0 { format!("{amount:.1}") } else { format!("{amount:.0}") };
                let at = v(at) + spread(n) * 14.0 + Vec2::Y * 18.0;
                text(&mut commands, w, at, 20.0, theme::type_color(kind), Fx::new(Kind::Number, 0.6, Vec2::new(0.0, 70.0)));
            }
            Event::SlopDied { at, spec } => {
                let look = world.setup.specs.get(spec as usize).map_or(SlopLook::Archetype(Archetype::ChadGpt), |s| s.look);
                let at = v(at);
                for k in 0..7 {
                    if particles >= MAX_PARTICLES {
                        break;
                    }
                    particles += 1;
                    particle(&mut commands, &art, at, 7.0, look_color(look), Fx::new(Kind::Particle, 0.45, spread(n * 7 + k) * 220.0));
                }
                say(&mut commands, "[removed]".into(), at + Vec2::Y * 10.0, 13.0, theme::QUIET, 0.7);
            }
            Event::SpawnWarning { at } => {
                commands.spawn((
                    Sprite { image: art.typing.clone(), custom_size: Some(Vec2::new(44.0, 28.0)), ..default() },
                    Transform::from_xyz(at.x, at.y, Z_FX),
                    Fx::new(Kind::Warning, SPAWN_WARNING as f32 / TICK_RATE as f32, Vec2::ZERO),
                ));
            }
            Event::Named { at, spec } => {
                if let Some(s) = world.setup.specs.get(spec as usize) {
                    let color = if s.elite { theme::PLAYER_RED } else { Color::srgb(0.85, 0.42, 0.1) };
                    say(&mut commands, format!("{}!", s.name), v(at) + Vec2::Y * 40.0, 16.0, color, 1.6);
                }
            }
            Event::PlayerHurt { amount } => {
                shake.trauma = (shake.trauma + 0.25 + amount * 0.015).min(1.0);
                say(&mut commands, format!("-{:.0}", amount.max(1.0)), me + Vec2::new(22.0, 34.0), 22.0, theme::DANGER, 0.7);
            }
            Event::Dodged => say(&mut commands, "Nope.".into(), me + Vec2::new(0.0, 40.0), 18.0, theme::QUIET, 0.8),
            Event::SlopDodged { at } => say(&mut commands, "dodged".into(), v(at) + Vec2::Y * 24.0, 14.0, theme::QUIET, 0.6),
            Event::Blocked => say(&mut commands, "Blocked!".into(), me + Vec2::new(0.0, 44.0), 18.0, theme::type_color(slop_sim::DamageType::Fact), 0.8),
            Event::SecondWind => say(&mut commands, "Good night's sleep!".into(), me + Vec2::new(0.0, 50.0), 22.0, theme::mental_health_color(1.0), 1.5),
            Event::Swing { at, angle, range, weapon, .. } => {
                let spray = world.player.weapons.get(weapon as usize).is_some_and(|w| matches!(w.def().pattern, Pattern::Cone { .. }));
                let image = if spray { art.spray.clone() } else { art.swing.clone() };
                let mut fx = Fx::new(Kind::Flash, if spray { 0.14 } else { 0.16 }, Vec2::ZERO);
                fx.base_alpha = if spray { 0.55 } else { 0.7 };
                commands.spawn((
                    Sprite { image, custom_size: Some(Vec2::splat(range * 2.0)), ..default() },
                    Transform::from_xyz(at.x, at.y, Z_FX - 0.5).with_rotation(Quat::from_rotation_z(angle)),
                    fx,
                ));
            }
            Event::Beam { from, to } => line(&mut commands, &art, v(from), v(to), 10.0, Color::srgba(1.0, 0.95, 0.6, 0.85), 0.18),
            Event::Chain { from, to } => line(&mut commands, &art, v(from), v(to), 4.0, theme::type_color(slop_sim::DamageType::Sarcasm), 0.25),
            Event::Splash { at, radius } => ring(&mut commands, &art, v(at), radius, Color::srgba(1.0, 0.85, 0.6, 0.9), 0.3),
            Event::Slam { at, radius } => {
                shake.trauma = (shake.trauma + 0.3).min(1.0);
                ring(&mut commands, &art, v(at), radius, theme::type_color(slop_sim::DamageType::Fact), 0.35);
                say(&mut commands, "BANNED".into(), v(at) + Vec2::Y * 20.0, 18.0, theme::type_color(slop_sim::DamageType::Fact), 0.6);
            }
            Event::Ring { at, radius } => ring(&mut commands, &art, v(at), radius, Color::srgba(1.0, 1.0, 1.0, 0.6), 0.2),
            Event::Reflected { at } => say(&mut commands, "No u.".into(), v(at) + Vec2::Y * 24.0, 16.0, theme::type_color(slop_sim::DamageType::Sarcasm), 0.8),
            Event::BubblePopped { at, line } => {
                let words = BUBBLE_LINES.get(line as usize).copied().unwrap_or("...");
                for (k, ch) in words.chars().filter(|c| c.is_alphanumeric()).step_by(2).take(6).enumerate() {
                    if particles < MAX_PARTICLES {
                        particles += 1;
                        text(&mut commands, ch.to_string(), v(at), 15.0, theme::INK, Fx::new(Kind::Particle, 0.5, spread(n * 11 + k as u32) * 160.0));
                    }
                }
            }
            Event::Agreed { at, line } => {
                let w = AGREE_LINES.get(line as usize).copied().unwrap_or("Type AMEN.");
                say(&mut commands, w.into(), v(at) + Vec2::Y * 40.0, 16.0, theme::AGREE, 1.4);
            }
            Event::Healed { at } => say(&mut commands, "+HP".into(), v(at) + Vec2::Y * 30.0, 15.0, theme::AGREE, 0.8),
            Event::Scattered { at } => say(&mut commands, "Listicle scattered!".into(), v(at) + Vec2::Y * 30.0, 16.0, theme::INK, 1.0),
            Event::Reposted { at } => say(&mut commands, "[repost]".into(), v(at) + Vec2::Y * 30.0, 15.0, theme::QUIET, 0.9),
            Event::Split { at } => say(&mut commands, "I came across your profile.".into(), v(at) + Vec2::Y * 34.0, 14.0, theme::INK, 1.2),
            Event::Hatched { at } => say(&mut commands, "Surprise!".into(), v(at) + Vec2::Y * 30.0, 16.0, theme::INK, 1.0),
            Event::Grabbed => say(&mut commands, "Grabbed!".into(), me + Vec2::new(0.0, 44.0), 18.0, theme::DANGER, 0.8),
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
            Event::Status { kind } => {
                let (w, color) = status_words(kind);
                say(&mut commands, w.into(), me + Vec2::new(0.0, 52.0), 18.0, color, 1.0);
            }
            Event::ActiveUsed { active } => say(&mut commands, active_words(active).into(), me + Vec2::new(0.0, 60.0), 20.0, theme::type_color(slop_sim::DamageType::Grass), 1.0),
            Event::Friend { from, to } => line(&mut commands, &art, v(from), v(to), 14.0, Color::srgba(0.4, 0.9, 0.5, 0.8), 0.35),
            Event::LootBox { at } => say(&mut commands, "Loot Box! Opens in the next shop.".into(), v(at) + Vec2::Y * 40.0, 16.0, Color::srgb(0.9, 0.7, 0.1), 1.6),
            Event::Hustle { level } => say(&mut commands, format!("Rise and grind! Speed x{level:.2}"), me + Vec2::new(0.0, 80.0), 20.0, theme::PLAYER_RED, 1.5),
            Event::WaveCleared => {
                for (k, s) in world.slop.iter().enumerate() {
                    let at = v(s.pos);
                    for j in 0..3 {
                        let dir = spread(n * 3 + k as u32 * 5 + j);
                        particle(&mut commands, &art, at, 7.0, look_color(world.spec(s).look), Fx::new(Kind::Particle, 0.6, dir * 200.0));
                    }
                }
            }
            Event::Burnout => shake.trauma = 1.0,
            Event::Beat => shake.trauma = shake.trauma.max(0.12),
            Event::BossIntro { .. } | Event::BossPhase { .. } | Event::Regenerate { .. } => shake.trauma = shake.trauma.max(0.5),
            Event::Fired { .. } | Event::LikeCollected { .. } | Event::BombWarning { .. } | Event::AdBreak => {}
        }
    }
}

#[allow(clippy::type_complexity)]
fn animate(mut commands: Commands, time: Res<Time>, mut fx: Query<(Entity, &mut Fx, &mut Transform, Option<&mut Sprite>, Option<&mut TextColor>)>) {
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
            Kind::Ring => {
                tf.scale = Vec3::splat(0.3 + 0.7 * (1.0 - left));
                f.base_alpha * left
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
