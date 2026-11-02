//! Draws the arena from the simulation every frame: the floor, you, the slop
//! with their health bars, projectiles and Likes (UI-02, UI-03).
//!
//! Every simulation object has a stable id; a view is spawned the first frame
//! an id appears and despawned the first frame it is gone.

use std::collections::{HashMap, HashSet};

use bevy::camera::ScalingMode;
use bevy::prelude::*;
use bevy::sprite::{BorderRect, SliceScaleMode, SpriteImageMode, TextureSlicer};
use slop_sim::content::{Look, BUBBLE_LINES};
use slop_sim::world::{Outcome, ARENA, FLASH_TICKS};
use slop_sim::SlopKind;

use crate::art::{Art, BUBBLE_BORDER, FACES, HUMAN_SIZE, SLOP_SCALE};
use crate::play::Simulation;
use crate::state::{Game, Screen};
use crate::theme;

/// Draw order, back to front.
const Z_FLOOR: f32 = 0.0;
const Z_LIKE: f32 = 2.0;
const Z_ACTOR: f32 = 3.0;
const Z_SHOT: f32 = 5.0;
const Z_BUBBLE: f32 = 6.0;
pub const Z_FX: f32 = 8.0;
pub const Z_TEXT: f32 = 9.0;

/// The smallest part of the arena any window shows.
pub const VIEW: Vec2 = Vec2::new(1600.0, 900.0);
/// The camera may show this much wall beyond the arena.
const WALL_SHOWN: f32 = 160.0;
/// How far ahead of you the camera looks, in seconds of movement.
const CAMERA_LEAD: f32 = 0.3;
const CAMERA_FOLLOW: f32 = 7.0;

pub struct ViewsPlugin;

impl Plugin for ViewsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Shake>()
            .add_systems(Startup, setup)
            .add_systems(Update, (sync, follow_camera).chain().after(Simulation));
    }
}

/// Screen shake: decays every frame; hits add to it.
#[derive(Resource, Default)]
pub struct Shake {
    pub trauma: f32,
}

struct SlopView {
    root: Entity,
    body: Entity,
    bar_back: Entity,
    bar_fill: Entity,
    /// "[citation needed]" while marked by the Red Pen.
    tag: Entity,
}

#[derive(Resource)]
struct Views {
    /// Seed of the wave the views belong to.
    seed: u64,
    tick: u32,
    player_root: Entity,
    player_body: Entity,
    slop: HashMap<u32, SlopView>,
    shots: HashMap<u32, Entity>,
    likes: HashMap<u32, Entity>,
}

#[derive(Component)]
pub struct MainCamera;

/// A view's root: moved to the object's position.
#[derive(Component)]
struct Root;

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let art = Art::build(&mut images);
    let centre = Vec2::splat(ARENA / 2.0);
    // Any window shows at least 1600 x 900 of the arena, so a bigger window
    // shows the same game, larger (UI-07).
    let projection = OrthographicProjection {
        scaling_mode: ScalingMode::AutoMin { min_width: VIEW.x, min_height: VIEW.y },
        ..OrthographicProjection::default_2d()
    };
    commands.spawn((Camera2d, MainCamera, Projection::from(projection), Transform::from_xyz(centre.x, centre.y, 100.0)));
    commands.spawn((
        Sprite {
            image: art.floor.clone(),
            custom_size: Some(Vec2::splat(ARENA)),
            image_mode: SpriteImageMode::Tiled { tile_x: true, tile_y: true, stretch_value: 1.0 },
            ..default()
        },
        Transform::from_xyz(centre.x, centre.y, Z_FLOOR),
    ));
    // Walls around the arena, and a dark rim on its edge.
    let thick = 900.0;
    for (x, y, w, h) in [
        (-thick / 2.0, centre.y, thick, ARENA + 2.0 * thick),
        (ARENA + thick / 2.0, centre.y, thick, ARENA + 2.0 * thick),
        (centre.x, -thick / 2.0, ARENA, thick),
        (centre.x, ARENA + thick / 2.0, ARENA, thick),
    ] {
        commands.spawn((Sprite::from_color(theme::WALL, Vec2::new(w, h)), Transform::from_xyz(x, y, Z_FLOOR + 0.1)));
    }
    let rim = 8.0;
    for (x, y, w, h) in [
        (-rim / 2.0, centre.y, rim, ARENA + 2.0 * rim),
        (ARENA + rim / 2.0, centre.y, rim, ARENA + 2.0 * rim),
        (centre.x, -rim / 2.0, ARENA, rim),
        (centre.x, ARENA + rim / 2.0, ARENA, rim),
    ] {
        commands.spawn((Sprite::from_color(theme::INK, Vec2::new(w, h)), Transform::from_xyz(x, y, Z_FLOOR + 0.2)));
    }

    let player_root = commands.spawn((Root, Transform::from_xyz(centre.x, centre.y, Z_ACTOR), Visibility::Hidden)).id();
    commands.spawn((
        Sprite { image: art.shadow.clone(), custom_size: Some(Vec2::new(44.0, 14.0)), ..default() },
        Transform::from_xyz(0.0, -HUMAN_SIZE * 0.42, -0.05),
        ChildOf(player_root),
    ));
    let player_body = commands
        .spawn((
            Sprite { image: art.human[0].clone(), custom_size: Some(Vec2::splat(HUMAN_SIZE)), ..default() },
            Transform::default(),
            ChildOf(player_root),
        ))
        .id();
    commands.insert_resource(Views {
        seed: u64::MAX,
        tick: 0,
        player_root,
        player_body,
        slop: HashMap::new(),
        shots: HashMap::new(),
        likes: HashMap::new(),
    });
    commands.insert_resource(art);
}

/// Depth by height on screen, so lower actors overlap higher ones.
fn depth(y: f32) -> f32 {
    (ARENA - y) / ARENA * 0.9
}

fn spawn_slop(commands: &mut Commands, art: &Art, kind: SlopKind, number: u8) -> SlopView {
    let r = kind.def().radius;
    let root = commands.spawn((Root, Transform::default(), Visibility::Visible)).id();
    commands.spawn((
        Sprite { image: art.shadow.clone(), custom_size: Some(Vec2::new(r * 2.6, r * 0.8)), ..default() },
        Transform::from_xyz(0.0, -r * 0.95, -0.05),
        ChildOf(root),
    ));
    let size = r * SLOP_SCALE;
    let body = commands
        .spawn((
            Sprite { image: art.slop[&kind].0.clone(), custom_size: Some(Vec2::splat(size)), ..default() },
            Transform::default(),
            ChildOf(root),
        ))
        .id();
    if kind == SlopKind::Listicle {
        commands.spawn((
            Text2d::new(number.to_string()),
            TextFont { font_size: 20.0, ..default() },
            TextColor(theme::INK),
            Transform::from_xyz(0.0, 0.0, 0.05),
            ChildOf(body),
        ));
    }
    let bar_y = size * 0.55 + 6.0;
    let bar_back = commands
        .spawn((
            Sprite::from_color(theme::INK, Vec2::new(r * 2.4 + 4.0, 8.0)),
            Transform::from_xyz(0.0, bar_y, 0.1),
            Visibility::Hidden,
            ChildOf(root),
        ))
        .id();
    let bar_fill = commands
        .spawn((
            Sprite::from_color(theme::PLAYER_RED, Vec2::new(r * 2.4, 4.0)),
            Transform::from_xyz(0.0, bar_y, 0.11),
            Visibility::Hidden,
            ChildOf(root),
        ))
        .id();
    let tag = commands
        .spawn((
            Text2d::new("[citation needed]"),
            TextFont { font_size: 12.0, ..default() },
            TextColor(theme::type_color(slop_sim::DamageType::Fact)),
            Transform::from_xyz(0.0, bar_y + 14.0, 0.12),
            Visibility::Hidden,
            ChildOf(root),
        ))
        .id();
    SlopView { root, body, bar_back, bar_fill, tag }
}

/// Bubble text is short; its frame is sized to it.
fn bubble_width(line: &str) -> f32 {
    line.len() as f32 * 7.2 + 26.0
}

fn spawn_shot(commands: &mut Commands, art: &Art, look: Look, line: u16) -> Entity {
    if look == Look::Bubble {
        let text = BUBBLE_LINES.get(line as usize).copied().unwrap_or("...");
        let slicer = TextureSlicer {
            border: BorderRect::all(BUBBLE_BORDER),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 0.5,
        };
        let root = commands
            .spawn((
                Root,
                Sprite {
                    image: art.bubble.clone(),
                    custom_size: Some(Vec2::new(bubble_width(text), 28.0)),
                    image_mode: SpriteImageMode::Sliced(slicer),
                    ..default()
                },
                Transform::default(),
            ))
            .id();
        commands.spawn((
            Text2d::new(text),
            TextFont { font_size: 13.0, ..default() },
            TextColor(theme::INK),
            Transform::from_xyz(0.0, 0.0, 0.05),
            ChildOf(root),
        ));
        return root;
    }
    let size = match look {
        Look::Rock => Vec2::splat(20.0),
        Look::Bullet => Vec2::new(16.0, 8.0),
        Look::Pen => Vec2::new(26.0, 9.0),
        Look::EyeRoll => Vec2::splat(22.0),
        Look::EmDash => Vec2::new(54.0, 11.0),
        Look::Bubble => unreachable!(),
    };
    commands.spawn((Root, Sprite { image: art.shots[&look].clone(), custom_size: Some(size), ..default() }, Transform::default())).id()
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
fn sync(
    mut commands: Commands,
    game: Res<Game>,
    art: Res<Art>,
    time: Res<Time>,
    mut views: ResMut<Views>,
    mut roots: Query<(&mut Transform, &mut Visibility), (With<Root>, Without<Sprite>)>,
    mut shots: Query<&mut Transform, (With<Root>, With<Sprite>)>,
    mut parts: Query<(&mut Transform, &mut Sprite, &mut Visibility), Without<Root>>,
    mut tags: Query<&mut Visibility, (With<Text2d>, Without<Sprite>, Without<Root>)>,
) {
    let views = &mut *views;
    let world = &game.world;
    let t = time.elapsed_secs();

    // A new wave: forget every view of the old one.
    if views.seed != world.seed || world.tick < views.tick {
        for v in views.slop.drain().map(|(_, v)| v) {
            commands.entity(v.root).despawn();
        }
        for e in views.shots.drain().map(|(_, e)| e).chain(views.likes.drain().map(|(_, e)| e)) {
            commands.entity(e).despawn();
        }
        views.seed = world.seed;
    }
    views.tick = world.tick;
    let playing = game.screen != Screen::Title;

    // You.
    if let Ok((mut tf, mut vis)) = roots.get_mut(views.player_root) {
        let p = world.player.pos;
        tf.translation = Vec3::new(p.x, p.y, Z_ACTOR + depth(p.y));
        *vis = if playing { Visibility::Visible } else { Visibility::Hidden };
    }
    if let Ok((mut tf, mut sprite, mut vis)) = parts.get_mut(views.player_body) {
        let p = &world.player;
        let share = (p.mental_health / p.stats.max_mental_health).clamp(0.0, 1.0);
        let face = ((1.0 - share) * FACES as f32).min(FACES as f32 - 1.0) as usize;
        sprite.image = if p.invulnerable > 24 { art.human_flash.clone() } else { art.human[face].clone() };
        if p.vel.x.abs() > 5.0 {
            sprite.flip_x = p.vel.x < 0.0;
        }
        let speed = p.vel.length() / slop_sim::world::PLAYER_SPEED;
        let bob = (t * if speed > 0.1 { 14.0 } else { 3.0 }).sin();
        tf.scale = Vec3::new(1.0 - 0.05 * bob * speed.max(0.3), 1.0 + 0.07 * bob * speed.max(0.3), 1.0);
        tf.translation.y = 3.0 * bob.abs() * speed;
        // Blink while invulnerable after a hit (FAIR-03).
        *vis = if p.invulnerable > 0 && (p.invulnerable / 4) % 2 == 1 { Visibility::Hidden } else { Visibility::Inherited };
    }

    // Slop. When the wave is cleared they are deleted by the effects instead.
    let showing = world.outcome != Outcome::Cleared;
    let mut seen = HashSet::new();
    for s in world.slop.iter().filter(|_| showing) {
        seen.insert(s.id);
        let v = views.slop.entry(s.id).or_insert_with(|| spawn_slop(&mut commands, &art, s.kind, s.number));
        if let Ok((mut tf, _)) = roots.get_mut(v.root) {
            tf.translation = Vec3::new(s.pos.x, s.pos.y, Z_ACTOR + depth(s.pos.y));
        }
        if let Ok((mut tf, mut sprite, _)) = parts.get_mut(v.body) {
            let (normal, flash) = &art.slop[&s.kind];
            sprite.image = if s.flash > 0 { flash.clone() } else { normal.clone() };
            if s.moving.x.abs() > 1.0 {
                sprite.flip_x = s.moving.x < 0.0;
            }
            sprite.color = if s.agreed > 0 { Color::srgb(0.8, 1.0, 0.8) } else { Color::WHITE };
            let phase = t * 9.0 + s.id as f32;
            let hit = s.flash as f32 / FLASH_TICKS as f32;
            let aiming = if s.charging > 0 { 1.0 + 0.12 * (t * 30.0).sin() } else { 1.0 };
            tf.scale = Vec3::new((1.0 + 0.04 * phase.sin() + 0.15 * hit) * aiming, (1.0 - 0.04 * phase.sin() + 0.15 * hit) * aiming, 1.0);
        }
        let show_bar = s.shows_health(world.tick);
        let full = s.kind.def().radius * 2.4;
        if let Ok((_, _, mut vis)) = parts.get_mut(v.bar_back) {
            *vis = if show_bar { Visibility::Inherited } else { Visibility::Hidden };
        }
        if let Ok((mut tf, mut sprite, mut vis)) = parts.get_mut(v.bar_fill) {
            *vis = if show_bar { Visibility::Inherited } else { Visibility::Hidden };
            let w = full * (s.hp / s.max_hp).clamp(0.0, 1.0);
            sprite.custom_size = Some(Vec2::new(w, 4.0));
            tf.translation.x = -(full - w) / 2.0;
        }
        if let Ok(mut vis) = tags.get_mut(v.tag) {
            *vis = if s.marked > 0 { Visibility::Inherited } else { Visibility::Hidden };
        }
    }
    views.slop.retain(|id, v| {
        let keep = seen.contains(id);
        if !keep {
            commands.entity(v.root).despawn();
        }
        keep
    });

    // Projectiles and speech bubbles.
    let mut seen = HashSet::new();
    for p in world.projectiles.iter().filter(|_| showing) {
        seen.insert(p.id);
        let e = *views.shots.entry(p.id).or_insert_with(|| spawn_shot(&mut commands, &art, p.look, p.line));
        if let Ok(mut tf) = shots.get_mut(e) {
            let z = if p.look == Look::Bubble { Z_BUBBLE } else { Z_SHOT };
            tf.translation = Vec3::new(p.pos.x, p.pos.y, z + (p.id % 100) as f32 * 0.001);
            tf.rotation = match p.look {
                Look::Bubble => Quat::IDENTITY,
                Look::Rock | Look::EyeRoll => Quat::from_rotation_z(t * 9.0 + p.id as f32),
                _ => Quat::from_rotation_z(p.vel.angle()),
            };
        }
    }
    views.shots.retain(|id, e| {
        let keep = seen.contains(id);
        if !keep {
            commands.entity(*e).despawn();
        }
        keep
    });

    // Likes.
    let mut seen = HashSet::new();
    for l in &world.likes {
        seen.insert(l.id);
        let e = *views.likes.entry(l.id).or_insert_with(|| {
            commands
                .spawn((Root, Sprite { image: art.like.clone(), custom_size: Some(Vec2::splat(18.0)), ..default() }, Transform::default()))
                .id()
        });
        if let Ok(mut tf) = shots.get_mut(e) {
            let pulse = 1.0 + 0.12 * (t * 6.0 + l.id as f32).sin();
            tf.translation = Vec3::new(l.pos.x, l.pos.y, Z_LIKE);
            tf.scale = Vec3::splat(pulse * if l.value > 1 { 1.25 } else { 1.0 });
        }
    }
    views.likes.retain(|id, e| {
        let keep = seen.contains(id);
        if !keep {
            commands.entity(*e).despawn();
        }
        keep
    });
}

fn follow_camera(
    time: Res<Time>,
    game: Res<Game>,
    mut shake: ResMut<Shake>,
    mut camera: Query<(&mut Transform, &Projection), With<MainCamera>>,
    mut focus: Local<Option<Vec2>>,
    mut last_seed: Local<u64>,
) {
    let Ok((mut tf, projection)) = camera.single_mut() else { return };
    let half = match projection {
        Projection::Orthographic(o) => o.area.half_size(),
        _ => VIEW / 2.0,
    };
    let dt = time.delta_secs();
    let p = &game.world.player;
    let target = if game.screen == Screen::Title {
        Vec2::splat(ARENA / 2.0)
    } else {
        Vec2::new(p.pos.x + p.vel.x * CAMERA_LEAD, p.pos.y + p.vel.y * CAMERA_LEAD)
    };
    if *last_seed != game.world.seed {
        *last_seed = game.world.seed;
        *focus = Some(target);
    }
    let f = focus.get_or_insert(target);
    *f += (target - *f) * (1.0 - (-CAMERA_FOLLOW * dt).exp());
    let lo = half - Vec2::splat(WALL_SHOWN);
    let hi = Vec2::splat(ARENA) - half + Vec2::splat(WALL_SHOWN);
    // A view wider than the arena stays centred on it.
    let at = Vec2::new(
        if lo.x < hi.x { f.x.clamp(lo.x, hi.x) } else { ARENA / 2.0 },
        if lo.y < hi.y { f.y.clamp(lo.y, hi.y) } else { ARENA / 2.0 },
    );
    shake.trauma = (shake.trauma - dt * 1.8).max(0.0);
    let k = shake.trauma * shake.trauma * 14.0;
    let wobble = Vec2::new((time.elapsed_secs() * 61.0).sin(), (time.elapsed_secs() * 47.0).cos()) * k;
    tf.translation.x = at.x + wobble.x;
    tf.translation.y = at.y + wobble.y;
}
