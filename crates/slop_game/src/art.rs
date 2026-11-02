//! Every sprite in the game, drawn in code at startup (ART-01).
//!
//! Shapes are described by signed distance functions and painted with a thick
//! outline and anti-aliased edges, at twice the size they are shown, so the
//! game ships without a single bought asset. Each slop also gets a white copy
//! for hit flashes.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use slop_sim::content::{Look, SlopKind};

/// Canvas pixels per world unit.
pub const DENSITY: f32 = 2.0;

type Rgba = [f32; 4];

const INK: Rgba = [0.13, 0.12, 0.17, 1.0];
/// Anything that hurts you is outlined in this pink, and nothing else is (FAIR-04).
pub const DANGER: Rgba = [1.0, 0.18, 0.58, 1.0];

fn hex(h: u32) -> Rgba {
    [((h >> 16) & 255) as f32 / 255.0, ((h >> 8) & 255) as f32 / 255.0, (h & 255) as f32 / 255.0, 1.0]
}

fn alpha(c: Rgba, a: f32) -> Rgba {
    [c[0], c[1], c[2], c[3] * a]
}

// Signed distances: negative inside.

fn circle(x: f32, y: f32, cx: f32, cy: f32, r: f32) -> f32 {
    ((x - cx).powi(2) + (y - cy).powi(2)).sqrt() - r
}

fn ellipse(x: f32, y: f32, cx: f32, cy: f32, rx: f32, ry: f32) -> f32 {
    let (dx, dy) = ((x - cx) / rx, (y - cy) / ry);
    ((dx * dx + dy * dy).sqrt() - 1.0) * rx.min(ry)
}

fn rbox(x: f32, y: f32, cx: f32, cy: f32, hw: f32, hh: f32, r: f32) -> f32 {
    let qx = (x - cx).abs() - hw + r;
    let qy = (y - cy).abs() - hh + r;
    (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt() + qx.max(qy).min(0.0) - r
}

fn capsule(x: f32, y: f32, ax: f32, ay: f32, bx: f32, by: f32, r: f32) -> f32 {
    let (px, py, vx, vy) = (x - ax, y - ay, bx - ax, by - ay);
    let h = ((px * vx + py * vy) / (vx * vx + vy * vy)).clamp(0.0, 1.0);
    ((px - vx * h).powi(2) + (py - vy * h).powi(2)).sqrt() - r
}

/// A wedge from (cx, cy) pointing right, `half` radians each side.
fn wedge(x: f32, y: f32, cx: f32, cy: f32, r: f32, half: f32) -> f32 {
    let (dx, dy) = (x - cx, y - cy);
    let d = (dx * dx + dy * dy).sqrt();
    let off = dy.atan2(dx).abs() - half;
    (d - r).max(off * d.max(1.0))
}

/// An RGBA painting surface; colours are sRGB with straight alpha.
pub struct Canvas {
    w: usize,
    h: usize,
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

/// The human's face for each band of Mental Health, best first.
pub const FACES: usize = 4;

#[derive(Resource)]
pub struct Art {
    /// Faces from calm to the thousand-yard stare.
    pub human: [Handle<Image>; FACES],
    pub human_flash: Handle<Image>,
    pub slop: HashMap<SlopKind, (Handle<Image>, Handle<Image>)>,
    pub shots: HashMap<Look, Handle<Image>>,
    pub bubble: Handle<Image>,
    pub like: Handle<Image>,
    pub shadow: Handle<Image>,
    pub typing: Handle<Image>,
    pub swing: Handle<Image>,
    pub spray: Handle<Image>,
    pub floor: Handle<Image>,
    pub dot: Handle<Image>,
}

/// World size each slop is drawn at, relative to its collision radius.
pub const SLOP_SCALE: f32 = 2.7;
pub const HUMAN_SIZE: f32 = 52.0;
/// Size of the speech bubble frame's corners, in canvas pixels.
pub const BUBBLE_BORDER: f32 = 14.0;

impl Art {
    pub fn build(images: &mut Assets<Image>) -> Art {
        let mut add = |c: &Canvas| images.add(c.image());
        let faces: Vec<Canvas> = (0..FACES).map(human).collect();
        let human_flash = add(&faces[0].whitened());
        let human = [add(&faces[0]), add(&faces[1]), add(&faces[2]), add(&faces[3])];
        let mut slop_art = HashMap::new();
        for kind in SlopKind::ALL {
            let c = slop(kind);
            slop_art.insert(kind, (add(&c), add(&c.whitened())));
        }
        let mut shots = HashMap::new();
        for look in [Look::Rock, Look::Bullet, Look::Pen, Look::EyeRoll, Look::EmDash] {
            shots.insert(look, add(&shot(look)));
        }
        Art {
            human,
            human_flash,
            slop: slop_art,
            shots,
            bubble: add(&bubble()),
            like: add(&like()),
            shadow: add(&shadow()),
            typing: add(&typing()),
            swing: add(&sector(1.05, [1.0, 1.0, 1.0, 0.85])),
            spray: add(&sector(0.45, [0.45, 0.75, 1.0, 0.8])),
            floor: add(&floor()),
            dot: add(&dot()),
        }
    }
}

/// The human: a round head over a hoodie, headphones around the neck.
fn human(face: usize) -> Canvas {
    let s = HUMAN_SIZE * DENSITY;
    let mut c = Canvas::new(s as usize, s as usize);
    let k = s / 104.0;
    let hoodie = hex(0xF28C38);
    let skin = hex(0xF6D2B0);
    c.shape(|x, y| rbox(x, y, 52.0 * k, 78.0 * k, 30.0 * k, 22.0 * k, 16.0 * k), hoodie, INK, 3.0 * k);
    c.fill(|x, y| capsule(x, y, 52.0 * k, 64.0 * k, 52.0 * k, 96.0 * k, 1.5 * k), alpha(INK, 0.35));
    c.shape(|x, y| circle(x, y, 52.0 * k, 40.0 * k, 28.0 * k), skin, INK, 3.0 * k);
    // Headphones resting on the neck.
    c.shape(|x, y| capsule(x, y, 30.0 * k, 66.0 * k, 74.0 * k, 66.0 * k, 5.0 * k), hex(0x3A3D4D), INK, 2.0 * k);
    c.shape(|x, y| circle(x, y, 28.0 * k, 66.0 * k, 8.0 * k), hex(0x3A3D4D), INK, 2.0 * k);
    c.shape(|x, y| circle(x, y, 76.0 * k, 66.0 * k, 8.0 * k), hex(0x3A3D4D), INK, 2.0 * k);
    let eye_r = if face == 3 { 5.5 } else { 4.0 };
    for ex in [41.0, 63.0] {
        if face == 3 {
            // Dark circles and tiny pupils: the thousand-yard stare.
            c.fill(|x, y| ellipse(x, y, ex * k, 44.0 * k, 8.0 * k, 4.0 * k), alpha(hex(0x8A6A9A), 0.6));
            c.shape(|x, y| circle(x, y, ex * k, 38.0 * k, eye_r * k), [1.0; 4], INK, 1.5 * k);
            c.fill(|x, y| circle(x, y, ex * k, 38.0 * k, 1.5 * k), INK);
        } else {
            c.fill(|x, y| circle(x, y, ex * k, 38.0 * k, eye_r * k), INK);
        }
    }
    match face {
        0 => c.fill(|x, y| circle(x, y, 52.0 * k, 46.0 * k, 9.0 * k).max(-circle(x, y, 52.0 * k, 41.0 * k, 10.0 * k)), INK),
        1 => c.fill(|x, y| capsule(x, y, 45.0 * k, 52.0 * k, 59.0 * k, 52.0 * k, 1.8 * k), INK),
        2 => {
            c.fill(|x, y| ellipse(x, y, 52.0 * k, 54.0 * k, 6.0 * k, 5.0 * k), INK);
            c.shape(|x, y| circle(x, y, 80.0 * k, 30.0 * k, 6.0 * k).min(capsule(x, y, 80.0 * k, 22.0 * k, 80.0 * k, 28.0 * k, 3.0 * k)), hex(0x8FD3FF), INK, 1.5 * k);
        }
        _ => c.fill(|x, y| capsule(x, y, 46.0 * k, 54.0 * k, 58.0 * k, 54.0 * k, 1.5 * k), INK),
    }
    c
}

fn slop(kind: SlopKind) -> Canvas {
    let s = kind.def().radius * SLOP_SCALE * DENSITY;
    let mut c = Canvas::new(s as usize, s as usize);
    let k = s / 100.0;
    let w = 3.5 * k;
    match kind {
        SlopKind::BubbleBot => {
            // A chat window that is always typing.
            c.shape(|x, y| rbox(x, y, 50.0 * k, 46.0 * k, 34.0 * k, 26.0 * k, 10.0 * k).min(capsule(x, y, 34.0 * k, 66.0 * k, 26.0 * k, 82.0 * k, 5.0 * k)), hex(0x8FE3F0), INK, w);
            c.fill(|x, y| rbox(x, y, 50.0 * k, 26.0 * k, 31.0 * k, 5.0 * k, 3.0 * k), hex(0x4FB8CC));
            for dx in [36.0, 50.0, 64.0] {
                c.fill(|x, y| circle(x, y, dx * k, 50.0 * k, 5.0 * k), INK);
            }
        }
        SlopKind::EmDashLancer => {
            c.shape(|x, y| rbox(x, y, 50.0 * k, 54.0 * k, 24.0 * k, 36.0 * k, 22.0 * k), hex(0x9B6BFF), INK, w);
            c.shape(|x, y| rbox(x, y, 50.0 * k, 40.0 * k, 18.0 * k, 8.0 * k, 6.0 * k), hex(0x2A2240), INK, 2.0 * k);
            c.fill(|x, y| rbox(x, y, 56.0 * k, 40.0 * k, 9.0 * k, 3.0 * k, 3.0 * k), DANGER);
            c.fill(|x, y| capsule(x, y, 30.0 * k, 72.0 * k, 70.0 * k, 72.0 * k, 3.0 * k), alpha(INK, 0.5));
        }
        SlopKind::Listicle => {
            // A sticky note with a folded corner; its number is drawn as text.
            c.shape(|x, y| rbox(x, y, 50.0 * k, 50.0 * k, 34.0 * k, 34.0 * k, 4.0 * k), hex(0xFFE066), INK, w);
            c.fill(|x, y| capsule(x, y, 72.0 * k, 72.0 * k, 82.0 * k, 82.0 * k, 7.0 * k), hex(0xE0B83A));
        }
        SlopKind::YesBot => {
            c.shape(|x, y| circle(x, y, 46.0 * k, 52.0 * k, 32.0 * k), hex(0x7BE07B), INK, w);
            for ex in [36.0, 56.0] {
                c.fill(|x, y| circle(x, y, ex * k, 44.0 * k, 4.5 * k), INK);
            }
            c.fill(|x, y| circle(x, y, 46.0 * k, 58.0 * k, 14.0 * k).max(-circle(x, y, 46.0 * k, 50.0 * k, 16.0 * k)), INK);
            // A thumb up.
            c.shape(|x, y| rbox(x, y, 82.0 * k, 60.0 * k, 9.0 * k, 9.0 * k, 4.0 * k).min(capsule(x, y, 80.0 * k, 50.0 * k, 80.0 * k, 34.0 * k, 5.0 * k)), hex(0xF6D2B0), INK, 2.5 * k);
        }
        SlopKind::SixFingerHand => {
            let skin = hex(0xF0C9A0);
            c.shape(
                |x, y| {
                    let mut d = rbox(x, y, 50.0 * k, 64.0 * k, 22.0 * k, 18.0 * k, 9.0 * k);
                    for f in 0..6 {
                        let a = -2.4 + f as f32 * 0.33;
                        let (tx, ty) = (50.0 + a.cos() * 40.0, 58.0 + a.sin() * 40.0);
                        d = d.min(capsule(x, y, 50.0 * k, 56.0 * k, tx * k, ty * k, 5.5 * k));
                    }
                    d
                },
                skin,
                INK,
                w,
            );
        }
    }
    c
}

fn shot(look: Look) -> Canvas {
    match look {
        Look::Rock => {
            let mut c = Canvas::new(40, 40);
            c.shape(|x, y| circle(x, y, 18.0, 21.0, 12.0).min(circle(x, y, 25.0, 17.0, 9.0)), hex(0x9A9A9A), INK, 2.5);
            c
        }
        Look::Bullet => {
            let mut c = Canvas::new(32, 16);
            c.shape(|x, y| capsule(x, y, 6.0, 8.0, 26.0, 8.0, 5.0), hex(0xFFD24A), INK, 2.0);
            c
        }
        Look::Pen => {
            let mut c = Canvas::new(48, 16);
            c.shape(|x, y| capsule(x, y, 6.0, 8.0, 34.0, 8.0, 5.0), hex(0xE23B3B), INK, 2.0);
            c.shape(|x, y| capsule(x, y, 34.0, 8.0, 43.0, 8.0, 2.5), hex(0x3A3D4D), INK, 1.0);
            c
        }
        Look::EyeRoll => {
            let mut c = Canvas::new(44, 44);
            c.shape(|x, y| circle(x, y, 22.0, 22.0, 18.0), [1.0; 4], INK, 2.5);
            c.fill(|x, y| circle(x, y, 22.0, 10.0, 7.0).max(circle(x, y, 22.0, 22.0, 16.0)), hex(0x4A7BD0));
            c
        }
        Look::EmDash => {
            let mut c = Canvas::new(96, 20);
            c.shape(|x, y| capsule(x, y, 8.0, 10.0, 88.0, 10.0, 6.0), hex(0x2A2240), DANGER, 3.0);
            c
        }
        Look::Bubble => bubble(),
    }
}

/// A white speech-bubble frame, outlined in danger pink, drawn as nine slices.
fn bubble() -> Canvas {
    let mut c = Canvas::new(48, 40);
    c.shape(|x, y| rbox(x, y, 24.0, 20.0, 22.0, 18.0, 12.0), [1.0; 4], DANGER, 3.5);
    c
}

fn like() -> Canvas {
    let mut c = Canvas::new(36, 36);
    c.shape(
        |x, y| {
            let lobes = circle(x, y, 12.0, 14.0, 7.5).min(circle(x, y, 24.0, 14.0, 7.5));
            // The point of the heart: a square turned 45 degrees.
            let k = std::f32::consts::FRAC_1_SQRT_2;
            let (u, v) = ((x - 18.0 + y - 19.0) * k, (x - 18.0 - (y - 19.0)) * k);
            lobes.min(rbox(u, v, 0.0, 0.0, 8.5, 8.5, 1.5))
        },
        hex(0xFF4F6D),
        [1.0; 4],
        2.5,
    );
    c
}

fn shadow() -> Canvas {
    let mut c = Canvas::new(64, 24);
    c.fill(|x, y| ellipse(x, y, 32.0, 12.0, 30.0, 10.0), [0.0, 0.0, 0.0, 0.22]);
    c
}

/// The "typing..." indicator that warns of a spawn.
fn typing() -> Canvas {
    let mut c = Canvas::new(64, 40);
    c.shape(|x, y| rbox(x, y, 32.0, 20.0, 28.0, 16.0, 14.0), [0.93, 0.93, 0.95, 0.95], DANGER, 3.0);
    for dx in [20.0, 32.0, 44.0] {
        c.fill(|x, y| circle(x, y, dx, 20.0, 4.5), INK);
    }
    c
}

/// A fading wedge for swings and sprays, pointing right from the left edge's middle.
fn sector(half: f32, color: Rgba) -> Canvas {
    let (w, h) = (256usize, 256usize);
    let mut c = Canvas::new(w, h);
    let (cx, cy, r) = (128.0, 128.0, 126.0);
    for y in 0..h {
        for x in 0..w {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let d = wedge(px, py, cx, cy, r, half);
            let cover = (0.5 - d).clamp(0.0, 1.0);
            if cover > 0.0 {
                let dist = ((px - cx).powi(2) + (py - cy).powi(2)).sqrt() / r;
                c.over(x, y, alpha(color, cover * dist.powf(1.5)));
            }
        }
    }
    c
}

/// One tile of the arena floor: a faint feed of posts nobody asked for.
fn floor() -> Canvas {
    let mut c = Canvas::new(256, 256);
    c.fill(|_, _| -1.0, hex(0xEFECE6));
    let card = [1.0, 1.0, 1.0, 0.4];
    let faint = alpha(hex(0xB9B4AA), 0.3);
    c.shape(|x, y| rbox(x, y, 128.0, 128.0, 104.0, 104.0, 14.0), card, faint, 2.0);
    c.fill(|x, y| circle(x, y, 52.0, 56.0, 14.0), faint);
    c.fill(|x, y| rbox(x, y, 120.0, 50.0, 46.0, 5.0, 5.0), faint);
    c.fill(|x, y| rbox(x, y, 100.0, 66.0, 26.0, 4.0, 4.0), faint);
    for (i, w) in [86.0f32, 70.0, 80.0].iter().enumerate() {
        let y0 = 104.0 + i as f32 * 18.0;
        c.fill(|x, y| rbox(x, y, 48.0 + w, y0, *w, 4.0, 4.0), faint);
    }
    c.fill(|x, y| rbox(x, y, 128.0, 186.0, 82.0, 18.0, 8.0), alpha(hex(0xD8D3C9), 0.5));
    c
}

fn dot() -> Canvas {
    let mut c = Canvas::new(8, 8);
    c.fill(|_, _| -1.0, [1.0; 4]);
    c
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
        let white = c.whitened();
        assert_eq!(white.px[20 * 40 + 20], [1.0, 1.0, 1.0, 1.0]);
    }
}
