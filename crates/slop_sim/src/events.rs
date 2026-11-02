//! What happened in a tick, for the client's effects and sounds. The
//! simulation never reads its own events.

use crate::content::BossKind;
use crate::stats::DamageType;
use crate::vec::V2;
use crate::world::Active;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusKind {
    Anxiety,
    Doomscroll,
    Popup,
    Ratio,
    Brainrot,
    /// Brainrot is coming: shown above you before it lands.
    BrainrotWarning,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    /// A slop took damage.
    SlopHit { at: V2, amount: f32, kind: DamageType },
    /// A slop died ("deleted").
    SlopDied { at: V2, spec: u16 },
    /// A slop will appear here in `SPAWN_WARNING` ticks.
    SpawnWarning { at: V2 },
    /// A brainrot or elite appeared and shouts its name.
    Named { at: V2, spec: u16 },
    /// You lost Mental Health.
    PlayerHurt { amount: f32 },
    /// Scepticism ignored a hit.
    Dodged,
    /// A slop (a cat) dodged its first hit.
    SlopDodged { at: V2 },
    /// An item blocked a hit or a bubble.
    Blocked,
    /// Good Night's Sleep saved you from Burnout.
    SecondWind,
    /// A melee swing or a spray: drawn as a fading sector.
    Swing { at: V2, angle: f32, range: f32, half_angle: f32, weapon: u8 },
    /// A weapon fired.
    Fired { weapon: u8 },
    /// A fact-check beam.
    Beam { from: V2, to: V2 },
    /// One link of Air Quotes or a chain letter.
    Chain { from: V2, to: V2 },
    /// Something exploded or landed with a splash.
    Splash { at: V2, radius: f32 },
    /// A Ban Hammer slam.
    Slam { at: V2, radius: f32 },
    /// A ring around you: keyboard clacks or a slow clap.
    Ring { at: V2, radius: f32 },
    /// An Uno Reverse turned a bubble around.
    Reflected { at: V2 },
    /// One of your projectiles popped a speech bubble.
    BubblePopped { at: V2, line: u16 },
    /// A Yes-Bot agreed with nearby slop.
    Agreed { at: V2, line: u16 },
    /// A cake healed slop around it.
    Healed { at: V2 },
    /// Number 1 of a Listicle pack died and the pack scattered.
    Scattered { at: V2 },
    /// A dead slop came back as a repost.
    Reposted { at: V2 },
    /// A Recruiter Bot split in two.
    Split { at: V2 },
    /// A Surprise Egg hatched.
    Hatched { at: V2 },
    /// A Six-Finger Hand grabbed you.
    Grabbed,
    /// Clawed is aiming from `from` along `dir`.
    LancerAim { from: V2, dir: V2 },
    /// A bomb will land here.
    BombWarning { at: V2, radius: f32 },
    /// A status effect landed on you.
    Status { kind: StatusKind },
    /// One of your actives fired.
    ActiveUsed { active: Active },
    /// Phone a Friend: a friend ran from `from` to `to`.
    Friend { from: V2, to: V2 },
    /// An elite dropped a Loot Box for the next shop.
    LootBox { at: V2 },
    /// A mid-roll ad started (MeTube).
    AdBreak,
    /// The beat dropped (ClockTok).
    Beat,
    BossIntro { kind: BossKind },
    BossPhase { kind: BossKind, phase: u8 },
    /// The Hustle Guru made everything faster.
    Hustle { level: f32 },
    /// The Model tried "Regenerate response".
    Regenerate { at: V2 },
    LikeCollected { value: u32 },
    WaveCleared,
    Burnout,
}
