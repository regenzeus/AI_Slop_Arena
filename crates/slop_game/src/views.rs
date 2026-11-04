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
use slop_sim::map::{chunk_seed, PropKind, PropState, WallKind, CHUNK};
use slop_sim::world::{Outcome, Shape, Slop, World, ZoneKind, FLASH_TICKS, NO_LINE, PLAYER_SPEED};

use crate::art::{Art, Pair, BUBBLE_BORDER, FACES, HUMAN_SIZE, SLOP_SCALE};
use crate::play::Simulation;
use crate::state::Game;
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

/// The smallest part of the Feed any window shows.
pub const VIEW: Vec2 = Vec2::new(1440.0, 810.0);
/// Side of the floor sprite; it is moved in whole tiles to follow the camera.
const FLOOR: f32 = 256.0 * 16.0;
const TILE: f32 = 256.0;
/// Map things farther than this from you are not drawn.
const DRAW_RANGE: f32 = 1400.0;
/// Feed scraps on the floor per chunk.
const DECOR_PER_CHUNK: u64 = 14;
/// Distance of carried weapons from your centre.
const HOLD: f32 = 38.0;
const WEAPON_SIZE: f32 = 34.0;
/// Height of a lobbed brick at the top of its arc.
const LOB_HEIGHT: f32 = 80.0;

pub struct ViewsPlugin;

impl Plugin for ViewsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Shake>().add_systems(Startup, setup).add_systems(Update, (sync, map_views, follow_camera).chain().after(Simulation));
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
    /// An elite's glowing aura (ELITE-02).
    aura: Option<Entity>,
    /// Feet that step as it walks (ICON-02).
    feet: Vec<Entity>,
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
    /// God Mode and Power Outlet aura; the Boundaries bubble.
    aura: Entity,
    bubble: Entity,
    slop: HashMap<u32, SlopView>,
    /// A projectile's view and the life it started with (for lob arcs).
    shots: HashMap<u32, (Entity, u16)>,
    likes: HashMap<u32, Entity>,
    zones: HashMap<u32, Entity>,
    /// A Dog Walk's dog and its leash, by weapon slot.
    dogs: HashMap<usize, (Entity, Entity)>,
    weapons: Vec<WeaponView>,
    /// The map (ART-06): walls and fences, props with their rings, pickups,
    /// and the feed scraps of each chunk.
    walls: HashMap<u32, Entity>,
    fences: HashMap<u32, (Entity, Entity)>,
    props: HashMap<u32, PropView>,
    pickups: HashMap<u32, Entity>,
    decor: HashMap<(i32, i32), Vec<Entity>>,
    /// The floor of each biome square near you (BIOME-01).
    biomes: HashMap<(i32, i32), Vec<Entity>>,
    /// Barrier segments, the Alt Account, and the livestream ring.
    barriers: Vec<Entity>,
    alt: Option<Entity>,
    arena: Option<(Entity, Entity)>,
    /// The seed the biome squares and scraps were made for.
    map_seed: u64,
}

struct PropView {
    root: Entity,
    body: Entity,
    ring: Option<Entity>,
    /// "Hold to use" progress, or what is left.
    meter: Option<(Entity, Entity)>,
    tag: Entity,
}

#[derive(Component)]
pub struct MainCamera;

/// A view's root, moved to the object's position.
#[derive(Component)]
struct Root;

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let art = Art::build(&mut images);
    let centre = Vec2::ZERO;
    // Any window shows at least 1440 x 810 of the Feed (UI-07).
    let projection = OrthographicProjection { scaling_mode: ScalingMode::AutoMin { min_width: VIEW.x, min_height: VIEW.y }, ..OrthographicProjection::default_2d() };
    commands.spawn((Camera2d, MainCamera, Projection::from(projection), Transform::from_xyz(centre.x, centre.y, 100.0)));
    // The endless floor: one big tiled sprite that follows the camera in
    // whole tiles, so the pattern never slides (MAP-01).
    let floor = commands
        .spawn((
            Sprite {
                image: art.floors[&Platform::FaceSlop].clone(),
                custom_size: Some(Vec2::splat(FLOOR)),
                image_mode: SpriteImageMode::Tiled { tile_x: true, tile_y: true, stretch_value: 1.0 },
                ..default()
            },
            Transform::from_xyz(centre.x, centre.y, Z_FLOOR),
        ))
        .id();

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
    // Effects on you are shown on you: a bubble, an aura, a tint (FAIR-05).
    let aura = commands.spawn((Sprite { image: art.ring.clone(), custom_size: Some(Vec2::splat(HUMAN_SIZE * 1.6)), color: Color::NONE, ..default() }, Transform::from_xyz(0.0, 0.0, 0.25), ChildOf(player_root))).id();
    let bubble = commands.spawn((Sprite { image: art.ball.clone(), custom_size: Some(Vec2::splat(HUMAN_SIZE * 1.5)), color: Color::NONE, ..default() }, Transform::from_xyz(0.0, 0.0, 0.24), ChildOf(player_root))).id();
    commands.insert_resource(Views {
        seed: u64::MAX,
        platform: None,
        floor,
        player_root,
        player_body,
        swirl,
        aura,
        bubble,
        slop: HashMap::new(),
        shots: HashMap::new(),
        likes: HashMap::new(),
        zones: HashMap::new(),
        dogs: HashMap::new(),
        weapons: Vec::new(),
        walls: HashMap::new(),
        fences: HashMap::new(),
        props: HashMap::new(),
        pickups: HashMap::new(),
        decor: HashMap::new(),
        biomes: HashMap::new(),
        barriers: Vec::new(),
        alt: None,
        arena: None,
        map_seed: u64::MAX,
    });
    commands.insert_resource(art);
}

/// Depth by height on screen, so lower actors overlap higher ones. The map
/// is endless, so heights count from the camera's.
fn depth(y: f32) -> f32 {
    (0.45 - (y - DEPTH_REF.with(|r| r.get())) / 4000.0).clamp(0.0, 0.9)
}

thread_local! {
    /// The height `depth` counts from: where you are this frame.
    static DEPTH_REF: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
}

/// Where something is drawn: between its last two ticks (FEEL-04).
fn lerp(prev: slop_sim::V2, pos: slop_sim::V2, alpha: f32) -> Vec2 {
    let (a, b) = (Vec2::new(prev.x, prev.y), Vec2::new(pos.x, pos.y));
    // A jump (moved back in front of you) is not smoothed.
    if a.distance_squared(b) > 200.0 * 200.0 {
        b
    } else {
        a.lerp(b, alpha)
    }
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
    // Elites stand out from the crowd: a glowing aura in their affix's colour,
    // a ring around them and their affix above (ELITE-02).
    let mut aura = None;
    if spec.elite {
        // A Nemesis wears its whole name: it is personal (NEMESIS-01).
        let nemesis = spec.traits.has(Traits::NEMESIS);
        let affix = if nemesis { "NEMESIS".to_string() } else { spec.name.split_whitespace().next().unwrap_or("ELITE").to_uppercase() };
        let color = affix_color(&affix);
        let glow = commands.spawn((Sprite { image: art.ball.clone(), custom_size: Some(Vec2::splat(spec.radius * 4.2)), color: color.with_alpha(0.35), ..default() }, Transform::from_xyz(0.0, 0.0, -0.08), ChildOf(root))).id();
        commands.spawn((Sprite { image: art.ring.clone(), custom_size: Some(Vec2::splat(spec.radius * 3.0)), color: color.with_alpha(0.9), ..default() }, Transform::from_xyz(0.0, -spec.radius * 0.3, -0.07), ChildOf(glow)));
        aura = Some(glow);
        let words = if nemesis { spec.name.clone() } else { affix };
        text(commands, words, 14.0, color, Vec3::new(0.0, bar_y + 26.0, 0.12), root);
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
    // Walkers without frames of their own get two feet that step.
    let walks = !matches!(spec.look, SlopLook::Boss(_)) && !matches!(spec.look, SlopLook::Archetype(a) if a.is_icon() || a == Archetype::LootGoblin) && !spec.traits.has(Traits::FLIES) && spec.movement != slop_sim::spec::Movement::Stand;
    let feet = if walks {
        (0..2)
            .map(|k| {
                let x = if k == 0 { -0.38 } else { 0.38 } * spec.radius;
                commands.spawn((Sprite { image: art.ball.clone(), custom_size: Some(Vec2::new(spec.radius * 0.55, spec.radius * 0.34)), color: theme::INK, ..default() }, Transform::from_xyz(x, -spec.radius * 0.88, -0.02), ChildOf(root))).id()
            })
            .collect()
    } else {
        Vec::new()
    };
    SlopView { root, body, parts, bar_back, bar_fill, tag, shield, stun, aura, feet }
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
        Look::Duck => Vec2::new(26.0, 24.0),
        Look::Plane => Vec2::new(30.0, 19.0),
        Look::Note => Vec2::splat(16.0),
        Look::Banana => Vec2::splat(28.0),
        Look::Needle => Vec2::new(24.0, 10.0),
        Look::Shard => Vec2::splat(18.0),
        Look::Bubble | Look::Post => unreachable!(),
    };
    let root = commands.spawn((Root, Sprite { image: art.shots[&look].clone(), custom_size: Some(size), ..default() }, Transform::default())).id();
    // Your weapons copied by The Model hurt you: they get the danger outline (FAIR-04).
    let danger_looks = !matches!(look, Look::EmDash | Look::Toast | Look::Arrow | Look::Letter | Look::Banana | Look::Needle | Look::Shard);
    if hostile && danger_looks {
        commands.spawn((Sprite { image: art.danger_ring.clone(), custom_size: Some(Vec2::splat(size.x.max(size.y) * 1.7)), ..default() }, Transform::from_xyz(0.0, 0.0, -0.01), ChildOf(root)));
    }
    root
}

/// A zone in its shape (ZONE-02): circles use their picture; rectangles and
/// rings are drawn as tinted areas with a word on them.
fn spawn_zone(commands: &mut Commands, art: &Art, kind: ZoneKind, radius: f32, shape: Shape, sunflowers: bool) -> Entity {
    let tint = match kind {
        ZoneKind::Downvote => Color::srgba(1.0, 0.35, 0.1, 0.26),
        ZoneKind::Filter => Color::srgba(1.0, 0.55, 0.85, 0.22),
        _ => Color::srgba(1.0, 1.0, 1.0, 0.2),
    };
    let word = match kind {
        ZoneKind::Downvote => "DOWNVOTED",
        ZoneKind::Filter => "#nofilter",
        _ => "",
    };
    match (kind, shape) {
        (ZoneKind::Banner, _) | (_, Shape::Circle) => {}
        (_, Shape::Rect(h)) => {
            let root = commands.spawn((Root, Sprite::from_color(tint, Vec2::new(h.x * 2.0, h.y * 2.0)), Transform::default())).id();
            text(commands, word, 16.0, tint.with_alpha(0.9), Vec3::new(0.0, 0.0, 0.05), root);
            return root;
        }
        (_, Shape::Ring(_)) => {
            let root = commands.spawn((Root, Sprite { image: art.ring.clone(), custom_size: Some(Vec2::splat(radius * 2.0)), color: tint.with_alpha(0.75), ..default() }, Transform::default())).id();
            text(commands, word, 16.0, tint.with_alpha(0.9), Vec3::new(0.0, radius - 20.0, 0.05), root);
            return root;
        }
    }
    let (image, size) = match kind {
        ZoneKind::Seed { .. } if sunflowers => (art.sunflowers.clone(), Vec2::splat(radius * 2.0)),
        ZoneKind::Seed { .. } => (art.seed.clone(), Vec2::splat(radius * 2.0)),
        ZoneKind::Bomb => (art.bomb.clone(), Vec2::splat(radius * 2.0)),
        ZoneKind::Downvote => (art.downvote.clone(), Vec2::splat(radius * 2.0)),
        ZoneKind::Filter => (art.filter.clone(), Vec2::splat(radius * 2.0)),
        ZoneKind::Fire => (art.fire.clone(), Vec2::splat(radius * 2.2)),
        ZoneKind::Roots => (art.roots.clone(), Vec2::splat(radius * 2.0)),
        ZoneKind::Buffering => {
            let root = commands.spawn((Root, Sprite { image: art.ring.clone(), custom_size: Some(Vec2::splat(radius * 2.0)), color: Color::srgba(0.9, 0.9, 0.95, 0.6), ..default() }, Transform::default())).id();
            commands.spawn((Sprite { image: art.dot.clone(), custom_size: Some(Vec2::splat(radius * 1.9)), color: Color::srgba(0.1, 0.1, 0.12, 0.35), ..default() }, Transform::from_xyz(0.0, 0.0, -0.01), ChildOf(root)));
            text(commands, "buffering...", 16.0, Color::srgba(0.95, 0.95, 1.0, 0.8), Vec3::new(0.0, 0.0, 0.05), root);
            return root;
        }
        ZoneKind::Brainrot => {
            let root = commands.spawn((Root, Sprite { image: art.brainrot_zone.clone(), custom_size: Some(Vec2::splat(radius * 2.0)), ..default() }, Transform::default())).id();
            text(commands, "BRAINROT ZONE\nleave before the drop", 15.0, Color::srgb(0.85, 0.7, 1.0), Vec3::new(0.0, 0.0, 0.05), root);
            return root;
        }
        ZoneKind::Banner => {
            let root = commands.spawn((Root, Sprite::from_color(Color::srgba(0.25, 0.65, 0.35, 0.35), Vec2::new(6000.0, radius * 2.0)), Transform::default())).id();
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

    // The floor under everything: the biome you are in.
    let platform = world.map(|w| w.biome);
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

    // You, drawn between ticks (FEEL-04).
    let p = &world.player;
    let alpha = game.alpha;
    let me = lerp(p.prev, p.pos, alpha);
    DEPTH_REF.with(|r| r.set(me.y));
    if let Ok((mut tf, _)) = roots.get_mut(views.player_root) {
        tf.translation = Vec3::new(me.x, me.y, Z_ACTOR + depth(me.y));
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
    if let Ok((mut tf, mut sprite, _)) = parts.get_mut(views.aura) {
        let pulse = 1.0 + 0.08 * (t * 10.0).sin();
        tf.scale = Vec3::splat(pulse);
        sprite.color = if p.god > 0 {
            Color::srgba(1.0, 0.85, 0.3, 0.9)
        } else if p.charged > 0 {
            Color::srgba(0.5, 0.9, 1.0, 0.45 + 0.3 * (t * 14.0).sin().abs())
        } else if p.status.brainrot > 0 || p.status.doomscroll > 0 || p.status.popup > 0 {
            Color::srgba(0.55, 0.45, 1.0, 0.5)
        } else {
            Color::NONE
        };
    }
    if let Ok((_, mut sprite, _)) = parts.get_mut(views.bubble) {
        let max = p.stats.boundaries.max(0.0);
        let share = if max > 0.0 { (p.shield / max).clamp(0.0, 1.0) } else { 0.0 };
        sprite.color = Color::srgba(0.31, 0.72, 0.91, 0.18 * share);
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
            let at = lerp(s.prev, s.pos, alpha);
            tf.translation = Vec3::new(at.x, at.y, Z_ACTOR + depth(at.y));
        }
        let spec = world.spec(s);
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
            // Walkers waddle and squash with their step; standing ones breathe
            // (ICON-02).
            let pace = (s.moving.length() / 120.0).min(1.5);
            let step = (t * (6.0 + 8.0 * pace) + s.id as f32).sin();
            let squash = 0.04 + 0.05 * pace;
            tf.scale = Vec3::new(flip * (1.0 + squash * step + 0.15 * hit) * aim * grow, (1.0 - squash * step + 0.15 * hit) * aim * grow, 1.0);
            tf.rotation = Quat::from_rotation_z(0.09 * pace * (t * (5.0 + 5.0 * pace) + s.id as f32).cos());
            tf.translation.y = 4.0 * pace * step.abs();
        }
        // Feet step in turn while it moves.
        let pace = (s.moving.length() / 120.0).min(1.5);
        let stride = t * (8.0 + 8.0 * pace) + s.id as f32;
        for (k, e) in v.feet.iter().enumerate() {
            if let Ok((mut tf, _, _)) = parts.get_mut(*e) {
                let phase = stride + k as f32 * std::f32::consts::PI;
                let base_x = if k == 0 { -0.38 } else { 0.38 } * spec.radius;
                tf.translation.x = base_x + phase.sin() * spec.radius * 0.18 * pace;
                tf.translation.y = -spec.radius * 0.88 + phase.cos().max(0.0) * spec.radius * 0.22 * pace;
            }
        }
        // A brainrot's object swings and its accessory bobs.
        if matches!(spec.look, SlopLook::Brainrot { .. }) {
            if let Some((e, _)) = v.parts.get(1) {
                if let Ok((mut tf, _, _)) = parts.get_mut(*e) {
                    tf.rotation = Quat::from_rotation_z(0.18 * (stride * 0.8).sin());
                }
            }
            if let Some((e, _)) = v.parts.get(2) {
                if let Ok((mut tf, _, _)) = parts.get_mut(*e) {
                    tf.translation.y = 2.5 * (stride * 1.3).sin().abs();
                }
            }
        }
        // Brainrot icons play their frames: legs, bats, propellers.
        let frames = match spec.look {
            SlopLook::Archetype(a) => art.icon_frames.get(&a),
            _ => None,
        };
        let frame = frames.map(|f| {
            let speed = if spec.movement == slop_sim::spec::Movement::Stand { 4.0 } else { 7.0 + s.moving.length() / 25.0 };
            &f[((t * speed) as usize + s.id as usize) % f.len()]
        });
        for (e, pair) in &v.parts {
            let pair = frame.unwrap_or(pair);
            if let Ok((_, mut sprite, _)) = parts.get_mut(*e) {
                let image = if s.flash > 0 { &pair.1 } else { &pair.0 };
                if sprite.image != *image {
                    sprite.image = image.clone();
                }
                // Injected slop are visibly not themselves (INJECT-01).
                let tint = if world.injection > 0 && s.confused > 0 {
                    Color::srgb(0.55, 1.0, 0.6)
                } else if world.freeze > 0 {
                    Color::srgb(0.6, 0.85, 1.0)
                } else if s.rooted > 0 {
                    Color::srgb(0.7, 1.0, 0.65)
                } else if s.confused > 0 {
                    Color::srgb(1.0, 0.75, 0.95)
                } else if s.agreed > 0 {
                    Color::srgb(0.8, 1.0, 0.8)
                } else if s.stunned > 0 {
                    Color::srgb(0.75, 0.85, 1.0)
                } else {
                    Color::WHITE
                };
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
        if let Some(a) = v.aura {
            if let Ok((mut tf, _, _)) = parts.get_mut(a) {
                tf.scale = Vec3::splat(1.0 + 0.12 * (t * 5.0 + s.id as f32).sin());
            }
        }
        if let Ok((_, _, mut vis)) = parts.get_mut(v.stun) {
            *vis = if s.stunned > 0 || s.confused > 0 { Visibility::Inherited } else { Visibility::Hidden };
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
            let at = lerp(pr.prev, pr.pos, alpha);
            tf.translation = Vec3::new(at.x, at.y + arc * LOB_HEIGHT, z + (pr.id % 100) as f32 * 0.001);
            tf.scale = Vec3::splat(1.0 + 0.6 * arc);
            tf.rotation = match pr.look {
                Look::Bubble | Look::Post | Look::Letter => Quat::IDENTITY,
                Look::Rock | Look::EyeRoll | Look::Frisbee | Look::Toast | Look::Brick | Look::Book | Look::Duck | Look::Note => Quat::from_rotation_z(t * if lob { 7.0 } else { 9.0 } + pr.id as f32),
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

    // Likes: experience. Fake ones from the Hallucinator glitch.
    let mut seen = HashSet::new();
    for l in &world.likes {
        seen.insert(l.id);
        let e = *views.likes.entry(l.id).or_insert_with(|| {
            commands.spawn((Root, Sprite { image: art.like.clone(), custom_size: Some(Vec2::splat(18.0)), ..default() }, Transform::default())).id()
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
        // Sunflower patches look like sunflowers; other seed patches like seeds.
        let sunflowers = matches!(z.kind, ZoneKind::Seed { .. })
            && world.player.weapons.get(z.spec as usize).is_some_and(|w| matches!(w.def().name, "Sunflower Patch" | "Rewilding" | "Hurricane Season"));
        let e = *views.zones.entry(z.id).or_insert_with(|| spawn_zone(&mut commands, &art, z.kind, z.radius, z.shape, sunflowers));
        if let Ok((mut tf, mut sprite)) = sprite_roots.get_mut(e) {
            tf.translation = Vec3::new(z.pos.x, z.pos.y, Z_ZONE + (z.id % 50) as f32 * 0.001);
            let left = z.ticks as f32 / z.max_ticks.max(1) as f32;
            match z.kind {
                ZoneKind::Bomb => sprite.color = Color::WHITE.with_alpha(0.5 + 0.5 * (t * (24.0 - 18.0 * left)).sin().abs()),
                ZoneKind::Seed { .. } | ZoneKind::Downvote | ZoneKind::Filter if z.shape == Shape::Circle => sprite.color = Color::WHITE.with_alpha((left * 4.0).min(1.0)),
                ZoneKind::Seed { .. } | ZoneKind::Downvote | ZoneKind::Filter => sprite.color = sprite.color.with_alpha((left * 4.0).min(1.0) * 0.26),
                ZoneKind::Fire => {
                    sprite.color = Color::WHITE.with_alpha((left * 2.0).min(1.0));
                    tf.scale = Vec3::splat(0.9 + 0.12 * (t * 14.0 + z.id as f32).sin());
                }
                ZoneKind::Roots => sprite.color = Color::WHITE.with_alpha((left * 4.0).min(1.0)),
                ZoneKind::Buffering => sprite.color = sprite.color.with_alpha(0.35 + 0.3 * (t * 4.0).sin().abs()),
                // Pulses to the beat, faster and brighter as the drop nears.
                ZoneKind::Brainrot => {
                    let beat = 1.0 - world.beat as f32 / slop_sim::hazards::BEAT_INTERVAL as f32;
                    sprite.color = Color::WHITE.with_alpha(0.55 + 0.45 * beat.max(0.0) * (1.0 - left).max(0.3));
                    tf.scale = Vec3::splat(1.0 + 0.06 * beat.max(0.0));
                }
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
            let at = slop_sim::V2::new(me.x, me.y) + slop_sim::V2::from_angle(w.phase) * radius;
            // Dogs walk on a leash; anything else circling you just spins.
            let is_dog = matches!(w.def().name, "Dog Walk" | "Dog Park");
            if let Ok((mut tf, mut sprite)) = sprite_roots.get_mut(dog) {
                tf.translation = Vec3::new(at.x, at.y + if is_dog { 3.0 * (t * 16.0).sin().abs() } else { 0.0 }, Z_ACTOR + depth(at.y));
                if is_dog {
                    sprite.flip_x = (w.phase + std::f32::consts::FRAC_PI_2).cos() < 0.0;
                } else {
                    sprite.image = art.weapon_icons[w.def].clone();
                    tf.rotation = Quat::from_rotation_z(t * 18.0);
                }
            }
            if let Ok((mut tf, mut sprite)) = sprite_roots.get_mut(leash) {
                let (a, b) = (me, Vec2::new(at.x, at.y));
                let mid = (a + b) / 2.0;
                tf.translation = Vec3::new(mid.x, mid.y, Z_ACTOR + depth(me.y) - 0.01);
                tf.rotation = Quat::from_rotation_z((b - a).to_angle());
                sprite.custom_size = Some(Vec2::new((b - a).length(), 2.5));
                sprite.color = if is_dog { Color::srgb(0.85, 0.2, 0.2) } else { Color::NONE };
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

    // Barriers circling you (DEF-02): each segment is its weapon's icon.
    let segments = world.barrier_segments();
    let offset = me - Vec2::new(world.player.pos.x, world.player.pos.y);
    while views.barriers.len() < segments.len() {
        let e = commands.spawn((Root, Sprite { image: art.dot.clone(), custom_size: Some(Vec2::splat(40.0)), ..default() }, Transform::default())).id();
        views.barriers.push(e);
    }
    for (k, e) in views.barriers.iter().enumerate() {
        if let Ok((mut tf, mut sprite)) = sprite_roots.get_mut(*e) {
            match segments.get(k) {
                Some(&(c, size, slot)) => {
                    let def = world.player.weapons[slot].def;
                    if sprite.image != art.weapon_icons[def] {
                        sprite.image = art.weapon_icons[def].clone();
                    }
                    sprite.custom_size = Some(Vec2::splat(size * 1.5));
                    sprite.color = Color::WHITE;
                    tf.translation = Vec3::new(c.x + offset.x, c.y + offset.y, Z_ACTOR + depth(c.y) + 0.01);
                    tf.rotation = Quat::from_rotation_z(t * 3.0 + k as f32);
                }
                None => sprite.color = Color::NONE,
            }
        }
    }

    // Your Alt Account, a see-through second you (ITEM-03).
    let alt = (world.player.perks.alt_account > 0.0).then(|| world.alt_position());
    if alt.is_some() && views.alt.is_none() {
        views.alt = Some(commands.spawn((Root, Sprite { image: art.human[&hero][0].clone(), custom_size: Some(Vec2::splat(HUMAN_SIZE)), color: Color::srgba(0.6, 0.85, 1.0, 0.55), ..default() }, Transform::default())).id());
    }
    if let Some(e) = views.alt {
        if let Ok((mut tf, mut sprite)) = sprite_roots.get_mut(e) {
            match alt {
                Some(a) => {
                    tf.translation = Vec3::new(a.x + offset.x, a.y + offset.y, Z_ACTOR + depth(a.y));
                    sprite.color = Color::srgba(0.6, 0.85, 1.0, 0.55 + 0.1 * (t * 4.0).sin());
                }
                None => sprite.color = Color::NONE,
            }
        }
    }

    // A livestream's ring (EVENT-02): blinking red while it closes, solid
    // with LIVE when it holds you.
    if views.arena.is_none() {
        let ring = commands.spawn((Root, Sprite { image: art.thin_ring.clone(), custom_size: Some(Vec2::splat(10.0)), color: Color::NONE, ..default() }, Transform::default())).id();
        let label = text(&mut commands, "", 22.0, Color::srgb(1.0, 0.3, 0.3), Vec3::new(0.0, 0.0, 0.1), ring);
        views.arena = Some((ring, label));
    }
    if let Some((ring, _)) = views.arena {
        if let Ok((mut tf, mut sprite)) = sprite_roots.get_mut(ring) {
            match world.arena {
                Some(a) => {
                    tf.translation = Vec3::new(a.centre.x, a.centre.y, Z_ZONE + 0.3);
                    sprite.custom_size = Some(Vec2::splat(a.radius * 2.0));
                    let alpha = if a.closed() { 0.95 } else { 0.4 + 0.5 * (t * 10.0).sin().abs() };
                    sprite.color = Color::srgba(1.0, 0.25, 0.25, alpha);
                }
                None => sprite.color = Color::NONE,
            }
        }
    }
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

/// The map near you (ART-06): walls, maintenance fences, props with their
/// rings and meters, pickups, and the feed scraps that make every chunk of
/// floor different.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn map_views(
    mut commands: Commands,
    game: Res<Game>,
    art: Res<Art>,
    time: Res<Time>,
    mut views: ResMut<Views>,
    mut sprites: Query<(&mut Transform, &mut Sprite, &mut Visibility), Without<MainCamera>>,
    mut words: Query<(&mut Text2d, &mut TextColor)>,
) {
    let views = &mut *views;
    let t = time.elapsed_secs();
    let Some(world) = game.world().filter(|w| w.outcome != Outcome::Cleared) else {
        for e in views.walls.drain().map(|(_, e)| e).chain(views.pickups.drain().map(|(_, e)| e)) {
            commands.entity(e).despawn();
        }
        for (_, (a, b)) in views.fences.drain() {
            commands.entity(a).despawn();
            commands.entity(b).despawn();
        }
        for (_, v) in views.props.drain() {
            commands.entity(v.root).despawn();
        }
        for (_, list) in views.decor.drain() {
            for e in list {
                commands.entity(e).despawn();
            }
        }
        return;
    };
    let me = lerp(world.player.prev, world.player.pos, game.alpha);
    let near = |x: f32, y: f32, r: f32| (Vec2::new(x, y) - me).length() < DRAW_RANGE + r;
    // Another run, another map: the floors and scraps are made again.
    if views.map_seed != world.seed {
        views.map_seed = world.seed;
        for (_, list) in views.decor.drain().chain(views.biomes.drain()) {
            for e in list {
                commands.entity(e).despawn();
            }
        }
    }

    // Walls.
    let mut seen = HashSet::new();
    for w in world.map.walls_near(world.player.pos, DRAW_RANGE).filter(|w| w.kind != WallKind::Fence) {
        let c = w.centre();
        let size = w.size();
        if !near(c.x, c.y, size.x.max(size.y)) {
            continue;
        }
        seen.insert(w.id);
        views.walls.entry(w.id).or_insert_with(|| {
            let slicer = TextureSlicer { border: BorderRect::all(10.0), center_scale_mode: SliceScaleMode::Tile { stretch_value: 1.0 }, sides_scale_mode: SliceScaleMode::Stretch, max_corner_scale: 1.0 };
            commands
                .spawn((
                    Sprite { image: art.walls[&w.kind].clone(), custom_size: Some(Vec2::new(size.x, size.y)), image_mode: SpriteImageMode::Sliced(slicer), ..default() },
                    Transform::from_xyz(c.x, c.y, Z_ACTOR + depth(w.max.y) - 0.02),
                ))
                .id()
        });
    }
    retain(&mut commands, &mut views.walls, &seen);
    for (id, e) in &views.walls {
        if let Some(w) = world.map.walls.iter().find(|w| w.id == *id) {
            if let Ok((mut tf, _, _)) = sprites.get_mut(*e) {
                tf.translation.z = Z_ACTOR + depth(w.max.y) - 0.02;
            }
        }
    }

    // Maintenance fences: striped while warning, solid with a countdown when closed.
    let mut seen_f = HashSet::new();
    for f in &world.map.fences {
        seen_f.insert(f.id);
        let c = f.centre();
        let size = f.size();
        let (body, label) = *views.fences.entry(f.id).or_insert_with(|| {
            let body = commands
                .spawn((
                    Sprite { image: art.walls[&WallKind::Fence].clone(), custom_size: Some(Vec2::new(size.x, size.y)), image_mode: SpriteImageMode::Tiled { tile_x: true, tile_y: true, stretch_value: 0.5 }, ..default() },
                    Transform::from_xyz(c.x, c.y, Z_ZONE + 0.2),
                ))
                .id();
            let label = commands.spawn((Text2d::new(""), TextFont { font_size: 14.0, ..default() }, TextColor(theme::PAPER), Transform::from_xyz(c.x, c.y, Z_TEXT))).id();
            (body, label)
        });
        let solid = f.solid();
        if let Ok((mut tf, mut sprite, _)) = sprites.get_mut(body) {
            let blink = if solid { 1.0 } else { 0.25 + 0.25 * (t * 10.0).sin().abs() };
            sprite.color = Color::WHITE.with_alpha(blink);
            tf.translation.z = if solid { Z_ACTOR + depth(f.max.y) - 0.02 } else { Z_ZONE + 0.2 };
        }
        if let Ok((mut text, mut color)) = words.get_mut(label) {
            let left = if solid { f.ticks } else { f.warning } as f32 / 60.0;
            // A barrier, never a danger: construction yellow, not pink.
            text.0 = if solid { format!("UNDER MAINTENANCE\nback in {left:.0} s") } else { format!("CLOSING FOR MAINTENANCE IN {left:.1}\n(blocks the way, harmless)") };
            color.0 = if solid { theme::PAPER } else { Color::srgb(0.95, 0.76, 0.19) };
        }
    }
    views.fences.retain(|id, (a, b)| {
        let keep = seen_f.contains(id);
        if !keep {
            commands.entity(*a).despawn();
            commands.entity(*b).despawn();
        }
        keep
    });

    // Props.
    let mut seen_p = HashSet::new();
    for p in world.map.props.iter().filter(|p| near(p.pos.x, p.pos.y, p.kind.radius())) {
        seen_p.insert(p.id);
        let v = views.props.entry(p.id).or_insert_with(|| spawn_prop(&mut commands, &art, p.kind, Vec2::new(p.pos.x, p.pos.y)));
        let done = p.state == PropState::Done;
        if let Ok((mut tf, mut sprite, _)) = sprites.get_mut(v.body) {
            tf.translation.z = Z_ACTOR + depth(p.pos.y);
            let bob = if done { 0.0 } else { 2.0 * (t * 2.0 + p.id as f32).sin() };
            tf.translation.y = p.pos.y + bob + if p.kind.breakable() { 0.0 } else { 18.0 };
            sprite.color = if done { Color::srgba(0.5, 0.5, 0.55, 0.5) } else { Color::WHITE };
            if done && p.kind.breakable() {
                sprite.color = Color::NONE;
            }
        }
        if let Some(ring) = v.ring {
            if let Ok((_, mut sprite, mut vis)) = sprites.get_mut(ring) {
                *vis = if done { Visibility::Hidden } else { Visibility::Inherited };
                let pulse = 0.35 + 0.2 * (t * 3.0).sin().abs();
                sprite.color = prop_color(p.kind).with_alpha(if p.state == PropState::Active { 0.85 } else { pulse });
            }
        }
        if let Some((back, fill)) = v.meter {
            let share = match p.kind {
                PropKind::Hotspot => p.left as f32 / (60.0 * 25.0),
                PropKind::DataCenter if p.state == PropState::Active => 1.0 - p.left as f32 / (60.0 * 20.0),
                _ => p.charge as f32 / p.kind.charge().max(1) as f32,
            };
            let show = !done && (share > 0.0 && share < 1.0 || p.state == PropState::Active);
            for e in [back, fill] {
                if let Ok((_, _, mut vis)) = sprites.get_mut(e) {
                    *vis = if show { Visibility::Inherited } else { Visibility::Hidden };
                }
            }
            if let Ok((mut tf, mut sprite, _)) = sprites.get_mut(fill) {
                let w = 90.0 * share.clamp(0.0, 1.0);
                sprite.custom_size = Some(Vec2::new(w, 8.0));
                tf.translation.x = p.pos.x - (90.0 - w) / 2.0;
            }
        }
        if let Ok((mut text, mut color)) = words.get_mut(v.tag) {
            let wanted = prop_label(p, world, game.shown_run());
            if text.0 != wanted {
                text.0 = wanted;
            }
            color.0 = if done { theme::QUIET } else { prop_color(p.kind) };
        }
    }
    views.props.retain(|id, v| {
        let keep = seen_p.contains(id);
        if !keep {
            commands.entity(v.root).despawn();
        }
        keep
    });

    // Pickups.
    let mut seen_k = HashSet::new();
    for k in world.pickups.iter().filter(|k| near(k.pos.x, k.pos.y, 0.0)) {
        seen_k.insert(k.id);
        let e = *views.pickups.entry(k.id).or_insert_with(|| {
            use slop_sim::world::PickupKind as K;
            // Coins come in three sizes (LOOT-03); a golden chest glows under
            // a pillar of light (LOOT-02).
            let (image, size) = match k.kind {
                K::Chest if k.value >= 3 => (art.golden_chest.clone(), 70.0),
                K::Chest => (art.pickups[&k.kind].clone(), 54.0),
                K::Clout if k.value >= 12 => (art.coins[2].clone(), 40.0),
                K::Clout if k.value >= 4 => (art.coins[1].clone(), 30.0),
                K::Clout => (art.coins[0].clone(), 20.0),
                _ => (art.pickups[&k.kind].clone(), 40.0),
            };
            let e = commands.spawn((Sprite { image, custom_size: Some(Vec2::splat(size)), ..default() }, Transform::default())).id();
            if k.kind == K::Chest && k.value >= 2 {
                let tint = if k.value >= 3 { Color::srgba(1.0, 0.85, 0.35, 0.9) } else { Color::srgba(0.7, 0.85, 1.0, 0.6) };
                commands.spawn((Sprite { image: art.light_beam.clone(), custom_size: Some(Vec2::new(70.0, 560.0)), color: tint, ..default() }, Transform::from_xyz(0.0, 270.0, -0.05), ChildOf(e)));
            }
            e
        });
        if let Ok((mut tf, _, _)) = sprites.get_mut(e) {
            let bob = 3.0 * (t * 4.0 + k.id as f32).sin();
            tf.translation = Vec3::new(k.pos.x, k.pos.y + bob, Z_LIKE + 0.1);
            let glow = if k.kind == slop_sim::world::PickupKind::Chest { 1.0 + 0.08 * (t * 6.0).sin() } else { 1.0 };
            tf.scale = Vec3::splat(glow);
        }
    }
    retain(&mut commands, &mut views.pickups, &seen_k);

    // Each biome square has its platform's floor, and a bright seam where two
    // platforms meet (BIOME-01).
    let seed = world.seed;
    let (bx, by) = slop_sim::map::biome_cell(slop_sim::V2::new(me.x, me.y));
    let mut cells = HashSet::new();
    for dy in -1..=1 {
        for dx in -1..=1 {
            cells.insert((bx + dx, by + dy));
        }
    }
    for c in &cells {
        views.biomes.entry(*c).or_insert_with(|| {
            let platform = slop_sim::map::biome_of_cell(seed, *c);
            let size = slop_sim::map::BIOME;
            let centre = Vec2::new(c.0 as f32 * size, c.1 as f32 * size);
            let floor = commands
                .spawn((
                    Sprite { image: art.floors[&platform].clone(), custom_size: Some(Vec2::splat(size)), image_mode: SpriteImageMode::Tiled { tile_x: true, tile_y: true, stretch_value: 1.0 }, ..default() },
                    Transform::from_xyz(centre.x, centre.y, Z_FLOOR + 0.01),
                ))
                .id();
            let seam = theme::platform_tint(platform).with_alpha(0.55);
            let left = commands.spawn((Sprite::from_color(seam, Vec2::new(10.0, size)), Transform::from_xyz(centre.x - size / 2.0 + 5.0, centre.y, Z_FLOOR + 0.02))).id();
            let bottom = commands.spawn((Sprite::from_color(seam, Vec2::new(size, 10.0)), Transform::from_xyz(centre.x, centre.y - size / 2.0 + 5.0, Z_FLOOR + 0.02))).id();
            // The platform's name, big and faint, written on its floor.
            let name = commands
                .spawn((Text2d::new(platform.name().to_uppercase()), TextFont { font_size: 120.0, ..default() }, TextColor(theme::platform_tint(platform).with_alpha(0.12)), Transform::from_xyz(centre.x, centre.y, Z_FLOOR + 0.03)))
                .id();
            vec![floor, left, bottom, name]
        });
    }
    views.biomes.retain(|c, list| {
        let keep = cells.contains(c);
        if !keep {
            for e in list.drain(..) {
                commands.entity(e).despawn();
            }
        }
        keep
    });

    // Feed scraps on the floor, chosen per chunk from the run's seed: no two
    // screens look alike.
    let (cx, cy) = slop_sim::map::chunk_of(slop_sim::V2::new(me.x, me.y));
    let mut wanted = HashSet::new();
    for dy in -1..=1 {
        for dx in -1..=1 {
            wanted.insert((cx + dx, cy + dy));
        }
    }
    for c in &wanted {
        views.decor.entry(*c).or_insert_with(|| {
            let mut rng = slop_sim::rng::Rng::new(chunk_seed(seed, *c), 23);
            let origin = Vec2::new(c.0 as f32 * CHUNK, c.1 as f32 * CHUNK);
            (0..DECOR_PER_CHUNK)
                .map(|_| {
                    let at = origin + Vec2::new(rng.range(0.0, CHUNK), rng.range(0.0, CHUNK));
                    let i = rng.below(crate::feed_art::DECOR as u32) as usize;
                    let size = rng.range(90.0, 200.0);
                    // Tinted for the biome the scrap lies in.
                    let tint = theme::platform_tint(slop_sim::map::biome_at(seed, slop_sim::V2::new(at.x, at.y)));
                    commands
                        .spawn((
                            Sprite { image: art.decor[i].clone(), custom_size: Some(Vec2::splat(size)), color: tint, ..default() },
                            Transform::from_xyz(at.x, at.y, Z_FLOOR + 0.05).with_rotation(Quat::from_rotation_z(rng.range(-0.25, 0.25))),
                        ))
                        .id()
                })
                .collect()
        });
    }
    views.decor.retain(|c, list| {
        let keep = wanted.contains(c);
        if !keep {
            for e in list.drain(..) {
                commands.entity(e).despawn();
            }
        }
        keep
    });
}

fn prop_color(kind: PropKind) -> Color {
    match kind {
        PropKind::Hotspot => Color::srgb(0.55, 0.85, 1.0),
        PropKind::Vending | PropKind::Gacha => Color::srgb(1.0, 0.8, 0.25),
        PropKind::Shrine => Color::srgb(1.0, 0.9, 0.5),
        PropKind::DataCenter => Color::srgb(0.45, 0.95, 0.6),
        PropKind::CommentSection => theme::DANGER,
        _ => theme::PAPER,
    }
}

/// The words over a prop: its name, and its price or state.
fn prop_label(p: &slop_sim::map::Prop, world: &World, run: Option<&slop_rules::Run>) -> String {
    let name = p.kind.name();
    if p.state == PropState::Done {
        return match p.kind {
            PropKind::DataCenter => "Uploaded".into(),
            PropKind::CommentSection => "Locked".into(),
            PropKind::Hotspot => "No signal".into(),
            _ if p.kind.breakable() => String::new(),
            _ => format!("{name}\n(used)"),
        };
    }
    let near = p.pos.distance(world.player.pos) < p.kind.radius() + 140.0;
    match p.kind {
        PropKind::Gacha => format!("{name}\n{} Clout", run.map_or(0, |r| r.gacha_price(p.id))),
        PropKind::DataCenter if p.state == PropState::Active => format!("UPLOADING {:.0}%", 100.0 - p.left as f32 / (60.0 * 20.0) * 100.0),
        PropKind::CommentSection if p.state == PropState::Active => "SURVIVE THE REPLIES".into(),
        _ if p.kind.breakable() => String::new(),
        _ if near => format!("{name}\n{}", p.kind.blurb()),
        _ => name.to_string(),
    }
}

fn spawn_prop(commands: &mut Commands, art: &Art, kind: PropKind, at: Vec2) -> PropView {
    let root = commands.spawn((Root, Transform::default(), Visibility::Visible)).id();
    let r = kind.radius();
    let ring = (!kind.breakable()).then(|| {
        commands
            .spawn((Sprite { image: art.ring.clone(), custom_size: Some(Vec2::splat(r * 2.0)), color: prop_color(kind).with_alpha(0.4), ..default() }, Transform::from_xyz(at.x, at.y, Z_ZONE + 0.1), ChildOf(root)))
            .id()
    });
    let size = if kind.breakable() { r * 2.4 } else { 96.0 };
    let body = commands.spawn((Sprite { image: art.props[&kind].clone(), custom_size: Some(Vec2::splat(size)), ..default() }, Transform::from_xyz(at.x, at.y, Z_ACTOR), ChildOf(root))).id();
    let meter = (!kind.breakable()).then(|| {
        let y = at.y - 70.0;
        let back = commands.spawn((Sprite::from_color(theme::INK, Vec2::new(94.0, 12.0)), Transform::from_xyz(at.x, y, Z_TEXT - 0.1), Visibility::Hidden, ChildOf(root))).id();
        let fill = commands.spawn((Sprite::from_color(prop_color(kind), Vec2::new(0.0, 8.0)), Transform::from_xyz(at.x, y, Z_TEXT), Visibility::Hidden, ChildOf(root))).id();
        (back, fill)
    });
    let tag = commands
        .spawn((Text2d::new(""), TextFont { font_size: 15.0, ..default() }, TextColor(theme::PAPER), TextLayout::new_with_justify(Justify::Center), Transform::from_xyz(at.x, at.y + 84.0, Z_TEXT), ChildOf(root)))
        .id();
    PropView { root, body, ring, meter, tag }
}

/// The camera stays on you, drawn between ticks: no lead, no lag (FEEL-04).
fn follow_camera(
    time: Res<Time>,
    game: Res<Game>,
    mut shake: ResMut<Shake>,
    views: Res<Views>,
    mut camera: Query<&mut Transform, With<MainCamera>>,
    mut floor: Query<&mut Transform, (Without<MainCamera>, With<Sprite>)>,
) {
    let Ok(mut tf) = camera.single_mut() else { return };
    let dt = time.delta_secs();
    let at = match game.world() {
        Some(w) => lerp(w.player.prev, w.player.pos, game.alpha),
        None => Vec2::new(tf.translation.x, tf.translation.y),
    };
    shake.trauma = (shake.trauma - dt * 1.8).max(0.0);
    let k = shake.trauma * shake.trauma * 14.0;
    let wobble = Vec2::new((time.elapsed_secs() * 61.0).sin(), (time.elapsed_secs() * 47.0).cos()) * k;
    tf.translation.x = at.x + wobble.x;
    tf.translation.y = at.y + wobble.y;
    if let Ok(mut f) = floor.get_mut(views.floor) {
        f.translation.x = (at.x / TILE).round() * TILE;
        f.translation.y = (at.y / TILE).round() * TILE;
    }
}

/// The colour of an elite's affix (ELITE-02).
fn affix_color(affix: &str) -> Color {
    match affix {
        "VERIFIED" => theme::type_color(slop_sim::DamageType::Fact),
        "VIRAL" => Color::srgb(0.35, 1.0, 0.45),
        "SPONSORED" => Color::srgb(1.0, 0.8, 0.2),
        "TRENDING" => Color::srgb(1.0, 0.5, 0.15),
        "TOXIC" => Color::srgb(0.72, 0.35, 1.0),
        "SUMMONER" => Color::srgb(0.3, 0.95, 0.95),
        "HULKING" => Color::srgb(1.0, 0.25, 0.25),
        "NEMESIS" => Color::srgb(1.0, 1.0, 1.0),
        _ => Color::WHITE,
    }
}
