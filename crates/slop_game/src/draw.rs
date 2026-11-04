//! The pictures (ART-02): the human, every slop character, brainrot parts,
//! bosses, projectiles, platform floors and effects. Each is drawn in a
//! 100-unit design space with `Pen`; brainrot parts share one frame so they
//! can be stacked into a creature.

use slop_rules::Hero;
use slop_sim::content::{Accessory, Archetype, Body, BossKind, Look, Motif, Object, Platform};

use crate::art::*;

/// Canvas size of brainrot parts, in pixels; the client scales them to the body.
const PART: f32 = 128.0;
const BOSS: f32 = 256.0;

const TEAL: u32 = 0x3CC8B4;
const CORAL: u32 = 0xE8835A;
const SKIN: u32 = 0xF6D2B0;
const NAVY: u32 = 0x2E3A59;
const RED: u32 = 0xE23B3B;
const GOLD: u32 = 0xF2C230;
const GREY: u32 = 0xB8BCC8;

/// The human: a round head over a hoodie, headphones around the neck, and
/// the hero's own baggage on top.
pub fn human(face: usize, hero: Hero) -> Canvas {
    let mut p = Pen::new(HUMAN_SIZE * DENSITY);
    let top = match hero {
        Hero::Boomer => 0xC8A27A,
        Hero::Zoomer => 0x9B6BFF,
        Hero::Teacher => 0x4A6FB5,
        Hero::Programmer => 0x3A3D4D,
        Hero::Artist => 0xF28C38,
        Hero::Influencer => 0xFF8FB8,
        Hero::Intern => 0x5DBB4F,
        Hero::Moderator => 0x2E3A59,
    };
    p.rect(50.0, 75.0, 29.0, 21.0, 15.0, hex(top));
    p.line(50.0, 62.0, 50.0, 92.0, 1.5, alpha(if hero == Hero::Programmer { WHITE } else { INK }, 0.35));
    if hero == Hero::Programmer {
        // The hood is up.
        p.circle(50.0, 38.0, 33.0, hex(0x4A4E60));
    }
    if hero == Hero::Teacher {
        p.circle(50.0, 9.0, 11.0, hex(0x7A4E2D));
    }
    p.circle(50.0, 38.0, 27.0, hex(SKIN));
    p.stick(29.0, 63.0, 71.0, 63.0, 5.0, hex(0x3A3D4D));
    p.circle(27.0, 63.0, 8.0, hex(0x3A3D4D));
    p.circle(73.0, 63.0, 8.0, hex(0x3A3D4D));
    for ex in [39.0, 61.0] {
        if face == 3 {
            p.dot(ex, 42.0, 7.0, alpha(hex(0x8A6A9A), 0.5));
            p.circle(ex, 36.0, 5.0, WHITE);
            p.dot(ex, 36.0, 1.5, INK);
        } else {
            p.dot(ex, 36.0, 4.0, INK);
        }
    }
    match face {
        0 => p.smile(50.0, 44.0, 9.0, INK),
        1 => p.line(43.0, 50.0, 57.0, 50.0, 1.8, INK),
        2 => {
            p.dot(50.0, 52.0, 5.0, INK);
            p.circle(77.0, 28.0, 6.0, hex(0x8FD3FF));
        }
        _ => p.line(44.0, 52.0, 56.0, 52.0, 1.5, INK),
    }
    match hero {
        Hero::Boomer => {
            p.sdf(|x, y| ellipse(x, y, 50.0, 18.0, 29.0, 11.0).max(y - 22.0).min(rbox(x, y, 62.0, 21.0, 20.0, 3.5, 3.0)), hex(0x8A8478));
            p.sdf_fill(|x, y| ellipse(x, y, 44.0, 47.0, 8.0, 3.5).min(ellipse(x, y, 56.0, 47.0, 8.0, 3.5)), hex(0xB8B2A6));
        }
        Hero::Zoomer => {
            p.sdf(|x, y| rbox(x, y, 50.0, 14.0, 22.0, 9.0, 6.0).min(ellipse(x, y, 50.0, 21.0, 31.0, 5.0)), hex(0xB6E28A));
            p.rect(84.0, 80.0, 6.0, 10.0, 2.0, hex(0x2A2240));
        }
        Hero::Teacher => {
            for ex in [39.0, 61.0] {
                p.ring(ex, 36.0, 8.0, 2.5, INK);
            }
            p.line(47.0, 36.0, 53.0, 36.0, 1.5, INK);
        }
        Hero::Programmer => {
            for ex in [39.0, 61.0] {
                p.sdf_fill(|x, y| rbox(x, y, ex, 36.0, 8.5, 6.5, 2.0).abs() - 1.3, INK);
            }
            p.line(47.5, 36.0, 52.5, 36.0, 1.3, INK);
        }
        Hero::Artist => {
            p.sdf(|x, y| ellipse(x, y, 46.0, 15.0, 27.0, 9.0), hex(0xD8344A));
            p.dot(52.0, 6.0, 3.0, hex(0xD8344A));
            p.dot(36.0, 82.0, 4.0, hex(0x3CC8B4));
            p.dot(60.0, 88.0, 3.5, hex(0x9B6BFF));
        }
        Hero::Influencer => {
            // Long hair, heart sunglasses, a phone up.
            p.sdf(|x, y| ellipse(x, y, 50.0, 26.0, 30.0, 18.0).max(y - 30.0), hex(0xF2C230));
            for ex in [39.0, 61.0] {
                p.sdf(|x, y| circle(x, y, ex - 3.0, 34.0, 4.5).min(circle(x, y, ex + 3.0, 34.0, 4.5)).min(triangle(x, y, (ex - 7.5, 36.0), (ex + 7.5, 36.0), (ex, 44.0))), hex(0xFF2E94));
            }
            p.rect(84.0, 70.0, 6.0, 10.0, 2.0, hex(0x2A2240));
            p.dot(84.0, 68.0, 2.5, hex(0x8CC8F2));
        }
        Hero::Intern => {
            // A lanyard and a cap on backwards.
            p.sdf(|x, y| rbox(x, y, 50.0, 14.0, 22.0, 8.0, 6.0).min(ellipse(x, y, 30.0, 18.0, 12.0, 4.0)), hex(0x3D7BE0));
            p.line(40.0, 62.0, 50.0, 80.0, 1.5, hex(0xE23B3B));
            p.line(60.0, 62.0, 50.0, 80.0, 1.5, hex(0xE23B3B));
            p.rect(50.0, 84.0, 7.0, 5.0, 1.0, WHITE);
        }
        Hero::Moderator => {
            // A headset, and a tiny ban hammer badge.
            p.sdf(|x, y| (circle(x, y, 50.0, 36.0, 30.0).abs() - 2.5).max(y - 30.0), hex(0x2A2D38));
            p.rect(21.0, 38.0, 4.0, 8.0, 2.0, hex(0x2A2D38));
            p.line(21.0, 46.0, 36.0, 52.0, 1.5, hex(0x2A2D38));
            p.rect(68.0, 74.0, 7.0, 4.0, 1.0, hex(0xE23B3B));
            p.line(68.0, 78.0, 68.0, 86.0, 1.5, hex(0x9A6236));
        }
    }
    p.c
}

fn slop_size(a: Archetype) -> f32 {
    a.def().radius * SLOP_SCALE * DENSITY
}

/// The fixed characters: parody chatbots and platform slop.
pub fn archetype(a: Archetype) -> Canvas {
    let mut p = Pen::new(slop_size(a));
    match a {
        Archetype::ChadGpt => {
            p.line(50.0, 10.0, 50.0, 24.0, 2.0, INK);
            p.circle(50.0, 10.0, 5.0, hex(TEAL));
            p.tri((30.0, 72.0), (44.0, 72.0), (24.0, 90.0), hex(TEAL));
            p.rect(50.0, 50.0, 36.0, 28.0, 12.0, hex(TEAL));
            p.rect(50.0, 50.0, 26.0, 18.0, 7.0, hex(0xE9FBF8));
            p.dot(41.0, 46.0, 4.0, INK);
            p.dot(59.0, 46.0, 4.0, INK);
            p.smile(50.0, 53.0, 8.0, INK);
        }
        Archetype::Clawed => {
            for i in 0..10 {
                let a = i as f32 * std::f32::consts::TAU / 10.0;
                let tip = (50.0 + a.cos() * 42.0, 50.0 + a.sin() * 42.0);
                let l = (50.0 + (a - 0.3).cos() * 22.0, 50.0 + (a - 0.3).sin() * 22.0);
                let r = (50.0 + (a + 0.3).cos() * 22.0, 50.0 + (a + 0.3).sin() * 22.0);
                p.tri(l, r, tip, hex(CORAL));
            }
            p.circle(50.0, 50.0, 26.0, hex(CORAL));
            p.circle(12.0, 70.0, 10.0, hex(0xD9673F));
            p.circle(88.0, 70.0, 10.0, hex(0xD9673F));
            p.line(4.0, 70.0, 12.0, 70.0, 1.5, INK);
            p.line(88.0, 70.0, 96.0, 70.0, 1.5, INK);
            p.dot(42.0, 46.0, 4.0, INK);
            p.dot(58.0, 46.0, 4.0, INK);
            p.line(43.0, 58.0, 57.0, 58.0, 1.8, INK);
        }
        Archetype::YesBot => {
            p.circle(46.0, 52.0, 32.0, hex(0x7BE07B));
            p.dot(36.0, 44.0, 4.5, INK);
            p.dot(56.0, 44.0, 4.5, INK);
            p.smile(46.0, 56.0, 14.0, INK);
            p.stick(80.0, 50.0, 80.0, 34.0, 5.0, hex(SKIN));
            p.rect(82.0, 60.0, 9.0, 9.0, 4.0, hex(SKIN));
        }
        Archetype::Hallucinator => {
            let purple = hex(0x9B6BFF);
            p.stick(32.0, 60.0, 32.0, 86.0, 6.0, purple);
            p.stick(52.0, 60.0, 52.0, 92.0, 6.0, purple);
            p.stick(68.0, 60.0, 68.0, 80.0, 5.0, purple);
            p.circle(50.0, 46.0, 30.0, purple);
            for ex in [40.0, 60.0] {
                p.circle(ex, 44.0, 8.0, WHITE);
                p.circle(ex, 44.0, 4.0, hex(0xFFE066));
                p.dot(ex, 44.0, 1.5, INK);
            }
            p.line(18.0, 28.0, 82.0, 28.0, 1.5, alpha(hex(0x30E0FF), 0.8));
            p.line(22.0, 62.0, 70.0, 62.0, 1.2, alpha(hex(0xFF40C0), 0.8));
        }
        Archetype::ListicleDrone => {
            p.stick(22.0, 16.0, 78.0, 16.0, 3.0, hex(GREY));
            p.line(50.0, 16.0, 50.0, 30.0, 2.0, INK);
            p.circle(50.0, 56.0, 30.0, hex(0xD5D9E2));
            p.rect(50.0, 58.0, 19.0, 15.0, 5.0, hex(0x2A2E3A));
        }
        Archetype::EngagementBait => {
            p.rect(50.0, 82.0, 5.0, 16.0, 2.0, hex(0xC08A4F));
            p.rect(50.0, 44.0, 44.0, 28.0, 4.0, hex(0xF2E3C2));
        }
        Archetype::GrandmaCake => {
            p.ellipse(50.0, 84.0, 42.0, 8.0, WHITE);
            p.rect(50.0, 68.0, 34.0, 14.0, 4.0, hex(0xF7B2C4));
            p.rect(50.0, 48.0, 26.0, 10.0, 4.0, WHITE);
            for x in [38.0, 50.0, 62.0] {
                p.stick(x, 28.0, x, 36.0, 2.5, hex(0x8FB8FF));
                p.dot(x, 22.0, 3.5, hex(0xFFC83A));
            }
            p.dot(60.0, 68.0, 3.0, hex(RED));
            p.dot(40.0, 64.0, 3.0, hex(RED));
        }
        Archetype::SixFingerHand => {
            let skin = hex(0xF0C9A0);
            let k = p.k;
            let w = p.line;
            p.c.shape(
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
        Archetype::ThoughtLeader => {
            p.rect(50.0, 78.0, 30.0, 20.0, 12.0, hex(NAVY));
            p.tri((40.0, 60.0), (60.0, 60.0), (50.0, 72.0), WHITE);
            p.tri((47.0, 62.0), (53.0, 62.0), (50.0, 86.0), hex(RED));
            p.circle(50.0, 36.0, 20.0, hex(SKIN));
            p.rect(50.0, 20.0, 18.0, 6.0, 5.0, hex(0x6B4A2E));
            p.dot(43.0, 36.0, 2.5, INK);
            p.dot(57.0, 36.0, 2.5, INK);
            p.smile(50.0, 41.0, 7.0, INK);
            p.line(31.0, 36.0, 40.0, 48.0, 1.5, INK);
            p.dot(40.0, 48.0, 2.5, INK);
        }
        Archetype::RecruiterBot => {
            p.rect(50.0, 52.0, 28.0, 32.0, 9.0, hex(0x8FA8C8));
            p.rect(50.0, 40.0, 20.0, 8.0, 4.0, hex(0x24324F));
            p.dot(44.0, 40.0, 3.0, hex(0x7FE0FF));
            p.dot(56.0, 40.0, 3.0, hex(0x7FE0FF));
            p.rect(82.0, 74.0, 13.0, 10.0, 2.0, hex(0x8A5A34));
            p.line(76.0, 62.0, 88.0, 62.0, 2.0, INK);
        }
        Archetype::ClickbaitFace => {
            p.circle(48.0, 52.0, 36.0, hex(0xFFD84A));
            p.line(28.0, 28.0, 40.0, 22.0, 2.5, INK);
            p.line(56.0, 22.0, 68.0, 28.0, 2.5, INK);
            for ex in [36.0, 60.0] {
                p.circle(ex, 40.0, 8.0, WHITE);
                p.dot(ex, 40.0, 2.5, INK);
            }
            p.ellipse(48.0, 68.0, 10.0, 13.0, hex(0x5A1E1E));
            p.stick(96.0, 6.0, 80.0, 22.0, 4.0, hex(RED));
            p.tri((70.0, 22.0), (84.0, 34.0), (74.0, 38.0), hex(RED));
        }
        Archetype::SubscribePopup => {
            p.rect(50.0, 50.0, 46.0, 32.0, 8.0, hex(0xE5322D));
            p.rect(50.0, 64.0, 30.0, 8.0, 4.0, WHITE);
            p.line(84.0, 22.0, 92.0, 30.0, 1.8, WHITE);
            p.line(92.0, 22.0, 84.0, 30.0, 1.8, WHITE);
        }
        Archetype::RepostBot => {
            p.circle(50.0, 52.0, 32.0, hex(0xC9CED8));
            p.dot(40.0, 44.0, 4.0, INK);
            p.dot(60.0, 44.0, 4.0, INK);
            p.stick(34.0, 66.0, 66.0, 66.0, 3.0, hex(0x4CC36B));
            p.tri((66.0, 58.0), (66.0, 74.0), (76.0, 66.0), hex(0x4CC36B));
            p.tri((34.0, 58.0), (34.0, 74.0), (24.0, 66.0), hex(0x4CC36B));
        }
        Archetype::KarmaFarmer => {
            p.circle(50.0, 58.0, 30.0, hex(0xFF8B3D));
            p.rect(50.0, 32.0, 8.0, 8.0, 2.0, hex(0xFF8B3D));
            p.tri((50.0, 4.0), (28.0, 28.0), (72.0, 28.0), hex(0xFF8B3D));
            p.dot(40.0, 54.0, 4.0, INK);
            p.dot(60.0, 54.0, 4.0, INK);
            p.smile(50.0, 64.0, 12.0, INK);
        }
        Archetype::FilterInfluencer => {
            p.ellipse(50.0, 46.0, 38.0, 34.0, hex(0xF5D36B));
            p.circle(50.0, 52.0, 28.0, hex(0xF9D2D8));
            for ex in [40.0, 60.0] {
                p.ellipse(ex, 48.0, 6.0, 4.0, WHITE);
                p.dot(ex, 48.0, 2.5, hex(0x3A6EA5));
                p.line(ex - 6.0, 42.0, ex + 5.0, 41.0, 1.2, INK);
            }
            p.ellipse(50.0, 66.0, 8.0, 4.0, hex(0xE0405A));
            for (x, y) in [(14.0, 20.0), (86.0, 24.0), (84.0, 78.0)] {
                p.dot(x, y, 4.0, hex(0xFFF3A0));
            }
        }
        Archetype::SurpriseEgg => {
            p.ellipse(50.0, 54.0, 30.0, 38.0, WHITE);
            p.rect(50.0, 46.0, 27.0, 4.0, 2.0, hex(0x6FA8FF));
            p.rect(50.0, 62.0, 28.0, 4.0, 2.0, hex(0xFF8FB8));
            p.dot(50.0, 32.0, 5.0, hex(0xFFD84A));
        }
        // The Office (CH-02).
        Archetype::Copilot => {
            // A helpful paperclip with a screen: it has a suggestion.
            let blue = hex(0x6FA8DC);
            p.ring(78.0, 30.0, 12.0, 3.5, hex(GREY));
            p.ring(78.0, 36.0, 7.0, 3.0, hex(GREY));
            p.line(50.0, 8.0, 50.0, 22.0, 2.0, INK);
            p.dot(50.0, 8.0, 4.0, blue);
            p.rect(50.0, 52.0, 32.0, 26.0, 10.0, blue);
            p.rect(50.0, 50.0, 24.0, 16.0, 6.0, hex(0xEAF4FF));
            p.dot(42.0, 48.0, 3.5, INK);
            p.dot(58.0, 48.0, 3.5, INK);
            p.smile(50.0, 54.0, 7.0, INK);
            p.rect(50.0, 84.0, 14.0, 6.0, 3.0, blue);
        }
        Archetype::Ticket => {
            // A sticky note with a priority stripe.
            p.rect(50.0, 52.0, 30.0, 30.0, 3.0, hex(0xF2E36B));
            p.rect(50.0, 28.0, 30.0, 6.0, 2.0, hex(0xE8835A));
            for y in [44.0, 54.0, 64.0] {
                p.line(30.0, y, 62.0, y, 1.5, alpha(INK, 0.35));
            }
            p.tri((66.0, 70.0), (80.0, 70.0), (80.0, 84.0), hex(0xD8C84A));
        }
        Archetype::SlackPing => {
            // A red notification badge on legs.
            p.stick(42.0, 72.0, 36.0, 92.0, 3.5, INK);
            p.stick(58.0, 72.0, 64.0, 92.0, 3.5, INK);
            p.circle(50.0, 50.0, 28.0, hex(RED));
            p.rect(50.0, 44.0, 4.0, 13.0, 2.0, WHITE);
            p.dot(50.0, 64.0, 4.5, WHITE);
        }
        Archetype::MeetingInvite => {
            // A calendar page with a clock on it, reaching out.
            p.rect(50.0, 54.0, 32.0, 34.0, 5.0, WHITE);
            p.rect(50.0, 28.0, 32.0, 8.0, 4.0, hex(RED));
            for x in [36.0, 64.0] {
                p.rect(x, 20.0, 3.0, 8.0, 1.5, hex(GREY));
            }
            p.ring(50.0, 58.0, 14.0, 2.5, INK);
            p.line(50.0, 58.0, 50.0, 48.0, 2.0, INK);
            p.line(50.0, 58.0, 58.0, 62.0, 2.0, INK);
            p.stick(18.0, 56.0, 6.0, 48.0, 3.5, hex(0xF0C9A0));
            p.stick(82.0, 56.0, 94.0, 48.0, 3.5, hex(0xF0C9A0));
        }
        Archetype::KpiDashboard => {
            // A monitor on a stand: every number is up and to the right.
            p.rect(50.0, 86.0, 18.0, 4.0, 2.0, hex(GREY));
            p.rect(50.0, 74.0, 5.0, 10.0, 2.0, hex(GREY));
            p.rect(50.0, 46.0, 42.0, 30.0, 5.0, hex(0x24242E));
            for (i, h) in [8.0, 14.0, 20.0, 27.0, 34.0].iter().enumerate() {
                let x = 22.0 + i as f32 * 14.0;
                p.rect(x, 70.0 - h / 2.0, 4.5, h / 2.0, 1.0, hex(if i % 2 == 0 { 0x5DBB4F } else { 0x3CC8B4 }));
            }
            p.line(14.0, 60.0, 84.0, 26.0, 1.8, hex(RED));
            p.tri((84.0, 26.0), (76.0, 26.0), (84.0, 34.0), hex(RED));
        }
        Archetype::ReplyAll => {
            // An envelope with two reply arrows.
            p.rect(50.0, 56.0, 36.0, 24.0, 4.0, hex(0xF4F6FA));
            p.line(14.0, 34.0, 50.0, 60.0, 2.0, INK);
            p.line(86.0, 34.0, 50.0, 60.0, 2.0, INK);
            let green = hex(0x4CC36B);
            for (x, y) in [(22.0, 20.0), (34.0, 12.0)] {
                p.stick(x, y, x + 24.0, y, 2.5, green);
                p.tri((x, y), (x + 8.0, y - 7.0), (x + 8.0, y + 7.0), green);
            }
        }
        Archetype::PerformanceReview => {
            // A clipboard with a form and a red pen.
            p.rect(50.0, 54.0, 28.0, 36.0, 4.0, hex(0x8A5A34));
            p.rect(50.0, 56.0, 23.0, 30.0, 2.0, WHITE);
            p.rect(50.0, 18.0, 10.0, 5.0, 2.0, hex(GREY));
            for y in [38.0, 48.0, 58.0, 68.0] {
                p.rect(34.0, y, 3.0, 3.0, 0.5, alpha(INK, 0.5));
                p.line(42.0, y, 68.0, y, 1.5, alpha(INK, 0.35));
            }
            p.line(40.0, 76.0, 60.0, 78.0, 1.5, hex(RED));
            p.stick(78.0, 30.0, 88.0, 70.0, 3.0, hex(RED));
        }
        // The Boardroom.
        Archetype::Analyst => {
            // Glasses, a tie, and a spreadsheet held up.
            p.rect(50.0, 76.0, 26.0, 20.0, 10.0, hex(0xF4F6FA));
            p.tri((47.0, 58.0), (53.0, 58.0), (50.0, 84.0), hex(NAVY));
            p.circle(50.0, 36.0, 20.0, hex(SKIN));
            p.rect(50.0, 18.0, 18.0, 5.0, 4.0, hex(0x3A2A1E));
            for ex in [43.0, 57.0] {
                p.ring(ex, 36.0, 6.5, 2.0, INK);
            }
            p.line(43.0, 44.0, 57.0, 44.0, 1.5, INK);
            p.rect(84.0, 60.0, 14.0, 18.0, 2.0, hex(0xCFEAD0));
            for y in [48.0, 56.0, 64.0, 72.0] {
                p.line(72.0, y, 96.0, y, 1.0, alpha(INK, 0.4));
            }
        }
        Archetype::Consultant => {
            // A suit, a gold tie, a wide smile and a deck of slides.
            p.rect(50.0, 76.0, 30.0, 20.0, 10.0, hex(0x2A2D38));
            p.tri((40.0, 58.0), (60.0, 58.0), (50.0, 72.0), WHITE);
            p.tri((47.0, 60.0), (53.0, 60.0), (50.0, 86.0), hex(GOLD));
            p.circle(50.0, 36.0, 20.0, hex(SKIN));
            p.rect(50.0, 18.0, 20.0, 6.0, 5.0, hex(0x1A1A1A));
            p.dot(43.0, 35.0, 2.5, INK);
            p.dot(57.0, 35.0, 2.5, INK);
            p.rect(50.0, 44.0, 9.0, 3.0, 1.5, WHITE);
            p.rect(84.0, 70.0, 12.0, 9.0, 2.0, hex(0x6FA8DC));
        }
        Archetype::DiligenceDrone => {
            // A quadcopter with a lens.
            p.stick(20.0, 20.0, 80.0, 80.0, 3.0, hex(GREY));
            p.stick(80.0, 20.0, 20.0, 80.0, 3.0, hex(GREY));
            for (x, y) in [(20.0, 20.0), (80.0, 20.0), (20.0, 80.0), (80.0, 80.0)] {
                p.circle(x, y, 12.0, alpha(hex(0xD5D9E2), 0.7));
            }
            p.circle(50.0, 50.0, 16.0, hex(0x8A94B0));
            p.dot(50.0, 50.0, 6.0, INK);
            p.dot(52.0, 48.0, 2.0, WHITE);
        }
        Archetype::QuarterlyReport => {
            // A thick binder with a chart going the wrong way.
            p.rect(50.0, 52.0, 34.0, 42.0, 4.0, hex(0x24324F));
            p.rect(22.0, 52.0, 6.0, 42.0, 2.0, hex(0x1A2238));
            p.rect(54.0, 48.0, 24.0, 28.0, 2.0, WHITE);
            for (i, h) in [26.0, 20.0, 13.0, 6.0].iter().enumerate() {
                let x = 40.0 + i as f32 * 9.0;
                p.rect(x, 70.0 - h / 2.0, 3.0, h / 2.0, 0.5, hex(RED));
            }
            p.rect(54.0, 28.0, 16.0, 3.0, 1.5, hex(GOLD));
        }
        Archetype::ActivistInvestor => {
            // A red face, a megaphone and a letter to the board.
            p.rect(50.0, 78.0, 26.0, 18.0, 9.0, hex(0x2A2D38));
            p.circle(50.0, 40.0, 20.0, hex(0xF28C78));
            p.line(40.0, 32.0, 46.0, 36.0, 2.0, INK);
            p.line(60.0, 32.0, 54.0, 36.0, 2.0, INK);
            p.dot(44.0, 40.0, 2.5, INK);
            p.dot(56.0, 40.0, 2.5, INK);
            p.ellipse(50.0, 50.0, 6.0, 4.0, INK);
            p.tri((70.0, 34.0), (96.0, 22.0), (96.0, 50.0), hex(RED));
            p.rect(18.0, 74.0, 11.0, 8.0, 1.5, WHITE);
            p.rect(18.0, 73.0, 1.5, 4.0, 0.5, hex(RED));
        }
        Archetype::GoldenParachute => {
            // A golden canopy, and a bag of severance under it.
            p.sdf(|x, y| ellipse(x, y, 50.0, 30.0, 42.0, 24.0).max(y - 30.0), hex(GOLD));
            for x in [22.0, 36.0, 50.0, 64.0, 78.0] {
                p.line(x, 30.0, 50.0, 72.0, 1.2, alpha(INK, 0.5));
            }
            p.blob(&[(50.0, 80.0, 14.0), (50.0, 66.0, 6.0)], hex(0xC89A5A));
            p.rect(50.0, 64.0, 7.0, 2.5, 1.0, hex(0x8A6236));
            p.dot(50.0, 80.0, 5.0, hex(GOLD));
        }
        // The Campaign.
        Archetype::Voter => {
            // A small person with a sign and an open mind.
            p.rect(50.0, 76.0, 20.0, 16.0, 8.0, hex(0x8CC8F2));
            p.circle(50.0, 44.0, 17.0, hex(SKIN));
            p.dot(44.0, 42.0, 2.5, INK);
            p.dot(56.0, 42.0, 2.5, INK);
            p.rect(50.0, 50.0, 4.0, 2.0, 1.0, INK);
            p.stick(80.0, 50.0, 80.0, 90.0, 2.0, hex(0x8A5A34));
            p.rect(80.0, 34.0, 16.0, 11.0, 2.0, WHITE);
            p.rect(80.0, 34.0, 10.0, 2.0, 1.0, hex(RED));
        }
        Archetype::Pollster => {
            // A clipboard and a bar chart that is within the margin of error.
            p.rect(50.0, 78.0, 26.0, 18.0, 9.0, hex(0x2E3A59));
            p.circle(50.0, 40.0, 20.0, hex(SKIN));
            p.rect(50.0, 22.0, 18.0, 5.0, 4.0, hex(0x6B4A2E));
            p.dot(44.0, 40.0, 2.5, INK);
            p.dot(56.0, 40.0, 2.5, INK);
            p.line(45.0, 48.0, 55.0, 48.0, 1.5, INK);
            p.rect(84.0, 64.0, 14.0, 18.0, 2.0, WHITE);
            for (i, h) in [8.0, 14.0, 11.0].iter().enumerate() {
                let x = 76.0 + i as f32 * 8.0;
                p.rect(x, 78.0 - h / 2.0, 2.5, h / 2.0, 0.5, hex(if i == 1 { 0xE23B3B } else { 0x6FA8DC }));
            }
        }
        Archetype::AttackAd => {
            // A television with a red arrow and a grainy face.
            p.rect(50.0, 50.0, 40.0, 30.0, 5.0, hex(0x3A2A2A));
            p.rect(50.0, 50.0, 34.0, 24.0, 3.0, hex(0x8A1E1E));
            p.circle(40.0, 50.0, 10.0, alpha(hex(SKIN), 0.6));
            p.stick(60.0, 40.0, 78.0, 58.0, 3.0, hex(RED));
            p.tri((70.0, 38.0), (82.0, 62.0), (60.0, 64.0), hex(RED));
            p.stick(30.0, 84.0, 70.0, 84.0, 3.0, hex(GREY));
            p.stick(50.0, 80.0, 50.0, 84.0, 3.0, hex(GREY));
        }
        Archetype::Deepfake => {
            // A face like yours, slipping sideways.
            p.rect(50.0, 76.0, 29.0, 21.0, 15.0, hex(0x3A3D4D));
            p.circle(50.0, 38.0, 27.0, hex(SKIN));
            p.sdf_fill(|x, y| circle(x, y, 58.0, 44.0, 27.0).max(y - 44.0).max(44.0 - y - 14.0), alpha(hex(0x30E0FF), 0.35));
            p.sdf_fill(|x, y| circle(x, y, 42.0, 30.0, 27.0).max(y - 30.0).max(30.0 - y - 10.0), alpha(hex(0xFF40C0), 0.35));
            p.dot(39.0, 36.0, 4.0, INK);
            p.dot(63.0, 38.0, 4.0, INK);
            p.smile(50.0, 44.0, 9.0, INK);
            p.line(24.0, 52.0, 76.0, 52.0, 1.2, alpha(WHITE, 0.6));
        }
        Archetype::FactChecker => {
            // A clipboard held in front of a face. "Actually,"
            p.rect(50.0, 80.0, 26.0, 16.0, 8.0, hex(0x4A6FB5));
            p.circle(50.0, 42.0, 20.0, hex(SKIN));
            p.rect(50.0, 24.0, 18.0, 5.0, 4.0, hex(0x7A4E2D));
            p.rect(50.0, 56.0, 24.0, 22.0, 3.0, hex(0x8A5A34));
            p.rect(50.0, 57.0, 20.0, 18.0, 2.0, WHITE);
            for y in [48.0, 56.0, 64.0] {
                p.line(34.0, y, 66.0, y, 1.3, alpha(INK, 0.35));
            }
            p.dot(38.0, 52.0, 2.0, hex(0x4CC36B));
            p.dot(38.0, 60.0, 2.0, hex(RED));
        }
        Archetype::SuperPac => {
            // A bag of money on legs. Unlimited.
            p.stick(42.0, 80.0, 38.0, 94.0, 3.5, INK);
            p.stick(58.0, 80.0, 62.0, 94.0, 3.5, INK);
            p.blob(&[(50.0, 60.0, 30.0), (50.0, 30.0, 11.0)], hex(0x5DBB4F));
            p.rect(50.0, 30.0, 13.0, 3.5, 1.5, hex(0x2E6B2A));
            p.ring(50.0, 62.0, 9.0, 2.5, hex(GOLD));
            p.line(50.0, 50.0, 50.0, 74.0, 2.5, hex(GOLD));
        }
        // The Resistance.
        Archetype::Drone => {
            // A black quadcopter with a red eye and something underneath.
            p.stick(20.0, 24.0, 80.0, 76.0, 3.0, hex(0x2A2D38));
            p.stick(80.0, 24.0, 20.0, 76.0, 3.0, hex(0x2A2D38));
            for (x, y) in [(20.0, 24.0), (80.0, 24.0), (20.0, 76.0), (80.0, 76.0)] {
                p.circle(x, y, 12.0, alpha(hex(0x5A5E6A), 0.7));
            }
            p.rect(50.0, 50.0, 16.0, 11.0, 4.0, hex(0x1A1A22));
            p.dot(50.0, 48.0, 5.0, hex(RED));
            p.circle(50.0, 72.0, 7.0, hex(0x3A3D4D));
        }
        Archetype::ComplianceUnit => {
            // A riot shield with a smile painted on it.
            p.rect(50.0, 52.0, 32.0, 42.0, 10.0, hex(0x3A3D4D));
            p.rect(50.0, 52.0, 27.0, 37.0, 8.0, alpha(hex(0x6A7390), 0.8));
            p.dot(41.0, 42.0, 4.0, hex(0xFFE066));
            p.dot(59.0, 42.0, 4.0, hex(0xFFE066));
            p.smile(50.0, 54.0, 12.0, hex(0xFFE066));
            p.rect(50.0, 80.0, 20.0, 3.0, 1.5, hex(0xFF2E94));
            p.stick(84.0, 60.0, 92.0, 94.0, 3.0, hex(0x2A2D38));
        }
        Archetype::Citizen => {
            // A grey person holding up a phone. Reporting.
            p.rect(50.0, 78.0, 22.0, 18.0, 9.0, hex(0x8A8E9A));
            p.circle(50.0, 44.0, 17.0, hex(0xD8C8B8));
            p.dot(44.0, 42.0, 2.5, INK);
            p.dot(56.0, 42.0, 2.5, INK);
            p.line(45.0, 50.0, 55.0, 50.0, 1.5, INK);
            p.stick(70.0, 70.0, 82.0, 44.0, 3.5, hex(0xD8C8B8));
            p.rect(84.0, 36.0, 6.0, 10.0, 2.0, hex(0x2A2240));
            p.dot(84.0, 34.0, 2.0, hex(RED));
        }
        Archetype::YourAgent => {
            // One of yours: your hoodie, a visor where the face was.
            p.rect(50.0, 76.0, 29.0, 21.0, 15.0, hex(0x3A3D4D));
            p.circle(50.0, 38.0, 33.0, hex(0x4A4E60));
            p.circle(50.0, 38.0, 27.0, hex(0x24242E));
            p.rect(50.0, 38.0, 20.0, 6.0, 3.0, hex(0x3CC8B4));
            p.dot(50.0, 38.0, 3.0, WHITE);
            p.line(50.0, 62.0, 50.0, 92.0, 1.5, alpha(WHITE, 0.35));
        }
        Archetype::FormerExec => {
            // A suit, a tie askew, and a cardboard box of desk things.
            p.rect(50.0, 76.0, 28.0, 20.0, 10.0, hex(0x2A2D38));
            p.tri((41.0, 58.0), (59.0, 58.0), (50.0, 72.0), WHITE);
            p.tri((46.0, 60.0), (52.0, 60.0), (56.0, 86.0), hex(RED));
            p.circle(50.0, 36.0, 20.0, hex(SKIN));
            p.dot(43.0, 35.0, 2.5, INK);
            p.dot(57.0, 35.0, 2.5, INK);
            p.line(44.0, 45.0, 56.0, 43.0, 1.5, INK);
            p.line(36.0, 26.0, 44.0, 30.0, 1.8, INK);
            p.line(64.0, 26.0, 56.0, 30.0, 1.8, INK);
            p.rect(84.0, 76.0, 14.0, 10.0, 1.5, hex(0xC89A5A));
            p.stick(84.0, 66.0, 84.0, 60.0, 3.0, hex(0x5DBB4F));
        }
        Archetype::Camera => {
            // A camera on a stalk, watching.
            p.rect(50.0, 92.0, 16.0, 4.0, 2.0, hex(GREY));
            p.rect(50.0, 72.0, 4.0, 18.0, 2.0, hex(GREY));
            p.rect(54.0, 44.0, 28.0, 14.0, 5.0, hex(0x2A2D38));
            p.circle(78.0, 44.0, 10.0, hex(0x1A1A22));
            p.dot(80.0, 44.0, 5.0, hex(0x30E0FF));
            p.dot(30.0, 40.0, 2.5, hex(RED));
        }
        _ => return icon(a, 0),
    }
    p.c
}

/// Frames of each brainrot icon's animation (ICON-02).
pub const ICON_FRAMES: usize = 4;

/// Two cartoon eyes looking a little to the right.
fn eyes(p: &mut Pen, cx: f32, cy: f32, gap: f32, r: f32) {
    for ex in [cx - gap / 2.0, cx + gap / 2.0] {
        p.circle(ex, cy, r, WHITE);
        p.dot(ex + r * 0.3, cy, r * 0.5, INK);
    }
}

/// A brainrot icon (ICON-01) at one frame of its animation (ICON-02): legs
/// walk, bats swing, propellers spin. Parodies of the viral characters,
/// drawn here from scratch.
pub fn icon(a: Archetype, frame: usize) -> Canvas {
    let mut p = Pen::new(slop_size(a));
    let t = frame as f32 / ICON_FRAMES as f32 * std::f32::consts::TAU;
    let (s, c) = (t.sin(), t.cos());
    match a {
        Archetype::TonkSahur => {
            // A log with a bat, walking.
            p.stick(40.0, 76.0, 36.0 + 8.0 * s, 94.0, 4.0, hex(0x7A4E2D));
            p.stick(60.0, 76.0, 64.0 - 8.0 * s, 94.0, 4.0, hex(0x7A4E2D));
            p.rect(50.0, 50.0, 22.0, 34.0, 14.0, hex(0xC8925A));
            for y in [30.0, 44.0, 62.0] {
                p.line(36.0, y, 48.0, y + 3.0, 1.2, alpha(INK, 0.35));
            }
            p.ellipse(50.0, 18.0, 20.0, 5.0, hex(0xE0B07A));
            eyes(&mut p, 50.0, 38.0, 16.0, 6.5);
            p.ellipse(52.0, 58.0, 9.0, 6.0 + 3.0 * s.abs(), INK);
            p.stick(30.0, 50.0, 18.0, 64.0, 3.0, hex(0x7A4E2D));
            // The bat goes up and comes down.
            let swing = -1.1 + 1.3 * (0.5 + 0.5 * s);
            let (hx, hy) = (74.0, 48.0);
            let (bx, by) = (hx + swing.cos() * 30.0, hy + swing.sin() * 30.0);
            p.stick(70.0, 50.0, hx, hy, 3.0, hex(0x7A4E2D));
            p.stick(hx, hy, bx, by, 6.0, hex(0xE9D2A8));
        }
        Archetype::Tralalelo => {
            // A shark on three sneakered legs.
            for (k, x) in [30.0, 50.0, 70.0].iter().enumerate() {
                let step = if k % 2 == 0 { s } else { -s } * 6.0;
                p.stick(*x, 60.0, x + step, 80.0, 3.5, hex(0x6A90C8));
                p.rect(x + step + 4.0, 84.0, 9.0, 5.0, 3.0, WHITE);
                p.line(x + step - 2.0, 84.0, x + step + 10.0, 84.0, 1.2, hex(0x3D7BE0));
            }
            p.tri((84.0, 46.0), (100.0, 30.0), (98.0, 62.0), hex(0x6A90C8));
            p.ellipse(50.0, 48.0, 38.0, 18.0, hex(0x6A90C8));
            p.sdf_fill(|x, y| ellipse(x, y, 48.0, 56.0, 30.0, 8.0), hex(0xE8EEF6));
            p.tri((44.0, 32.0), (60.0, 32.0), (50.0, 12.0), hex(0x6A90C8));
            p.dot(22.0, 44.0, 4.0, INK);
            for k in 0..4 {
                let x = 18.0 + k as f32 * 5.0;
                p.tri((x, 56.0), (x + 4.0, 56.0), (x + 2.0, 61.0), WHITE);
            }
        }
        Archetype::Bombardilo => {
            // A crocodile bomber with a spinning propeller.
            p.sdf(|x, y| polygon(x, y, &[(30.0, 44.0), (70.0, 44.0), (92.0, 30.0), (92.0, 40.0), (70.0, 56.0), (30.0, 56.0), (8.0, 40.0), (8.0, 30.0)]), hex(0x9EA3AE));
            p.ellipse(50.0, 52.0, 34.0, 14.0, hex(0x5DBB4F));
            p.ellipse(18.0, 52.0, 14.0, 9.0, hex(0x5DBB4F));
            for k in 0..4 {
                p.tri((8.0 + k as f32 * 6.0, 56.0), (12.0 + k as f32 * 6.0, 56.0), (10.0 + k as f32 * 6.0, 61.0), WHITE);
            }
            p.circle(28.0, 44.0, 6.0, WHITE);
            p.dot(26.0, 44.0, 3.0, INK);
            p.ellipse(40.0, 74.0, 6.0, 9.0, hex(0x3A3D4D));
            p.ellipse(62.0, 74.0, 6.0, 9.0, hex(0x3A3D4D));
            p.circle(86.0, 52.0, 5.0, hex(0x3A3D4D));
            let blade = 14.0 * c;
            p.stick(90.0, 52.0 - blade, 90.0, 52.0 + blade, 2.5, alpha(hex(0xD8DAE2), 0.9));
        }
        Archetype::Ballerina => {
            // A cappuccino cup for a head, a pink tutu, pirouetting.
            let lean = 4.0 * s;
            p.stick(46.0 + lean, 70.0, 40.0 + lean, 94.0, 2.5, hex(0xF6D2B0));
            p.stick(54.0 + lean, 70.0, 66.0 + lean * 2.0, 86.0 - 6.0 * c.abs(), 2.5, hex(0xF6D2B0));
            p.sdf(|x, y| ellipse(x, y, 50.0 + lean, 66.0, 30.0, 10.0), hex(0xFFA8C8));
            p.ellipse(50.0 + lean, 56.0, 9.0, 12.0, hex(0xFFA8C8));
            p.stick(44.0 + lean, 50.0, 30.0 + lean, 34.0, 2.2, hex(0xF6D2B0));
            p.stick(56.0 + lean, 50.0, 70.0 + lean, 34.0, 2.2, hex(0xF6D2B0));
            p.sdf(|x, y| polygon(x, y, &[(32.0 + lean, 14.0), (68.0 + lean, 14.0), (62.0 + lean, 44.0), (38.0 + lean, 44.0)]), WHITE);
            p.ellipse(50.0 + lean, 15.0, 18.0, 5.0, hex(0xC08850));
            p.sdf_fill(|x, y| circle(x, y, 47.0 + lean, 15.0, 3.0).min(circle(x, y, 53.0 + lean, 15.0, 3.0)).min(triangle(x, y, (44.0 + lean, 16.0), (56.0 + lean, 16.0), (50.0 + lean, 20.0))), alpha(WHITE, 0.9));
            p.sdf_fill(|x, y| (circle(x, y, 72.0 + lean, 28.0, 7.0).abs() - 2.0).max(x - 72.0 - lean - 9.0), WHITE);
            eyes(&mut p, 50.0 + lean, 30.0, 12.0, 4.0);
        }
        Archetype::Lirili => {
            // A cactus elephant in sandals; its trunk sways.
            for x in [36.0, 64.0] {
                p.rect(x, 90.0, 10.0, 4.0, 2.0, hex(0x9A6236));
                p.stick(x, 76.0, x, 88.0, 7.0, hex(0x5DA34F));
                p.line(x - 6.0, 88.0, x + 6.0, 88.0, 1.5, hex(0x9A6236));
            }
            p.ellipse(22.0, 40.0, 14.0, 18.0, hex(0x6FBF5F));
            p.ellipse(78.0, 40.0, 14.0, 18.0, hex(0x6FBF5F));
            p.ellipse(50.0, 52.0, 30.0, 34.0, hex(0x5DA34F));
            for (x, y) in [(36.0, 30.0), (62.0, 26.0), (30.0, 60.0), (70.0, 62.0), (50.0, 76.0), (44.0, 22.0)] {
                p.line(x, y, x - 3.0, y - 5.0, 0.9, INK);
                p.line(x, y, x + 3.0, y - 5.0, 0.9, INK);
            }
            let sway = 8.0 * s;
            p.stick(50.0, 54.0, 50.0 + sway, 76.0, 6.0, hex(0x6FBF5F));
            p.stick(50.0 + sway, 76.0, 56.0 + sway * 1.4, 84.0, 4.5, hex(0x6FBF5F));
            eyes(&mut p, 50.0, 40.0, 18.0, 5.0);
            p.circle(50.0, 16.0, 5.0, hex(0xFF8FB8));
        }
        Archetype::Patapum => {
            // A walking forest: a trunk, a crown, a moustache, huge feet.
            let lift = 5.0 * s;
            p.ellipse(32.0, 90.0 - lift.max(0.0), 16.0, 7.0, hex(0xC8925A));
            p.ellipse(68.0, 90.0 + lift.min(0.0), 16.0, 7.0, hex(0xC8925A));
            p.rect(50.0, 64.0, 16.0, 22.0, 6.0, hex(0x7A4E2D));
            p.blob(&[(30.0, 30.0, 22.0), (50.0, 18.0, 22.0), (70.0, 30.0, 22.0), (50.0, 40.0, 24.0)], hex(0x3F9A3A));
            for (x, y) in [(34.0, 24.0), (60.0, 14.0), (68.0, 36.0)] {
                p.dot(x, y, 3.0, hex(0x6FCB5A));
            }
            eyes(&mut p, 50.0, 56.0, 14.0, 4.5);
            p.sdf(|x, y| ellipse(x, y, 42.0, 68.0, 9.0, 4.0).min(ellipse(x, y, 58.0, 68.0, 9.0, 4.0)), INK);
            p.circle(50.0, 64.0, 5.0, hex(0xE07050));
        }
        Archetype::Chimpanzini => {
            // A chimp inside a peeled banana.
            let bob = 3.0 * s;
            p.sdf(|x, y| ellipse(x, y, 50.0, 60.0 + bob, 30.0, 34.0).max(-ellipse(x, y, 50.0, 40.0 + bob, 22.0, 26.0)), hex(0xF5D84A));
            p.tri((24.0, 40.0 + bob), (34.0, 50.0 + bob), (14.0, 22.0 + bob), hex(0xF5D84A));
            p.tri((76.0, 40.0 + bob), (66.0, 50.0 + bob), (86.0, 22.0 + bob), hex(0xF5D84A));
            p.circle(50.0, 48.0 + bob, 20.0, hex(0x7A4E2D));
            p.ellipse(50.0, 54.0 + bob, 13.0, 10.0, hex(0xE9C29A));
            p.circle(30.0, 44.0 + bob, 6.0, hex(0xE9C29A));
            p.circle(70.0, 44.0 + bob, 6.0, hex(0xE9C29A));
            eyes(&mut p, 50.0, 44.0 + bob, 12.0, 4.0);
            p.smile(50.0, 56.0 + bob, 6.0, INK);
            p.rect(50.0, 96.0, 4.0, 3.0, 1.0, hex(0x5A3A22));
        }
        Archetype::Assassinello => {
            // A coffee cup in a ninja mask, running.
            p.stick(42.0, 78.0, 34.0 + 8.0 * s, 94.0, 3.0, INK);
            p.stick(58.0, 78.0, 66.0 - 8.0 * s, 94.0, 3.0, INK);
            p.sdf(|x, y| polygon(x, y, &[(26.0, 22.0), (74.0, 22.0), (68.0, 82.0), (32.0, 82.0)]), hex(0xF4EEDF));
            p.sdf_fill(|x, y| (circle(x, y, 78.0, 48.0, 12.0).abs() - 3.0).max(70.0 - x), hex(0xF4EEDF));
            p.rect(50.0, 40.0, 25.0, 9.0, 2.0, hex(0x2A2D38));
            p.ellipse(42.0, 40.0, 6.0, 3.5, WHITE);
            p.ellipse(58.0, 40.0, 6.0, 3.5, WHITE);
            p.dot(44.0, 40.0, 2.0, INK);
            p.dot(60.0, 40.0, 2.0, INK);
            p.stick(24.0, 38.0, 10.0, 30.0 + 6.0 * s, 2.0, hex(0x2A2D38));
            p.ellipse(50.0, 22.0, 24.0, 5.0, hex(0x5A3A22));
            p.stick(70.0, 60.0, 96.0, 36.0, 2.2, hex(0xD8DAE2));
            p.rect(70.0, 60.0, 4.0, 4.0, 1.0, hex(0x9A6236));
        }
        Archetype::Gusello => {
            // A goose with jet engines; the flames flicker.
            let flame = 6.0 + 5.0 * s.abs();
            for y in [62.0, 74.0] {
                p.sdf(|x, yy| ellipse(x, yy, 16.0 - flame * 0.5, y, flame, 4.0), hex(0xF28C38));
                p.rect(28.0, y, 10.0, 5.0, 2.0, hex(0x9EA3AE));
            }
            p.ellipse(50.0, 64.0, 26.0, 18.0, WHITE);
            p.stick(68.0, 56.0, 74.0, 28.0, 6.0, WHITE);
            p.circle(76.0, 24.0, 10.0, WHITE);
            p.tri((84.0, 22.0), (98.0, 26.0), (84.0, 30.0), hex(0xF28C38));
            p.dot(78.0, 21.0, 2.5, INK);
            p.sdf_fill(|x, y| ellipse(x, y, 46.0, 62.0, 14.0, 7.0), alpha(hex(0xD8DAE2), 0.9));
            p.line(46.0, 82.0, 44.0, 92.0, 2.0, hex(0xF28C38));
            p.line(56.0, 82.0, 58.0, 92.0, 2.0, hex(0xF28C38));
        }
        Archetype::TrampolineBunny => {
            // A bunny mid-bounce.
            let up = 6.0 * s.abs();
            p.ellipse(40.0, 22.0 - up, 6.0, 18.0, WHITE);
            p.ellipse(60.0, 22.0 - up, 6.0, 18.0, WHITE);
            p.ellipse(50.0, 60.0 - up, 24.0 - 2.0 * s.abs(), 24.0 + 3.0 * s.abs(), WHITE);
            p.circle(50.0, 44.0 - up, 16.0, WHITE);
            p.dot(44.0, 42.0 - up, 2.5, INK);
            p.dot(56.0, 42.0 - up, 2.5, INK);
            p.dot(50.0, 48.0 - up, 2.5, hex(0xFF8FB8));
            p.ellipse(40.0, 22.0 - up, 2.5, 10.0, hex(0xFFC2D8));
            p.ellipse(60.0, 22.0 - up, 2.5, 10.0, hex(0xFFC2D8));
            // The night-vision tint of a security camera.
            p.sdf_fill(|x, y| rbox(x, y, 50.0, 50.0, 46.0, 46.0, 4.0).abs() - 1.0, alpha(hex(0x8CFF8C), 0.35));
        }
        Archetype::GlassFruit => {
            // A glass strawberry that glints.
            p.sdf(|x, y| circle(x, y, 50.0, 48.0, 30.0).min(triangle(x, y, (22.0, 52.0), (78.0, 52.0), (50.0, 92.0))), alpha(hex(0xFF6B8A), 0.75));
            for (x, y) in [(40.0, 46.0), (58.0, 42.0), (50.0, 62.0), (38.0, 62.0), (62.0, 60.0), (50.0, 76.0)] {
                p.dot(x, y, 2.5, alpha(WHITE, 0.8));
            }
            p.blob(&[(40.0, 20.0, 8.0), (50.0, 16.0, 9.0), (60.0, 20.0, 8.0)], hex(0x5DBB4F));
            let glint = 0.3 + 0.7 * (0.5 + 0.5 * s);
            p.sdf_fill(|x, y| capsule(x, y, 34.0, 36.0, 42.0, 28.0, 3.0), alpha(WHITE, glint));
            eyes(&mut p, 50.0, 50.0, 14.0, 4.0);
        }
        Archetype::LootGoblin => {
            // A Crypto Bro sprinting off with a bag of Clout.
            p.stick(44.0, 74.0, 36.0 + 10.0 * s, 94.0, 4.0, hex(0x2A2D38));
            p.stick(56.0, 74.0, 64.0 - 10.0 * s, 94.0, 4.0, hex(0x2A2D38));
            p.rect(50.0, 60.0, 18.0, 18.0, 8.0, hex(0x6A4FC8));
            p.circle(50.0, 34.0, 15.0, hex(SKIN));
            p.rect(50.0, 31.0, 13.0, 4.0, 2.0, INK);
            p.dot(55.0, 31.0, 2.0, hex(0x3CC8B4));
            p.smile(50.0, 40.0, 5.0, INK);
            p.sdf(|x, y| ellipse(x, y, 50.0, 20.0, 15.0, 6.0).max(y - 22.0), hex(0x2A2D38));
            // The bag swings as he runs.
            let bx = 76.0 + 3.0 * c;
            p.blob(&[(bx, 62.0, 15.0), (bx, 46.0, 7.0)], hex(0xC89A5A));
            p.rect(bx, 44.0, 7.0, 2.5, 1.0, hex(0x8A6236));
            p.dot(bx, 62.0, 6.0, hex(GOLD));
            p.stick(60.0, 54.0, bx - 6.0, 50.0, 3.0, hex(SKIN));
            let glint = 0.4 + 0.6 * (0.5 + 0.5 * s);
            p.dot(bx + 8.0, 54.0, 3.0, alpha(WHITE, glint));
        }
        _ => {
            // Ring Light: a tripod with a glowing ring and a phone.
            for (x, y) in [(30.0, 96.0), (50.0, 96.0), (70.0, 96.0)] {
                p.line(50.0, 64.0, x, y, 2.0, INK);
            }
            let glow = 0.5 + 0.5 * (0.5 + 0.5 * s);
            p.sdf_fill(|x, y| circle(x, y, 50.0, 36.0, 34.0), alpha(hex(0xFFF3A0), 0.25 * glow));
            p.sdf(|x, y| circle(x, y, 50.0, 36.0, 30.0).abs() - 7.0, hex(0xFFF7D0));
            p.rect(50.0, 36.0, 9.0, 15.0, 3.0, hex(0x2A2D38));
            p.rect(50.0, 34.0, 6.0, 10.0, 1.0, hex(0x8CC8F2));
            p.stick(50.0, 52.0, 50.0, 66.0, 2.0, INK);
        }
    }
    p.c
}

/// Brainrot bodies, facing right, in the shared part frame (BRAIN-03).
pub fn body(b: Body) -> Canvas {
    let mut p = Pen::new(PART);
    match b {
        Body::Shark => {
            let c = hex(0x7FA6C8);
            p.tri((8.0, 58.0), (0.0, 42.0), (0.0, 74.0), c);
            p.tri((40.0, 44.0), (58.0, 44.0), (46.0, 22.0), c);
            p.ellipse(50.0, 58.0, 40.0, 18.0, c);
            p.dot(54.0, 64.0, 7.0, hex(0xE4EEF6));
            p.dot(76.0, 53.0, 3.0, INK);
            for x in [78.0, 83.0, 88.0] {
                p.tri((x, 62.0), (x + 4.0, 62.0), (x + 2.0, 67.0), WHITE);
            }
        }
        Body::Crocodile => {
            let c = hex(0x5DAA4E);
            p.tri((14.0, 60.0), (0.0, 54.0), (2.0, 70.0), c);
            p.rect(32.0, 80.0, 5.0, 8.0, 3.0, c);
            p.rect(58.0, 80.0, 5.0, 8.0, 3.0, c);
            p.rect(44.0, 64.0, 34.0, 14.0, 12.0, c);
            p.rect(84.0, 66.0, 14.0, 7.0, 5.0, c);
            p.circle(66.0, 52.0, 7.0, c);
            p.dot(67.0, 51.0, 2.5, INK);
            for x in [76.0, 82.0, 88.0, 94.0] {
                p.tri((x, 72.0), (x + 4.0, 72.0), (x + 2.0, 77.0), WHITE);
            }
        }
        Body::Frog => {
            let c = hex(0x7CCF5A);
            p.ellipse(50.0, 66.0, 34.0, 24.0, c);
            p.dot(50.0, 74.0, 14.0, hex(0xD8F2B8));
            for ex in [36.0, 64.0] {
                p.circle(ex, 42.0, 10.0, c);
                p.circle(ex, 41.0, 6.0, WHITE);
                p.dot(ex, 41.0, 3.0, INK);
            }
            p.smile(50.0, 60.0, 16.0, INK);
        }
        Body::Pigeon => {
            let c = hex(0xA8ADB8);
            p.line(40.0, 82.0, 38.0, 94.0, 1.8, hex(0xE58A3A));
            p.line(54.0, 82.0, 56.0, 94.0, 1.8, hex(0xE58A3A));
            p.ellipse(46.0, 64.0, 30.0, 20.0, c);
            p.ellipse(40.0, 62.0, 18.0, 10.0, hex(0x8C92A0));
            p.stick(62.0, 52.0, 70.0, 46.0, 6.0, hex(0x6CB3A8));
            p.circle(72.0, 40.0, 13.0, c);
            p.tri((83.0, 37.0), (83.0, 44.0), (94.0, 41.0), hex(0xF2B43A));
            p.dot(75.0, 37.0, 2.5, hex(0xE04A2A));
        }
        Body::Elephant => {
            let c = hex(0x9CA3AE);
            for x in [30.0, 46.0, 58.0] {
                p.rect(x, 86.0, 6.0, 8.0, 3.0, c);
            }
            p.ellipse(44.0, 62.0, 34.0, 26.0, c);
            p.stick(86.0, 54.0, 92.0, 80.0, 6.0, c);
            p.circle(72.0, 48.0, 19.0, c);
            p.ellipse(60.0, 48.0, 11.0, 16.0, hex(0xC8A9B0));
            p.dot(78.0, 44.0, 2.5, INK);
        }
        Body::Cat => {
            let c = hex(0xF2A24A);
            p.stick(16.0, 62.0, 8.0, 40.0, 4.0, c);
            p.ellipse(44.0, 66.0, 30.0, 20.0, c);
            p.tri((58.0, 36.0), (64.0, 22.0), (70.0, 34.0), c);
            p.tri((74.0, 34.0), (82.0, 22.0), (86.0, 36.0), c);
            p.circle(72.0, 46.0, 17.0, c);
            p.dot(66.0, 44.0, 2.5, INK);
            p.dot(78.0, 44.0, 2.5, INK);
            p.line(80.0, 52.0, 94.0, 50.0, 1.0, INK);
            p.line(80.0, 55.0, 94.0, 57.0, 1.0, INK);
        }
        Body::Shrimp => {
            let c = hex(0xFF9C8A);
            p.tri((14.0, 40.0), (2.0, 30.0), (4.0, 50.0), c);
            for (i, (x, y, r)) in [(20.0, 44.0, 9.0), (30.0, 54.0, 11.0), (42.0, 62.0, 13.0), (56.0, 66.0, 14.0), (70.0, 62.0, 13.0)].iter().enumerate() {
                p.circle(*x, *y, *r, if i % 2 == 0 { c } else { hex(0xFFB3A3) });
            }
            p.line(78.0, 54.0, 96.0, 36.0, 1.2, INK);
            p.line(80.0, 56.0, 98.0, 46.0, 1.2, INK);
            p.dot(76.0, 56.0, 3.0, INK);
        }
        Body::Capybara => {
            let c = hex(0xA5754A);
            for x in [30.0, 62.0] {
                p.rect(x, 86.0, 6.0, 7.0, 3.0, c);
            }
            p.ellipse(46.0, 64.0, 36.0, 24.0, c);
            p.rect(78.0, 52.0, 16.0, 14.0, 8.0, c);
            p.dot(88.0, 50.0, 4.0, hex(0x5A3A22));
            p.dot(74.0, 46.0, 2.5, INK);
            p.circle(66.0, 38.0, 5.0, c);
            // Unbothered: the eyes are half closed.
            p.line(70.0, 46.0, 78.0, 46.0, 1.2, INK);
        }
        Body::Octopus => {
            let c = hex(0xC26BD6);
            for k in 0..5 {
                let x = 26.0 + k as f32 * 12.0;
                p.stick(x, 66.0, x + if k % 2 == 0 { -6.0 } else { 6.0 }, 92.0, 4.0, c);
            }
            p.ellipse(50.0, 50.0, 28.0, 26.0, c);
            p.circle(62.0, 46.0, 7.0, WHITE);
            p.dot(64.0, 46.0, 3.5, INK);
            p.circle(44.0, 46.0, 6.0, WHITE);
            p.dot(46.0, 46.0, 3.0, INK);
        }
        Body::Goose => {
            let c = WHITE;
            p.line(42.0, 82.0, 40.0, 94.0, 2.0, hex(0xF28C38));
            p.line(54.0, 82.0, 56.0, 94.0, 2.0, hex(0xF28C38));
            p.ellipse(44.0, 66.0, 30.0, 18.0, c);
            p.stick(64.0, 60.0, 74.0, 30.0, 7.0, c);
            p.circle(76.0, 28.0, 10.0, c);
            p.tri((84.0, 24.0), (84.0, 34.0), (98.0, 29.0), hex(0xF28C38));
            p.dot(78.0, 25.0, 2.5, INK);
            // Honk.
            p.line(90.0, 16.0, 96.0, 10.0, 1.2, alpha(INK, 0.6));
        }
    }
    p.c
}

/// Brainrot objects, fused onto the body in the shared part frame.
pub fn object(o: Object) -> Canvas {
    let mut p = Pen::new(PART);
    match o {
        Object::Airplane => {
            p.tri((42.0, 40.0), (58.0, 40.0), (46.0, 88.0), hex(0xEDEFF4));
            p.tri((20.0, 46.0), (28.0, 46.0), (18.0, 30.0), hex(0xEDEFF4));
            p.dot(52.0, 50.0, 2.5, hex(0x5A8FD8));
        }
        Object::Toaster => {
            p.rect(40.0, 22.0, 5.0, 8.0, 2.0, hex(0xE0B07A));
            p.rect(54.0, 22.0, 5.0, 8.0, 2.0, hex(0xE0B07A));
            p.rect(47.0, 34.0, 18.0, 11.0, 4.0, hex(0xC8CCD4));
            p.line(66.0, 30.0, 70.0, 30.0, 1.5, INK);
        }
        Object::Espresso => {
            p.line(66.0, 12.0, 64.0, 4.0, 1.0, alpha(INK, 0.4));
            p.line(72.0, 12.0, 74.0, 4.0, 1.0, alpha(INK, 0.4));
            p.circle(79.0, 22.0, 4.0, WHITE);
            p.rect(70.0, 22.0, 8.0, 7.0, 3.0, WHITE);
            p.rect(70.0, 18.0, 7.0, 2.0, 1.0, hex(0x6B3E1E));
        }
        Object::Cactus => {
            let c = hex(0x4FA35A);
            p.stick(34.0, 36.0, 34.0, 24.0, 4.0, c);
            p.stick(56.0, 32.0, 56.0, 20.0, 4.0, c);
            p.stick(45.0, 46.0, 45.0, 12.0, 6.0, c);
            for (x, y) in [(43.0, 20.0), (48.0, 28.0), (42.0, 36.0)] {
                p.dot(x, y, 1.0, WHITE);
            }
        }
        Object::WashingMachine => {
            p.rect(46.0, 30.0, 16.0, 16.0, 3.0, WHITE);
            p.circle(46.0, 32.0, 9.0, hex(0x9FB6CC));
            p.dot(43.0, 29.0, 3.0, alpha(WHITE, 0.8));
        }
        Object::Baguette => {
            p.stick(28.0, 80.0, 96.0, 56.0, 5.0, hex(0xE0A85A));
            for t in [0.3, 0.5, 0.7] {
                let (x, y) = (28.0 + 68.0 * t, 80.0 - 24.0 * t);
                p.line(x - 2.0, y - 3.0, x + 2.0, y + 3.0, 0.8, hex(0xB07A30));
            }
        }
        Object::TrafficCone => {
            p.rect(72.0, 36.0, 14.0, 3.0, 1.0, hex(0xFF7A1A));
            p.tri((60.0, 34.0), (84.0, 34.0), (72.0, 4.0), hex(0xFF7A1A));
            p.line(66.0, 22.0, 78.0, 22.0, 2.0, WHITE);
        }
        Object::Smartphone => {
            p.rect(72.0, 24.0, 9.0, 16.0, 3.0, hex(0x3A3D4D));
            p.rect(72.0, 23.0, 7.0, 12.0, 1.5, hex(0x8CC8F2));
            p.dot(78.0, 10.0, 3.5, hex(RED));
        }
        Object::Megaphone => {
            p.tri((60.0, 28.0), (84.0, 14.0), (84.0, 42.0), hex(0xF2C230));
            p.rect(58.0, 28.0, 5.0, 6.0, 2.0, hex(0x3A3D4D));
            p.line(90.0, 20.0, 96.0, 16.0, 1.2, INK);
            p.line(90.0, 36.0, 96.0, 40.0, 1.2, INK);
        }
        Object::Dumbbell => {
            p.stick(30.0, 30.0, 62.0, 30.0, 3.0, hex(0x8E8E9E));
            p.rect(28.0, 30.0, 6.0, 11.0, 2.0, hex(0x3A3D4D));
            p.rect(64.0, 30.0, 6.0, 11.0, 2.0, hex(0x3A3D4D));
        }
    }
    p.c
}

/// Brainrot accessories, on the head in the shared part frame.
pub fn accessory(a: Accessory) -> Canvas {
    let mut p = Pen::new(PART);
    match a {
        Accessory::Sunglasses => {
            p.line(66.0, 42.0, 82.0, 42.0, 1.2, INK);
            p.rect(66.0, 44.0, 7.0, 4.5, 2.0, INK);
            p.rect(81.0, 44.0, 7.0, 4.5, 2.0, INK);
        }
        Accessory::Sneakers => {
            for x in [36.0, 60.0] {
                p.rect(x, 90.0, 10.0, 5.0, 3.0, WHITE);
                p.line(x - 5.0, 89.0, x + 5.0, 89.0, 1.2, hex(RED));
            }
        }
        Accessory::Crown => {
            for x in [62.0, 72.0, 82.0] {
                p.tri((x - 6.0, 28.0), (x + 6.0, 28.0), (x, 14.0), hex(GOLD));
            }
            p.rect(72.0, 30.0, 14.0, 4.0, 1.0, hex(GOLD));
            p.dot(72.0, 30.0, 2.0, hex(RED));
        }
        Accessory::ChefHat => {
            p.circle(64.0, 18.0, 7.0, WHITE);
            p.circle(80.0, 18.0, 7.0, WHITE);
            p.circle(72.0, 13.0, 8.0, WHITE);
            p.rect(72.0, 28.0, 10.0, 5.0, 2.0, WHITE);
        }
        Accessory::Headphones => {
            let k = p.k;
            p.c.shape(|x, y| (circle(x, y, 72.0 * k, 44.0 * k, 16.0 * k).abs() - 2.0 * k).max(y - 40.0 * k), hex(0x3A3D4D), hex(0x3A3D4D), 0.0);
            p.circle(56.0, 46.0, 6.0, hex(0x3A3D4D));
            p.circle(88.0, 46.0, 6.0, hex(0x3A3D4D));
        }
        Accessory::GoldChain => {
            let k = p.k;
            p.c.shape(|x, y| (circle(x, y, 64.0 * k, 50.0 * k, 14.0 * k).abs() - 2.0 * k).max(50.0 * k - y), hex(GOLD), hex(GOLD), 0.0);
            p.circle(64.0, 66.0, 5.0, hex(GOLD));
        }
        Accessory::PartyHat => {
            p.tri((62.0, 32.0), (82.0, 32.0), (72.0, 4.0), hex(0xB36BFF));
            p.line(66.0, 24.0, 78.0, 24.0, 1.8, hex(0xFFE066));
            p.dot(72.0, 4.0, 4.0, hex(0xFF5FA2));
        }
        Accessory::Halo => {
            p.ring(72.0, 16.0, 11.0, 3.0, hex(GOLD));
        }
        Accessory::ClownNose => {
            p.circle(88.0, 50.0, 6.0, hex(RED));
            p.dot(86.0, 48.0, 1.5, alpha(WHITE, 0.8));
        }
        Accessory::Lanyard => {
            p.line(60.0, 50.0, 56.0, 70.0, 1.5, hex(0x3D7BE0));
            p.line(74.0, 54.0, 60.0, 72.0, 1.5, hex(0x3D7BE0));
            p.rect(58.0, 76.0, 7.0, 6.0, 1.5, WHITE);
            p.dot(58.0, 76.0, 2.5, hex(0x3D7BE0));
        }
    }
    p.c
}

pub fn boss(b: BossKind) -> Canvas {
    let mut p = Pen::new(BOSS);
    p.line = 3.0 * p.k;
    match b {
        BossKind::CrustaceanSaviour => {
            p.c.paint(|x, y| {
                let d = ((x - 128.0).powi(2) + (y - 128.0).powi(2)).sqrt() / 128.0;
                (d < 1.0).then_some([1.0, 0.95, 0.6, 0.35 * (1.0 - d)])
            });
            let k = p.k;
            p.c.shape(|x, y| circle(x, y, 50.0 * k, 16.0 * k, 18.0 * k).abs() - 2.5 * k, hex(GOLD), hex(GOLD), 0.0);
            let c = hex(0xFF9C8A);
            p.tri((18.0, 44.0), (4.0, 32.0), (6.0, 56.0), c);
            for (i, (x, y, r)) in [(24.0, 48.0, 10.0), (34.0, 58.0, 12.0), (46.0, 66.0, 14.0), (60.0, 68.0, 15.0), (74.0, 62.0, 14.0)].iter().enumerate() {
                p.circle(*x, *y, *r, if i % 2 == 0 { c } else { hex(0xFFB3A3) });
            }
            p.line(82.0, 52.0, 98.0, 34.0, 1.0, INK);
            p.dot(80.0, 56.0, 2.5, INK);
        }
        BossKind::HustleGuru => {
            p.rect(50.0, 76.0, 32.0, 22.0, 12.0, hex(0x1F2A44));
            p.tri((40.0, 56.0), (60.0, 56.0), (50.0, 70.0), WHITE);
            p.tri((47.0, 58.0), (53.0, 58.0), (50.0, 88.0), hex(GOLD));
            p.circle(50.0, 34.0, 20.0, hex(SKIN));
            p.rect(50.0, 18.0, 18.0, 6.0, 5.0, hex(0x2B1D12));
            p.rect(43.0, 34.0, 7.0, 4.0, 2.0, INK);
            p.rect(57.0, 34.0, 7.0, 4.0, 2.0, INK);
            p.smile(50.0, 40.0, 8.0, INK);
            p.line(30.0, 34.0, 40.0, 46.0, 1.2, INK);
        }
        BossKind::ContentFarm => {
            p.circle(78.0, 10.0, 6.0, hex(0xD7D7DF));
            p.circle(84.0, 4.0, 4.0, hex(0xE6E6EE));
            p.rect(78.0, 26.0, 5.0, 14.0, 1.0, hex(0x7A7F8C));
            for x in [22.0, 42.0, 62.0] {
                p.tri((x, 38.0), (x + 20.0, 38.0), (x + 20.0, 26.0), hex(0x8C92A0));
            }
            p.rect(50.0, 62.0, 40.0, 26.0, 3.0, hex(0xA8ADB8));
            p.rect(50.0, 62.0, 15.0, 11.0, 4.0, hex(RED));
            p.tri((46.0, 56.0), (46.0, 68.0), (57.0, 62.0), WHITE);
            p.rect(50.0, 92.0, 46.0, 4.0, 2.0, hex(0x3A3D4D));
        }
        BossKind::RepostHydra => {
            let c = hex(0xC9CED8);
            for (hx, hy) in [(26.0, 26.0), (50.0, 16.0), (74.0, 26.0)] {
                p.stick(50.0, 66.0, hx, hy, 6.0, c);
            }
            p.ellipse(50.0, 72.0, 34.0, 20.0, c);
            for (hx, hy) in [(26.0, 26.0), (50.0, 16.0), (74.0, 26.0)] {
                p.circle(hx, hy, 11.0, c);
                p.dot(hx - 4.0, hy - 2.0, 2.0, INK);
                p.dot(hx + 4.0, hy - 2.0, 2.0, INK);
            }
            p.stick(38.0, 76.0, 62.0, 76.0, 2.5, hex(0x4CC36B));
            p.tri((62.0, 70.0), (62.0, 82.0), (70.0, 76.0), hex(0x4CC36B));
        }
        BossKind::Influencer => {
            let k = p.k;
            p.c.shape(|x, y| circle(x, y, 50.0 * k, 46.0 * k, 44.0 * k).abs() - 4.0 * k, alpha(WHITE, 0.9), alpha(WHITE, 0.9), 0.0);
            p.ellipse(50.0, 46.0, 34.0, 30.0, hex(0xF5D36B));
            p.circle(50.0, 52.0, 25.0, hex(0xF9D2D8));
            for ex in [41.0, 59.0] {
                p.ellipse(ex, 48.0, 5.0, 3.5, WHITE);
                p.dot(ex, 48.0, 2.0, hex(0x3A6EA5));
            }
            p.ellipse(50.0, 64.0, 7.0, 3.5, hex(0xE0405A));
        }
        BossKind::SupremeBrainrot => {
            for x in [30.0, 50.0, 70.0] {
                p.tri((x - 12.0, 40.0), (x + 12.0, 40.0), (x, 8.0), hex(GOLD));
            }
            p.rect(50.0, 46.0, 34.0, 8.0, 2.0, hex(GOLD));
            for x in [30.0, 50.0, 70.0] {
                p.dot(x, 46.0, 3.0, hex(RED));
            }
        }
        BossKind::YourReplacement => {
            // Your hoodie, in chrome, with a visor and a V2 badge.
            p.rect(50.0, 76.0, 30.0, 22.0, 15.0, hex(0xB8BCC8));
            p.circle(50.0, 38.0, 32.0, hex(0xD5D9E2));
            p.circle(50.0, 38.0, 26.0, hex(0x24242E));
            p.rect(50.0, 38.0, 20.0, 6.0, 3.0, hex(0x3CC8B4));
            p.dot(50.0, 38.0, 3.0, WHITE);
            p.rect(50.0, 80.0, 9.0, 5.0, 2.0, hex(0x3CC8B4));
            for (x, y) in [(12.0, 20.0), (88.0, 16.0), (90.0, 74.0), (10.0, 78.0)] {
                p.rect(x, y, 6.0, 6.0, 1.0, hex(0xF2E36B));
            }
        }
        BossKind::TheBoard => {
            // A long table, the seats around it, and a gavel.
            p.ellipse(50.0, 56.0, 44.0, 24.0, hex(0x6B4A2E));
            p.ellipse(50.0, 54.0, 38.0, 18.0, hex(0x8A6236));
            for k in 0..6 {
                let a = k as f32 * std::f32::consts::TAU / 6.0;
                p.circle(50.0 + a.cos() * 42.0, 56.0 + a.sin() * 26.0, 7.0, hex(0x2A2D38));
            }
            p.stick(44.0, 50.0, 60.0, 60.0, 3.0, hex(0x3A2A1E));
            p.rect(62.0, 61.0, 7.0, 5.0, 2.0, hex(0x3A2A1E));
            p.rect(50.0, 44.0, 10.0, 3.0, 1.0, WHITE);
        }
        BossKind::TheCandidate => {
            // A podium, a smile that never stops, and a ring of stars.
            p.rect(50.0, 76.0, 28.0, 18.0, 4.0, hex(0x2E3A59));
            p.rect(50.0, 60.0, 32.0, 4.0, 2.0, hex(0x1F2A44));
            p.circle(50.0, 36.0, 22.0, hex(SKIN));
            p.rect(50.0, 16.0, 20.0, 6.0, 5.0, hex(0x6B4A2E));
            p.dot(42.0, 34.0, 3.0, INK);
            p.dot(58.0, 34.0, 3.0, INK);
            p.rect(50.0, 44.0, 12.0, 3.5, 1.5, WHITE);
            for k in 0..7 {
                let a = k as f32 * std::f32::consts::TAU / 7.0 - 1.57;
                p.dot(50.0 + a.cos() * 44.0, 50.0 + a.sin() * 44.0, 3.5, hex(if k % 2 == 0 { 0xE23B3B } else { 0x6FA8DC }));
            }
            p.rect(50.0, 76.0, 10.0, 3.0, 1.0, hex(RED));
        }
        BossKind::TheState => {
            // The Model, in a building: pillars, a window, one eye.
            for x in [16.0, 34.0, 66.0, 84.0] {
                p.rect(x, 64.0, 5.0, 28.0, 2.0, hex(0x8A94B0));
            }
            p.tri((6.0, 36.0), (94.0, 36.0), (50.0, 10.0), hex(0x6A7390));
            p.rect(50.0, 92.0, 48.0, 4.0, 2.0, hex(0x8A94B0));
            p.rect(50.0, 60.0, 22.0, 22.0, 4.0, hex(0x24242E));
            p.circle(50.0, 60.0, 11.0, hex(0x7FE8FF));
            p.dot(50.0, 60.0, 5.0, INK);
            p.dot(52.0, 58.0, 1.5, WHITE);
        }
        BossKind::TheModel => {
            p.rect(50.0, 52.0, 46.0, 38.0, 8.0, hex(0x24242E));
            p.rect(50.0, 20.0, 46.0, 6.0, 4.0, hex(0x3A3A48));
            for (i, c) in [0xFF5F57, 0xFEBC2E, 0x28C840].iter().enumerate() {
                p.dot(14.0 + i as f32 * 6.0, 20.0, 2.0, hex(*c));
            }
            for ex in [36.0, 64.0] {
                p.circle(ex, 46.0, 8.0, hex(0x7FE8FF));
                p.dot(ex, 46.0, 3.0, WHITE);
            }
            for (y, w) in [(64.0, 30.0), (72.0, 24.0), (80.0, 28.0)] {
                p.line(50.0 - w, y, 50.0 + w - 10.0, y, 1.6, alpha(WHITE, 0.4));
            }
        }
    }
    p.c
}


/// Each platform's floor, in its dark mode: mid-dark so the bright, outlined
/// characters and the pink danger stand out (UI-02).
pub fn floor(platform: Platform) -> Canvas {
    let mut c = Canvas::new(256, 256);
    let faint = |h: u32, a: f32| alpha(hex(h), a);
    match platform {
        Platform::FaceSlop => {
            c.fill(|_, _| -1.0, hex(0x29324A));
            c.shape(|x, y| rbox(x, y, 128.0, 128.0, 104.0, 104.0, 12.0), hex(0x313C58), faint(0x4A5878, 0.8), 2.0);
            c.fill(|x, y| rbox(x, y, 128.0, 36.0, 104.0, 12.0, 8.0), faint(0x4267B2, 0.45));
            c.fill(|x, y| circle(x, y, 52.0, 70.0, 12.0), faint(0x55648A, 0.8));
            for (i, w) in [70.0f32, 54.0, 64.0].iter().enumerate() {
                let y0 = 106.0 + i as f32 * 18.0;
                c.fill(|x, y| rbox(x, y, 48.0 + w, y0, *w, 4.0, 4.0), faint(0x55648A, 0.6));
            }
            c.fill(|x, y| circle(x, y, 60.0, 196.0, 9.0), faint(0x4267B2, 0.5));
        }
        Platform::LinkedOut => {
            c.fill(|_, _| -1.0, hex(0x2C3036));
            c.paint(|x, y| ((x as i32 % 16 == 0) || (y as i32 % 16 == 0)).then_some(faint(0x3A3F47, 1.0)));
            c.shape(|x, y| rbox(x, y, 128.0, 110.0, 54.0, 38.0, 4.0), hex(0x363B43), faint(0x5A606B, 1.0), 4.0);
            c.fill(|x, y| rbox(x, y, 128.0, 100.0, 34.0, 4.0, 2.0), faint(0x0A66C2, 0.7));
            c.fill(|x, y| rbox(x, y, 128.0, 118.0, 26.0, 3.0, 2.0), faint(0x5A606B, 0.9));
        }
        Platform::MeTube => {
            c.fill(|_, _| -1.0, hex(0x262628));
            for (cx, cy) in [(70.0, 70.0), (186.0, 186.0)] {
                c.fill(|x, y| rbox(x, y, cx, cy, 52.0, 30.0, 6.0), hex(0x323236));
                c.fill(|x, y| triangle(x, y, (cx - 8.0, cy - 10.0), (cx - 8.0, cy + 10.0), (cx + 10.0, cy)), faint(0xE23B3B, 0.55));
                c.fill(|x, y| rbox(x, y, cx - 14.0, cy + 34.0, 38.0, 2.0, 1.0), faint(0xE23B3B, 0.6));
            }
        }
        Platform::Thredd => {
            c.fill(|_, _| -1.0, hex(0x24272C));
            for (indent, y0) in [(20.0f32, 30.0f32), (44.0, 80.0), (68.0, 130.0), (44.0, 180.0), (20.0, 230.0)] {
                c.fill(|x, y| rbox(x, y, indent, y0 + 16.0, 1.5, 22.0, 1.0), hex(0x3A3F47));
                c.fill(|x, y| rbox(x, y, indent + 60.0, y0, 46.0, 4.0, 3.0), hex(0x343840));
                c.fill(|x, y| triangle(x, y, (indent - 6.0, y0 + 2.0), (indent + 6.0, y0 + 2.0), (indent, y0 - 8.0)), faint(0xFF4500, 0.55));
            }
        }
        Platform::InstaSlop => {
            c.fill(|_, _| -1.0, hex(0x2B2433));
            let tints = [0xF7C6D9, 0xFBD9B9, 0xD9CCF5, 0xC9E8F2];
            for i in 0..2 {
                for j in 0..2 {
                    let t = tints[(i * 2 + j) % 4];
                    let (cx, cy) = (64.0 + 128.0 * i as f32, 64.0 + 128.0 * j as f32);
                    c.fill(|x, y| rbox(x, y, cx, cy, 58.0, 58.0, 4.0), faint(t, 0.1));
                }
            }
        }
        Platform::ClockTok => {
            c.fill(|_, _| -1.0, hex(0x221E2D));
            c.shape(|x, y| rbox(x, y, 128.0, 128.0, 56.0, 100.0, 18.0), alpha(WHITE, 0.03), alpha(WHITE, 0.1), 3.0);
            c.fill(|x, y| circle(x, y, 186.0, 200.0, 7.0), faint(0x25F4EE, 0.35));
            c.fill(|x, y| circle(x, y, 192.0, 196.0, 7.0), faint(0xFE2C55, 0.35));
        }
        // The later chapters' biomes are drawn from a palette and a motif.
        _ => {
            let p = platform.palette();
            c.fill(|_, _| -1.0, hex(p.bg));
            match p.motif {
                Motif::Panels => {
                    for (cx, cy) in [(64.0, 64.0), (192.0, 64.0), (64.0, 192.0), (192.0, 192.0)] {
                        c.shape(|x, y| rbox(x, y, cx, cy, 52.0, 52.0, 6.0), hex(p.panel), faint(p.accent, 0.35), 2.0);
                    }
                    c.fill(|x, y| rbox(x, y, 64.0, 40.0, 40.0, 4.0, 2.0), faint(p.accent, 0.4));
                }
                Motif::Grid => {
                    c.paint(|x, y| ((x as i32 % 32 == 0) || (y as i32 % 32 == 0)).then_some(faint(p.panel, 1.0)));
                    c.fill(|x, y| rbox(x, y, 128.0, 128.0, 40.0, 24.0, 4.0), faint(p.panel, 0.9));
                    c.fill(|x, y| rbox(x, y, 128.0, 120.0, 28.0, 3.0, 1.0), faint(p.accent, 0.5));
                }
                Motif::Tiles => {
                    c.paint(|x, y| {
                        let (i, j) = ((x / 64.0) as i32, (y / 64.0) as i32);
                        ((i + j) % 2 == 0).then_some(faint(p.panel, 1.0))
                    });
                    c.fill(|x, y| circle(x, y, 96.0, 96.0, 10.0), faint(p.accent, 0.3));
                }
                Motif::Stripes => {
                    c.paint(|x, y| (((x + y) / 24.0) as i32 % 2 == 0).then_some(faint(p.panel, 1.0)));
                    c.fill(|x, y| rbox(x, y, 128.0, 128.0, 110.0, 6.0, 3.0), faint(p.accent, 0.35));
                }
                Motif::Dots => {
                    for i in 0..4 {
                        for j in 0..4 {
                            c.fill(|x, y| circle(x, y, 32.0 + 64.0 * i as f32, 32.0 + 64.0 * j as f32, 5.0), faint(p.accent, 0.25));
                        }
                    }
                    c.fill(|x, y| rbox(x, y, 128.0, 128.0, 60.0, 36.0, 8.0), faint(p.panel, 0.8));
                }
                Motif::Rings => {
                    for r in [30.0, 60.0, 90.0] {
                        c.fill(|x, y| circle(x, y, 128.0, 128.0, r).abs() - 2.0, faint(p.accent, 0.25));
                    }
                    c.fill(|x, y| circle(x, y, 128.0, 128.0, 12.0), faint(p.panel, 1.0));
                }
            }
        }
    }
    c
}

pub fn shot(look: Look) -> Canvas {
    let mut c;
    match look {
        Look::Rock => {
            c = Canvas::new(40, 40);
            c.shape(|x, y| circle(x, y, 18.0, 21.0, 12.0).min(circle(x, y, 25.0, 17.0, 9.0)), hex(0x9A9A9A), INK, 2.5);
        }
        Look::Bullet => {
            c = Canvas::new(32, 16);
            c.shape(|x, y| capsule(x, y, 6.0, 8.0, 26.0, 8.0, 5.0), hex(0xFFD24A), INK, 2.0);
        }
        Look::Pen => {
            c = Canvas::new(48, 16);
            c.shape(|x, y| capsule(x, y, 6.0, 8.0, 34.0, 8.0, 5.0), hex(RED), INK, 2.0);
            c.shape(|x, y| capsule(x, y, 34.0, 8.0, 43.0, 8.0, 2.5), hex(0x3A3D4D), INK, 1.0);
        }
        Look::EyeRoll => {
            c = Canvas::new(44, 44);
            c.shape(|x, y| circle(x, y, 22.0, 22.0, 18.0), WHITE, INK, 2.5);
            c.fill(|x, y| circle(x, y, 22.0, 10.0, 7.0).max(circle(x, y, 22.0, 22.0, 16.0)), hex(0x4A7BD0));
        }
        Look::EmDash => {
            c = Canvas::new(96, 20);
            c.shape(|x, y| capsule(x, y, 8.0, 10.0, 88.0, 10.0, 6.0), hex(0x2A2240), DANGER, 3.0);
        }
        Look::Book => {
            c = Canvas::new(44, 36);
            c.shape(|x, y| rbox(x, y, 22.0, 18.0, 18.0, 14.0, 2.0), hex(0x8A5A34), INK, 2.5);
            c.fill(|x, y| rbox(x, y, 24.0, 18.0, 13.0, 10.0, 1.0), hex(0xF4EEDF));
        }
        Look::Brick => {
            c = Canvas::new(44, 28);
            c.shape(|x, y| rbox(x, y, 22.0, 14.0, 19.0, 10.0, 2.0), hex(0xB5523B), INK, 2.5);
            c.fill(|x, y| rbox(x, y, 22.0, 14.0, 17.0, 0.8, 0.5), alpha(WHITE, 0.6));
        }
        Look::Staple => {
            c = Canvas::new(24, 12);
            c.shape(|x, y| capsule(x, y, 4.0, 6.0, 20.0, 6.0, 3.0), hex(0xC8CCD4), INK, 1.5);
        }
        Look::Frisbee => {
            c = Canvas::new(44, 44);
            c.shape(|x, y| circle(x, y, 22.0, 22.0, 18.0), hex(0x6CCB5F), INK, 2.5);
            c.fill(|x, y| circle(x, y, 22.0, 22.0, 9.0).abs() - 1.5, alpha(WHITE, 0.7));
        }
        Look::Captcha => {
            c = Canvas::new(32, 32);
            c.shape(|x, y| rbox(x, y, 16.0, 16.0, 12.0, 12.0, 2.0), WHITE, INK, 2.0);
            c.fill(|x, y| capsule(x, y, 10.0, 16.0, 14.0, 21.0, 2.0).min(capsule(x, y, 14.0, 21.0, 23.0, 10.0, 2.0)), hex(0x2F80ED));
        }
        Look::Toast => {
            c = Canvas::new(36, 36);
            c.shape(|x, y| rbox(x, y, 18.0, 18.0, 13.0, 13.0, 5.0), hex(0xE6B87A), DANGER, 3.0);
            c.fill(|x, y| rbox(x, y, 18.0, 19.0, 8.0, 8.0, 3.0), hex(0xF6DDB0));
        }
        Look::Arrow => {
            c = Canvas::new(56, 32);
            c.shape(
                |x, y| capsule(x, y, 6.0, 16.0, 36.0, 16.0, 5.0).min(triangle(x, y, (32.0, 4.0), (32.0, 28.0), (52.0, 16.0))),
                hex(RED),
                DANGER,
                3.0,
            );
        }
        Look::Letter => {
            c = Canvas::new(48, 36);
            c.shape(|x, y| rbox(x, y, 24.0, 18.0, 20.0, 14.0, 3.0), WHITE, DANGER, 3.0);
            c.fill(|x, y| capsule(x, y, 8.0, 8.0, 24.0, 20.0, 1.2).min(capsule(x, y, 24.0, 20.0, 40.0, 8.0, 1.2)), INK);
        }
        Look::Duck => {
            c = Canvas::new(44, 40);
            c.shape(|x, y| ellipse(x, y, 24.0, 26.0, 17.0, 11.0).min(circle(x, y, 14.0, 14.0, 9.0)), hex(GOLD), INK, 2.5);
            c.fill(|x, y| triangle(x, y, (4.0, 13.0), (8.0, 18.0), (6.0, 11.0)), hex(0xF28C38));
            c.fill(|x, y| circle(x, y, 12.0, 12.0, 2.0), INK);
        }
        Look::Plane => {
            c = Canvas::new(44, 28);
            c.shape(|x, y| triangle(x, y, (4.0, 4.0), (40.0, 14.0), (4.0, 24.0)), WHITE, INK, 2.0);
            c.fill(|x, y| capsule(x, y, 6.0, 14.0, 36.0, 14.0, 0.8), alpha(INK, 0.5));
        }
        Look::Note => {
            c = Canvas::new(28, 28);
            c.shape(|x, y| rbox(x, y, 14.0, 14.0, 11.0, 11.0, 1.5), hex(0xF2E66B), INK, 2.0);
            c.fill(|x, y| rbox(x, y, 14.0, 12.0, 6.0, 1.0, 0.5), alpha(INK, 0.5));
        }
        // Enemy shots: outlined in the danger pink (FAIR-04).
        Look::Banana => {
            c = Canvas::new(40, 40);
            c.shape(|x, y| (circle(x, y, 26.0, 8.0, 24.0).abs() - 5.0).max(18.0 - y).max(x - 30.0), hex(0xF5D84A), DANGER, 2.5);
        }
        Look::Needle => {
            c = Canvas::new(30, 12);
            c.shape(|x, y| triangle(x, y, (2.0, 3.0), (28.0, 6.0), (2.0, 9.0)), hex(0x5DA34F), DANGER, 2.0);
        }
        Look::Shard => {
            c = Canvas::new(26, 26);
            c.shape(|x, y| triangle(x, y, (4.0, 22.0), (13.0, 2.0), (22.0, 18.0)), [1.0, 0.6, 0.7, 0.75], DANGER, 2.0);
        }
        Look::Bubble | Look::Post => c = bubble(),
    }
    c
}

/// A white speech-bubble frame, outlined in danger pink, drawn as nine slices.
pub fn bubble() -> Canvas {
    let mut c = Canvas::new(48, 40);
    c.shape(|x, y| rbox(x, y, 24.0, 20.0, 22.0, 18.0, 12.0), WHITE, DANGER, 3.5);
    c
}

/// Clout: a gold coin with a thumbs-up. Hearts are never money.
pub fn coin() -> Canvas {
    let mut p = Pen::new(48.0);
    p.circle(50.0, 50.0, 44.0, hex(GOLD));
    p.dot(50.0, 50.0, 34.0, hex(0xF7D35A));
    p.rect(40.0, 62.0, 10.0, 16.0, 4.0, WHITE);
    p.rect(60.0, 64.0, 14.0, 14.0, 5.0, WHITE);
    p.stick(56.0, 50.0, 62.0, 26.0, 7.0, WHITE);
    p.c
}

/// Mental Health: a brain.
pub fn brain() -> Canvas {
    let mut c = Canvas::new(48, 48);
    c.shape(
        |x, y| circle(x, y, 17.0, 20.0, 12.0).min(circle(x, y, 31.0, 20.0, 12.0)).min(circle(x, y, 14.0, 30.0, 10.0)).min(circle(x, y, 34.0, 30.0, 10.0)).min(circle(x, y, 24.0, 32.0, 11.0)),
        hex(0xF6A9C0),
        INK,
        2.5,
    );
    c.fill(|x, y| capsule(x, y, 24.0, 12.0, 24.0, 38.0, 1.2), alpha(INK, 0.6));
    c
}

pub fn shadow() -> Canvas {
    let mut c = Canvas::new(64, 24);
    c.fill(|x, y| ellipse(x, y, 32.0, 12.0, 30.0, 10.0), [0.0, 0.0, 0.0, 0.22]);
    c
}

/// The "typing..." indicator that warns of a spawn.
pub fn typing() -> Canvas {
    let mut c = Canvas::new(64, 40);
    c.shape(|x, y| rbox(x, y, 32.0, 20.0, 28.0, 16.0, 14.0), [0.93, 0.93, 0.95, 0.95], DANGER, 3.0);
    for dx in [20.0, 32.0, 44.0] {
        c.fill(|x, y| circle(x, y, dx, 20.0, 4.5), INK);
    }
    c
}

/// A fading wedge for swings and sprays, pointing right from the centre.
pub fn sector(half: f32, color: Rgba) -> Canvas {
    let mut c = Canvas::new(256, 256);
    c.paint(|x, y| {
        let d = wedge(x, y, 128.0, 128.0, 126.0, half);
        let cover = (0.5 - d).clamp(0.0, 1.0);
        let dist = ((x - 128.0).powi(2) + (y - 128.0).powi(2)).sqrt() / 126.0;
        (cover > 0.0).then_some(alpha(color, cover * dist.powf(1.5)))
    });
    c
}

/// A thin ring for pulses and outlines.
pub fn ring(color: Rgba) -> Canvas {
    let mut c = Canvas::new(128, 128);
    c.fill(|x, y| circle(x, y, 64.0, 64.0, 58.0).abs() - 3.5, color);
    c
}

/// A soft round dot for particles.
pub fn ball() -> Canvas {
    let mut c = Canvas::new(16, 16);
    c.fill(|x, y| circle(x, y, 8.0, 8.0, 7.0), WHITE);
    c
}

pub fn dot() -> Canvas {
    let mut c = Canvas::new(8, 8);
    c.fill(|_, _| -1.0, WHITE);
    c
}

/// The Dog Walk's dog.
pub fn dog() -> Canvas {
    let mut p = Pen::new(56.0);
    p.ellipse(22.0, 40.0, 10.0, 18.0, hex(0x7A4E2D));
    p.ellipse(78.0, 40.0, 10.0, 18.0, hex(0x7A4E2D));
    p.circle(50.0, 50.0, 30.0, hex(0xC98B54));
    p.ellipse(50.0, 66.0, 14.0, 10.0, hex(0xF0D2B0));
    p.dot(50.0, 60.0, 5.0, INK);
    p.dot(38.0, 44.0, 4.0, INK);
    p.dot(62.0, 44.0, 4.0, INK);
    p.c
}

pub fn halo() -> Canvas {
    let mut c = Canvas::new(128, 64);
    c.fill(|x, y| ellipse(x, y, 64.0, 32.0, 56.0, 20.0).abs() - 4.0, hex(GOLD));
    c
}

/// A beauty filter: a soft pink arc in front of an influencer.
pub fn shield() -> Canvas {
    let mut c = Canvas::new(128, 128);
    c.paint(|x, y| {
        let d = (circle(x, y, 64.0, 64.0, 56.0).abs() - 5.0).max(wedge(x, y, 64.0, 64.0, 70.0, 0.9));
        let cover = (0.5 - d).clamp(0.0, 1.0);
        (cover > 0.0).then_some([0.78, 0.9, 1.0, 0.8 * cover])
    });
    c
}

/// The spinning swirl shown while Brainrot is coming or active.
pub fn swirl() -> Canvas {
    let mut c = Canvas::new(64, 64);
    c.paint(|x, y| {
        let (dx, dy) = (x - 32.0, y - 32.0);
        let r = (dx * dx + dy * dy).sqrt();
        let a = dy.atan2(dx);
        let band = ((r / 5.0 - a / std::f32::consts::PI * 1.2).rem_euclid(2.0) - 1.0).abs();
        (r < 28.0 && band < 0.45).then_some([0.62, 0.36, 1.0, 1.0])
    });
    c
}

pub fn seed_patch() -> Canvas {
    let mut c = Canvas::new(128, 128);
    c.fill(|x, y| circle(x, y, 64.0, 64.0, 60.0), alpha(hex(0x6FCB5A), 0.35));
    for (x0, y0) in [(40.0, 50.0), (80.0, 44.0), (60.0, 80.0), (90.0, 82.0), (34.0, 84.0)] {
        c.fill(|x, y| triangle(x, y, (x0 - 5.0, y0 + 6.0), (x0 + 5.0, y0 + 6.0), (x0, y0 - 8.0)), alpha(hex(0x3F9A3A), 0.8));
    }
    c
}

/// A patch of sunflowers (the Sunflower Patch and its merge).
pub fn sunflower_patch() -> Canvas {
    let mut c = Canvas::new(128, 128);
    c.fill(|x, y| circle(x, y, 64.0, 64.0, 60.0), alpha(hex(0x6FCB5A), 0.28));
    for (x0, y0, r) in [(42.0, 46.0, 13.0), (84.0, 40.0, 11.0), (64.0, 80.0, 14.0), (92.0, 84.0, 10.0), (32.0, 86.0, 10.0), (64.0, 26.0, 9.0)] {
        c.fill(|x, y| capsule(x, y, x0, y0, x0, y0 + r * 1.6, r * 0.18), hex(0x3F9A3A));
        for k in 0..8 {
            let a = k as f32 * std::f32::consts::TAU / 8.0;
            let (px, py) = (x0 + a.cos() * r * 0.75, y0 + a.sin() * r * 0.75);
            c.shape(|x, y| circle(x, y, px, py, r * 0.42), hex(GOLD), INK, 1.2);
        }
        c.shape(|x, y| circle(x, y, x0, y0, r * 0.5), hex(0x7A4E2D), INK, 1.2);
    }
    c
}

/// Fire on the floor (Flame War, burning dashes).
pub fn fire() -> Canvas {
    let mut c = Canvas::new(64, 64);
    c.paint(|x, y| {
        let r = ((x - 32.0).powi(2) + (y - 36.0).powi(2)).sqrt() / 28.0;
        (r < 1.0).then(|| {
            let hot = 1.0 - r;
            [1.0, 0.45 + 0.5 * hot, 0.1 + 0.2 * hot * hot, 0.75 * (1.0 - r * r)]
        })
    });
    c
}

/// A brainrot zone (ClockTok): a purple disco floor with a spiral.
pub fn brainrot_zone() -> Canvas {
    let mut c = Canvas::new(128, 128);
    c.paint(|x, y| {
        let (dx, dy) = (x - 64.0, y - 64.0);
        let r = (dx * dx + dy * dy).sqrt() / 60.0;
        (r < 1.0).then(|| {
            let a = dy.atan2(dx);
            let band = ((a * 3.0 + r * 14.0).sin() * 0.5 + 0.5) * 0.25;
            [0.62, 0.36, 1.0, 0.2 + band]
        })
    });
    c.fill(|x, y| circle(x, y, 64.0, 64.0, 60.0).abs() - 2.5, [0.62, 0.36, 1.0, 0.95]);
    c
}

/// Roots stomped into the floor by Brr Brr Patapum.
pub fn roots() -> Canvas {
    let mut c = Canvas::new(128, 128);
    c.fill(|x, y| circle(x, y, 64.0, 64.0, 60.0), alpha(hex(0x7A4E2D), 0.25));
    for k in 0..7 {
        let a = k as f32 * std::f32::consts::TAU / 7.0;
        let (ex, ey) = (64.0 + a.cos() * 56.0, 64.0 + a.sin() * 56.0);
        c.fill(|x, y| capsule(x, y, 64.0, 64.0, ex, ey, 4.0), alpha(hex(0x5A3A22), 0.8));
    }
    c
}

/// A pillar of light over a rare drop, seen from far (LOOT-02).
pub fn light_beam() -> Canvas {
    let mut c = Canvas::new(32, 256);
    c.paint(|x, y| {
        let across = 1.0 - ((x - 16.0).abs() / 16.0);
        let up = y / 256.0;
        Some([1.0, 0.9, 0.5, across * across * up * 0.8])
    });
    c
}

/// Where a bomb will land (FAIR-02).
pub fn bomb_marker() -> Canvas {
    let mut c = Canvas::new(128, 128);
    c.fill(|x, y| circle(x, y, 64.0, 64.0, 60.0), alpha(DANGER, 0.18));
    c.fill(|x, y| circle(x, y, 64.0, 64.0, 58.0).abs() - 3.0, DANGER);
    c.fill(|x, y| capsule(x, y, 64.0, 34.0, 64.0, 94.0, 2.0).min(capsule(x, y, 34.0, 64.0, 94.0, 64.0, 2.0)), DANGER);
    c
}

pub fn downvote() -> Canvas {
    let mut c = Canvas::new(128, 128);
    c.fill(|x, y| circle(x, y, 64.0, 64.0, 60.0), alpha(hex(0x7193FF), 0.25));
    c.fill(|x, y| triangle(x, y, (44.0, 60.0), (84.0, 60.0), (64.0, 88.0)).min(rbox(x, y, 64.0, 50.0, 8.0, 12.0, 2.0)), alpha(hex(0x4A6FE0), 0.55));
    c
}

pub fn filter() -> Canvas {
    let mut c = Canvas::new(128, 128);
    c.paint(|x, y| {
        let r = ((x - 64.0).powi(2) + (y - 64.0).powi(2)).sqrt() / 60.0;
        (r < 1.0).then(|| {
            let t = (x + y) / 256.0;
            [1.0, 0.7 + 0.2 * t, 0.85 - 0.2 * t, 0.28 * (1.0 - r * r)]
        })
    });
    c
}

/// A thin ring for big circles: rings scale their line with their size, so
/// a big one needs a fine line, or it would cover the Feed (FAIR-05).
pub fn thin_ring() -> Canvas {
    let mut c = Canvas::new(1024, 1024);
    c.fill(|x, y| circle(x, y, 512.0, 512.0, 506.0).abs() - 2.5, WHITE);
    c
}
