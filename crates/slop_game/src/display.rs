//! The window (UI-16): fullscreen by default, Alt+Enter or F11 to switch to
//! a window and back, F12 or Print Screen to save a screenshot to
//! Pictures\AI Slop Arena.

use std::path::PathBuf;

use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured};
use bevy::window::{MonitorSelection, PrimaryWindow, WindowMode};

use crate::state::Game;

pub struct DisplayPlugin;

impl Plugin for DisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (toggle_fullscreen, screenshot));
    }
}

pub const FULLSCREEN: WindowMode = WindowMode::BorderlessFullscreen(MonitorSelection::Current);

fn toggle_fullscreen(keyboard: Res<ButtonInput<KeyCode>>, mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    let alt = keyboard.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);
    let enter = keyboard.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter]);
    if !(keyboard.just_pressed(KeyCode::F11) || alt && enter) {
        return;
    }
    if let Ok(mut w) = windows.single_mut() {
        w.mode = if w.mode == WindowMode::Windowed { FULLSCREEN } else { WindowMode::Windowed };
    }
}

/// Pictures\AI Slop Arena, or the working directory without a home.
fn folder() -> PathBuf {
    let home = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")).map(PathBuf::from);
    home.map_or_else(|| PathBuf::from("screenshots"), |h| h.join("Pictures").join("AI Slop Arena"))
}

fn screenshot(mut commands: Commands, keyboard: Res<ButtonInput<KeyCode>>, mut raw: MessageReader<KeyboardInput>) {
    // Windows reports only the release of Print Screen, and a release without
    // a press never reaches `ButtonInput`: read the raw key events instead.
    let print = raw.read().any(|k| k.key_code == KeyCode::PrintScreen && k.state == ButtonState::Released);
    if !keyboard.just_pressed(KeyCode::F12) && !print {
        return;
    }
    let dir = folder();
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("cannot create {}: {e}", dir.display());
        return;
    }
    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis());
    let path = dir.join(format!("slop-{stamp}.png"));
    let shown = path.display().to_string();
    // The message comes after the capture, so it is not in the picture.
    commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path)).observe(move |_: On<ScreenshotCaptured>, mut game: ResMut<Game>| {
        game.say(format!("Screenshot saved: {shown}"));
    });
}
