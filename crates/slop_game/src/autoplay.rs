//! Autoplay: the bot plays a wave and the client screenshots the title, the
//! wave in progress and the end screen (UI-06). This is how the client is
//! checked without a human:
//! `slop_game --autoplay --screenshots shots --exit-on-end`.

use std::path::PathBuf;

use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};

use crate::state::{Game, Screen};

/// Ticks into the wave at which to take a screenshot.
const WAVE_SHOTS: [u32; 3] = [480, 1800, 3060];

#[derive(Resource)]
pub struct Autoplay {
    shots: Option<PathBuf>,
    exit_on_end: bool,
    frames: u32,
    taken: Vec<u32>,
    end_shot: bool,
    count: u32,
}

impl Autoplay {
    pub fn new(shots: Option<PathBuf>, exit_on_end: bool) -> Autoplay {
        Autoplay { shots, exit_on_end, frames: 0, taken: Vec::new(), end_shot: false, count: 0 }
    }

    fn shoot(&mut self, commands: &mut Commands, name: &str) {
        if let Some(dir) = &self.shots {
            self.count += 1;
            let path = dir.join(format!("{:02}_{name}.png", self.count));
            commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
        }
    }
}

pub struct AutoplayPlugin;

impl Plugin for AutoplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, drive);
    }
}

fn drive(mut commands: Commands, mut game: ResMut<Game>, mut auto: ResMut<Autoplay>, mut exit: MessageWriter<AppExit>) {
    auto.frames += 1;
    match game.screen {
        Screen::Title => {
            if auto.frames == 40 {
                auto.shoot(&mut commands, "title");
            } else if auto.frames > 55 {
                game.bot = true;
                game.start();
            }
        }
        Screen::Wave => {
            game.bot = true;
            let tick = game.world.tick;
            if let Some(&at) = WAVE_SHOTS.iter().find(|&&at| tick >= at && !auto.taken.contains(&at)) {
                auto.taken.push(at);
                auto.shoot(&mut commands, &format!("wave_{:02}s", at / 60));
            }
        }
        Screen::End => {
            if game.since_end > 2.2 && !auto.end_shot {
                auto.end_shot = true;
                auto.shoot(&mut commands, "end");
            } else if game.since_end > 3.2 && auto.exit_on_end {
                exit.write(AppExit::Success);
            }
        }
    }
}
