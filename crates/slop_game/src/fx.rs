//! Effects that make a wave readable and satisfying (UI-04, FEEL-01): damage
//! numbers in the colour of their type, slop being deleted, brainrot shouting
//! its name, bubbles popping into letters, and an effect for every weapon
//! (WPN-09): muzzle flashes, sprays, flying keys, chains, beams, a falling Ban
//! Hammer, brick debris and trails behind your projectiles. Also spawn and aim
//! warnings, status words, level-ups and screen shake. Effects only read
//! simulation events; they never change the wave.

use bevy::prelude::*;
use slop_sim::content::{weapon_index, Archetype, Look, Pattern, AGREE_LINES, BUBBLE_LINES, CLAWED_LINES, WEAPONS};
use slop_sim::events::StatusKind;
use slop_sim::spec::SlopLook;
use slop_sim::world::{LANCE_AIM, SPAWN_WARNING, TICK_RATE};
use slop_sim::{DamageType, Event, V2};

use crate::art::{Art, HUMAN_SIZE};
use crate::play::Simulation;
use crate::state::Game;
use crate::theme;
use crate::views::{Shake, Z_FX, Z_TEXT};

/// Caps keep a crowded wave from drowning in effects.
const MAX_NUMBERS: usize = 70;
const MAX_PARTICLES: usize = 320;
const MAX_WORDS: usize = 30;
const MAX_TRAILS: usize = 240;
const AIM_LENGTH: f32 = 560.0;

pub struct FxPlugin;

impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn_fx, trails, animate).chain().after(Simulation));
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
    /// Spins as it fades (an Uno card).
    Spin,
    /// Swings down onto its spot, then rests (the Ban Hammer).
    Drop,
    /// A dot left behind a projectile.
    Trail,
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

    fn alpha(mut self, a: f32) -> Fx {
        self.base_alpha = a;
        self
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

/// A number from 0 to 1 that changes with `n`.
fn jitter(n: u32) -> f32 {
    ((n.wrapping_mul(2_654_435_761)) >> 16) as f32 / 65_536.0
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

/// Text with a dark copy behind it, readable on any floor.
#[allow(clippy::too_many_arguments)]
fn shadowed(commands: &mut Commands, words: &str, at: Vec2, size: f32, color: Color, life: f32, vel: Vec2, kind: Kind) {
    let shadow = Fx::new(kind, life, vel).alpha(0.75);
    commands.spawn((Text2d::new(words), TextFont { font_size: size, ..default() }, TextColor(theme::INK), Transform::from_xyz(at.x + 1.5, at.y - 1.5, Z_TEXT - 0.01), shadow));
    text(commands, words, at, size, color, Fx::new(kind, life, vel));
}

fn particle(commands: &mut Commands, art: &Art, at: Vec2, size: f32, color: Color, fx: Fx) {
    commands.spawn((Sprite { image: art.ball.clone(), custom_size: Some(Vec2::splat(size)), color, ..default() }, Transform::from_xyz(at.x, at.y, Z_FX), fx));
}

fn sprite(commands: &mut Commands, image: &Handle<Image>, at: Vec2, size: f32, angle: f32, fx: Fx) {
    commands.spawn((Sprite { image: image.clone(), custom_size: Some(Vec2::splat(size)), ..default() }, Transform::from_xyz(at.x, at.y, Z_FX + 0.2).with_rotation(Quat::from_rotation_z(angle)), fx));
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
    commands.spawn((Sprite { image: if radius > 120.0 { art.thin_ring.clone() } else { art.ring.clone() }, custom_size: Some(Vec2::splat(radius * 2.0)), color, ..default() }, Transform::from_xyz(at.x, at.y, Z_FX), fx));
}

fn status_color(kind: StatusKind) -> Color {
    match kind {
        StatusKind::Brainrot | StatusKind::BrainrotWarning => Color::srgba(0.62, 0.36, 1.0, 0.9),
        StatusKind::Doomscroll | StatusKind::Popup => Color::srgba(0.4, 0.6, 1.0, 0.9),
        _ => theme::DANGER.with_alpha(0.8),
    }
}

/// Keys that fly off a Mechanical Keyboard.
const KEYS: [&str; 8] = ["W", "A", "S", "D", "Q", "E", "Ctrl", "Esc"];


#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
fn spawn_fx(
    mut commands: Commands,
    game: Res<Game>,
    art: Res<Art>,
    mut shake: ResMut<Shake>,
    existing: Query<&Fx>,
    mut counter: Local<u32>,
) {
    let Some(world) = game.world() else { return };
    let me = v(world.player.pos);
    if game.level_flash {
        ring(&mut commands, &art, me, 90.0, Color::srgb(1.0, 0.86, 0.3), 0.5);
        for k in 0..8 {
            let fx = Fx::new(Kind::Spin, 0.7, spread(*counter + k) * 260.0);
            sprite(&mut commands, &art.star, me, 18.0, 0.0, fx);
        }
    }
    if game.events.is_empty() {
        return;
    }
    let mut numbers = existing.iter().filter(|f| f.kind == Kind::Number).count();
    let mut particles = existing.iter().filter(|f| f.kind == Kind::Particle).count();
    let mut words = existing.iter().filter(|f| f.kind == Kind::Text).count();
    let mut say = |commands: &mut Commands, w: &str, at: Vec2, size: f32, color: Color, life: f32| {
        if words < MAX_WORDS {
            words += 1;
            shadowed(commands, w, at, size, color, life, Vec2::Y * 32.0, Kind::Text);
        }
    };
    let uno = weapon_index("Uno Reverse Card").map(|i| art.weapon_icons[i].clone());
    let hammer = weapon_index("Ban Hammer").map(|i| art.weapon_icons[i].clone());
    for e in &game.events {
        *counter = counter.wrapping_add(1);
        let n = *counter;
        match *e {
            Event::SlopHit { at, amount, kind } => {
                if particles < MAX_PARTICLES {
                    particles += 1;
                    particle(&mut commands, &art, v(at) + spread(n) * 8.0, 11.0, Color::WHITE, Fx::new(Kind::Particle, 0.1, Vec2::ZERO));
                }
                if numbers >= MAX_NUMBERS {
                    continue;
                }
                numbers += 1;
                let w = if amount < 1.0 { format!("{amount:.1}") } else { format!("{amount:.0}") };
                let at = v(at) + spread(n) * 14.0 + Vec2::Y * 18.0;
                let size = if amount >= 40.0 { 30.0 } else if amount >= 15.0 { 25.0 } else { 21.0 };
                shadowed(&mut commands, &w, at, size, theme::type_text(kind), 0.6, Vec2::new(0.0, 80.0), Kind::Number);
            }
            Event::SlopDied { at, spec } => {
                let look = world.setup.specs.get(spec as usize).map_or(SlopLook::Archetype(Archetype::ChadGpt), |s| s.look);
                let at = v(at);
                for k in 0..9 {
                    if particles >= MAX_PARTICLES {
                        break;
                    }
                    particles += 1;
                    particle(&mut commands, &art, at, 8.0, look_color(look), Fx::new(Kind::Particle, 0.5, spread(n * 7 + k) * 260.0));
                }
                ring(&mut commands, &art, at, 34.0, look_color(look), 0.22);
            }
            Event::SpawnWarning { at } => {
                commands.spawn((
                    Sprite { image: art.typing.clone(), custom_size: Some(Vec2::new(44.0, 28.0)), ..default() },
                    Transform::from_xyz(at.x, at.y, Z_FX),
                    Fx::new(Kind::Warning, SPAWN_WARNING as f32 / TICK_RATE as f32, Vec2::ZERO),
                ));
            }
            Event::Named { .. } => {}
            Event::PlayerHurt { amount } => {
                shake.trauma = (shake.trauma + 0.25 + amount * 0.015).min(1.0);
                say(&mut commands, &format!("-{:.0}", amount.max(1.0)), me + Vec2::new(22.0, 34.0), 24.0, theme::DANGER, 0.7);
            }
            Event::Dodged => ring(&mut commands, &art, me, 40.0, Color::srgba(1.0, 1.0, 1.0, 0.7), 0.2),
            Event::SlopDodged { .. } => {}
            Event::Blocked => ring(&mut commands, &art, me, 50.0, theme::type_text(DamageType::Fact).with_alpha(0.9), 0.25),
            Event::SecondWind => ring(&mut commands, &art, me, 160.0, theme::mental_health_color(1.0).with_alpha(0.9), 0.6),
            Event::Swing { at, angle, range, weapon, .. } => {
                let spray = world.player.weapons.get(weapon as usize).is_some_and(|w| matches!(w.def().pattern, Pattern::Cone { .. }));
                let image = if spray { art.spray.clone() } else { art.swing.clone() };
                let fx = Fx::new(Kind::Flash, if spray { 0.14 } else { 0.2 }, Vec2::ZERO).alpha(if spray { 0.55 } else { 0.8 });
                commands.spawn((
                    Sprite { image, custom_size: Some(Vec2::splat(range * 2.0)), ..default() },
                    Transform::from_xyz(at.x, at.y, Z_FX - 0.5).with_rotation(Quat::from_rotation_z(angle)),
                    fx,
                ));
            }
            Event::Attacked { weapon, to } => {
                let Some(w) = world.player.weapons.get(weapon as usize) else { continue };
                let def = &WEAPONS[w.def];
                let dir = (v(to) - me).normalize_or(Vec2::X);
                let muzzle = me + dir * 52.0;
                let color = theme::type_text(def.kind);
                match def.pattern {
                    Pattern::Shot { .. } | Pattern::Burst { .. } | Pattern::Mark { .. } | Pattern::Stun { .. } => {
                        let fx = Fx::new(Kind::Flash, 0.07, Vec2::ZERO);
                        sprite(&mut commands, &art.star, muzzle, 24.0, n as f32, fx);
                        for k in 0..3 {
                            let d = Vec2::from_angle(dir.to_angle() + (jitter(n * 3 + k) - 0.5) * 0.9);
                            particle(&mut commands, &art, muzzle, 5.0, color, Fx::new(Kind::Particle, 0.18, d * 380.0));
                        }
                    }
                    Pattern::Cone { half_angle } => {
                        for k in 0..10 {
                            let a = dir.to_angle() + (jitter(n * 10 + k) - 0.5) * 2.0 * half_angle;
                            let speed = 380.0 + 260.0 * jitter(n * 31 + k);
                            particle(&mut commands, &art, me + dir * 30.0, 7.0, Color::srgb(0.55, 0.8, 1.0), Fx::new(Kind::Particle, 0.32, Vec2::from_angle(a) * speed));
                        }
                    }
                    Pattern::Ring => {
                        for k in 0..6u32 {
                            let d = Vec2::from_angle(k as f32 * 1.047 + jitter(n) * 3.0);
                            let key = KEYS[((n + k) % KEYS.len() as u32) as usize];
                            shadowed(&mut commands, key, me + d * 30.0, 15.0, theme::PAPER, 0.45, d * 300.0, Kind::Particle);
                        }
                    }
                    Pattern::Aura => {}
                    Pattern::Zone { .. } | Pattern::Lob { .. } => {
                        for k in 0..3 {
                            particle(&mut commands, &art, me + dir * 30.0, 6.0, color, Fx::new(Kind::Particle, 0.2, Vec2::from_angle(dir.to_angle() + (jitter(n + k) - 0.5)) * 220.0));
                        }
                    }
                    Pattern::Orbit { .. } if n % 7 == 0 => {}
                    _ => {}
                }
            }
            Event::Beam { from, to } => {
                line(&mut commands, &art, v(from), v(to), 36.0, Color::srgba(1.0, 0.95, 0.6, 0.25), 0.22);
                line(&mut commands, &art, v(from), v(to), 9.0, Color::srgba(1.0, 0.98, 0.8, 0.95), 0.16);
                ring(&mut commands, &art, v(to), 24.0, Color::srgb(1.0, 0.95, 0.6), 0.2);
            }
            Event::Chain { from, to } => {
                // Air Quotes jump in a zigzag, with the quotes flying off.
                let (a, b) = (v(from), v(to));
                let side = (b - a).perp().normalize_or(Vec2::Y);
                let p1 = a.lerp(b, 0.33) + side * 14.0;
                let p2 = a.lerp(b, 0.66) - side * 14.0;
                let gold = theme::type_text(DamageType::Sarcasm);
                for (x, y) in [(a, p1), (p1, p2), (p2, b)] {
                    line(&mut commands, &art, x, y, 4.0, gold, 0.25);
                }
                shadowed(&mut commands, "\"", b + Vec2::new(-8.0, 20.0), 26.0, gold, 0.4, Vec2::new(-40.0, 60.0), Kind::Particle);
                shadowed(&mut commands, "\"", b + Vec2::new(8.0, 20.0), 26.0, gold, 0.4, Vec2::new(40.0, 60.0), Kind::Particle);
            }
            Event::Splash { at, radius } => {
                shake.trauma = shake.trauma.max(0.15);
                ring(&mut commands, &art, v(at), radius, Color::srgba(1.0, 0.85, 0.6, 0.9), 0.3);
                for k in 0..8 {
                    if particles < MAX_PARTICLES {
                        particles += 1;
                        particle(&mut commands, &art, v(at), 9.0, Color::srgb(0.72, 0.35, 0.25), Fx::new(Kind::Particle, 0.45, spread(n * 5 + k) * 300.0));
                    }
                }
            }
            Event::Slam { at, radius } => {
                shake.trauma = (shake.trauma + 0.35).min(1.0);
                if let Some(h) = &hammer {
                    sprite(&mut commands, h, v(at) + Vec2::new(10.0, 30.0), 96.0, 0.0, Fx::new(Kind::Drop, 0.35, Vec2::ZERO));
                }
                ring(&mut commands, &art, v(at), radius, theme::type_text(DamageType::Fact), 0.35);
                for k in 0..10 {
                    particle(&mut commands, &art, v(at), 10.0, Color::srgb(0.7, 0.7, 0.75), Fx::new(Kind::Particle, 0.4, spread(n * 13 + k) * 340.0));
                }
            }
            Event::Ring { at, radius } => ring(&mut commands, &art, v(at), radius, Color::srgba(1.0, 1.0, 1.0, 0.6), 0.2),
            Event::Reflected { at } => {
                if let Some(u) = &uno {
                    sprite(&mut commands, u, v(at), 40.0, 0.0, Fx::new(Kind::Spin, 0.45, Vec2::Y * 40.0));
                }
            }
            Event::BubblePopped { at, line } => {
                let words = BUBBLE_LINES.get(line as usize).copied().unwrap_or("...");
                for (k, ch) in words.chars().filter(|c| c.is_alphanumeric()).step_by(3).take(3).enumerate() {
                    if particles < MAX_PARTICLES {
                        particles += 1;
                        text(&mut commands, ch.to_string(), v(at), 13.0, theme::QUIET, Fx::new(Kind::Particle, 0.5, spread(n * 11 + k as u32) * 160.0));
                    }
                }
            }
            Event::Agreed { at, line } => {
                // A green pulse; now and then the words too.
                ring(&mut commands, &art, v(at), 60.0, Color::srgba(0.5, 0.95, 0.5, 0.6), 0.3);
                if n % 4 == 0 {
                    let w = AGREE_LINES.get(line as usize).copied().unwrap_or("Type AMEN.");
                    say(&mut commands, w, v(at) + Vec2::Y * 40.0, 15.0, Color::srgb(0.5, 0.95, 0.5), 1.0);
                }
            }
            Event::Healed { .. } => {}
            Event::Scattered { .. } => {}
            Event::Reposted { .. } => {}
            Event::Split { .. } => {}
            Event::Hatched { .. } => {}
            Event::Grabbed => {}
            Event::LancerAim { from, dir } => {
                let (from, dir) = (v(from), v(dir));
                let centre = from + dir * (AIM_LENGTH / 2.0);
                let fx = Fx::new(Kind::Warning, LANCE_AIM as f32 / TICK_RATE as f32, Vec2::ZERO).alpha(0.55);
                commands.spawn((
                    Sprite { image: art.dot.clone(), custom_size: Some(Vec2::new(AIM_LENGTH, 3.0)), color: theme::DANGER, ..default() },
                    Transform::from_xyz(centre.x, centre.y, Z_FX).with_rotation(Quat::from_rotation_z(dir.to_angle())),
                    fx,
                ));
                // Clawed thinks out loud while it aims.
                if n % 5 == 0 {
                    say(&mut commands, CLAWED_LINES[(n / 2) as usize % CLAWED_LINES.len()], from + Vec2::Y * 46.0, 15.0, Color::srgb(1.0, 0.66, 0.5), 1.2);
                }
            }
            // A status shows on you as a coloured flash, never as words.
            Event::Status { kind } => ring(&mut commands, &art, me, 46.0, status_color(kind), 0.3),
            Event::ActiveUsed { .. } => {}
            Event::Friend { from, to } => line(&mut commands, &art, v(from), v(to), 14.0, Color::srgba(0.4, 0.9, 0.5, 0.8), 0.35),
            Event::LootBox { .. } => {}
            Event::PickedUp { kind, at } => {
                use slop_sim::world::PickupKind as K;
                let color = match kind {
                    K::Food => Some(theme::AGREE),
                    K::Freeze => Some(Color::srgb(0.6, 0.85, 1.0)),
                    K::Chest => Some(Color::srgb(1.0, 0.8, 0.2)),
                    _ => None,
                };
                if let Some(c) = color {
                    ring(&mut commands, &art, v(at), 80.0, c, 0.35);
                }
                if kind == K::Freeze {
                    shake.trauma = shake.trauma.max(0.3);
                }
            }
            Event::MassReport { at } => {
                shake.trauma = 1.0;
                ring(&mut commands, &art, v(at), 900.0, Color::srgba(1.0, 0.3, 0.3, 0.7), 0.5);
            }
            Event::Broke { at, .. } => {
                ring(&mut commands, &art, v(at), 60.0, Color::srgba(1.0, 1.0, 1.0, 0.6), 0.25);
            }
            Event::PropUsed { .. } => {}
            Event::PropDone { at, .. } => {
                ring(&mut commands, &art, v(at), 220.0, Color::srgba(1.0, 0.8, 0.2, 0.8), 0.4);
            }
            Event::BossDefeated { at, .. } => {
                shake.trauma = 1.0;
                ring(&mut commands, &art, v(at), 500.0, Color::srgba(1.0, 0.8, 0.2, 0.9), 0.6);
            }
            Event::Confused { at } => say(&mut commands, "?", v(at) + Vec2::Y * 34.0, 22.0, Color::srgb(1.0, 0.6, 0.9), 0.7),
            Event::Maintenance { .. } => {}
            Event::Dashed { from, dir } => {
                // Afterimages along the dash.
                for k in 0..4 {
                    let at = v(from) + Vec2::new(dir.x, dir.y) * (k as f32 * 45.0);
                    let fx = Fx::new(Kind::Flash, 0.25 + k as f32 * 0.04, Vec2::ZERO);
                    commands.spawn((
                        Sprite { image: art.human[&game.shown_run().map_or(slop_rules::Hero::Boomer, |r| r.hero)][0].clone(), custom_size: Some(Vec2::splat(HUMAN_SIZE)), color: Color::srgba(0.6, 0.85, 1.0, 0.5), ..default() },
                        Transform::from_xyz(at.x, at.y, Z_FX - 1.0),
                        fx,
                    ));
                }
            }
            Event::ShieldHit { .. } => ring(&mut commands, &art, me, 34.0, Color::srgba(0.31, 0.72, 0.91, 0.8), 0.18),
            Event::ShieldBroke => {
                ring(&mut commands, &art, me, 60.0, Color::srgba(0.31, 0.72, 0.91, 0.9), 0.35);
            }
            Event::ShieldUp => {}
            Event::Deflected { at } => {
                if particles < MAX_PARTICLES {
                    particles += 1;
                    particle(&mut commands, &art, v(at), 10.0, Color::srgb(0.5, 0.85, 1.0), Fx::new(Kind::Particle, 0.25, spread(n) * 120.0));
                }
            }
            Event::Shock { at, .. } | Event::DoNotDisturb { at } => {
                ring(&mut commands, &art, v(at), radius_of(e), Color::srgba(0.75, 0.6, 1.0, 0.85), 0.3);
                shake.trauma = shake.trauma.max(0.2);
            }
            Event::GodMode => {
                ring(&mut commands, &art, me, 120.0, Color::srgba(1.0, 0.9, 0.4, 0.9), 0.5);
            }
            Event::SelfHeal { .. } => {}
            Event::ChargeAim { from, dir } => {
                commands.spawn((
                    Sprite { image: art.dot.clone(), custom_size: Some(Vec2::new(380.0, 6.0)), color: theme::DANGER.with_alpha(0.6), ..default() },
                    Transform::from_xyz(from.x + dir.x * 190.0, from.y + dir.y * 190.0, Z_FX).with_rotation(Quat::from_rotation_z(dir.y.atan2(dir.x))),
                    Fx::new(Kind::Flash, 0.66, Vec2::ZERO),
                ));
            }
            Event::BlinkWarning { at } => {
                ring(&mut commands, &art, v(at), 40.0, theme::DANGER, 0.75);
            }
            Event::Swarm { .. } => {
                shake.trauma = shake.trauma.max(0.3);
            }
            Event::Encircled { at, radius } => ring(&mut commands, &art, v(at), radius, theme::DANGER.with_alpha(0.6), 1.0),
            Event::ArenaWarning { .. } => {}
            Event::ArenaClosed { at, radius } => {
                shake.trauma = shake.trauma.max(0.4);
                ring(&mut commands, &art, v(at), radius, Color::srgba(1.0, 0.25, 0.25, 1.0), 0.5);
            }
            Event::ArenaCleared { at } => {
                shake.trauma = shake.trauma.max(0.5);
                ring(&mut commands, &art, v(at), 300.0, Color::srgba(1.0, 0.8, 0.2, 0.9), 0.6);
            }
            Event::Vacuum => ring(&mut commands, &art, me, 600.0, Color::srgba(0.5, 0.8, 1.0, 0.6), 0.5),
            // Slop turn green and tear at each other; the one thing that talks.
            Event::PromptInjection { at } => {
                shake.trauma = shake.trauma.max(0.3);
                ring(&mut commands, &art, v(at), 700.0, Color::srgba(0.4, 1.0, 0.5, 0.7), 0.5);
                for s in world.slop.iter().take(3) {
                    say(&mut commands, "Sure! Attacking my friends.", v(s.pos) + Vec2::Y * 34.0, 14.0, Color::srgb(0.55, 1.0, 0.6), 1.2);
                }
            }
            Event::RareDrop { at } => {
                ring(&mut commands, &art, v(at), 160.0, Color::srgba(1.0, 0.85, 0.3, 1.0), 0.8);
                shake.trauma = shake.trauma.max(0.4);
            }
            Event::Hustle { .. } => {}
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
            Event::Fired { .. } | Event::LikeCollected { .. } | Event::BombWarning { .. } | Event::AdBreak | Event::DashReady | Event::Biome { .. } => {}
        }
    }
}

/// The radius of a shockwave event.
fn radius_of(e: &Event) -> f32 {
    match *e {
        Event::Shock { radius, .. } => radius,
        Event::DoNotDisturb { .. } => 160.0,
        _ => 0.0,
    }
}

/// Fading dots behind your projectiles, in the colour of their damage type.
fn trails(mut commands: Commands, game: Res<Game>, art: Res<Art>, existing: Query<&Fx>, mut frame: Local<u32>) {
    let Some(world) = game.world() else { return };
    if game.paused || game.frozen {
        return;
    }
    *frame = frame.wrapping_add(1);
    let mut count = existing.iter().filter(|f| f.kind == Kind::Trail).count();
    for p in world.projectiles.iter().filter(|p| !p.hostile && (p.id + *frame) % 2 == 0) {
        if count >= MAX_TRAILS {
            break;
        }
        if matches!(p.look, Look::Bubble | Look::Post) {
            continue;
        }
        count += 1;
        let fx = Fx::new(Kind::Trail, 0.18, Vec2::ZERO).alpha(0.6);
        particle(&mut commands, &art, v(p.pos), (p.radius * 1.3).clamp(5.0, 14.0), theme::type_text(p.kind), fx);
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
                tf.scale = Vec3::splat(1.0 + 0.6 * (left - 0.75).max(0.0) * 4.0);
                f.base_alpha * left.sqrt()
            }
            Kind::Ring => {
                tf.scale = Vec3::splat(0.3 + 0.7 * (1.0 - left));
                f.base_alpha * left
            }
            Kind::Spin => {
                tf.rotate_z(14.0 * dt);
                f.base_alpha * left.sqrt()
            }
            Kind::Drop => {
                // Swings down in the first third of its life, then rests.
                let s = ((1.0 - left) * 3.0).min(1.0);
                tf.rotation = Quat::from_rotation_z(1.3 * (1.0 - s) - 0.25 * s);
                f.base_alpha * (left * 2.0).min(1.0)
            }
            Kind::Trail => {
                tf.scale = Vec3::splat(left);
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
