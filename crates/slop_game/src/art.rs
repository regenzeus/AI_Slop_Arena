//! Every sprite in the game, drawn in code at startup (ART-01).
//!
//! Shapes are signed distance functions painted with a thick outline and
//! anti-aliased edges, at twice the size they are shown. The pictures
//! themselves are in `draw`; this module holds the painter and the handles.
//! Each character also gets a white copy for hit flashes.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use slop_sim::content::{Accessory, Archetype, Body, BossKind, Look, Object, Platform};

use slop_rules::items::{StatKind, ITEMS};
use slop_rules::Hero;
use slop_sim::content::WEAPONS;
use slop_sim::DamageType;

use crate::{draw, feed_art, icons};
use slop_sim::map::{PropKind, WallKind};
use slop_sim::world::PickupKind;

/// Canvas pixels per world unit.
pub const DENSITY: f32 = 2.0;

pub type Rgba = [f32; 4];

pub const INK: Rgba = [0.13, 0.12, 0.17, 1.0];
pub const WHITE: Rgba = [1.0, 1.0, 1.0, 1.0];
/// Anything that hurts you is outlined in this pink, and nothing else is (FAIR-04).
pub const DANGER: Rgba = [1.0, 0.18, 0.58, 1.0];

pub fn hex(h: u32) -> Rgba {
    [((h >> 16) & 255) as f32 / 255.0, ((h >> 8) & 255) as f32 / 255.0, (h & 255) as f32 / 255.0, 1.0]
}

pub fn alpha(c: Rgba, a: f32) -> Rgba {
    [c[0], c[1], c[2], c[3] * a]
}

// Signed distances: negative inside.

pub fn circle(x: f32, y: f32, cx: f32, cy: f32, r: f32) -> f32 {
    ((x - cx).powi(2) + (y - cy).powi(2)).sqrt() - r
}

pub fn ellipse(x: f32, y: f32, cx: f32, cy: f32, rx: f32, ry: f32) -> f32 {
    let (dx, dy) = ((x - cx) / rx, (y - cy) / ry);
    ((dx * dx + dy * dy).sqrt() - 1.0) * rx.min(ry)
}

pub fn rbox(x: f32, y: f32, cx: f32, cy: f32, hw: f32, hh: f32, r: f32) -> f32 {
    let qx = (x - cx).abs() - hw + r;
    let qy = (y - cy).abs() - hh + r;
    (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt() + qx.max(qy).min(0.0) - r
}

pub fn capsule(x: f32, y: f32, ax: f32, ay: f32, bx: f32, by: f32, r: f32) -> f32 {
    let (px, py, vx, vy) = (x - ax, y - ay, bx - ax, by - ay);
    let h = ((px * vx + py * vy) / (vx * vx + vy * vy)).clamp(0.0, 1.0);
    ((px - vx * h).powi(2) + (py - vy * h).powi(2)).sqrt() - r
}

/// A triangle with corners a, b, c.
pub fn triangle(x: f32, y: f32, a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> f32 {
    let edge = |p0: (f32, f32), p1: (f32, f32)| {
        let (ex, ey) = (p1.0 - p0.0, p1.1 - p0.1);
        let (wx, wy) = (x - p0.0, y - p0.1);
        let h = ((wx * ex + wy * ey) / (ex * ex + ey * ey)).clamp(0.0, 1.0);
        let d = ((wx - ex * h).powi(2) + (wy - ey * h).powi(2)).sqrt();
        (d, ex * wy - ey * wx)
    };
    let (d0, s0) = edge(a, b);
    let (d1, s1) = edge(b, c);
    let (d2, s2) = edge(c, a);
    let d = d0.min(d1).min(d2);
    let inside = (s0 >= 0.0 && s1 >= 0.0 && s2 >= 0.0) || (s0 <= 0.0 && s1 <= 0.0 && s2 <= 0.0);
    if inside {
        -d
    } else {
        d
    }
}

/// Any simple polygon, by its corners in order.
pub fn polygon(x: f32, y: f32, pts: &[(f32, f32)]) -> f32 {
    let mut d = f32::MAX;
    let mut inside = false;
    for i in 0..pts.len() {
        let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
        let (ex, ey) = (b.0 - a.0, b.1 - a.1);
        let (wx, wy) = (x - a.0, y - a.1);
        let h = ((wx * ex + wy * ey) / (ex * ex + ey * ey)).clamp(0.0, 1.0);
        d = d.min(((wx - ex * h).powi(2) + (wy - ey * h).powi(2)).sqrt());
        if (a.1 > y) != (b.1 > y) && x < a.0 + (y - a.1) / (b.1 - a.1) * ex {
            inside = !inside;
        }
    }
    if inside {
        -d
    } else {
        d
    }
}

/// A wedge from (cx, cy) pointing right, `half` radians each side.
pub fn wedge(x: f32, y: f32, cx: f32, cy: f32, r: f32, half: f32) -> f32 {
    let (dx, dy) = (x - cx, y - cy);
    let d = (dx * dx + dy * dy).sqrt();
    let off = dy.atan2(dx).abs() - half;
    (d - r).max(off * d.max(1.0))
}

/// An RGBA painting surface; colours are sRGB with straight alpha.
pub struct Canvas {
    pub w: usize,
    pub h: usize,
    px: Vec<Rgba>,
}

impl Canvas {
    pub fn new(w: usize, h: usize) -> Canvas {
        Canvas { w, h, px: vec![[0.0; 4]; w * h] }
    }

    /// Paints a shape with a fill and an outline of `width` pixels inside its edge.
    pub fn shape(&mut self, sdf: impl Fn(f32, f32) -> f32, fill: Rgba, line: Rgba, width: f32) {
        for y in 0..self.h {
            for x in 0..self.w {
                let d = sdf(x as f32 + 0.5, y as f32 + 0.5);
                let cover = (0.5 - d).clamp(0.0, 1.0);
                if cover <= 0.0 {
                    continue;
                }
                let t = if width > 0.0 { (0.5 - (d + width)).clamp(0.0, 1.0) } else { 1.0 };
                let mut c = [0.0; 4];
                for k in 0..4 {
                    c[k] = line[k] + (fill[k] - line[k]) * t;
                }
                c[3] *= cover;
                self.over(x, y, c);
            }
        }
    }

    /// Fills a shape with no outline.
    pub fn fill(&mut self, sdf: impl Fn(f32, f32) -> f32, color: Rgba) {
        self.shape(sdf, color, color, 0.0);
    }

    /// Paints with a per-pixel colour (gradients, fades).
    pub fn paint(&mut self, f: impl Fn(f32, f32) -> Option<Rgba>) {
        for y in 0..self.h {
            for x in 0..self.w {
                if let Some(c) = f(x as f32 + 0.5, y as f32 + 0.5) {
                    self.over(x, y, c);
                }
            }
        }
    }

    fn over(&mut self, x: usize, y: usize, src: Rgba) {
        let dst = &mut self.px[y * self.w + x];
        let a = src[3] + dst[3] * (1.0 - src[3]);
        if a <= 0.0 {
            return;
        }
        for k in 0..3 {
            dst[k] = (src[k] * src[3] + dst[k] * dst[3] * (1.0 - src[3])) / a;
        }
        dst[3] = a;
    }

    #[cfg(test)]
    pub fn pixel(&self, x: usize, y: usize) -> Rgba {
        self.px[y * self.w + x]
    }

    #[cfg(test)]
    pub fn alpha_at(&self, i: usize) -> f32 {
        self.px[i][3]
    }

    /// A hash of the picture, to tell two drawings apart in tests.
    #[cfg(test)]
    pub fn fingerprint(&self) -> u64 {
        self.px.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, p| p.iter().fold(h, |h, v| (h ^ (v * 255.0) as u64).wrapping_mul(0x100_0000_01b3)))
    }

    /// Paints `src` over this canvas, scaled by `scale`, its top-left corner at
    /// (`ox`, `oy`) in this canvas's pixels (nearest-neighbour sampling).
    pub fn stamp(&mut self, src: &Canvas, ox: f32, oy: f32, scale: f32) {
        let (w, h) = ((src.w as f32 * scale) as usize, (src.h as f32 * scale) as usize);
        for y in 0..h {
            for x in 0..w {
                let (tx, ty) = (ox as i64 + x as i64, oy as i64 + y as i64);
                if tx < 0 || ty < 0 || tx >= self.w as i64 || ty >= self.h as i64 {
                    continue;
                }
                let sx = ((x as f32 + 0.5) / scale) as usize;
                let sy = ((y as f32 + 0.5) / scale) as usize;
                let c = src.px[sy.min(src.h - 1) * src.w + sx.min(src.w - 1)];
                if c[3] > 0.0 {
                    self.over(tx as usize, ty as usize, c);
                }
            }
        }
    }

    /// The same silhouette in solid white, for hit flashes.
    pub fn whitened(&self) -> Canvas {
        let px = self.px.iter().map(|p| [1.0, 1.0, 1.0, p[3]]).collect();
        Canvas { w: self.w, h: self.h, px }
    }

    pub fn image(&self) -> Image {
        let data: Vec<u8> = self.px.iter().flat_map(|p| p.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8)).collect();
        Image::new(
            Extent3d { width: self.w as u32, height: self.h as u32, depth_or_array_layers: 1 },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        )
    }
}

/// A canvas drawn in a 100-unit design space, scaled to `size` pixels, with
/// a standard outline. Keeps the pictures in `draw` short.
pub struct Pen {
    pub c: Canvas,
    pub k: f32,
    pub line: f32,
}

impl Pen {
    pub fn new(size: f32) -> Pen {
        let s = size.ceil().max(8.0) as usize;
        Pen { c: Canvas::new(s, s), k: s as f32 / 100.0, line: 3.5 * s as f32 / 100.0 }
    }

    pub fn circle(&mut self, cx: f32, cy: f32, r: f32, fill: Rgba) {
        let k = self.k;
        self.c.shape(|x, y| circle(x, y, cx * k, cy * k, r * k), fill, INK, self.line);
    }

    pub fn ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, fill: Rgba) {
        let k = self.k;
        self.c.shape(|x, y| ellipse(x, y, cx * k, cy * k, rx * k, ry * k), fill, INK, self.line);
    }

    pub fn rect(&mut self, cx: f32, cy: f32, hw: f32, hh: f32, r: f32, fill: Rgba) {
        let k = self.k;
        self.c.shape(|x, y| rbox(x, y, cx * k, cy * k, hw * k, hh * k, r * k), fill, INK, self.line);
    }

    pub fn stick(&mut self, ax: f32, ay: f32, bx: f32, by: f32, r: f32, fill: Rgba) {
        let k = self.k;
        self.c.shape(|x, y| capsule(x, y, ax * k, ay * k, bx * k, by * k, r * k), fill, INK, self.line);
    }

    pub fn tri(&mut self, a: (f32, f32), b: (f32, f32), c: (f32, f32), fill: Rgba) {
        let k = self.k;
        let s = |p: (f32, f32)| (p.0 * k, p.1 * k);
        self.c.shape(|x, y| triangle(x, y, s(a), s(b), s(c)), fill, INK, self.line);
    }

    /// A filled dot with no outline (eyes, details).
    pub fn dot(&mut self, cx: f32, cy: f32, r: f32, color: Rgba) {
        let k = self.k;
        self.c.fill(|x, y| circle(x, y, cx * k, cy * k, r * k), color);
    }

    /// A line with no outline.
    pub fn line(&mut self, ax: f32, ay: f32, bx: f32, by: f32, r: f32, color: Rgba) {
        let k = self.k;
        self.c.fill(|x, y| capsule(x, y, ax * k, ay * k, bx * k, by * k, r * k), color);
    }

    /// Any shape given as a distance in design units, outlined.
    pub fn sdf(&mut self, f: impl Fn(f32, f32) -> f32, fill: Rgba) {
        let k = self.k;
        self.c.shape(|x, y| f(x / k, y / k) * k, fill, INK, self.line);
    }

    /// Any shape given as a distance in design units, with no outline.
    pub fn sdf_fill(&mut self, f: impl Fn(f32, f32) -> f32, color: Rgba) {
        let k = self.k;
        self.c.fill(|x, y| f(x / k, y / k) * k, color);
    }

    /// Circles merged into one outlined lump.
    pub fn blob(&mut self, parts: &[(f32, f32, f32)], fill: Rgba) {
        self.sdf(|x, y| parts.iter().map(|&(cx, cy, r)| circle(x, y, cx, cy, r)).fold(f32::MAX, f32::min), fill);
    }

    /// A ring of width `w` with no outline.
    pub fn ring(&mut self, cx: f32, cy: f32, r: f32, w: f32, color: Rgba) {
        self.sdf_fill(|x, y| circle(x, y, cx, cy, r).abs() - w / 2.0, color);
    }

    /// A smile: the lower part of a ring.
    pub fn smile(&mut self, cx: f32, cy: f32, r: f32, color: Rgba) {
        let k = self.k;
        self.c.fill(|x, y| circle(x, y, cx * k, cy * k, r * k).max(-circle(x, y, cx * k, (cy - r * 0.5) * k, r * 1.05 * k)), color);
    }
}

/// The human's face for each band of Mental Health, best first.
pub const FACES: usize = 4;
/// World size of slop sprites relative to their collision radius.
pub const SLOP_SCALE: f32 = 2.7;
pub const HUMAN_SIZE: f32 = 52.0;
/// Size of the speech bubble frame's corners, in canvas pixels.
pub const BUBBLE_BORDER: f32 = 14.0;

/// A sprite and its white flash copy.
pub type Pair = (Handle<Image>, Handle<Image>);

#[derive(Resource)]
pub struct Art {
    /// Each hero's faces, from calm to the thousand-yard stare.
    pub human: HashMap<Hero, [Handle<Image>; FACES]>,
    pub human_flash: HashMap<Hero, Handle<Image>>,
    /// Icons, in `WEAPONS` and `ITEMS` order (ART-04).
    pub weapon_icons: Vec<Handle<Image>>,
    pub item_icons: Vec<Handle<Image>>,
    pub stat_icons: HashMap<StatKind, Handle<Image>>,
    /// In `DamageType::ALL` order.
    pub type_icons: Vec<Handle<Image>>,
    pub reroll: Handle<Image>,
    pub star: Handle<Image>,
    pub archetypes: HashMap<Archetype, Pair>,
    pub bodies: HashMap<Body, Pair>,
    pub objects: HashMap<Object, Pair>,
    pub accessories: HashMap<Accessory, Pair>,
    pub bosses: HashMap<BossKind, Pair>,
    pub floors: HashMap<Platform, Handle<Image>>,
    pub shots: HashMap<Look, Handle<Image>>,
    pub bubble: Handle<Image>,
    pub coin: Handle<Image>,
    pub brain: Handle<Image>,
    pub shadow: Handle<Image>,
    pub typing: Handle<Image>,
    pub swing: Handle<Image>,
    pub spray: Handle<Image>,
    pub ring: Handle<Image>,
    pub danger_ring: Handle<Image>,
    pub dot: Handle<Image>,
    /// A round dot, for particles.
    pub ball: Handle<Image>,
    pub dog: Handle<Image>,
    pub halo: Handle<Image>,
    pub shield: Handle<Image>,
    pub swirl: Handle<Image>,
    pub seed: Handle<Image>,
    pub bomb: Handle<Image>,
    pub downvote: Handle<Image>,
    pub filter: Handle<Image>,
    /// The Feed (ART-06).
    pub pickups: HashMap<PickupKind, Handle<Image>>,
    pub props: HashMap<PropKind, Handle<Image>>,
    pub walls: HashMap<WallKind, Handle<Image>>,
    pub decor: Vec<Handle<Image>>,
    pub blessings: Vec<Handle<Image>>,
    pub like: Handle<Image>,
    /// The frames of each brainrot icon's animation (ICON-02).
    pub icon_frames: HashMap<Archetype, Vec<Pair>>,
    /// In `GLITCHES` order (GLITCH-01).
    pub glitch_icons: Vec<Handle<Image>>,
    /// The easy option (AUTO-01).
    pub delegate: Handle<Image>,
    /// Coin, stack and bag (LOOT-03).
    pub coins: [Handle<Image>; 3],
    pub golden_chest: Handle<Image>,
    pub light_beam: Handle<Image>,
    pub sunflowers: Handle<Image>,
    pub fire: Handle<Image>,
    pub brainrot_zone: Handle<Image>,
    pub roots: Handle<Image>,
    /// A fine line for big circles.
    pub thin_ring: Handle<Image>,
}

impl Art {
    pub fn build(images: &mut Assets<Image>) -> Art {
        let mut human = HashMap::new();
        let mut human_flash = HashMap::new();
        for hero in Hero::ALL {
            let faces: Vec<Canvas> = (0..FACES).map(|f| draw::human(f, hero)).collect();
            human_flash.insert(hero, images.add(faces[0].whitened().image()));
            human.insert(hero, [images.add(faces[0].image()), images.add(faces[1].image()), images.add(faces[2].image()), images.add(faces[3].image())]);
        }
        let mut pair = |c: Canvas| (images.add(c.image()), images.add(c.whitened().image()));
        let archetypes = Archetype::ALL.iter().map(|&a| (a, pair(draw::archetype(a)))).collect();
        let icon_frames = Archetype::ICONS.iter().chain([Archetype::LootGoblin].iter()).map(|&a| (a, (0..draw::ICON_FRAMES).map(|f| pair(draw::icon(a, f))).collect())).collect();
        let bodies = Body::ALL.iter().map(|&b| (b, pair(draw::body(b)))).collect();
        let objects = Object::ALL.iter().map(|&o| (o, pair(draw::object(o)))).collect();
        let accessories = Accessory::ALL.iter().map(|&a| (a, pair(draw::accessory(a)))).collect();
        let bosses = BossKind::ALL.iter().map(|&b| (b, pair(draw::boss(b)))).collect();
        let mut add = |c: Canvas| images.add(c.image());
        let floors = Platform::ALL.iter().map(|&p| (p, add(draw::floor(p)))).collect();
        let shots = Look::ALL.iter().filter(|l| **l != Look::Bubble && **l != Look::Post).map(|&l| (l, add(draw::shot(l)))).collect();
        Art {
            human,
            human_flash,
            weapon_icons: (0..WEAPONS.len()).map(|i| add(icons::weapon(i))).collect(),
            item_icons: (0..ITEMS.len()).map(|i| add(icons::item(i))).collect(),
            stat_icons: StatKind::ALL.iter().map(|&k| (k, add(icons::stat(k)))).collect(),
            type_icons: DamageType::ALL.iter().map(|&t| add(icons::damage_type(t))).collect(),
            reroll: add(icons::reroll()),
            star: add(icons::star()),
            archetypes,
            bodies,
            objects,
            accessories,
            bosses,
            floors,
            shots,
            bubble: add(draw::bubble()),
            coin: add(draw::coin()),
            brain: add(draw::brain()),
            shadow: add(draw::shadow()),
            typing: add(draw::typing()),
            swing: add(draw::sector(1.05, [1.0, 1.0, 1.0, 0.85])),
            spray: add(draw::sector(0.45, [0.45, 0.75, 1.0, 0.8])),
            ring: add(draw::ring(WHITE)),
            danger_ring: add(draw::ring(DANGER)),
            dot: add(draw::dot()),
            ball: add(draw::ball()),
            dog: add(draw::dog()),
            halo: add(draw::halo()),
            shield: add(draw::shield()),
            swirl: add(draw::swirl()),
            seed: add(draw::seed_patch()),
            bomb: add(draw::bomb_marker()),
            downvote: add(draw::downvote()),
            filter: add(draw::filter()),
            pickups: [PickupKind::Food, PickupKind::Magnet, PickupKind::Bomb, PickupKind::Freeze, PickupKind::Clout, PickupKind::Chest, PickupKind::Inject].iter().map(|&k| (k, add(feed_art::pickup(k)))).collect(),
            props: PropKind::ALL.iter().map(|&k| (k, add(feed_art::prop(k)))).collect(),
            walls: [WallKind::ServerRack, WallKind::Billboard, WallKind::Desk, WallKind::Fence].iter().map(|&k| (k, add(feed_art::wall(k)))).collect(),
            decor: (0..feed_art::DECOR).map(|i| add(feed_art::decor(i))).collect(),
            blessings: (0..slop_rules::items::BLESSINGS.len()).map(|i| add(feed_art::blessing(i))).collect(),
            like: add(feed_art::like()),
            icon_frames,
            glitch_icons: (0..slop_rules::items::GLITCHES.len()).map(|i| add(icons::glitch(i))).collect(),
            delegate: add(icons::delegate()),
            coins: [add(icons::coins(0)), add(icons::coins(1)), add(icons::coins(2))],
            golden_chest: add(feed_art::golden_chest()),
            light_beam: add(draw::light_beam()),
            sunflowers: add(draw::sunflower_patch()),
            fire: add(draw::fire()),
            brainrot_zone: add(draw::brainrot_zone()),
            roots: add(draw::roots()),
            thin_ring: add(draw::thin_ring()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: ART-01
    #[test]
    fn shapes_have_soft_edges_and_outlines() {
        let mut c = Canvas::new(40, 40);
        c.shape(|x, y| circle(x, y, 20.0, 20.0, 15.0), [1.0, 0.0, 0.0, 1.0], INK, 3.0);
        let at = |x: usize, y: usize| c.px[y * 40 + x];
        assert_eq!(at(20, 20), [1.0, 0.0, 0.0, 1.0], "filled inside");
        assert!(at(20, 6)[0] < 0.3, "outline at the edge");
        assert_eq!(at(0, 0)[3], 0.0, "transparent outside");
        let edge = at(20, 5)[3];
        assert!(edge > 0.0 && edge < 1.0, "anti-aliased edge, alpha {edge}");
        assert_eq!(c.whitened().px[20 * 40 + 20], [1.0, 1.0, 1.0, 1.0]);
        assert!(triangle(5.0, 5.0, (0.0, 0.0), (10.0, 0.0), (5.0, 10.0)) < 0.0);
        assert!(triangle(20.0, 20.0, (0.0, 0.0), (10.0, 0.0), (5.0, 10.0)) > 0.0);
    }

    // REQ: ART-02
    #[test]
    fn every_character_is_drawn_and_not_empty() {
        let opaque = |c: &Canvas| c.px.iter().filter(|p| p[3] > 0.5).count();
        for a in Archetype::ALL {
            assert!(opaque(&draw::archetype(a)) > 200, "{a:?}");
        }
        for b in Body::ALL {
            assert!(opaque(&draw::body(b)) > 200, "{b:?}");
        }
        for b in BossKind::ALL {
            assert!(opaque(&draw::boss(b)) > 1000, "{b:?}");
        }
    }
}
