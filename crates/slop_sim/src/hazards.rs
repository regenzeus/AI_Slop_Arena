//! Each platform's hazard (PLAT-02): chain letters, banners, ads, downvote
//! zones, filter zones and the beat.

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

impl World {
    pub(crate) fn hazards(&mut self) {
        let platform = self.setup.platform;
        self.beat = self.beat.saturating_add(1);
        if platform == Some(Platform::ClockTok) && self.beat >= BEAT_INTERVAL {
            self.beat = 0;
            self.events.push(Event::Beat);
        }
        let Some(platform) = platform else { return };
        self.hazard_timer -= 1;
        if self.hazard_timer > 0 {
            return;
        }
        let me = self.player.pos;
        self.hazard_timer = match platform {
            Platform::FaceSlop => {
                let edge = self.edge_point();
                let dir = (me - edge).normalized();
                self.hostile_shot(edge, dir * LETTER_SPEED, LETTER_DAMAGE, Look::Letter, NO_LINE, NO_SPEC);
                420
            }
            Platform::LinkedOut => {
                let below = self.rng_ai.chance(0.5);
                let y = (me.y + if below { -620.0 } else { 620.0 }).clamp(0.0, ARENA);
                let id = self.id();
                let vel = V2::new(0.0, if below { BANNER_SPEED } else { -BANNER_SPEED });
                self.zones.push(Zone { id, kind: ZoneKind::Banner, pos: V2::new(ARENA / 2.0, y), radius: 45.0, ticks: 1200, max_ticks: 1200, vel, damage: 0.0, spec: NO_SPEC });
                600
            }
            Platform::MeTube => {
                self.ad_break = AD_TICKS;
                self.events.push(Event::AdBreak);
                self.apply_status(StatusKind::Popup);
                900
            }
            Platform::Thredd => {
                for _ in 0..3 {
                    let at = self.near(me, 180.0, 480.0);
                    let id = self.id();
                    self.zones.push(Zone { id, kind: ZoneKind::Downvote, pos: at, radius: 90.0, ticks: 480, max_ticks: 480, vel: V2::ZERO, damage: 0.0, spec: NO_SPEC });
                }
                480
            }
            Platform::InstaSlop => {
                for _ in 0..2 {
                    let at = self.near(me, 250.0, 600.0);
                    let id = self.id();
                    self.zones.push(Zone { id, kind: ZoneKind::Filter, pos: at, radius: 150.0, ticks: 600, max_ticks: 600, vel: V2::ZERO, damage: 0.0, spec: NO_SPEC });
                }
                600
            }
            Platform::ClockTok => {
                self.apply_status(StatusKind::BrainrotWarning);
                1500
            }
        };
    }

    /// A random point between `min` and `max` from `at`, inside the arena.
    fn near(&mut self, at: V2, min: f32, max: f32) -> V2 {
        let angle = self.rng_ai.range(0.0, std::f32::consts::TAU);
        clamp_to_arena(at + V2::from_angle(angle) * self.rng_ai.range(min, max), 60.0)
    }

    /// A random point on the arena's edge.
    fn edge_point(&mut self) -> V2 {
        let t = self.rng_ai.range(0.0, ARENA);
        match self.rng_ai.below(4) {
            0 => V2::new(t, 10.0),
            1 => V2::new(t, ARENA - 10.0),
            2 => V2::new(10.0, t),
            _ => V2::new(ARENA - 10.0, t),
        }
    }
}
