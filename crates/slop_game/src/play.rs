//! Runs the wave at its fixed 60 ticks per second, turns keys into input and
//! actions, and lets the bots play during autoplay (UI-01).

use bevy::prelude::*;
use slop_rules::bot::Policy;
use slop_rules::Phase;
use slop_sim::rng::Rng;
use slop_sim::world::{Input, DT};

use crate::state::{Game, Screen, UiAction};

/// Ticks allowed in one frame, so a slow frame does not snowball.
const MAX_TICKS_PER_FRAME: u32 = 5;
/// Seconds after a wave ends before choices are taken (lets the effects play).
pub const END_PAUSE: f32 = 1.2;

pub struct PlayPlugin;

impl Plugin for PlayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BotRng(Rng::new(5, 13))).add_systems(Update, (keys, step, timers).chain().in_set(Simulation));
    }
}

/// Everything that changes the run; views and effects run after it.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Simulation;

#[derive(Resource)]
pub struct BotRng(pub Rng);

const NUMBER_KEYS: [KeyCode; 6] = [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4, KeyCode::Digit5, KeyCode::Digit6];

fn keys(keyboard: Res<ButtonInput<KeyCode>>, mut game: ResMut<Game>, mut exit: MessageWriter<AppExit>) {
    if game.bot {
        return;
    }
    let go = keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::Enter);
    let number = NUMBER_KEYS.iter().position(|k| keyboard.just_pressed(*k));
    let mut actions = Vec::new();
    match game.screen {
        Screen::Title if go => actions.push(UiAction::ToHeroes),
        Screen::Heroes => {
            if let Some(i) = number.filter(|i| *i < 5) {
                actions.push(UiAction::PickHero(i));
            }
            if keyboard.just_pressed(KeyCode::ArrowRight) || keyboard.just_pressed(KeyCode::KeyD) {
                actions.push(UiAction::PickHero((game.hero + 1) % 5));
            }
            if keyboard.just_pressed(KeyCode::ArrowLeft) || keyboard.just_pressed(KeyCode::KeyA) {
                actions.push(UiAction::PickHero((game.hero + 4) % 5));
            }
            if go {
                actions.push(UiAction::StartRun);
            }
        }
        Screen::Run => {
            let phase = game.run.as_ref().map(|r| r.phase.clone());
            let settled = game.since_end > END_PAUSE;
            match phase {
                Some(Phase::Wave) => {
                    if keyboard.just_pressed(KeyCode::KeyP) || keyboard.just_pressed(KeyCode::Escape) {
                        game.paused = !game.paused;
                    }
                }
                Some(Phase::LevelUp(_)) if settled => {
                    if let Some(i) = number.filter(|i| *i < 4) {
                        actions.push(UiAction::LevelPick(i));
                    }
                }
                Some(Phase::PatchNotes(_)) if settled && go => actions.push(UiAction::ClosePatchNotes),
                Some(Phase::Shop) => {
                    if let Some(i) = number {
                        actions.push(UiAction::Buy(i));
                    }
                    if keyboard.just_pressed(KeyCode::KeyR) {
                        actions.push(UiAction::Reroll);
                    }
                    if go {
                        actions.push(UiAction::NextWave);
                    }
                }
                Some(Phase::Over { .. }) if settled && go => actions.push(UiAction::NewRun),
                _ => {}
            }
        }
        _ => {}
    }
    let quit_allowed = game.screen != Screen::Run || game.paused || game.run.as_ref().is_some_and(|r| r.is_over());
    if keyboard.just_pressed(KeyCode::KeyQ) && quit_allowed {
        exit.write(AppExit::Success);
    }
    for a in actions {
        if !game.act(a) {
            exit.write(AppExit::Success);
        }
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

fn step(time: Res<Time>, keyboard: Res<ButtonInput<KeyCode>>, mut game: ResMut<Game>, mut rng: ResMut<BotRng>) {
    let game = &mut *game;
    game.events.clear();
    if game.screen != Screen::Run || game.paused {
        return;
    }
    let bot = game.bot;
    let Some(run) = game.run.as_mut() else { return };
    if !matches!(run.phase, Phase::Wave) {
        game.since_end += time.delta_secs();
        // The bots choose between waves during autoplay.
        if bot && game.since_end > END_PAUSE + 0.6 && !run.is_over() {
            slop_rules::bot::decide(run, Policy::Heuristic, &mut rng.0);
            game.dirty = true;
            if matches!(run.phase, Phase::Wave) {
                let run = game.run.take().expect("still here");
                game.wave_banner(&run);
                game.run = Some(run);
            }
        }
        return;
    }
    let budget = DT * (MAX_TICKS_PER_FRAME * game.speed) as f32;
    game.accumulator = (game.accumulator + time.delta_secs() * game.speed as f32).min(budget);
    while game.accumulator >= DT {
        game.accumulator -= DT;
        let input = if bot { slop_sim::bot::steer(run.world.as_ref().expect("a wave runs")) } else { movement(&keyboard) };
        let ended = run.step(input);
        if let Some(w) = run.world.as_mut() {
            game.events.extend(w.drain_events());
        }
        if ended {
            game.since_end = 0.0;
            game.dirty = true;
            break;
        }
    }
    for e in &game.events {
        match e {
            slop_sim::Event::BossIntro { kind } => game.banner = Some((kind.name().to_uppercase(), kind.puzzle().to_string(), 4.0)),
            slop_sim::Event::Regenerate { .. } => game.banner = Some(("REGENERATE RESPONSE".into(), "The Model tries again. A varied build would have stopped this.".into(), 2.5)),
            slop_sim::Event::BossPhase { .. } => game.banner = Some(("PHASE 2".into(), "It is getting serious.".into(), 1.8)),
            _ => {}
        }
    }
}

fn timers(time: Res<Time>, mut game: ResMut<Game>) {
    let dt = time.delta_secs();
    if let Some((_, left)) = &mut game.message {
        *left -= dt;
        if *left <= 0.0 {
            game.message = None;
            game.dirty = true;
        }
    }
    if let Some((_, _, left)) = &mut game.banner {
        *left -= dt;
        if *left <= 0.0 {
            game.banner = None;
        }
    }
}
