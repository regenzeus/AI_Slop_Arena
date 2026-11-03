//! Each platform's hazard (PLAT-02): chain letters, banners, ads, downvote
//! zones, filter zones and the beat. And on every platform, maintenance:
//! parts of the map are fenced off for a while (MAP-04).

use crate::content::{Look, Platform};
use crate::events::{Event, StatusKind};
use crate::vec::V2;
use crate::world::*;

/// Ticks between beats on ClockTok (100 beats per minute).
pub const BEAT_INTERVAL: u16 = 36;
const LETTER_SPEED: f32 = 170.0;
const LETTER_DAMAGE: f32 = 8.0;
const BANNER_SPEED: f32 = 150.0;
const AD_TICKS: u16 = 180;
/// Ticks between maintenance fences, and their warning.
const MAINTENANCE_EVERY: u32 = 60 * 50;
const FENCE_WARNING: u16 = 150;

impl World {
    pub(crate) fn hazards(&mut self) {
        let platform = self.setup.platform;
        self.beat = self.beat.saturating_add(1);
        if platform == Some(Platform::ClockTok) && self.beat >= BEAT_INTERVAL {
            self.beat = 0;
            self.events.push(Event::Beat);
        }
        let Some(platform) = platform else { return };
        if self.setup.seconds <= 0.0 && self.tick % MAINTENANCE_EVERY == MAINTENANCE_EVERY / 2 {
            self.maintenance();
        }
        self.hazard_timer -= 1;
        if self.hazard_timer > 0 {
            return;
        }
        let me = self.player.pos;
        self.hazard_timer = match platform {
            Platform::FaceSlop => {
                let edge = self.near(me, 880.0, 940.0);
                let dir = (me - edge).normalized();
                self.hostile_shot(edge, dir * LETTER_SPEED, LETTER_DAMAGE, Look::Letter, NO_LINE, NO_SPEC);
                420
            }
            Platform::LinkedOut => {
                let below = self.rng_ai.chance(0.5);
                let y = me.y + if below { -620.0 } else { 620.0 };
                let id = self.id();
                let mut z = Zone::new(id, ZoneKind::Banner, V2::new(me.x, y), 45.0, 1200);
                z.shape = Shape::Rect(V2::new(3000.0, 45.0));
                z.vel = V2::new(0.0, if below { BANNER_SPEED } else { -BANNER_SPEED });
                self.zones.push(z);
                600
            }
            Platform::MeTube => {
                self.ad_break = AD_TICKS;
                self.events.push(Event::AdBreak);
                self.apply_status(StatusKind::Popup);
                900
            }
            Platform::Thredd => {
                // Downvotes come in every shape: puddles, strips and rings.
                for _ in 0..3 {
                    let at = self.near(me, 180.0, 520.0);
                    let id = self.id();
                    let mut z = Zone::new(id, ZoneKind::Downvote, at, 90.0, 480);
                    z.shape = match self.rng_ai.below(3) {
                        0 => Shape::Circle,
                        1 => {
                            let long = self.rng_ai.range(150.0, 260.0);
                            if self.rng_ai.chance(0.5) {
                                Shape::Rect(V2::new(long, 40.0))
                            } else {
                                Shape::Rect(V2::new(40.0, long))
                            }
                        }
                        _ => {
                            z.radius = 170.0;
                            Shape::Ring(120.0)
                        }
                    };
                    self.zones.push(z);
                }
                480
            }
            Platform::InstaSlop => {
                for _ in 0..2 {
                    let at = self.near(me, 250.0, 600.0);
                    let id = self.id();
                    let mut z = Zone::new(id, ZoneKind::Filter, at, 150.0, 600);
                    if self.rng_ai.chance(0.5) {
                        // A square photo frame.
                        z.shape = Shape::Rect(V2::new(140.0, 140.0));
                    }
                    self.zones.push(z);
                }
                600
            }
            Platform::ClockTok => {
                self.apply_status(StatusKind::BrainrotWarning);
                1500
            }
        };
    }

    /// Fences off a block of the map ahead of you, after a warning. It
    /// never closes on top of you: it is placed beside where you go.
    fn maintenance(&mut self) {
        let me = self.player.pos;
        let heading = if self.player.vel.length_sq() > 100.0 { self.player.vel.normalized() } else { self.player.facing };
        let half = V2::new(self.rng_ai.range(140.0, 300.0), self.rng_ai.range(120.0, 260.0));
        let side = if self.rng_ai.chance(0.5) { 1.0 } else { -1.0 };
        let at = me + heading * self.rng_ai.range(380.0, 560.0) + heading.perp() * (side * self.rng_ai.range(0.0, 220.0));
        let gap = V2::new(half.x + PLAYER_RADIUS + 60.0, half.y + PLAYER_RADIUS + 60.0);
        if (me.x - at.x).abs() < gap.x && (me.y - at.y).abs() < gap.y {
            return;
        }
        let ticks = 60 * (14 + self.rng_ai.below(12)) as u16;
        self.map.add_fence(at, half, FENCE_WARNING, ticks);
        self.events.push(Event::Maintenance { at, half });
    }

    /// A random point between `min` and `max` from `at`.
    fn near(&mut self, at: V2, min: f32, max: f32) -> V2 {
        let angle = self.rng_ai.range(0.0, std::f32::consts::TAU);
        at + V2::from_angle(angle) * self.rng_ai.range(min, max)
    }
}
