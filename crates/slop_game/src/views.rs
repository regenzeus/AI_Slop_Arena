//! Draws the arena from the simulation every frame: the platform floor, you,
//! the slop as characters with health bars, projectiles, Clout coins, zones
//! and the dog (UI-02, UI-03, ART-03).
//!
//! Every simulation object has a stable id; a view is spawned the first frame
//! an id appears and despawned the first frame it is gone.

use std::collections::{HashMap, HashSet};

use bevy::camera::ScalingMode;
use bevy::prelude::*;
use bevy::sprite::{BorderRect, SliceScaleMode, SpriteImageMode, TextureSlicer};
use slop_rules::Hero;
use slop_sim::content::{Archetype, BossKind, Look, Pattern, Platform, BUBBLE_LINES, POST_LINES, WEAPONS};
use slop_sim::Event;
use slop_sim::spec::{SlopLook, SlopSpec, Traits};
use slop_sim::world::{Outcome, Slop, World, ZoneKind, ARENA, FLASH_TICKS, NO_LINE, PLAYER_SPEED};

use crate::art::{Art, Pair, BUBBLE_BORDER, FACES, HUMAN_SIZE, SLOP_SCALE};
use crate::play::Simulation;
use crate::state::{Game, Screen};
use crate::theme;

/// Draw order, back to front.
const Z_FLOOR: f32 = 0.0;
const Z_ZONE: f32 = 1.0;
const Z_LIKE: f32 = 2.0;
const Z_ACTOR: f32 = 3.0;
const Z_SHOT: f32 = 5.0;
const Z_BUBBLE: f32 = 6.0;
pub const Z_FX: f32 = 8.0;
pub const Z_TEXT: f32 = 9.0;

/// The smallest part of the arena any window shows.
pub const VIEW: Vec2 = Vec2::new(1440.0, 810.0);
/// The camera may show this much wall beyond the arena.
const WALL_SHOWN: f32 = 160.0;
/// How fast the camera catches up with you. It never looks ahead, so turning
/// around does not swing the view.
const CAMERA_FOLLOW: f32 = 14.0;
/// Distance of carried weapons from your centre.
const HOLD: f32 = 38.0;
const WEAPON_SIZE: f32 = 34.0;
/// Height of a lobbed brick at the top of its arc.
const LOB_HEIGHT: f32 = 80.0;

pub struct ViewsPlugin;

impl Plugin for ViewsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Shake>().add_systems(Startup, setup).add_systems(Update, (sync, follow_camera).chain().after(Simulation));
    }
}

/// Screen shake: decays every frame; hits add to it.
#[derive(Resource, Default)]
pub struct Shake {
    pub trauma: f32,
}

struct SlopView {
    root: Entity,
    /// Turned to face where the slop moves; holds the part sprites.
    body: Entity,
    parts: Vec<(Entity, Pair)>,
    bar_back: Entity,
    bar_fill: Entity,
    /// "[citation needed]" while marked by the Red Pen.
    tag: Entity,
    shield: Option<Entity>,
    stun: Entity,
}

/// How a carried weapon moves when it attacks.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Motion {
    /// Kicks back, like a gun.
    Recoil,
    /// Lunges forward and spins a little, like a throw.
    Throw,
    /// Sweeps through its arc, out and back.
    Swing,
    /// Swells and shrinks: claps, keys, eyes.
    Pulse,
    /// Not drawn in your hand (the dog walks itself).
    Hidden,
}

/// The direction a weapon's icon points, in degrees (None: always upright),
/// and how it moves (WPN-09).
fn hold(def: usize) -> (Option<f32>, Motion) {
    match WEAPONS[def].pattern {
        Pattern::Orbit { .. } => (None, Motion::Hidden),
        Pattern::Arc { .. } => (Some(48.0), Motion::Swing),
        Pattern::Ring | Pattern::Aura | Pattern::Chain { .. } | Pattern::Reflect | Pattern::Ricochet { .. } => (None, Motion::Pulse),
        Pattern::Lob { .. } | Pattern::Boomerang { .. } | Pattern::Zone { .. } | Pattern::Slam { .. } => (None, Motion::Throw),
        Pattern::Cone { .. } => (Some(35.0), Motion::Recoil),
        Pattern::Beam { .. } => (Some(36.0), Motion::Recoil),
        Pattern::Mark { .. } => (Some(46.0), Motion::Recoil),
        Pattern::Shot { look: Look::Rock, .. } if WEAPONS[def].name == "Rock" => (None, Motion::Throw),
        Pattern::Shot { look: Look::Rock, .. } => (Some(90.0), Motion::Recoil),
        Pattern::Shot { look: Look::Book, .. } => (None, Motion::Throw),
        Pattern::Stun { .. } => (None, Motion::Recoil),
        _ => (Some(0.0), Motion::Recoil),
    }
}

struct WeaponView {
    e: Entity,
    def: usize,
    /// Current and wanted aim, in radians.
    aim: f32,
    target: f32,
    /// 1 right after an attack, decaying.
    kick: f32,
    /// Swing progress left, 1 to 0, and the swing's half-width.
    swing: f32,
    half: f32,
}

#[derive(Resource)]
struct Views {
    /// Seed of the wave the views belong to.
    seed: u64,
    platform: Option<Platform>,
    floor: Entity,
    player_root: Entity,
    player_body: Entity,
    swirl: Entity,
    slop: HashMap<u32, SlopView>,
    /// A projectile's view and the life it started with (for lob arcs).
    shots: HashMap<u32, (Entity, u16)>,
    likes: HashMap<u32, Entity>,
    zones: HashMap<u32, Entity>,
    /// A Dog Walk's dog and its leash, by weapon slot.
    dogs: HashMap<usize, (Entity, Entity)>,
    weapons: Vec<WeaponView>,
}

#[derive(Component)]
pub struct MainCamera;

/// A view's root, moved to the object's position.
#[derive(Component)]
struct Root;

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let art = Art::build(&mut images);
    let centre = Vec2::splat(ARENA / 2.0);
    // Any window shows at least 1600 x 900 of the arena (UI-07).
    let projection = OrthographicProjection { scaling_mode: ScalingMode::AutoMin { min_width: VIEW.x, min_height: VIEW.y }, ..OrthographicProjection::default_2d() };
    commands.spawn((Camera2d, MainCamera, Projection::from(projection), Transform::from_xyz(centre.x, centre.y, 100.0)));
    let floor = commands
        .spawn((
            Sprite {
                image: art.floors[&Platform::FaceSlop].clone(),
                custom_size: Some(Vec2::splat(ARENA)),
                image_mode: SpriteImageMode::Tiled { tile_x: true, tile_y: true, stretch_value: 1.0 },
                ..default()
            },
            Transform::from_xyz(centre.x, centre.y, Z_FLOOR),
        ))
        .id();
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
        .spawn((Sprite { image: art.human[&Hero::Boomer][0].clone(), custom_size: Some(Vec2::splat(HUMAN_SIZE)), ..default() }, Transform::default(), ChildOf(player_root)))
        .id();
    let swirl = commands
        .spawn((
            Sprite { image: art.swirl.clone(), custom_size: Some(Vec2::splat(30.0)), ..default() },
            Transform::from_xyz(0.0, HUMAN_SIZE * 0.75, 0.2),
            Visibility::Hidden,
            ChildOf(player_root),
        ))
        .id();
    commands.insert_resource(Views {
        seed: u64::MAX,
        platform: None,
        floor,
        player_root,
        player_body,
        swirl,
        slop: HashMap::new(),
        shots: HashMap::new(),
        likes: HashMap::new(),
        zones: HashMap::new(),
        dogs: HashMap::new(),
        weapons: Vec::new(),
    });
    commands.insert_resource(art);
}

/// Depth by height on screen, so lower actors overlap higher ones.
fn depth(y: f32) -> f32 {
    (ARENA - y) / ARENA * 0.9
}

fn text(commands: &mut Commands, words: impl Into<String>, size: f32, color: Color, at: Vec3, parent: Entity) -> Entity {
    commands.spawn((Text2d::new(words), TextFont { font_size: size, ..default() }, TextColor(color), Transform::from_translation(at), ChildOf(parent))).id()
}

/// The sprites a slop is made of: one picture, or a brainrot's three parts.
fn parts_of(art: &Art, spec: &SlopSpec, world: &World) -> Vec<Pair> {
    match spec.look {
        SlopLook::Archetype(a) => vec![art.archetypes[&a].clone()],
        SlopLook::Brainrot { body, object, accessory } => vec![art.bodies[&body].clone(), art.objects[&object].clone(), art.accessories[&accessory].clone()],
        SlopLook::Boss(BossKind::SupremeBrainrot) => {
            let mut v = Vec::new();
            if let Some((b, o, a)) = world.setup.model.parts {
                v = vec![art.bodies[&b].clone(), art.objects[&o].clone(), art.accessories[&a].clone()];
            }
            v.push(art.bosses[&BossKind::SupremeBrainrot].clone());
            v
        }
        SlopLook::Boss(kind) => vec![art.bosses[&kind].clone()],
    }
}

fn spawn_slop(commands: &mut Commands, art: &Art, world: &World, s: &Slop) -> SlopView {
    let spec = world.spec(s);
    // Brainrot parts leave room in their frame for each other, so they are drawn larger.
    let size = spec.radius * SLOP_SCALE * match spec.look {
        SlopLook::Boss(_) => 0.85,
        SlopLook::Brainrot { .. } => 1.25,
        SlopLook::Archetype(_) => 1.0,
    };
    let root = commands.spawn((Root, Transform::default(), Visibility::Visible)).id();
    commands.spawn((
        Sprite { image: art.shadow.clone(), custom_size: Some(Vec2::new(spec.radius * 2.6, spec.radius * 0.8)), ..default() },
        Transform::from_xyz(0.0, -spec.radius * 0.95, -0.05),
        ChildOf(root),
    ));
    if matches!(spec.look, SlopLook::Boss(BossKind::CrustaceanSaviour)) {
        commands.spawn((Sprite { image: art.halo.clone(), custom_size: Some(Vec2::new(size * 0.6, size * 0.3)), ..default() }, Transform::from_xyz(0.0, size * 0.42, 0.3), ChildOf(root)));
    }
    let body = commands.spawn((Transform::default(), Visibility::Inherited, ChildOf(root))).id();
    let mut parts = Vec::new();
    let clone = spec.traits.has(Traits::CLONE);
    for (k, pair) in parts_of(art, spec, world).into_iter().enumerate() {
        let mut part_size = size;
        if k == 3 {
            // The Supreme Brainrot's crown sits on top of its parts.
            part_size = size * 0.6;
        }
        let y = if k == 3 { size * 0.35 } else { 0.0 };
        let color = if clone { Color::srgba(1.0, 1.0, 1.0, 0.75) } else { Color::WHITE };
        let e = commands
            .spawn((Sprite { image: pair.0.clone(), custom_size: Some(Vec2::splat(part_size)), color, ..default() }, Transform::from_xyz(0.0, y, 0.01 * k as f32), ChildOf(body)))
            .id();
        parts.push((e, pair));
    }
    match spec.look {
        SlopLook::Archetype(Archetype::ListicleDrone) => {
            text(commands, s.number.to_string(), 18.0, Color::WHITE, Vec3::new(0.0, -size * 0.07, 0.1), root);
        }
        SlopLook::Archetype(Archetype::EngagementBait) => {
            text(commands, "TYPE\nAMEN", 12.0, theme::INK, Vec3::new(0.0, size * 0.07, 0.1), root);
        }
        SlopLook::Archetype(Archetype::SubscribePopup) => {
            text(commands, "SUBSCRIBE", 11.0, Color::WHITE, Vec3::new(0.0, size * 0.02, 0.1), root);
        }
        _ => {}
    }
    let bar_y = size * 0.55 + 6.0;
    if spec.elite {
        text(commands, "TRENDING", 12.0, theme::PLAYER_RED, Vec3::new(0.0, bar_y + 26.0, 0.12), root);
    }
    if s.reposted {
        text(commands, "[repost]", 12.0, theme::QUIET, Vec3::new(0.0, bar_y + 26.0, 0.12), root);
    }
    let w = spec.radius * 2.4;
    let bar_back = commands.spawn((Sprite::from_color(theme::INK, Vec2::new(w + 4.0, 8.0)), Transform::from_xyz(0.0, bar_y, 0.1), Visibility::Hidden, ChildOf(root))).id();
    let bar_fill = commands.spawn((Sprite::from_color(theme::PLAYER_RED, Vec2::new(w, 4.0)), Transform::from_xyz(0.0, bar_y, 0.11), Visibility::Hidden, ChildOf(root))).id();
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
    let shield = spec.traits.has(Traits::FRONT_SHIELD).then(|| {
        commands.spawn((Sprite { image: art.shield.clone(), custom_size: Some(Vec2::splat(size * 1.1)), ..default() }, Transform::from_xyz(0.0, 0.0, 0.2), ChildOf(root))).id()
    });
    let stun = commands
        .spawn((Sprite { image: art.swirl.clone(), custom_size: Some(Vec2::splat(20.0)), color: Color::srgb(0.5, 0.8, 1.0), ..default() }, Transform::from_xyz(0.0, bar_y + 8.0, 0.2), Visibility::Hidden, ChildOf(root)))
        .id();
    SlopView { root, body, parts, bar_back, bar_fill, tag, shield, stun }
}

/// Bubble text is short; its frame is sized to it.
fn bubble_width(line: &str) -> f32 {
    line.len() as f32 * 7.2 + 26.0
}

fn spawn_shot(commands: &mut Commands, art: &Art, look: Look, line: u16, hostile: bool) -> Entity {
    if matches!(look, Look::Bubble | Look::Post) {
        let lines = if look == Look::Post { POST_LINES } else { BUBBLE_LINES };
        let text_line = if line == NO_LINE { "..." } else { lines.get(line as usize).copied().unwrap_or("...") };
        let slicer = TextureSlicer { border: BorderRect::all(BUBBLE_BORDER), center_scale_mode: SliceScaleMode::Stretch, sides_scale_mode: SliceScaleMode::Stretch, max_corner_scale: 0.5 };
        let size = Vec2::new(bubble_width(text_line), if look == Look::Post { 32.0 } else { 28.0 });
        let root = commands
            .spawn((Root, Sprite { image: art.bubble.clone(), custom_size: Some(size), image_mode: SpriteImageMode::Sliced(slicer), ..default() }, Transform::default()))
            .id();
        let color = if look == Look::Post { Color::srgb(0.04, 0.4, 0.76) } else { theme::INK };
        text(commands, text_line, 13.0, color, Vec3::new(0.0, 0.0, 0.05), root);
        return root;
    }
    let size = match look {
        Look::Rock => Vec2::splat(20.0),
        Look::Bullet => Vec2::new(16.0, 8.0),
        Look::Pen => Vec2::new(26.0, 9.0),
        Look::EyeRoll | Look::Frisbee => Vec2::splat(22.0),
        Look::EmDash => Vec2::new(54.0, 11.0),
        Look::Book => Vec2::new(26.0, 21.0),
        Look::Brick => Vec2::new(26.0, 16.0),
        Look::Staple => Vec2::new(14.0, 7.0),
        Look::Captcha => Vec2::splat(18.0),
        Look::Toast => Vec2::splat(22.0),
        Look::Arrow => Vec2::new(34.0, 20.0),
        Look::Letter => Vec2::new(36.0, 27.0),
        Look::Bubble | Look::Post => unreachable!(),
    };
    let root = commands.spawn((Root, Sprite { image: art.shots[&look].clone(), custom_size: Some(size), ..default() }, Transform::default())).id();
    // Your weapons copied by The Model hurt you: they get the danger outline (FAIR-04).
    let danger_looks = !matches!(look, Look::EmDash | Look::Toast | Look::Arrow | Look::Letter);
    if hostile && danger_looks {
        commands.spawn((Sprite { image: art.danger_ring.clone(), custom_size: Some(Vec2::splat(size.x.max(size.y) * 1.7)), ..default() }, Transform::from_xyz(0.0, 0.0, -0.01), ChildOf(root)));
    }
    root
}

fn spawn_zone(commands: &mut Commands, art: &Art, kind: ZoneKind, radius: f32) -> Entity {
    let (image, size) = match kind {
        ZoneKind::Seed { .. } => (art.seed.clone(), Vec2::splat(radius * 2.0)),
        ZoneKind::Bomb => (art.bomb.clone(), Vec2::splat(radius * 2.0)),
        ZoneKind::Downvote => (art.downvote.clone(), Vec2::splat(radius * 2.0)),
        ZoneKind::Filter => (art.filter.clone(), Vec2::splat(radius * 2.0)),
        ZoneKind::Banner => {
            let root = commands.spawn((Root, Sprite::from_color(Color::srgba(0.25, 0.65, 0.35, 0.35), Vec2::new(ARENA, radius * 2.0)), Transform::default())).id();
            for x in [-800.0, -400.0, 0.0, 400.0, 800.0] {
                text(commands, "#OPENTOWORK", 22.0, Color::srgba(0.1, 0.4, 0.2, 0.9), Vec3::new(x, 0.0, 0.05), root);
            }
            return root;
        }
    };
    commands.spawn((Root, Sprite { image, custom_size: Some(size), ..default() }, Transform::default())).id()
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn sync(
    mut commands: Commands,
    game: Res<Game>,
    art: Res<Art>,
    time: Res<Time>,
    mut views: ResMut<Views>,
    mut roots: Query<(&mut Transform, &mut Visibility), (With<Root>, Without<Sprite>)>,
    mut sprite_roots: Query<(&mut Transform, &mut Sprite), (With<Root>, With<Sprite>)>,
    mut parts: Query<(&mut Transform, &mut Sprite, &mut Visibility), Without<Root>>,
    mut groups: Query<&mut Transform, (Without<Root>, Without<Sprite>, Without<Text2d>, Without<Camera>)>,
    mut tags: Query<&mut Visibility, (With<Text2d>, Without<Sprite>, Without<Root>)>,
) {
    let views = &mut *views;
    let t = time.elapsed_secs();
    let world = game.world();

    // The platform's floor.
    let platform = game.shown_run().map(|r| r.platform);
    if platform != views.platform {
        views.platform = platform;
        if let Ok((_, mut sprite, _)) = parts.get_mut(views.floor) {
            sprite.image = art.floors[&platform.unwrap_or(Platform::FaceSlop)].clone();
        }
    }

    // A new wave, or none: forget every view of the old one.
    let seed = world.map_or(u64::MAX - 1, |w| w.seed);
    if views.seed != seed {
        for v in views.slop.drain().map(|(_, v)| v) {
            commands.entity(v.root).despawn();
        }
        let olds: Vec<Entity> = views
            .shots
            .drain()
            .map(|(_, (e, _))| e)
            .chain(views.likes.drain().chain(views.zones.drain()).map(|(_, e)| e))
            .chain(views.dogs.drain().flat_map(|(_, (a, b))| [a, b]))
            .collect();
        for e in olds {
            commands.entity(e).despawn();
        }
        views.seed = seed;
    }
    let showing = world.is_some();
    let hero = game.shown_run().map_or(Hero::Boomer, |r| r.hero);
    if let Ok((_, mut vis)) = roots.get_mut(views.player_root) {
        *vis = if showing { Visibility::Visible } else { Visibility::Hidden };
    }
    let Some(world) = world.filter(|_| showing) else { return };

    // You.
    let p = &world.player;
    if let Ok((mut tf, _)) = roots.get_mut(views.player_root) {
        tf.translation = Vec3::new(p.pos.x, p.pos.y, Z_ACTOR + depth(p.pos.y));
    }
    if let Ok((mut tf, mut sprite, mut vis)) = parts.get_mut(views.player_body) {
        let share = (p.mental_health / p.stats.max_mental_health).clamp(0.0, 1.0);
        let face = ((1.0 - share) * FACES as f32).min(FACES as f32 - 1.0) as usize;
        sprite.image = if p.invulnerable > 24 { art.human_flash[&hero].clone() } else { art.human[&hero][face].clone() };
        if p.vel.x.abs() > 5.0 {
            sprite.flip_x = p.vel.x < 0.0;
        }
        let speed = p.vel.length() / PLAYER_SPEED;
        let bob = (t * if speed > 0.1 { 14.0 } else { 3.0 }).sin();
        tf.scale = Vec3::new(1.0 - 0.05 * bob * speed.max(0.3), 1.0 + 0.07 * bob * speed.max(0.3), 1.0);
        tf.translation.y = 3.0 * bob.abs() * speed;
        // Blink while invulnerable after a hit (FAIR-03).
        *vis = if p.invulnerable > 0 && (p.invulnerable / 4) % 2 == 1 { Visibility::Hidden } else { Visibility::Inherited };
    }
    if let Ok((mut tf, _, mut vis)) = parts.get_mut(views.swirl) {
        let on = p.status.brainrot > 0 || p.status.brainrot_warning > 0;
        *vis = if on { Visibility::Inherited } else { Visibility::Hidden };
        tf.rotation = Quat::from_rotation_z(-t * 8.0);
    }

    // Your weapons, in your hands: they aim, kick, swing and throw (WPN-09).
    let defs: Vec<usize> = p.weapons.iter().map(|w| w.def).collect();
    if views.weapons.iter().map(|w| w.def).ne(defs.iter().copied()) {
        for w in views.weapons.drain(..) {
            commands.entity(w.e).despawn();
        }
        for &def in &defs {
            let e = commands.spawn((Sprite { image: art.weapon_icons[def].clone(), custom_size: Some(Vec2::splat(WEAPON_SIZE)), ..default() }, Transform::default(), ChildOf(views.player_root))).id();
            views.weapons.push(WeaponView { e, def, aim: 0.0, target: 0.0, kick: 0.0, swing: 0.0, half: 0.0 });
        }
    }
    for e in &game.events {
        if let Event::Attacked { weapon, to } = *e {
            if let Some(w) = views.weapons.get_mut(weapon as usize) {
                let d = to - p.pos;
                if d.length_sq() > 1.0 {
                    w.target = d.angle();
                }
                w.kick = 1.0;
                if let Pattern::Arc { half_angle } = WEAPONS[w.def].pattern {
                    w.swing = 1.0;
                    w.half = half_angle;
                    w.aim = w.target;
                }
            }
        }
    }
    let dt = time.delta_secs();
    let n = views.weapons.len().max(1) as f32;
    for (k, w) in views.weapons.iter_mut().enumerate() {
        let (forward, motion) = hold(w.def);
        let delta = (w.target - w.aim + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
        w.aim += delta * (1.0 - (-18.0 * dt).exp());
        w.kick *= (-9.0 * dt).exp();
        w.swing = (w.swing - dt / 0.22).max(0.0);
        let home_angle = std::f32::consts::TAU * k as f32 / n + 0.8;
        let home = Vec2::from_angle(home_angle) * HOLD + Vec2::new(0.0, 4.0 * (t * 2.5 + k as f32).sin());
        let aim_dir = Vec2::from_angle(w.aim);
        let (mut at, mut angle, mut scale) = (home, w.aim, 1.0);
        match motion {
            Motion::Recoil => at -= aim_dir * 10.0 * w.kick,
            Motion::Throw => {
                at += aim_dir * 16.0 * w.kick;
                angle += 1.2 * w.kick;
            }
            Motion::Pulse => scale = 1.0 + 0.45 * w.kick,
            Motion::Swing if w.swing > 0.0 => {
                let s = 1.0 - w.swing;
                let ease = s * s * (3.0 - 2.0 * s);
                let sweep = w.aim - w.half + 2.0 * w.half * ease;
                at = Vec2::from_angle(sweep) * (HOLD + 46.0 * (std::f32::consts::PI * s).sin());
                angle = sweep;
                scale = 1.25;
            }
            Motion::Swing | Motion::Hidden => {}
        }
        if let Ok((mut tf, mut sprite, mut vis)) = parts.get_mut(w.e) {
            *vis = if motion == Motion::Hidden { Visibility::Hidden } else { Visibility::Inherited };
            let left = angle.cos() < 0.0;
            let rotation = match forward {
                // Mirrored when aiming left, so nothing is held upside down.
                Some(f) if left => {
                    sprite.flip_y = true;
                    angle + f.to_radians()
                }
                Some(f) => {
                    sprite.flip_y = false;
                    angle - f.to_radians()
                }
                None => {
                    sprite.flip_x = left;
                    if motion == Motion::Throw { 1.2 * w.kick } else { 0.0 }
                }
            };
            tf.translation = Vec3::new(at.x, at.y, if at.y > 8.0 { -0.1 } else { 0.3 });
            tf.rotation = Quat::from_rotation_z(rotation);
            tf.scale = Vec3::splat(scale);
        }
    }

    // Slop. When the wave is cleared they are deleted by the effects instead.
    let alive = world.outcome != Outcome::Cleared;
    let mut seen = HashSet::new();
    for s in world.slop.iter().filter(|_| alive) {
        seen.insert(s.id);
        let v = views.slop.entry(s.id).or_insert_with(|| spawn_slop(&mut commands, &art, world, s));
        if let Ok((mut tf, _)) = roots.get_mut(v.root) {
            tf.translation = Vec3::new(s.pos.x, s.pos.y, Z_ACTOR + depth(s.pos.y));
        }
        let spec = world.spec(s);
        let phase = t * 9.0 + s.id as f32;
        let hit = s.flash as f32 / FLASH_TICKS as f32;
        let aim = if s.charging > 0 { 1.0 + 0.12 * (t * 30.0).sin() } else { 1.0 };
        let grow = s.radius / spec.radius;
        if let Ok(mut tf) = groups.get_mut(v.body) {
            let flip = if s.moving.x < -1.0 {
                -1.0
            } else if s.moving.x > 1.0 {
                1.0
            } else {
                tf.scale.x.signum()
            };
            tf.scale = Vec3::new(flip * (1.0 + 0.04 * phase.sin() + 0.15 * hit) * aim * grow, (1.0 - 0.04 * phase.sin() + 0.15 * hit) * aim * grow, 1.0);
        }
        for (e, pair) in &v.parts {
            if let Ok((_, mut sprite, _)) = parts.get_mut(*e) {
                let image = if s.flash > 0 { &pair.1 } else { &pair.0 };
                if sprite.image != *image {
                    sprite.image = image.clone();
                }
                let tint = if s.agreed > 0 { Color::srgb(0.8, 1.0, 0.8) } else if s.stunned > 0 { Color::srgb(0.75, 0.85, 1.0) } else { sprite.color.with_alpha(1.0) };
                if !spec.traits.has(Traits::CLONE) {
                    sprite.color = tint;
                }
            }
        }
        let show_bar = s.shows_health(world.tick);
        let full = spec.radius * 2.4;
        if let Ok((_, _, mut vis)) = parts.get_mut(v.bar_back) {
            *vis = if show_bar { Visibility::Inherited } else { Visibility::Hidden };
        }
        if let Ok((mut tf, mut sprite, mut vis)) = parts.get_mut(v.bar_fill) {
            *vis = if show_bar { Visibility::Inherited } else { Visibility::Hidden };
            let w = full * (s.hp / s.max_hp).clamp(0.0, 1.0);
            sprite.custom_size = Some(Vec2::new(w, 4.0));
            tf.translation.x = -(full - w) / 2.0;
        }
        if let Ok((_, _, mut vis)) = parts.get_mut(v.stun) {
            *vis = if s.stunned > 0 { Visibility::Inherited } else { Visibility::Hidden };
        }
        if let Ok(mut vis) = tags.get_mut(v.tag) {
            *vis = if s.marked > 0 { Visibility::Inherited } else { Visibility::Hidden };
        }
        if let Some(sh) = v.shield {
            if let Ok((mut tf, _, mut vis)) = parts.get_mut(sh) {
                *vis = if s.shield_off == 0 { Visibility::Inherited } else { Visibility::Hidden };
                let facing = if s.moving.length_sq() > 1.0 { s.moving } else { world.player.pos - s.pos };
                let a = facing.angle();
                tf.rotation = Quat::from_rotation_z(a);
                tf.translation = Vec3::new(a.cos() * spec.radius * 0.6, a.sin() * spec.radius * 0.6, 0.2);
            }
        }
    }
    views.slop.retain(|id, v| {
        let keep = seen.contains(id);
        if !keep {
            commands.entity(v.root).despawn();
        }
        keep
    });

    // Projectiles and speech bubbles. Lobbed bricks fly in an arc.
    let mut seen = HashSet::new();
    for pr in world.projectiles.iter().filter(|_| alive) {
        seen.insert(pr.id);
        let (e, start) = *views.shots.entry(pr.id).or_insert_with(|| (spawn_shot(&mut commands, &art, pr.look, pr.line, pr.hostile), pr.life));
        if let Ok((mut tf, _)) = sprite_roots.get_mut(e) {
            let z = if matches!(pr.look, Look::Bubble | Look::Post) { Z_BUBBLE } else { Z_SHOT };
            let lob = pr.splash > 0.0 && !pr.hostile;
            let arc = if lob {
                let s = 1.0 - pr.life as f32 / start.max(1) as f32;
                4.0 * s * (1.0 - s)
            } else {
                0.0
            };
            tf.translation = Vec3::new(pr.pos.x, pr.pos.y + arc * LOB_HEIGHT, z + (pr.id % 100) as f32 * 0.001);
            tf.scale = Vec3::splat(1.0 + 0.6 * arc);
            tf.rotation = match pr.look {
                Look::Bubble | Look::Post | Look::Letter => Quat::IDENTITY,
                Look::Rock | Look::EyeRoll | Look::Frisbee | Look::Toast | Look::Brick | Look::Book => Quat::from_rotation_z(t * if lob { 7.0 } else { 9.0 } + pr.id as f32),
                _ => Quat::from_rotation_z(pr.vel.angle()),
            };
        }
    }
    views.shots.retain(|id, (e, _)| {
        let keep = seen.contains(id);
        if !keep {
            commands.entity(*e).despawn();
        }
        keep
    });

    // Clout coins; fake ones from the Hallucinator glitch.
    let mut seen = HashSet::new();
    for l in &world.likes {
        seen.insert(l.id);
        let e = *views.likes.entry(l.id).or_insert_with(|| {
            commands.spawn((Root, Sprite { image: art.coin.clone(), custom_size: Some(Vec2::splat(20.0)), ..default() }, Transform::default())).id()
        });
        if let Ok((mut tf, mut sprite)) = sprite_roots.get_mut(e) {
            let pulse = 1.0 + 0.12 * (t * 6.0 + l.id as f32).sin();
            let jitter = if l.fake && (t * 20.0) as u32 % 7 == 0 { 3.0 } else { 0.0 };
            tf.translation = Vec3::new(l.pos.x + jitter, l.pos.y, Z_LIKE);
            tf.scale = Vec3::splat(pulse * if l.value > 1 { 1.25 } else { 1.0 });
            if l.fake {
                sprite.color = Color::srgb(0.85, 0.7, 1.0);
            }
        }
    }
    retain(&mut commands, &mut views.likes, &seen);

    // Zones on the floor.
    let mut seen = HashSet::new();
    for z in &world.zones {
        seen.insert(z.id);
        let e = *views.zones.entry(z.id).or_insert_with(|| spawn_zone(&mut commands, &art, z.kind, z.radius));
        if let Ok((mut tf, mut sprite)) = sprite_roots.get_mut(e) {
            tf.translation = Vec3::new(if z.kind == ZoneKind::Banner { ARENA / 2.0 } else { z.pos.x }, z.pos.y, Z_ZONE + (z.id % 50) as f32 * 0.001);
            let left = z.ticks as f32 / z.max_ticks.max(1) as f32;
            match z.kind {
                ZoneKind::Bomb => sprite.color = Color::WHITE.with_alpha(0.5 + 0.5 * (t * (24.0 - 18.0 * left)).sin().abs()),
                ZoneKind::Seed { .. } | ZoneKind::Downvote | ZoneKind::Filter => sprite.color = Color::WHITE.with_alpha((left * 4.0).min(1.0)),
                ZoneKind::Banner => {}
            }
        }
    }
    retain(&mut commands, &mut views.zones, &seen);

    // The dog of a Dog Walk, on its leash.
    let mut seen = HashSet::new();
    for (slot, w) in world.player.weapons.iter().enumerate() {
        if let Pattern::Orbit { radius } = w.def().pattern {
            seen.insert(slot);
            let (dog, leash) = *views.dogs.entry(slot).or_insert_with(|| {
                let dog = commands.spawn((Root, Sprite { image: art.dog.clone(), custom_size: Some(Vec2::splat(34.0)), ..default() }, Transform::default())).id();
                let leash = commands.spawn((Root, Sprite::from_color(Color::srgb(0.85, 0.2, 0.2), Vec2::new(1.0, 2.5)), Transform::default())).id();
                (dog, leash)
            });
            let at = p.pos + slop_sim::V2::from_angle(w.phase) * radius;
            if let Ok((mut tf, mut sprite)) = sprite_roots.get_mut(dog) {
                tf.translation = Vec3::new(at.x, at.y + 3.0 * (t * 16.0).sin().abs(), Z_ACTOR + depth(at.y));
                sprite.flip_x = (w.phase + std::f32::consts::FRAC_PI_2).cos() < 0.0;
            }
            if let Ok((mut tf, mut sprite)) = sprite_roots.get_mut(leash) {
                let (a, b) = (Vec2::new(p.pos.x, p.pos.y), Vec2::new(at.x, at.y));
                let mid = (a + b) / 2.0;
                tf.translation = Vec3::new(mid.x, mid.y, Z_ACTOR + depth(p.pos.y) - 0.01);
                tf.rotation = Quat::from_rotation_z((b - a).to_angle());
                sprite.custom_size = Some(Vec2::new((b - a).length(), 2.5));
            }
        }
    }
    views.dogs.retain(|slot, (dog, leash)| {
        let keep = seen.contains(slot);
        if !keep {
            commands.entity(*dog).despawn();
            commands.entity(*leash).despawn();
        }
        keep
    });
}

fn retain(commands: &mut Commands, map: &mut HashMap<u32, Entity>, seen: &HashSet<u32>) {
    map.retain(|id, e| {
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
    let target = match game.world().filter(|_| game.screen == Screen::Run) {
        Some(w) => Vec2::new(w.player.pos.x, w.player.pos.y),
        None => Vec2::splat(ARENA / 2.0) + Vec2::new((time.elapsed_secs() * 0.1).sin() * 300.0, 0.0),
    };
    let seed = game.world().map_or(0, |w| w.seed);
    if *last_seed != seed {
        *last_seed = seed;
        *focus = Some(target);
    }
    let f = focus.get_or_insert(target);
    *f += (target - *f) * (1.0 - (-CAMERA_FOLLOW * dt).exp());
    let lo = half - Vec2::splat(WALL_SHOWN);
    let hi = Vec2::splat(ARENA) - half + Vec2::splat(WALL_SHOWN);
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
