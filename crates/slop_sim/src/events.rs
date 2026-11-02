//! What happened in a tick, for the client's effects and sounds. The
//! simulation never reads its own events.

use crate::content::SlopKind;
use crate::stats::DamageType;
use crate::vec::V2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    /// A slop took damage.
    SlopHit { at: V2, amount: f32, kind: DamageType },
    /// A slop died ("deleted").
    SlopDied { at: V2, kind: SlopKind },
    /// A slop will appear here in `SPAWN_WARNING` ticks.
    SpawnWarning { at: V2, kind: SlopKind },
    /// You lost Mental Health.
    PlayerHurt { amount: f32 },
    /// Scepticism ignored a hit.
    Dodged,
    /// A melee swing or a spray: drawn as a fading sector.
    Swing { at: V2, angle: f32, range: f32, half_angle: f32, weapon: u8 },
    /// A weapon fired a projectile.
    Fired { weapon: u8 },
    /// One of your projectiles popped a speech bubble.
    BubblePopped { at: V2, line: u16 },
    /// A Yes-Bot agreed with nearby slop.
    Agreed { at: V2, line: u16 },
    /// Number 1 of a Listicle died and the pack scattered.
    Scattered { at: V2 },
    /// A Six-Finger Hand grabbed you.
    Grabbed,
    /// An Em-Dash Lancer is aiming from `from` along `dir`.
    LancerAim { from: V2, dir: V2 },
    LikeCollected { value: u32 },
    WaveCleared,
    Burnout,
}
