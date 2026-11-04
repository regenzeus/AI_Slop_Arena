//! AI Slop Arena desktop client.
//!
//! ```text
//! slop_game                          play
//! slop_game --autoplay               watch the bots play a whole run
//!   --seed N                         choose the run's seed
//!   --screenshots DIR                save a screenshot of every kind of screen
//!   --exit-on-end                    close when the run ends
//!   --speed N                        autoplay fast-forward (default 8)
//! slop_game --windowed               start in a window (autoplay always does)
//!   --size WxH                       the window's size (default 1600x900)
//! ```
//!
//! Alt+Enter or F11 switches between fullscreen and a window; F12 saves a
//! screenshot.

mod art;
mod assets;
mod audio;
mod autoplay;
mod controls;
mod display;
mod draw;
mod feed_art;
mod fx;
mod hud;
mod icons;
mod music;
mod play;
mod screens;
mod state;
mod theme;
mod ui;
mod views;
mod winput;

use std::path::PathBuf;

use bevy::prelude::*;
use bevy::window::{WindowMode, WindowResolution};

use crate::state::Game;

struct Args {
    autoplay: bool,
    exit_on_end: bool,
    screenshots: Option<PathBuf>,
    seed: Option<u64>,
    speed: u32,
    windowed: bool,
    size: (u32, u32),
}

fn parse_args() -> Args {
    let mut a = Args { autoplay: false, exit_on_end: false, screenshots: None, seed: None, speed: 8, windowed: false, size: (1600, 900) };
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--autoplay" => a.autoplay = true,
            "--exit-on-end" => a.exit_on_end = true,
            "--screenshots" => a.screenshots = it.next().map(PathBuf::from),
            "--seed" => a.seed = it.next().and_then(|v| v.parse().ok()),
            "--speed" => a.speed = it.next().and_then(|v| v.parse().ok()).unwrap_or(8),
            "--windowed" => a.windowed = true,
            "--size" => {
                if let Some((w, h)) = it.next().as_deref().and_then(|v| v.split_once('x')).and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?))) {
                    a.size = (w, h);
                }
            }
            other => eprintln!("ignoring unknown argument {other}"),
        }
    }
    a
}

fn main() {
    let args = parse_args();
    let clock_seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(1, |d| d.as_secs() % 1_000_000);
    let mut game = Game::new(args.seed.unwrap_or(clock_seed));
    // Autoplay plays against a blank memory and never writes to it.
    if !args.autoplay {
        game.memory = state::load_memory();
        // A first run starts calm; after that the choice is yours (TEMP-01).
        if game.memory.runs == 0 {
            game.temperature = slop_rules::memory::Temperature::Low;
        }
    }

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "AI Slop Arena".into(),
            // A fixed scale keeps the 1600x900 view on screen under Windows display scaling.
            resolution: WindowResolution::new(args.size.0, args.size.1).with_scale_factor_override(1.0),
            mode: if args.windowed || args.autoplay { WindowMode::Windowed } else { display::FULLSCREEN },
            ..default()
        }),
        ..default()
    }))
    // Keep running at full speed in the background, so a controller plays on
    // while you type somewhere else (UI-20).
    .insert_resource(bevy::winit::WinitSettings { focused_mode: bevy::winit::UpdateMode::Continuous, unfocused_mode: bevy::winit::UpdateMode::Continuous })
    .insert_resource(ClearColor(theme::VOID))
    .insert_resource(game)
    // Files in assets/ replace drawn art, generated music and synthesised sounds (ASSET-01).
    .insert_resource(assets::Overrides::find())
    .add_plugins((play::PlayPlugin, views::ViewsPlugin, fx::FxPlugin, hud::HudPlugin, screens::ScreensPlugin, ui::UiKitPlugin, audio::AudioFxPlugin, music::MusicPlugin, display::DisplayPlugin, controls::ControlsPlugin));

    if args.autoplay {
        if let Some(dir) = &args.screenshots {
            if let Err(e) = std::fs::create_dir_all(dir) {
                eprintln!("cannot create {}: {e}", dir.display());
            }
        }
        app.insert_resource(autoplay::Autoplay::new(args.screenshots, args.exit_on_end, args.speed)).add_plugins(autoplay::AutoplayPlugin);
    }
    app.run();
}
