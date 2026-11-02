//! Runs the simulation at its fixed 60 ticks per second and turns keys into
//! input (UI-01).

use bevy::prelude::*;
use slop_sim::bot;
use slop_sim::world::{Input, Outcome, DT};

use crate::state::{Game, Screen};

/// Ticks allowed in one frame, so a slow frame does not snowball.
const MAX_TICKS_PER_FRAME: u32 = 5;

pub struct PlayPlugin;

impl Plugin for PlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (keys, step).chain().in_set(Simulation));
    }
}

/// Everything that changes the wave; views and effects run after it.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Simulation;

const WEAPON_KEYS: [KeyCode; 6] =
    [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4, KeyCode::Digit5, KeyCode::Digit6];

fn keys(keyboard: Res<ButtonInput<KeyCode>>, mut game: ResMut<Game>, mut exit: MessageWriter<AppExit>) {
    let go = keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::Enter);
    match game.screen {
        Screen::Title if go => game.start(),
        Screen::End if go && game.since_end > 1.5 => game.start(),
        Screen::Wave => {
            if keyboard.just_pressed(KeyCode::KeyP) || keyboard.just_pressed(KeyCode::Escape) {
                game.paused = !game.paused;
            }
            for (i, key) in WEAPON_KEYS.iter().enumerate() {
                if keyboard.just_pressed(*key) {
                    if let Some(w) = game.world.player.weapons.get_mut(i) {
                        w.enabled = !w.enabled;
                    }
                }
            }
        }
        _ => {}
    }
    if keyboard.just_pressed(KeyCode::KeyQ) && (game.screen != Screen::Wave || game.paused) {
        exit.write(AppExit::Success);
    }
}

fn movement(keyboard: &ButtonInput<KeyCode>) -> Input {
    let held = |a: KeyCode, b: KeyCode| keyboard.pressed(a) || keyboard.pressed(b);
    let mut input = Input::default();
    if held(KeyCode::KeyA, KeyCode::ArrowLeft) {
        input.x -= 1.0;
    }
    if held(KeyCode::KeyD, KeyCode::ArrowRight) {
        input.x += 1.0;
    }
    if held(KeyCode::KeyW, KeyCode::ArrowUp) {
        input.y += 1.0;
    }
    if held(KeyCode::KeyS, KeyCode::ArrowDown) {
        input.y -= 1.0;
    }
    input
}

fn step(time: Res<Time>, keyboard: Res<ButtonInput<KeyCode>>, mut game: ResMut<Game>) {
    let game = &mut *game;
    game.events.clear();
    match game.screen {
        Screen::Wave if !game.paused => {}
        Screen::End => {
            game.since_end += time.delta_secs();
            return;
        }
        _ => return,
    }
    game.accumulator = (game.accumulator + time.delta_secs()).min(DT * MAX_TICKS_PER_FRAME as f32);
    while game.accumulator >= DT {
        game.accumulator -= DT;
        let input = if game.bot { bot::steer(&game.world) } else { movement(&keyboard) };
        game.world.step(input);
        game.events.extend(game.world.drain_events());
        if game.world.outcome != Outcome::Running {
            game.screen = Screen::End;
            game.since_end = 0.0;
            break;
        }
    }
}
