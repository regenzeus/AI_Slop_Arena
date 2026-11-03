//! AI Slop Arena desktop client.
//!
//! ```text
//! slop_game                          play
//! slop_game --autoplay               watch the bots play a whole run
//!   --seed N                         choose the run's seed
//!   --screenshots DIR                save a screenshot of every kind of screen
//!   --exit-on-end                    close when the run ends
//!   --speed N                        autoplay fast-forward (default 8)
//! ```

mod art;
mod audio;
mod autoplay;
mod draw;
mod fx;
mod hud;
mod icons;
mod play;
mod screens;
mod state;
mod theme;
mod ui;
mod views;

use std::path::PathBuf;

use bevy::prelude::*;
use bevy::window::WindowResolution;

use crate::state::Game;

struct Args {
    autoplay: bool,
    exit_on_end: bool,
    screenshots: Option<PathBuf>,
    seed: Option<u64>,
    speed: u32,
}

fn parse_args() -> Args {
    let mut a = Args { autoplay: false, exit_on_end: false, screenshots: None, seed: None, speed: 8 };
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--autoplay" => a.autoplay = true,
            "--exit-on-end" => a.exit_on_end = true,
            "--screenshots" => a.screenshots = it.next().map(PathBuf::from),
            "--seed" => a.seed = it.next().and_then(|v| v.parse().ok()),
            "--speed" => a.speed = it.next().and_then(|v| v.parse().ok()).unwrap_or(8),
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
    let game = Game::new(args.seed.unwrap_or(clock_seed));

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "AI Slop Arena".into(),
            // A fixed scale keeps the 1600x900 view on screen under Windows display scaling.
            resolution: WindowResolution::new(1600, 900).with_scale_factor_override(1.0),
            ..default()
        }),
        ..default()
    }))
    .insert_resource(ClearColor(theme::VOID))
    .insert_resource(game)
    .add_plugins((play::PlayPlugin, views::ViewsPlugin, fx::FxPlugin, hud::HudPlugin, screens::ScreensPlugin, ui::UiKitPlugin, audio::AudioFxPlugin));

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
