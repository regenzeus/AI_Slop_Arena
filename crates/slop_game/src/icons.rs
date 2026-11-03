//! Icons for every weapon, item, stat and damage type, and the shop's
//! buttons (ART-04). Drawn like the characters: thick outlines, flat colours,
//! in the 100-unit design space of `Pen`. The weapon icons double as the
//! weapons you carry in the arena.

use slop_rules::items::StatKind;
use slop_sim::DamageType;

use crate::art::*;

/// Canvas size of an icon in pixels; shown at 24 to 72.
pub const ICON: f32 = 96.0;

const GREY: u32 = 0x9EA3AE;
const DARK: u32 = 0x3A3D4D;
const WOOD: u32 = 0x9A6236;
const RED: u32 = 0xE23B3B;
const GOLD: u32 = 0xF2C230;
const GREEN: u32 = 0x5DBB4F;
const LEAF: u32 = 0x3F9A3A;
const BLUE: u32 = 0x3D7BE0;
const SKY: u32 = 0x8CC8F2;
const PAPER: u32 = 0xF4EEDF;
const SKIN: u32 = 0xF6D2B0;
const PINK: u32 = 0xF6A9C0;
const ORANGE: u32 = 0xF28C38;
const PURPLE: u32 = 0x9B6BFF;

fn pen() -> Pen {
    Pen::new(ICON)
}

fn rock(p: &mut Pen, cx: f32, cy: f32, s: f32) {
    p.blob(&[(cx - 8.0 * s, cy + 4.0 * s, 20.0 * s), (cx + 10.0 * s, cy - 4.0 * s, 16.0 * s), (cx + 6.0 * s, cy + 10.0 * s, 14.0 * s)], hex(GREY));
    p.dot(cx - 10.0 * s, cy - 4.0 * s, 4.0 * s, alpha(WHITE, 0.6));
}

fn book(p: &mut Pen, cx: f32, cy: f32, cover: Rgba) {
    p.rect(cx, cy, 28.0, 36.0, 4.0, cover);
    p.rect(cx + 4.0, cy, 21.0, 30.0, 2.0, hex(PAPER));
    p.rect(cx - 2.0, cy, 21.0, 30.0, 3.0, cover);
    p.rect(cx - 2.0, cy - 12.0, 12.0, 5.0, 2.0, hex(PAPER));
}

fn eye(p: &mut Pen, cx: f32, cy: f32, r: f32) {
    p.circle(cx, cy, r, WHITE);
    p.dot(cx, cy - r * 0.55, r * 0.42, hex(BLUE));
    p.dot(cx, cy - r * 0.6, r * 0.2, INK);
}

fn leaf(p: &mut Pen, cx: f32, cy: f32, s: f32, color: Rgba) {
    p.sdf(|x, y| circle(x, y, cx - 12.0 * s, cy, 22.0 * s).max(circle(x, y, cx + 12.0 * s, cy, 22.0 * s)), color);
    p.line(cx, cy - 16.0 * s, cx, cy + 18.0 * s, 1.5 * s, alpha(INK, 0.5));
}

fn dog_face(p: &mut Pen, cx: f32, cy: f32, s: f32) {
    p.ellipse(cx - 24.0 * s, cy - 6.0 * s, 9.0 * s, 17.0 * s, hex(0x7A4E2D));
    p.ellipse(cx + 24.0 * s, cy - 6.0 * s, 9.0 * s, 17.0 * s, hex(0x7A4E2D));
    p.circle(cx, cy, 26.0 * s, hex(0xC98B54));
    p.ellipse(cx, cy + 13.0 * s, 12.0 * s, 9.0 * s, hex(0xF0D2B0));
    p.dot(cx, cy + 8.0 * s, 4.5 * s, INK);
    p.dot(cx - 10.0 * s, cy - 6.0 * s, 3.5 * s, INK);
    p.dot(cx + 10.0 * s, cy - 6.0 * s, 3.5 * s, INK);
}

fn phone(p: &mut Pen, cx: f32, cy: f32, screen: Rgba) {
    p.rect(cx, cy, 20.0, 36.0, 7.0, hex(DARK));
    p.rect(cx, cy - 2.0, 15.0, 28.0, 3.0, screen);
}

/// Speed lines trailing to the left.
fn speed_lines(p: &mut Pen, x: f32, ys: &[f32]) {
    for (i, y) in ys.iter().enumerate() {
        p.line(x - 14.0 + i as f32 * 4.0, *y, x, *y, 2.5, alpha(INK, 0.6));
    }
}

/// The icon of weapon `index` in `WEAPONS` order.
pub fn weapon(index: usize) -> Canvas {
    let mut p = pen();
    match index {
        // Rock
        0 => rock(&mut p, 50.0, 52.0, 1.3),
        // Stick
        1 => {
            p.stick(22.0, 84.0, 78.0, 16.0, 6.0, hex(WOOD));
            p.stick(56.0, 44.0, 76.0, 46.0, 4.0, hex(WOOD));
            p.ellipse(82.0, 44.0, 9.0, 6.0, hex(GREEN));
        }
        // Slingshot
        2 => {
            p.stick(50.0, 90.0, 50.0, 58.0, 7.0, hex(WOOD));
            p.stick(50.0, 60.0, 26.0, 20.0, 6.0, hex(WOOD));
            p.stick(50.0, 60.0, 74.0, 20.0, 6.0, hex(WOOD));
            p.line(26.0, 22.0, 50.0, 44.0, 2.5, hex(RED));
            p.line(74.0, 22.0, 50.0, 44.0, 2.5, hex(RED));
            p.circle(50.0, 44.0, 8.0, hex(GREY));
        }
        // Brick
        3 => {
            p.rect(50.0, 56.0, 40.0, 24.0, 3.0, hex(0xB5523B));
            p.line(12.0, 56.0, 88.0, 56.0, 2.0, alpha(hex(0xF0D9C8), 0.8));
            for (x, y) in [(36.0, 44.0), (64.0, 44.0), (50.0, 68.0)] {
                p.line(x, y - 9.0, x, y + 9.0, 2.0, alpha(hex(0xF0D9C8), 0.8));
            }
        }
        // Baseball Bat
        4 => {
            p.stick(16.0, 86.0, 44.0, 56.0, 5.0, hex(0xD9A066));
            p.stick(40.0, 60.0, 80.0, 20.0, 12.0, hex(0xD9A066));
            p.circle(14.0, 88.0, 7.0, hex(0xD9A066));
            p.line(22.0, 80.0, 30.0, 72.0, 3.0, hex(RED));
        }
        // Pistol: a toy, with the orange tip.
        5 => {
            p.stick(30.0, 50.0, 22.0, 80.0, 9.0, hex(DARK));
            p.rect(48.0, 40.0, 34.0, 11.0, 4.0, hex(0x5A5F70));
            p.rect(84.0, 40.0, 5.0, 9.0, 2.0, hex(ORANGE));
            p.sdf_fill(|x, y| circle(x, y, 42.0, 56.0, 8.0).abs() - 2.0, hex(DARK));
        }
        // Stapler
        6 => {
            p.rect(50.0, 72.0, 40.0, 8.0, 4.0, hex(DARK));
            p.stick(18.0, 58.0, 84.0, 50.0, 10.0, hex(RED));
            p.circle(18.0, 62.0, 8.0, hex(GREY));
            p.line(70.0, 66.0, 86.0, 66.0, 2.0, hex(GREY));
        }
        // Mechanical Keyboard
        7 => {
            p.rect(50.0, 56.0, 44.0, 26.0, 6.0, hex(DARK));
            for row in 0..3 {
                for col in 0..6 {
                    let (x, y) = (22.0 + col as f32 * 11.2, 42.0 + row as f32 * 13.0);
                    let blue = row == 1 && col == 3;
                    p.rect(x, y, 4.5, 4.5, 1.5, if blue { hex(SKY) } else { hex(0xD8DAE2) });
                }
            }
            p.rect(50.0, 76.0, 18.0, 3.0, 1.5, hex(0xD8DAE2));
        }
        // Red Pen
        8 => {
            p.stick(22.0, 82.0, 70.0, 34.0, 8.0, hex(RED));
            p.tri((66.0, 30.0), (74.0, 38.0), (86.0, 16.0), hex(0xF0E6D8));
            p.stick(30.0, 74.0, 44.0, 60.0, 9.0, hex(0xB82525));
            p.line(14.0, 92.0, 36.0, 88.0, 2.5, hex(RED));
        }
        // Library Book
        9 => book(&mut p, 50.0, 52.0, hex(0x2E5AAC)),
        // Newspaper Roll
        10 => {
            p.stick(22.0, 78.0, 76.0, 24.0, 15.0, hex(0xE8E6E0));
            for t in [0.3f32, 0.5, 0.7] {
                let (x, y) = (22.0 + 54.0 * t, 78.0 - 54.0 * t);
                p.line(x - 8.0, y - 8.0, x + 8.0, y + 8.0, 1.8, alpha(INK, 0.5));
            }
            p.circle(78.0, 22.0, 13.0, hex(0xD0CEC6));
            p.ring(78.0, 22.0, 7.0, 2.0, alpha(INK, 0.5));
        }
        // Fact-Check Flashlight
        11 => {
            p.sdf_fill(|x, y| triangle(x, y, (60.0, 34.0), (98.0, 4.0), (98.0, 64.0)), alpha(hex(0xFFF1A0), 0.8));
            p.stick(14.0, 76.0, 50.0, 46.0, 10.0, hex(DARK));
            p.sdf(|x, y| triangle(x, y, (44.0, 40.0), (66.0, 22.0), (66.0, 58.0)).min(circle(x, y, 64.0, 40.0, 16.0)), hex(0xC9CED8));
            p.dot(66.0, 40.0, 9.0, hex(0xFFF1A0));
        }
        // Eye-Roll
        12 => eye(&mut p, 50.0, 52.0, 36.0),
        // Air Quotes
        13 => {
            for (x, y) in [(18.0, 24.0), (44.0, 24.0), (56.0, 60.0), (82.0, 60.0)] {
                p.sdf(|px, py| circle(px, py, x, y, 13.0).min(triangle(px, py, (x - 1.0, y + 6.0), (x + 12.0, y + 3.0), (x - 12.0, y + 32.0))), hex(GOLD));
            }
        }
        // Slow Clap
        14 => {
            p.stick(30.0, 82.0, 44.0, 30.0, 12.0, hex(SKIN));
            p.stick(70.0, 82.0, 56.0, 30.0, 12.0, hex(SKIN));
            for (a, b) in [((40.0, 14.0), (34.0, 4.0)), ((50.0, 12.0), (50.0, 0.0)), ((60.0, 14.0), (66.0, 4.0))] {
                p.line(a.0, a.1, b.0, b.1, 2.5, alpha(INK, 0.6));
            }
        }
        // Uno Reverse Card
        15 => {
            p.rect(50.0, 50.0, 30.0, 42.0, 7.0, hex(RED));
            p.sdf(|x, y| ellipse(x, y, 50.0, 50.0, 20.0, 30.0), WHITE);
            p.stick(40.0, 62.0, 56.0, 40.0, 5.0, hex(GOLD));
            p.tri((50.0, 34.0), (64.0, 34.0), (60.0, 48.0), hex(GOLD));
            p.stick(60.0, 38.0, 44.0, 60.0, 5.0, hex(BLUE));
            p.tri((50.0, 66.0), (36.0, 66.0), (40.0, 52.0), hex(BLUE));
        }
        // Garden Hose
        16 => {
            p.ring(42.0, 56.0, 26.0, 9.0, hex(GREEN));
            p.ring(42.0, 56.0, 13.0, 7.0, hex(0x4FA544));
            p.stick(60.0, 36.0, 80.0, 22.0, 6.0, hex(GOLD));
            for (x, y) in [(88.0, 12.0), (92.0, 24.0), (84.0, 6.0)] {
                p.dot(x, y, 4.0, hex(SKY));
            }
        }
        // Frisbee
        17 => {
            p.ellipse(50.0, 54.0, 40.0, 26.0, hex(0x6CCB5F));
            p.sdf_fill(|x, y| ellipse(x, y, 50.0, 50.0, 24.0, 14.0).abs() - 2.0, alpha(WHITE, 0.7));
            speed_lines(&mut p, 10.0, &[30.0, 40.0]);
        }
        // Dog Walk
        18 => {
            p.line(12.0, 12.0, 40.0, 50.0, 2.5, hex(RED));
            dog_face(&mut p, 58.0, 60.0, 1.1);
        }
        // Seed Bomb
        19 => {
            p.circle(50.0, 62.0, 26.0, hex(0x8A5A34));
            p.dot(42.0, 56.0, 3.0, alpha(INK, 0.5));
            p.dot(58.0, 70.0, 3.0, alpha(INK, 0.5));
            p.stick(50.0, 38.0, 50.0, 22.0, 3.0, hex(LEAF));
            p.ellipse(38.0, 20.0, 12.0, 7.0, hex(GREEN));
            p.ellipse(62.0, 20.0, 12.0, 7.0, hex(GREEN));
        }
        // Ban Hammer
        20 => {
            p.stick(20.0, 86.0, 58.0, 44.0, 6.0, hex(WOOD));
            p.stick(36.0, 26.0, 80.0, 66.0, 15.0, hex(0x6A7080));
            p.stick(46.0, 34.0, 70.0, 56.0, 15.5, hex(RED));
        }
        // CAPTCHA Turret
        21 => {
            p.rect(50.0, 82.0, 26.0, 8.0, 3.0, hex(DARK));
            p.rect(50.0, 44.0, 32.0, 32.0, 5.0, WHITE);
            p.sdf_fill(|x, y| capsule(x, y, 36.0, 44.0, 46.0, 56.0, 4.5).min(capsule(x, y, 46.0, 56.0, 66.0, 30.0, 4.5)), hex(0x2F80ED));
        }
        _ => return feed_weapon(index),
    }
    p.c
}

/// Weapons added with the Feed (WPN-11), and merged weapons (WPN-12).
fn feed_weapon(index: usize) -> Canvas {
    let def = &slop_sim::content::WEAPONS[index];
    if def.evolved {
        return merged(index);
    }
    let mut p = pen();
    match def.name {
        "Rubber Duck" => {
            p.ellipse(50.0, 64.0, 32.0, 22.0, hex(GOLD));
            p.circle(36.0, 38.0, 17.0, hex(GOLD));
            p.tri((18.0, 38.0), (6.0, 44.0), (18.0, 46.0), hex(ORANGE));
            p.dot(32.0, 34.0, 3.5, INK);
            p.sdf_fill(|x, y| ellipse(x, y, 62.0, 60.0, 14.0, 7.0), alpha(WHITE, 0.4));
        }
        "Office Memo" => {
            p.sdf(|x, y| polygon(x, y, &[(10.0, 60.0), (90.0, 22.0), (48.0, 78.0)]), WHITE);
            p.sdf(|x, y| polygon(x, y, &[(48.0, 78.0), (90.0, 22.0), (56.0, 58.0)]), hex(0xD8DAE2));
            p.line(56.0, 58.0, 44.0, 70.0, 1.5, alpha(INK, 0.6));
        }
        "Sticky Notes" => {
            p.rect(42.0, 46.0, 24.0, 24.0, 2.0, hex(0xF2E66B));
            p.rect(58.0, 58.0, 24.0, 24.0, 2.0, hex(PINK));
            p.line(44.0, 54.0, 70.0, 54.0, 2.0, alpha(INK, 0.5));
            p.line(44.0, 64.0, 64.0, 64.0, 2.0, alpha(INK, 0.5));
        }
        "Side-Eye" => {
            p.ellipse(50.0, 50.0, 38.0, 20.0, WHITE);
            p.circle(76.0, 50.0, 12.0, hex(PURPLE));
            p.dot(80.0, 50.0, 5.0, INK);
            p.stick(16.0, 26.0, 80.0, 22.0, 4.0, hex(DARK));
        }
        "Ratio" => {
            p.rect(50.0, 50.0, 34.0, 26.0, 6.0, hex(DARK));
            p.sdf_fill(|x, y| capsule(x, y, 36.0, 58.0, 50.0, 40.0, 4.0).min(capsule(x, y, 50.0, 40.0, 64.0, 58.0, 4.0)), hex(RED));
            p.line(28.0, 66.0, 72.0, 66.0, 3.0, hex(RED));
        }
        "Sunflower Patch" => {
            p.stick(50.0, 56.0, 50.0, 92.0, 4.0, hex(LEAF));
            leaf(&mut p, 62.0, 76.0, 0.4, hex(GREEN));
            for k in 0..10 {
                let a = k as f32 * std::f32::consts::TAU / 10.0;
                p.ellipse(50.0 + a.cos() * 22.0, 40.0 + a.sin() * 22.0, 9.0, 9.0, hex(GOLD));
            }
            p.circle(50.0, 40.0, 16.0, hex(0x7A4E2D));
        }
        _ => {
            // Leaf Blower
            p.rect(36.0, 58.0, 22.0, 18.0, 6.0, hex(ORANGE));
            p.stick(52.0, 52.0, 90.0, 34.0, 7.0, hex(0x5A5F70));
            p.stick(26.0, 74.0, 30.0, 90.0, 4.0, hex(DARK));
            leaf(&mut p, 84.0, 64.0, 0.35, hex(GREEN));
        }
    }
    p.c
}

/// A merged weapon: its two parents on a golden burst (WPN-12).
fn merged(index: usize) -> Canvas {
    let name = slop_sim::content::WEAPONS[index].name;
    let (a, b) = slop_sim::content::MERGES
        .iter()
        .find(|m| m.2 == name)
        .and_then(|m| Some((slop_sim::content::weapon_index(m.0)?, slop_sim::content::weapon_index(m.1)?)))
        .unwrap_or((0, 1));
    let mut p = pen();
    let points: Vec<(f32, f32)> = (0..16)
        .map(|k| {
            let ang = k as f32 * std::f32::consts::TAU / 16.0;
            let r = if k % 2 == 0 { 48.0 } else { 34.0 };
            (50.0 + ang.cos() * r, 50.0 + ang.sin() * r)
        })
        .collect();
    p.sdf_fill(|x, y| polygon(x, y, &points), alpha(hex(GOLD), 0.55));
    p.sdf_fill(|x, y| circle(x, y, 50.0, 50.0, 30.0), alpha(WHITE, 0.25));
    let size = p.c.w as f32;
    p.c.stamp(&weapon(a), -size * 0.04, -size * 0.04, 0.78);
    p.c.stamp(&weapon(b), size * 0.36, size * 0.36, 0.62);
    p.c
}

/// The icon of item `index` in `ITEMS` order.
pub fn item(index: usize) -> Canvas {
    let mut p = pen();
    match index {
        // Meditation
        0 => {
            p.ring(50.0, 48.0, 40.0, 3.0, alpha(hex(PURPLE), 0.5));
            p.ellipse(50.0, 78.0, 32.0, 10.0, hex(0x6A8BD8));
            p.tri((30.0, 74.0), (70.0, 74.0), (50.0, 36.0), hex(0x6A8BD8));
            p.circle(50.0, 30.0, 13.0, hex(SKIN));
        }
        // Therapy Dog
        1 => {
            dog_face(&mut p, 50.0, 50.0, 1.25);
            p.rect(50.0, 86.0, 14.0, 9.0, 3.0, hex(BLUE));
            p.line(50.0, 80.0, 50.0, 92.0, 2.0, WHITE);
            p.line(44.0, 86.0, 56.0, 86.0, 2.0, WHITE);
        }
        // Houseplant
        2 => {
            p.ellipse(34.0, 34.0, 12.0, 22.0, hex(GREEN));
            p.ellipse(66.0, 34.0, 12.0, 22.0, hex(GREEN));
            p.ellipse(50.0, 26.0, 11.0, 24.0, hex(LEAF));
            p.sdf(|x, y| rbox(x, y, 50.0, 72.0, 22.0, 18.0, 4.0).max(triangle(x, y, (14.0, 54.0), (86.0, 54.0), (50.0, 150.0))), hex(0xC8643C));
            p.rect(50.0, 56.0, 26.0, 5.0, 2.0, hex(0xD9784E));
        }
        // Journal
        3 => {
            p.rect(46.0, 52.0, 30.0, 38.0, 5.0, hex(0x8A5A34));
            p.rect(46.0, 52.0, 6.0, 38.0, 1.0, hex(0x6E4527));
            p.stick(62.0, 14.0, 62.0, 34.0, 4.0, hex(RED));
            p.stick(70.0, 86.0, 88.0, 50.0, 5.0, hex(GOLD));
        }
        // Noise-Cancelling Headphones
        4 => {
            p.sdf(|x, y| (circle(x, y, 50.0, 56.0, 34.0).abs() - 5.0).max(y - 56.0), hex(DARK));
            p.rect(18.0, 64.0, 11.0, 18.0, 6.0, hex(RED));
            p.rect(82.0, 64.0, 11.0, 18.0, 6.0, hex(RED));
        }
        // Ad Blocker
        5 => {
            p.sdf(|x, y| rbox(x, y, 50.0, 36.0, 32.0, 24.0, 6.0).min(triangle(x, y, (18.0, 48.0), (82.0, 48.0), (50.0, 92.0))), hex(0x4FB06A));
            p.rect(50.0, 44.0, 18.0, 11.0, 3.0, WHITE);
            p.line(30.0, 66.0, 70.0, 22.0, 4.0, hex(RED));
        }
        // Weighted Blanket
        6 => {
            for (i, color) in [0x6A8BD8u32, 0xE8835A, 0x6A8BD8].iter().enumerate() {
                p.rect(50.0, 34.0 + i as f32 * 18.0, 38.0, 10.0, 5.0, hex(*color));
            }
            p.line(18.0, 52.0, 82.0, 52.0, 1.5, alpha(WHITE, 0.6));
        }
        // Library Card
        7 => {
            p.rect(50.0, 52.0, 40.0, 26.0, 5.0, hex(SKY));
            p.rect(30.0, 52.0, 11.0, 13.0, 2.0, WHITE);
            p.dot(30.0, 50.0, 5.0, hex(SKIN));
            for y in [44.0, 54.0, 64.0] {
                p.line(48.0, y, 78.0, y, 2.5, alpha(INK, 0.5));
            }
        }
        // Running Shoes
        8 => {
            p.sdf(|x, y| ellipse(x, y, 52.0, 56.0, 30.0, 18.0).min(rbox(x, y, 60.0, 66.0, 30.0, 8.0, 6.0)), hex(BLUE));
            p.rect(60.0, 74.0, 32.0, 5.0, 3.0, WHITE);
            p.line(44.0, 48.0, 56.0, 46.0, 2.0, WHITE);
            speed_lines(&mut p, 20.0, &[46.0, 58.0, 70.0]);
        }
        // Good Night's Sleep
        9 => {
            p.sdf(|x, y| circle(x, y, 44.0, 54.0, 30.0).max(-circle(x, y, 60.0, 42.0, 26.0)), hex(GOLD));
            for (x, y, s) in [(70.0, 30.0, 10.0), (82.0, 14.0, 7.0)] {
                p.line(x - s, y - s, x + s, y - s, 2.5, hex(PURPLE));
                p.line(x + s, y - s, x - s, y + s, 2.5, hex(PURPLE));
                p.line(x - s, y + s, x + s, y + s, 2.5, hex(PURPLE));
            }
        }
        // Group Chat
        10 => {
            p.sdf(|x, y| rbox(x, y, 40.0, 38.0, 28.0, 18.0, 9.0).min(triangle(x, y, (22.0, 50.0), (36.0, 50.0), (20.0, 64.0))), hex(GREEN));
            p.sdf(|x, y| rbox(x, y, 60.0, 66.0, 28.0, 18.0, 9.0).min(triangle(x, y, (64.0, 78.0), (78.0, 78.0), (80.0, 92.0))), hex(SKY));
            for x in [50.0, 60.0, 70.0] {
                p.dot(x, 66.0, 3.0, INK);
            }
        }
        // Touch-Grass Badge
        11 => {
            p.tri((34.0, 66.0), (48.0, 66.0), (30.0, 96.0), hex(RED));
            p.tri((52.0, 66.0), (66.0, 66.0), (70.0, 96.0), hex(RED));
            p.circle(50.0, 44.0, 32.0, hex(GOLD));
            p.dot(50.0, 44.0, 24.0, hex(0xF7D35A));
            for x in [40.0f32, 50.0, 60.0] {
                p.sdf_fill(|px, py| triangle(px, py, (x - 5.0, 56.0), (x + 5.0, 56.0), (x + 1.0, 30.0)), hex(LEAF));
            }
        }
        // Self-Help Book
        12 => {
            book(&mut p, 50.0, 52.0, hex(GOLD));
            p.dot(42.0, 50.0, 3.0, INK);
            p.dot(54.0, 50.0, 3.0, INK);
            p.smile(48.0, 58.0, 8.0, INK);
        }
        // Coffee
        13 => {
            p.sdf(|x, y| circle(x, y, 72.0, 62.0, 12.0).abs() - 4.0, hex(RED));
            p.rect(46.0, 62.0, 26.0, 26.0, 7.0, hex(RED));
            p.rect(46.0, 42.0, 21.0, 5.0, 2.0, hex(0x6E4527));
            p.rect(46.0, 66.0, 16.0, 4.0, 2.0, alpha(WHITE, 0.8));
            for x in [36.0, 50.0] {
                p.line(x, 30.0, x + 4.0, 20.0, 2.5, alpha(INK, 0.4));
                p.line(x + 4.0, 20.0, x, 10.0, 2.5, alpha(INK, 0.4));
            }
        }
        // Energy Drink
        14 => {
            p.rect(50.0, 54.0, 22.0, 38.0, 6.0, hex(0x7BE34A));
            p.rect(50.0, 18.0, 16.0, 4.0, 2.0, hex(GREY));
            p.sdf(|x, y| triangle(x, y, (54.0, 28.0), (38.0, 58.0), (52.0, 58.0)).min(triangle(x, y, (48.0, 52.0), (62.0, 52.0), (46.0, 82.0))), hex(GOLD));
        }
        // Doomscrolling
        15 => {
            phone(&mut p, 50.0, 50.0, hex(0xE6ECF5));
            for y in [32.0, 42.0, 52.0, 62.0] {
                p.line(40.0, y, 60.0, y, 2.5, alpha(INK, 0.4));
            }
            p.tri((76.0, 66.0), (92.0, 66.0), (84.0, 82.0), hex(RED));
            p.stick(84.0, 40.0, 84.0, 66.0, 3.0, hex(RED));
        }
        // Hot Take
        16 => {
            p.sdf(|x, y| circle(x, y, 50.0, 64.0, 26.0).min(triangle(x, y, (26.0, 58.0), (74.0, 58.0), (54.0, 6.0))), hex(ORANGE));
            p.sdf_fill(|x, y| circle(x, y, 50.0, 70.0, 14.0).min(triangle(x, y, (37.0, 66.0), (63.0, 66.0), (52.0, 34.0))), hex(GOLD));
        }
        // Dumbphone
        17 => {
            p.rect(50.0, 52.0, 20.0, 38.0, 6.0, hex(0x7A8090));
            p.rect(50.0, 30.0, 13.0, 10.0, 2.0, hex(0xB6E28A));
            for row in 0..3 {
                for col in 0..3 {
                    p.dot(40.0 + col as f32 * 10.0, 54.0 + row as f32 * 10.0, 3.0, hex(0xE8E6E0));
                }
            }
        }
        // Burner Account
        18 => {
            p.ellipse(50.0, 38.0, 40.0, 7.0, hex(DARK));
            p.rect(50.0, 26.0, 22.0, 14.0, 6.0, hex(DARK));
            p.ring(34.0, 62.0, 11.0, 4.0, INK);
            p.ring(66.0, 62.0, 11.0, 4.0, INK);
            p.line(45.0, 62.0, 55.0, 62.0, 2.0, INK);
        }
        // Clickbait Thumbnail
        19 => {
            p.rect(50.0, 52.0, 42.0, 30.0, 5.0, hex(GOLD));
            p.circle(36.0, 52.0, 16.0, hex(SKIN));
            p.dot(31.0, 48.0, 2.5, INK);
            p.dot(41.0, 48.0, 2.5, INK);
            p.circle(36.0, 59.0, 4.0, INK);
            p.sdf(|x, y| capsule(x, y, 58.0, 70.0, 74.0, 46.0, 4.0).min(triangle(x, y, (66.0, 40.0), (84.0, 36.0), (80.0, 54.0))), hex(RED));
        }
        // Side Hustle
        20 => {
            p.sdf_fill(|x, y| rbox(x, y, 50.0, 28.0, 14.0, 10.0, 4.0).abs() - 3.0, hex(0x6E4527));
            p.rect(50.0, 58.0, 38.0, 26.0, 5.0, hex(0x8A5A34));
            p.rect(50.0, 54.0, 6.0, 5.0, 1.5, hex(GOLD));
        }
        // Savings Account
        21 => {
            p.ellipse(50.0, 58.0, 36.0, 26.0, hex(PINK));
            p.tri((30.0, 38.0), (42.0, 36.0), (32.0, 24.0), hex(PINK));
            p.rect(84.0, 56.0, 7.0, 9.0, 3.0, hex(PINK));
            p.rect(36.0, 84.0, 6.0, 8.0, 2.0, hex(PINK));
            p.rect(62.0, 84.0, 6.0, 8.0, 2.0, hex(PINK));
            p.dot(68.0, 50.0, 3.0, INK);
            p.rect(48.0, 36.0, 10.0, 2.5, 1.0, INK);
            p.circle(50.0, 18.0, 9.0, hex(GOLD));
        }
        // Referral Link
        22 => {
            p.sdf(|x, y| capsule(x, y, 24.0, 64.0, 46.0, 42.0, 13.0).abs() - 4.0, hex(GREY));
            p.sdf(|x, y| capsule(x, y, 54.0, 58.0, 76.0, 36.0, 13.0).abs() - 4.0, hex(GREY));
        }
        // Coupon Clipper
        23 => {
            p.rect(50.0, 50.0, 40.0, 24.0, 4.0, hex(0xF7D35A));
            p.sdf_fill(|x, y| rbox(x, y, 50.0, 50.0, 34.0, 18.0, 2.0).abs() - 1.0, alpha(INK, 0.5));
            p.dot(40.0, 42.0, 5.0, INK);
            p.dot(60.0, 58.0, 5.0, INK);
            p.line(62.0, 38.0, 38.0, 62.0, 3.0, INK);
        }
        // Garage Sale
        24 => {
            p.sdf(|x, y| rbox(x, y, 56.0, 56.0, 28.0, 22.0, 4.0).min(triangle(x, y, (30.0, 34.0), (30.0, 78.0), (8.0, 56.0))), hex(0xF7D35A));
            p.dot(24.0, 56.0, 4.0, WHITE);
            p.line(24.0, 56.0, 10.0, 20.0, 2.0, INK);
            p.line(46.0, 50.0, 70.0, 50.0, 3.0, alpha(INK, 0.6));
            p.line(46.0, 62.0, 64.0, 62.0, 3.0, alpha(INK, 0.6));
        }
        // Influencer Deal
        25 => {
            p.ring(50.0, 46.0, 34.0, 10.0, hex(0xFFF1A0));
            p.sdf(|x, y| circle(x, y, 50.0, 46.0, 39.0).max(-circle(x, y, 50.0, 46.0, 29.0)), hex(0xFFF1A0));
            phone(&mut p, 50.0, 50.0, hex(PINK));
            p.stick(50.0, 86.0, 50.0, 96.0, 3.0, hex(DARK));
        }
        // Crypto Wallet
        26 => {
            p.circle(50.0, 52.0, 34.0, hex(ORANGE));
            p.dot(50.0, 52.0, 26.0, hex(0xF7A94A));
            p.sdf(|x, y| capsule(x, y, 36.0, 66.0, 48.0, 50.0, 3.5).min(capsule(x, y, 48.0, 50.0, 56.0, 58.0, 3.5)).min(capsule(x, y, 56.0, 58.0, 68.0, 36.0, 3.5)), WHITE);
            p.tri((62.0, 32.0), (74.0, 30.0), (70.0, 42.0), WHITE);
        }
        // Log Off
        27 => {
            p.sdf(|x, y| (circle(x, y, 50.0, 54.0, 30.0).abs() - 5.0).max(-triangle(x, y, (50.0, 52.0), (30.0, 10.0), (70.0, 10.0))), hex(RED));
            p.stick(50.0, 18.0, 50.0, 50.0, 5.0, hex(RED));
        }
        // Block Button
        28 => {
            p.sdf(|x, y| circle(x, y, 50.0, 50.0, 36.0).abs() - 6.0, hex(RED));
            p.sdf(|x, y| capsule(x, y, 28.0, 28.0, 72.0, 72.0, 6.0).max(circle(x, y, 50.0, 50.0, 32.0)), hex(RED));
        }
        // Digital Detox
        29 => {
            phone(&mut p, 50.0, 56.0, hex(0x40445A));
            p.line(42.0, 46.0, 58.0, 62.0, 3.0, hex(RED));
            p.line(58.0, 46.0, 42.0, 62.0, 3.0, hex(RED));
            p.ellipse(70.0, 20.0, 14.0, 8.0, hex(GREEN));
            p.ellipse(52.0, 14.0, 10.0, 6.0, hex(LEAF));
        }
        // Unsubscribe
        30 => {
            p.rect(46.0, 54.0, 34.0, 24.0, 4.0, WHITE);
            p.sdf_fill(|x, y| capsule(x, y, 14.0, 34.0, 46.0, 58.0, 2.0).min(capsule(x, y, 46.0, 58.0, 78.0, 34.0, 2.0)), INK);
            p.circle(78.0, 76.0, 15.0, hex(RED));
            p.line(72.0, 70.0, 84.0, 82.0, 3.0, WHITE);
            p.line(84.0, 70.0, 72.0, 82.0, 3.0, WHITE);
        }
        // Phone a Friend
        _ => {
            p.sdf(
                |x, y| {
                    let band = (circle(x, y, 74.0, 26.0, 50.0).abs() - 8.0).max(x - 74.0).max(26.0 - y);
                    band.min(rbox(x, y, 30.0, 30.0, 14.0, 10.0, 6.0)).min(rbox(x, y, 70.0, 74.0, 10.0, 14.0, 6.0))
                },
                hex(GREEN),
            );
            for r in [10.0, 18.0] {
                p.sdf_fill(|x, y| (circle(x, y, 74.0, 26.0, r).abs() - 1.5).max(-(x - 74.0)).max(y - 26.0), alpha(INK, 0.6));
            }
        }
    }
    p.c
}

/// The icon of a stat (UI-13).
pub fn stat(kind: StatKind) -> Canvas {
    let mut p = pen();
    match kind {
        StatKind::MentalHealth => {
            p.blob(&[(36.0, 40.0, 22.0), (64.0, 40.0, 22.0), (30.0, 60.0, 19.0), (70.0, 60.0, 19.0), (50.0, 64.0, 22.0)], hex(PINK));
            p.line(50.0, 24.0, 50.0, 80.0, 2.0, alpha(INK, 0.6));
            p.line(30.0, 46.0, 40.0, 52.0, 2.0, alpha(INK, 0.4));
            p.line(70.0, 46.0, 60.0, 52.0, 2.0, alpha(INK, 0.4));
        }
        StatKind::Composure => {
            p.sdf(|x, y| rbox(x, y, 50.0, 36.0, 32.0, 24.0, 6.0).min(triangle(x, y, (18.0, 48.0), (82.0, 48.0), (50.0, 94.0))), hex(BLUE));
            p.sdf_fill(|x, y| rbox(x, y, 50.0, 36.0, 20.0, 14.0, 4.0).min(triangle(x, y, (30.0, 46.0), (70.0, 46.0), (50.0, 78.0))), alpha(WHITE, 0.35));
        }
        StatKind::SelfCare => {
            p.stick(22.0, 74.0, 78.0, 26.0, 16.0, hex(0xE9C29A));
            p.rect(50.0, 50.0, 11.0, 11.0, 3.0, hex(0xF6DCC0));
            for (x, y) in [(46.0, 46.0), (54.0, 54.0), (46.0, 54.0), (54.0, 46.0)] {
                p.dot(x, y, 1.8, alpha(INK, 0.5));
            }
        }
        StatKind::Focus => {
            p.rect(50.0, 14.0, 8.0, 6.0, 2.0, hex(GREY));
            p.circle(50.0, 56.0, 36.0, WHITE);
            p.stick(50.0, 56.0, 50.0, 32.0, 3.5, hex(RED));
            p.stick(50.0, 56.0, 66.0, 62.0, 3.0, INK);
            p.dot(50.0, 56.0, 4.0, INK);
        }
        StatKind::Conviction => {
            p.sdf(|x, y| triangle(x, y, (28.0, 50.0), (80.0, 20.0), (80.0, 80.0)).min(rbox(x, y, 26.0, 50.0, 10.0, 12.0, 3.0)), hex(RED));
            p.rect(80.0, 50.0, 6.0, 30.0, 3.0, hex(DARK));
            p.stick(36.0, 62.0, 42.0, 84.0, 6.0, hex(DARK));
        }
        StatKind::Reach => {
            p.circle(72.0, 50.0, 22.0, WHITE);
            p.ring(72.0, 50.0, 12.0, 6.0, hex(RED));
            p.dot(72.0, 50.0, 4.0, hex(RED));
            p.stick(10.0, 50.0, 62.0, 50.0, 4.0, hex(WOOD));
            p.tri((54.0, 40.0), (54.0, 60.0), (70.0, 50.0), hex(GREY));
        }
        StatKind::Speed => {
            p.sdf(|x, y| ellipse(x, y, 56.0, 52.0, 28.0, 18.0).min(rbox(x, y, 62.0, 62.0, 28.0, 8.0, 6.0)), hex(ORANGE));
            p.rect(62.0, 70.0, 30.0, 5.0, 3.0, WHITE);
            speed_lines(&mut p, 24.0, &[42.0, 54.0, 66.0]);
        }
        StatKind::Scepticism => {
            p.stick(60.0, 60.0, 86.0, 86.0, 7.0, hex(DARK));
            p.circle(42.0, 42.0, 28.0, hex(SKY));
            p.ring(42.0, 42.0, 28.0, 5.0, hex(GREY));
            p.sdf_fill(|x, y| (circle(x, y, 42.0, 36.0, 9.0).abs() - 2.5).max(-(x - 40.0).max(y - 38.0)), INK);
            p.dot(42.0, 56.0, 3.0, INK);
            p.line(42.0, 45.0, 42.0, 49.0, 2.5, INK);
        }
        StatKind::Curiosity => {
            p.sdf(|x, y| (circle(x, y, 50.0, 44.0, 26.0).abs() - 10.0).max(y - 44.0).min(rbox(x, y, 26.0, 62.0, 10.0, 18.0, 1.0)).min(rbox(x, y, 74.0, 62.0, 10.0, 18.0, 1.0)), hex(RED));
            p.rect(26.0, 76.0, 10.0, 6.0, 1.0, hex(0xD8DAE2));
            p.rect(74.0, 76.0, 10.0, 6.0, 1.0, hex(0xD8DAE2));
        }
        StatKind::Luck => {
            p.stick(50.0, 54.0, 62.0, 92.0, 3.5, hex(LEAF));
            p.blob(&[(38.0, 38.0, 15.0), (62.0, 38.0, 15.0), (38.0, 62.0, 15.0), (62.0, 62.0, 15.0)], hex(GREEN));
            p.line(50.0, 26.0, 50.0, 74.0, 1.5, alpha(INK, 0.4));
            p.line(26.0, 50.0, 74.0, 50.0, 1.5, alpha(INK, 0.4));
        }
    }
    p.c
}

/// A small badge for a damage type, in its colour.
pub fn damage_type(t: DamageType) -> Canvas {
    let mut p = pen();
    match t {
        DamageType::Analog => rock(&mut p, 50.0, 52.0, 1.3),
        DamageType::Fact => book(&mut p, 50.0, 52.0, hex(0x2E5AAC)),
        DamageType::Sarcasm => eye(&mut p, 50.0, 52.0, 36.0),
        DamageType::Grass => {
            leaf(&mut p, 40.0, 54.0, 1.6, hex(GREEN));
            leaf(&mut p, 64.0, 46.0, 1.2, hex(LEAF));
        }
    }
    p.c
}

/// Two arrows chasing each other.
pub fn reroll() -> Canvas {
    let mut p = pen();
    p.sdf(|x, y| (circle(x, y, 50.0, 50.0, 30.0).abs() - 6.0).max(-rbox(x, y, 50.0, 50.0, 60.0, 6.0, 0.0)), hex(SKY));
    p.tri((70.0, 44.0), (94.0, 44.0), (82.0, 62.0), hex(SKY));
    p.tri((30.0, 56.0), (6.0, 56.0), (18.0, 38.0), hex(SKY));
    p.c
}

/// A level-up: a five-pointed star.
pub fn star() -> Canvas {
    let mut p = pen();
    let corners: Vec<(f32, f32)> = (0..10)
        .map(|k| {
            let a = std::f32::consts::FRAC_PI_2 + k as f32 * std::f32::consts::TAU / 10.0;
            let r = if k % 2 == 0 { 46.0 } else { 19.0 };
            (50.0 + r * a.cos(), 54.0 - r * a.sin())
        })
        .collect();
    p.sdf(|x, y| polygon(x, y, &corners), hex(GOLD));
    p.c
}

#[cfg(test)]
mod tests {
    use super::*;
    use slop_rules::items::ITEMS;
    use slop_sim::content::WEAPONS;

    // REQ: ART-04
    #[test]
    fn every_weapon_item_and_stat_has_a_distinct_icon() {
        let opaque = |c: &Canvas| (0..c.w * c.h).filter(|i| c.alpha_at(*i) > 0.5).count();
        let mut seen = Vec::new();
        for (i, w) in WEAPONS.iter().enumerate() {
            let c = weapon(i);
            assert!(opaque(&c) > 600, "weapon {}", w.name);
            seen.push(c.fingerprint());
        }
        for (i, it) in ITEMS.iter().enumerate() {
            let c = item(i);
            assert!(opaque(&c) > 600, "item {}", it.name);
            seen.push(c.fingerprint());
        }
        for k in StatKind::ALL {
            assert!(opaque(&stat(k)) > 600, "{k:?}");
        }
        let n = seen.len();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), n, "two icons are identical");
    }
}

/// Writes every icon on one sheet, for looking at them:
/// `cargo test -p slop_game icon_sheet -- --ignored`.
#[cfg(test)]
#[test]
#[ignore]
fn icon_sheet() {
    use slop_rules::items::ITEMS;
    use slop_sim::content::WEAPONS;
    let mut all: Vec<Canvas> = (0..WEAPONS.len()).map(weapon).collect();
    all.extend((0..ITEMS.len()).map(item));
    all.extend(StatKind::ALL.iter().map(|k| stat(*k)));
    all.extend(DamageType::ALL.iter().map(|t| damage_type(*t)));
    all.extend([reroll(), star()]);
    let s = ICON as u32;
    let cols = 12;
    let rows = all.len() as u32 / cols + 1;
    let mut sheet = image::RgbaImage::from_pixel(cols * s, rows * s, image::Rgba([235, 232, 226, 255]));
    for (n, c) in all.iter().enumerate() {
        let (ox, oy) = ((n as u32 % cols) * s, (n as u32 / cols) * s);
        for y in 0..c.h {
            for x in 0..c.w {
                let p = c.pixel(x, y);
                let a = p[3];
                let bg = sheet.get_pixel(ox + x as u32, oy + y as u32).0;
                let mix = |k: usize| ((p[k] * a + bg[k] as f32 / 255.0 * (1.0 - a)) * 255.0) as u8;
                sheet.put_pixel(ox + x as u32, oy + y as u32, image::Rgba([mix(0), mix(1), mix(2), 255]));
            }
        }
    }
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/icon_sheet.png");
    sheet.save(&out).unwrap();
    println!("wrote {}", out.display());
}
