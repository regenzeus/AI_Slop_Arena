//! One input layer for the keyboard and gamepads, such as an Xbox 360
//! controller (UI-17). Everything else reads `Controls`, never the devices.
//!
//! ```text
//! Keyboard                     Gamepad
//! WASD / arrows   move, choose  left stick / d-pad
//! Enter / E       press         A
//! Space           continue      A (menus), Start (pause)
//! Esc / P         pause, back   Start / B
//! L / X           lock, other   X
//! R               reroll        Y
//! 1-8             pick          -
//! ```

use bevy::input::gamepad::{Gamepad, GamepadButton};
use bevy::input::InputSystems;
use bevy::prelude::*;

/// Stick deflection that counts as a menu move, and the repeat timing.
const FLICK: f32 = 0.6;
const REPEAT_FIRST: f32 = 0.35;
const REPEAT_NEXT: f32 = 0.12;
/// Below this the stick is ignored for movement.
const DEADZONE: f32 = 0.18;

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Controls>().add_systems(PreUpdate, read.after(InputSystems));
    }
}

#[derive(Resource, Default, Debug)]
pub struct Controls {
    /// Movement wanted, up to length 1, y up.
    pub move_dir: Vec2,
    /// A menu direction this frame, in UI coordinates (y down).
    pub nav: Option<Vec2>,
    /// Press the highlighted thing.
    pub confirm: bool,
    /// The screen's "continue" button.
    pub cont: bool,
    pub back: bool,
    pub pause: bool,
    /// Lock or the other action of the highlighted thing.
    pub alt: bool,
    pub reroll: bool,
    pub number: Option<usize>,
    /// Alt is held (Alt+Enter is fullscreen, not a press).
    pub alt_key: bool,
    /// A gamepad was used last, for the hint bar.
    pub pad: bool,
    stick_timer: f32,
    stick_dir: Option<Vec2>,
}

const NUMBER_KEYS: [KeyCode; 8] = [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4, KeyCode::Digit5, KeyCode::Digit6, KeyCode::Digit7, KeyCode::Digit8];

fn read(time: Res<Time>, keys: Res<ButtonInput<KeyCode>>, pads: Query<&Gamepad>, mut c: ResMut<Controls>) {
    let held = |a: KeyCode, b: KeyCode| keys.pressed(a) || keys.pressed(b);
    let tap = |a: KeyCode, b: KeyCode| keys.just_pressed(a) || keys.just_pressed(b);
    let mut mv = Vec2::ZERO;
    if held(KeyCode::KeyA, KeyCode::ArrowLeft) {
        mv.x -= 1.0;
    }
    if held(KeyCode::KeyD, KeyCode::ArrowRight) {
        mv.x += 1.0;
    }
    if held(KeyCode::KeyW, KeyCode::ArrowUp) {
        mv.y += 1.0;
    }
    if held(KeyCode::KeyS, KeyCode::ArrowDown) {
        mv.y -= 1.0;
    }
    let mut nav = if tap(KeyCode::ArrowLeft, KeyCode::KeyA) {
        Some(Vec2::NEG_X)
    } else if tap(KeyCode::ArrowRight, KeyCode::KeyD) {
        Some(Vec2::X)
    } else if tap(KeyCode::ArrowUp, KeyCode::KeyW) {
        Some(Vec2::NEG_Y)
    } else if tap(KeyCode::ArrowDown, KeyCode::KeyS) {
        Some(Vec2::Y)
    } else {
        None
    };
    let alt_key = keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);
    let mut confirm = !alt_key && keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter, KeyCode::KeyE]);
    let mut cont = keys.just_pressed(KeyCode::Space);
    let mut back = keys.just_pressed(KeyCode::Escape);
    let mut pause = keys.any_just_pressed([KeyCode::KeyP, KeyCode::Escape]);
    let mut alt = keys.any_just_pressed([KeyCode::KeyL, KeyCode::KeyX]);
    let mut reroll = keys.just_pressed(KeyCode::KeyR);
    let keyboard_used = keys.get_just_pressed().next().is_some();

    let mut pad_used = false;
    let mut stick = Vec2::ZERO;
    for pad in &pads {
        let s = pad.left_stick();
        let d = pad.dpad();
        if s.length() > DEADZONE {
            stick = s;
        }
        if d.length() > 0.5 {
            stick = d;
        }
        let just = |b: GamepadButton| pad.just_pressed(b);
        if just(GamepadButton::DPadLeft) {
            nav = Some(Vec2::NEG_X);
        } else if just(GamepadButton::DPadRight) {
            nav = Some(Vec2::X);
        } else if just(GamepadButton::DPadUp) {
            nav = Some(Vec2::NEG_Y);
        } else if just(GamepadButton::DPadDown) {
            nav = Some(Vec2::Y);
        }
        confirm |= just(GamepadButton::South);
        back |= just(GamepadButton::East);
        pause |= just(GamepadButton::Start);
        cont |= just(GamepadButton::Start);
        alt |= just(GamepadButton::West);
        reroll |= just(GamepadButton::North);
        pad_used |= pad.get_just_pressed().next().is_some() || s.length() > 0.5;
    }
    // A flick of the stick moves through menus, repeating while held.
    let flick = (stick.length() > FLICK).then(|| if stick.x.abs() > stick.y.abs() { Vec2::new(stick.x.signum(), 0.0) } else { Vec2::new(0.0, -stick.y.signum()) });
    match flick {
        Some(dir) if c.stick_dir != Some(dir) => {
            nav = nav.or(Some(dir));
            c.stick_dir = Some(dir);
            c.stick_timer = REPEAT_FIRST;
        }
        Some(dir) => {
            c.stick_timer -= time.delta_secs();
            if c.stick_timer <= 0.0 {
                nav = nav.or(Some(dir));
                c.stick_timer = REPEAT_NEXT;
            }
        }
        None => c.stick_dir = None,
    }
    if stick != Vec2::ZERO {
        // The stick's own strength sets the pace; the keys always run.
        mv = if mv == Vec2::ZERO { stick } else { mv };
    }
    c.move_dir = mv.clamp_length_max(1.0);
    c.nav = nav;
    c.confirm = confirm;
    c.cont = cont;
    c.back = back;
    c.pause = pause;
    c.alt = alt;
    c.reroll = reroll;
    c.alt_key = alt_key;
    c.number = NUMBER_KEYS.iter().position(|k| keys.just_pressed(*k));
    if pad_used {
        c.pad = true;
    } else if keyboard_used {
        c.pad = false;
    }
}
