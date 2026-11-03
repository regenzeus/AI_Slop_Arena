//! The Feed's pictures (ART-06): pickups, the things on the map, walls and
//! fences, and the scraps of feed that litter every floor. Drawn like
//! everything else, in the 100-unit design space of `Pen`.

use slop_sim::map::{PropKind, WallKind};
use slop_sim::world::PickupKind;

use crate::art::*;

const GOLD: u32 = 0xF2C230;
const GREEN: u32 = 0x5DBB4F;
const RED: u32 = 0xE23B3B;
const SKY: u32 = 0x8CC8F2;
const BLUE: u32 = 0x3D7BE0;
const DARK: u32 = 0x3A3D4D;
const PURPLE: u32 = 0x9B6BFF;
const ORANGE: u32 = 0xF28C38;

/// Pickups on the floor, 64 pixels.
pub fn pickup(kind: PickupKind) -> Canvas {
    let mut p = Pen::new(64.0);
    match kind {
        // Touch Grass: a tuft in a pot.
        PickupKind::Food => {
            p.rect(50.0, 74.0, 22.0, 16.0, 5.0, hex(0xC0714A));
            for (x, h) in [(34.0, 30.0), (44.0, 20.0), (56.0, 24.0), (66.0, 34.0)] {
                p.sdf(|px, py| capsule(px, py, x, 60.0, x + (x - 50.0) * 0.3, h, 6.0), hex(GREEN));
            }
        }
        // Algorithm Boost: a horseshoe magnet.
        PickupKind::Magnet => {
            p.sdf(|x, y| (circle(x, y, 50.0, 46.0, 30.0).abs() - 10.0).max(y - 52.0).min(rbox(x, y, 30.0, 66.0, 10.0, 16.0, 1.0)).min(rbox(x, y, 70.0, 66.0, 10.0, 16.0, 1.0)), hex(RED));
            p.rect(30.0, 80.0, 10.0, 6.0, 1.0, hex(0xD8DAE2));
            p.rect(70.0, 80.0, 10.0, 6.0, 1.0, hex(0xD8DAE2));
        }
        // Mass Report: a red flag.
        PickupKind::Bomb => {
            p.stick(30.0, 14.0, 30.0, 90.0, 4.0, hex(DARK));
            p.sdf(|x, y| polygon(x, y, &[(32.0, 16.0), (82.0, 26.0), (64.0, 40.0), (82.0, 54.0), (32.0, 54.0)]), hex(RED));
            p.dot(52.0, 35.0, 5.0, WHITE);
        }
        // Server Outage: a snowflake on a blue server light.
        PickupKind::Freeze => {
            p.circle(50.0, 50.0, 36.0, hex(SKY));
            for a in 0..3 {
                let t = a as f32 * std::f32::consts::PI / 3.0;
                let (dx, dy) = (t.cos() * 28.0, t.sin() * 28.0);
                p.line(50.0 - dx, 50.0 - dy, 50.0 + dx, 50.0 + dy, 4.0, WHITE);
            }
        }
        PickupKind::Clout => {
            p.circle(50.0, 50.0, 40.0, hex(GOLD));
            p.dot(50.0, 50.0, 30.0, hex(0xF7D35A));
            p.stick(46.0, 62.0, 46.0, 40.0, 7.0, WHITE);
            p.rect(58.0, 58.0, 9.0, 9.0, 4.0, WHITE);
        }
        // A chest: a gift box with a ribbon.
        PickupKind::Chest => {
            p.rect(50.0, 62.0, 36.0, 28.0, 4.0, hex(PURPLE));
            p.rect(50.0, 36.0, 40.0, 10.0, 3.0, hex(0xB28CFF));
            p.rect(50.0, 58.0, 7.0, 32.0, 1.0, hex(GOLD));
            p.blob(&[(40.0, 22.0, 9.0), (60.0, 22.0, 9.0)], hex(GOLD));
        }
    }
    p.c
}

/// A Like: experience, a blue thumbs-up token. Clout is the gold coin.
pub fn like() -> Canvas {
    let mut p = Pen::new(40.0);
    p.circle(50.0, 50.0, 40.0, hex(BLUE));
    p.stick(42.0, 64.0, 42.0, 40.0, 8.0, WHITE);
    p.rect(58.0, 60.0, 12.0, 12.0, 5.0, WHITE);
    p.stick(56.0, 48.0, 62.0, 26.0, 6.0, WHITE);
    p.c
}

/// Things on the map, 128 pixels; rings are drawn by the view.
pub fn prop(kind: PropKind) -> Canvas {
    let mut p = Pen::new(128.0);
    match kind {
        PropKind::AdBillboard => {
            p.stick(36.0, 60.0, 36.0, 94.0, 4.0, hex(DARK));
            p.stick(64.0, 60.0, 64.0, 94.0, 4.0, hex(DARK));
            p.rect(50.0, 40.0, 44.0, 28.0, 3.0, hex(0xFFE36E));
            p.rect(50.0, 30.0, 34.0, 6.0, 2.0, hex(RED));
            p.rect(40.0, 50.0, 22.0, 4.0, 2.0, alpha(INK, 0.6));
            p.circle(78.0, 50.0, 8.0, hex(RED));
        }
        PropKind::PopupAd => {
            p.rect(50.0, 52.0, 38.0, 30.0, 4.0, WHITE);
            p.rect(50.0, 28.0, 38.0, 7.0, 3.0, hex(BLUE));
            p.dot(80.0, 28.0, 4.0, hex(RED));
            p.rect(50.0, 52.0, 24.0, 5.0, 2.0, hex(GOLD));
            p.rect(50.0, 68.0, 16.0, 5.0, 2.0, hex(GREEN));
        }
        PropKind::CryptoMiner => {
            p.rect(50.0, 56.0, 34.0, 30.0, 5.0, hex(DARK));
            for k in 0..3 {
                p.dot(30.0 + k as f32 * 20.0, 74.0, 4.0, hex(GREEN));
            }
            p.circle(50.0, 46.0, 16.0, hex(ORANGE));
            p.line(44.0, 40.0, 44.0, 52.0, 3.0, WHITE);
            p.line(44.0, 40.0, 56.0, 40.0, 3.0, WHITE);
            p.line(44.0, 46.0, 54.0, 46.0, 3.0, WHITE);
            p.line(44.0, 52.0, 56.0, 52.0, 3.0, WHITE);
        }
        PropKind::Hotspot => {
            p.stick(50.0, 60.0, 50.0, 92.0, 5.0, hex(DARK));
            p.circle(50.0, 58.0, 9.0, hex(SKY));
            for r in [20.0, 32.0, 44.0] {
                p.sdf(|x, y| (circle(x, y, 50.0, 58.0, r).abs() - 3.5).max(-(58.0 - y) + (x - 50.0).abs() * 0.9), hex(SKY));
            }
        }
        PropKind::Vending => {
            p.rect(50.0, 52.0, 30.0, 44.0, 4.0, hex(RED));
            p.rect(44.0, 44.0, 18.0, 28.0, 2.0, hex(SKY));
            for (x, y) in [(36.0, 28.0), (50.0, 28.0), (36.0, 44.0), (50.0, 44.0), (36.0, 60.0), (50.0, 60.0)] {
                p.dot(x, y, 4.0, hex([GOLD, GREEN, ORANGE][((x + y) as usize / 14) % 3]));
            }
            p.rect(72.0, 40.0, 4.0, 10.0, 1.0, hex(DARK));
            p.rect(50.0, 86.0, 18.0, 4.0, 1.0, hex(DARK));
        }
        PropKind::Gacha => {
            p.rect(50.0, 76.0, 24.0, 18.0, 4.0, hex(PURPLE));
            p.circle(50.0, 40.0, 28.0, alpha(hex(SKY), 0.7));
            for (x, y, c) in [(40.0, 46.0, RED), (56.0, 50.0, GOLD), (48.0, 30.0, GREEN), (62.0, 34.0, BLUE)] {
                p.circle(x, y, 7.0, hex(c));
            }
            p.circle(64.0, 80.0, 5.0, hex(GOLD));
        }
        PropKind::Shrine => {
            p.rect(50.0, 84.0, 34.0, 8.0, 2.0, hex(0x8E8E9E));
            p.rect(50.0, 50.0, 22.0, 30.0, 3.0, hex(0xB8BCC8));
            p.sdf(|x, y| polygon(x, y, &[(50.0, 34.0), (58.0, 50.0), (50.0, 66.0), (42.0, 50.0)]), hex(GOLD));
            p.dot(50.0, 50.0, 4.0, WHITE);
            p.ring(50.0, 50.0, 30.0, 3.0, alpha(hex(GOLD), 0.6));
        }
        PropKind::DataCenter => {
            for (k, x) in [30.0, 50.0, 70.0].iter().enumerate() {
                p.rect(*x, 54.0, 9.0, 34.0, 2.0, hex(DARK));
                for j in 0..4 {
                    p.dot(*x, 32.0 + j as f32 * 14.0, 2.5, hex(if (k + j) % 2 == 0 { GREEN } else { SKY }));
                }
            }
        }
        PropKind::CommentSection => {
            p.rect(50.0, 50.0, 38.0, 38.0, 6.0, hex(0x2A2433));
            for (k, w) in [26.0, 20.0, 30.0].iter().enumerate() {
                let y = 30.0 + k as f32 * 18.0;
                p.dot(24.0, y, 5.0, hex(DANGER_HEX));
                p.rect(34.0 + w, y, *w, 4.0, 2.0, alpha(WHITE, 0.8));
            }
            p.sdf(|x, y| polygon(x, y, &[(70.0, 76.0), (86.0, 76.0), (78.0, 90.0)]), hex(0x2A2433));
        }
        PropKind::LostPhone => {
            p.rect(50.0, 50.0, 18.0, 32.0, 6.0, hex(DARK));
            p.rect(50.0, 48.0, 14.0, 24.0, 2.0, hex(SKY));
            p.line(42.0, 36.0, 58.0, 58.0, 1.5, alpha(WHITE, 0.8));
            p.dot(60.0, 30.0, 5.0, hex(RED));
        }
    }
    p.c
}

const DANGER_HEX: u32 = 0xFF2E94;

/// A wall's texture, stretched over its rectangle.
pub fn wall(kind: WallKind) -> Canvas {
    let mut c = Canvas::new(64, 64);
    match kind {
        WallKind::ServerRack => {
            c.shape(|x, y| rbox(x, y, 32.0, 32.0, 29.0, 29.0, 4.0), hex(0x1E2230), INK, 3.0);
            for k in 0..5 {
                let y0 = 12.0 + k as f32 * 10.0;
                c.fill(|x, y| rbox(x, y, 28.0, y0, 18.0, 3.0, 1.0), hex(0x323848));
                c.fill(|x, y| circle(x, y, 52.0, y0, 2.2), hex(if k % 2 == 0 { GREEN } else { SKY }));
            }
        }
        WallKind::Billboard => {
            c.shape(|x, y| rbox(x, y, 32.0, 32.0, 30.0, 30.0, 3.0), hex(0x3B3550), INK, 3.0);
            c.fill(|x, y| rbox(x, y, 32.0, 32.0, 22.0, 6.0, 3.0), alpha(hex(GOLD), 0.7));
        }
        WallKind::Desk => {
            c.shape(|x, y| rbox(x, y, 32.0, 32.0, 29.0, 29.0, 5.0), hex(0x7A5636), INK, 3.0);
            c.fill(|x, y| rbox(x, y, 22.0, 26.0, 9.0, 7.0, 1.5), hex(0x2C3040));
            c.fill(|x, y| rbox(x, y, 22.0, 26.0, 7.0, 5.0, 1.0), hex(SKY));
            c.fill(|x, y| circle(x, y, 46.0, 40.0, 4.5), WHITE);
        }
        WallKind::Fence => {
            // Hazard stripes, tiled by the view.
            c.paint(|x, y| Some(if ((x + y) / 16.0) as i32 % 2 == 0 { hex(0xF2C230) } else { hex(0x2A2433) }));
        }
    }
    c
}

/// Faint scraps of feed scattered on the floor: posts, thumbnails, comments,
/// hearts, play buttons and hashtags (ART-06). `i` picks one of `DECOR`.
pub const DECOR: usize = 10;

pub fn decor(i: usize) -> Canvas {
    let mut c = Canvas::new(128, 128);
    let ghost = |h: u32| alpha(hex(h), 0.16);
    let line = |h: u32| alpha(hex(h), 0.28);
    match i % DECOR {
        // A post card with an avatar and lines of text.
        0 => {
            c.fill(|x, y| rbox(x, y, 64.0, 64.0, 56.0, 40.0, 8.0), ghost(0xFFFFFF));
            c.fill(|x, y| circle(x, y, 26.0, 40.0, 9.0), line(0xFFFFFF));
            for k in 0..3 {
                let w = [40.0, 30.0, 36.0][k];
                c.fill(|x, y| rbox(x, y, 44.0 + w, 62.0 + k as f32 * 12.0, w, 3.0, 3.0), line(0xFFFFFF));
            }
        }
        // A video thumbnail with a play button.
        1 => {
            c.fill(|x, y| rbox(x, y, 64.0, 64.0, 56.0, 34.0, 6.0), ghost(0xE23B3B));
            c.fill(|x, y| triangle(x, y, (56.0, 50.0), (56.0, 78.0), (78.0, 64.0)), line(0xFFFFFF));
        }
        // A heart reaction.
        2 => c.fill(|x, y| circle(x, y, 52.0, 52.0, 18.0).min(circle(x, y, 76.0, 52.0, 18.0)).min(triangle(x, y, (36.0, 60.0), (92.0, 60.0), (64.0, 96.0))), ghost(0xFF6FA0)),
        // A comment thread.
        3 => {
            for k in 0..3 {
                let x0 = 20.0 + k as f32 * 14.0;
                let y0 = 30.0 + k as f32 * 28.0;
                c.fill(|x, y| rbox(x, y, x0 + 36.0, y0, 36.0, 10.0, 8.0), ghost(0xFFFFFF));
                c.fill(|x, y| rbox(x, y, x0 - 4.0, y0 + 14.0, 1.5, 14.0, 1.0), line(0xFFFFFF));
            }
        }
        // A thumbs-up.
        4 => {
            c.fill(|x, y| rbox(x, y, 50.0, 76.0, 14.0, 22.0, 5.0).min(rbox(x, y, 74.0, 78.0, 18.0, 20.0, 8.0)).min(capsule(x, y, 66.0, 60.0, 74.0, 30.0, 9.0)), ghost(0x8CC8F2));
        }
        // A hashtag.
        5 => {
            for (a, b, cc, d) in [(48.0, 30.0, 40.0, 98.0), (78.0, 30.0, 70.0, 98.0), (26.0, 52.0, 100.0, 52.0), (24.0, 76.0, 98.0, 76.0)] {
                c.fill(|x, y| capsule(x, y, a, b, cc, d, 5.0), ghost(0xFFFFFF));
            }
        }
        // A profile picture in a ring.
        6 => {
            c.fill(|x, y| circle(x, y, 64.0, 64.0, 40.0).abs() - 4.0, line(0xF2C230));
            c.fill(|x, y| circle(x, y, 64.0, 54.0, 14.0).min(ellipse(x, y, 64.0, 92.0, 26.0, 16.0)), ghost(0xFFFFFF));
        }
        // A share arrow.
        7 => c.fill(|x, y| triangle(x, y, (70.0, 30.0), (70.0, 98.0), (108.0, 64.0)).min(rbox(x, y, 48.0, 64.0, 26.0, 12.0, 4.0)), ghost(0x5DBB4F)),
        // A notification badge.
        8 => {
            c.fill(|x, y| rbox(x, y, 58.0, 70.0, 34.0, 30.0, 10.0), ghost(0xFFFFFF));
            c.fill(|x, y| circle(x, y, 90.0, 40.0, 14.0), alpha(hex(0xE23B3B), 0.35));
        }
        // A loading spinner.
        _ => c.fill(|x, y| (circle(x, y, 64.0, 64.0, 30.0).abs() - 6.0).max(-(y - 64.0)), ghost(0xFFFFFF)),
    }
    c
}

/// A blessing's icon: a halo over a glowing gem.
pub fn blessing(i: usize) -> Canvas {
    let mut p = Pen::new(96.0);
    let hues = [0xFF6FA0, 0xF28C38, 0x8CC8F2, 0x5DBB4F, 0x9B6BFF, 0xF2C230, 0x3CC8B4, 0xE23B3B];
    let h = hues[i % hues.len()];
    p.ring(50.0, 22.0, 18.0, 5.0, hex(GOLD));
    p.sdf(|x, y| polygon(x, y, &[(50.0, 34.0), (74.0, 58.0), (50.0, 92.0), (26.0, 58.0)]), hex(h));
    p.sdf_fill(|x, y| polygon(x, y, &[(50.0, 40.0), (62.0, 58.0), (50.0, 66.0), (38.0, 58.0)]), alpha(WHITE, 0.5));
    p.c
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: ART-06
    #[test]
    fn the_feed_has_a_picture_for_everything_on_the_map() {
        let opaque = |c: &Canvas| (0..c.w * c.h).filter(|&i| c.alpha_at(i) > 0.5).count();
        for k in PropKind::ALL {
            assert!(opaque(&prop(k)) > 600, "{k:?}");
        }
        for k in [PickupKind::Food, PickupKind::Magnet, PickupKind::Bomb, PickupKind::Freeze, PickupKind::Clout, PickupKind::Chest] {
            assert!(opaque(&pickup(k)) > 300, "{k:?}");
        }
        let prints: Vec<u64> = (0..DECOR).map(|i| decor(i).fingerprint()).collect();
        for i in 0..DECOR {
            for j in i + 1..DECOR {
                assert_ne!(prints[i], prints[j], "decor {i} and {j}");
            }
        }
    }
}
