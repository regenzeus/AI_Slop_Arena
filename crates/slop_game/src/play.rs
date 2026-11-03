//! Runs the Feed at its fixed 60 ticks per second, turns input into
//! movement and actions, and lets the bots play during autoplay (UI-01).

use bevy::prelude::*;
use slop_rules::bot::Policy;
use slop_rules::Phase;
use slop_sim::rng::Rng;
use slop_sim::world::{Input, DT};

use crate::controls::Controls;
use crate::screens::key;
use crate::state::{Cue, Game, Screen, UiAction};
use crate::ui::Nav;

/// Ticks allowed in one frame, so a slow frame does not snowball.
const MAX_TICKS_PER_FRAME: u32 = 5;
/// Seconds after Burnout before the end screen (lets the effects play).
pub const END_PAUSE: f32 = 1.2;
/// Seconds a toast stays.
const TOAST: f32 = 5.0;

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

/// Shortcuts; menu navigation and presses are handled by `ui`.
fn keys(mut controls: ResMut<Controls>, mut game: ResMut<Game>, mut nav: ResMut<Nav>, mut exit: MessageWriter<AppExit>) {
    if game.bot {
        return;
    }
    let mut actions = Vec::new();
    match game.screen {
        Screen::Heroes => {
            if let Some(i) = controls.number.filter(|i| *i < slop_rules::Hero::ALL.len()) {
                nav.key = Some(key::HERO + i as u32);
                game.hero = i;
            }
            if controls.back {
                game.screen = Screen::Title;
                game.dirty = true;
            }
        }
        Screen::Run => {
            let phase = game.run.as_ref().map(|r| r.phase.clone());
            match phase {
                // Pause and resume take the whole press, so the screen that
                // opens does not act on it too.
                Some(Phase::Playing) if controls.pause || (game.paused && controls.back) => {
                    game.paused = !game.paused;
                    game.dirty = true;
                    controls.consume();
                }
                Some(Phase::Playing) if controls.dash && !game.paused => {
                    game.dash_queued = true;
                }
                Some(Phase::Glitch(offers)) => {
                    if let Some(i) = controls.number.filter(|i| *i < offers.len()) {
                        actions.push(UiAction::Glitch(i));
                    }
                }
                Some(Phase::LevelUp(choices)) => {
                    if let Some(i) = controls.number.filter(|i| *i < choices.len()) {
                        actions.push(UiAction::Choose(i));
                    }
                    if controls.reroll {
                        actions.push(UiAction::Reroll);
                    }
                }
                Some(Phase::Vending { offers, .. }) => {
                    if let Some(i) = controls.number.filter(|i| *i < offers.len()) {
                        actions.push(UiAction::Buy(i));
                    }
                    if controls.back {
                        actions.push(UiAction::Close);
                    }
                }
                Some(Phase::Shrine { offers, .. }) => {
                    if let Some(i) = controls.number.filter(|i| *i < offers.len()) {
                        actions.push(UiAction::Bless(i));
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    for a in actions {
        if !game.act(a) {
            exit.write(AppExit::Success);
        }
    }
}

/// The attract mode: the bots play the Feed behind the menus (UI-15).
fn demo(game: &mut Game, time: &Time, rng: &mut Rng) {
    let fresh = game.demo.as_ref().is_none_or(|r| r.is_over() || r.minutes() > 24.0);
    if fresh {
        let hero = slop_rules::Hero::ALL[rng.below(5) as usize];
        game.demo = Some(slop_rules::Run::showcase(game.seed.wrapping_add(rng.below(1000) as u64), hero, 4.0 + rng.below(12) as f32));
        game.accumulator = 0.0;
    }
    let run = game.demo.as_mut().expect("just made");
    game.accumulator = (game.accumulator + time.delta_secs()).min(DT * MAX_TICKS_PER_FRAME as f32);
    while game.accumulator >= DT {
        game.accumulator -= DT;
        if !matches!(run.phase, Phase::Playing) {
            slop_rules::bot::decide(run, Policy::Heuristic, rng);
            continue;
        }
        let input = slop_sim::bot::steer(run.world.as_ref().expect("the Feed runs"));
        run.step(input);
        if let Some(w) = run.world.as_mut() {
            game.events.extend(w.drain_events());
        }
        // The demo is silent and never shows its own news.
        run.toasts.clear();
    }
    game.alpha = game.accumulator / DT;
}

fn step(time: Res<Time>, controls: Res<Controls>, mut game: ResMut<Game>, mut rng: ResMut<BotRng>) {
    let game = &mut *game;
    game.events.clear();
    game.level_flash = false;
    if matches!(game.screen, Screen::Title | Screen::Heroes) {
        demo(game, &time, &mut rng.0);
        return;
    }
    let Some(run) = game.run.as_mut() else { return };
    if run.level > game.shown_level {
        game.shown_level = run.level;
        game.level_flash = true;
    }
    if game.paused || game.frozen {
        return;
    }
    let bot = game.bot;
    if run.is_over() {
        game.since_end += time.delta_secs();
        return;
    }
    if !matches!(run.phase, Phase::Playing) {
        game.since_choice += time.delta_secs();
        // The bots choose after a beat during autoplay, so it can be seen.
        let wait = if matches!(run.phase, Phase::Chest(_)) { 3.2 } else { 0.5 };
        if bot && game.since_choice > wait {
            slop_rules::bot::decide(run, Policy::Heuristic, &mut rng.0);
            game.since_choice = 0.0;
            game.dirty = true;
        }
        return;
    }
    let budget = DT * (MAX_TICKS_PER_FRAME * game.speed) as f32;
    game.accumulator = (game.accumulator + time.delta_secs() * game.speed as f32).min(budget);
    while game.accumulator >= DT {
        game.accumulator -= DT;
        let input = if bot {
            slop_sim::bot::steer(run.world.as_ref().expect("the Feed runs"))
        } else {
            // A dash pressed between ticks goes with the next one.
            let dash = std::mem::take(&mut game.dash_queued);
            Input { x: controls.move_dir.x, y: controls.move_dir.y, dash }
        };
        let changed = run.step(input);
        if let Some(w) = run.world.as_mut() {
            game.events.extend(w.drain_events());
        }
        if changed {
            game.since_choice = 0.0;
            game.since_end = 0.0;
            game.dirty = true;
            if let Phase::Chest(reward) = &run.phase {
                game.cues.push(if reward.best >= 4 { Cue::Legendary } else { Cue::Chest });
                game.chest_opened = 0.0;
            }
            if matches!(run.phase, Phase::Glitch(_)) {
                game.cues.push(Cue::Legendary);
            }
            break;
        }
    }
    game.alpha = game.accumulator / DT;
    for t in run.toasts.drain(..) {
        game.toasts.push((t, TOAST));
    }
    for e in &game.events {
        match e {
            slop_sim::Event::BossIntro { kind } => game.banner = Some((kind.name().to_uppercase(), kind.puzzle().to_string(), 4.0)),
            slop_sim::Event::Regenerate { .. } => game.banner = Some(("REGENERATE RESPONSE".into(), "The Model tries again. A varied build would have stopped this.".into(), 2.5)),
            slop_sim::Event::BossPhase { .. } => game.banner = Some(("PHASE 2".into(), "It is getting serious.".into(), 1.8)),
            slop_sim::Event::BossDefeated { kind, .. } => game.banner = Some(("DELETED".into(), format!("{} is gone. Grab its chest: a glitch waits inside.", kind.name()), 3.0)),
            slop_sim::Event::Biome { platform } => {
                game.banner = Some((format!("NOW ENTERING {}", platform.name().to_uppercase()), platform.hazard().to_string(), 2.6));
            }
            slop_sim::Event::ArenaWarning { .. } => game.banner = Some(("YOU'RE LIVE IN 3...".into(), "Stay inside the ring. Survive the stream, get the chest.".into(), 2.5)),
            slop_sim::Event::ArenaCleared { .. } => game.banner = Some(("STREAM ENDED".into(), "Thanks for watching. Grab your chest.".into(), 2.5)),
            slop_sim::Event::GodMode => game.banner = Some(("GOD MODE".into(), "Untouchable for 2 seconds.".into(), 1.2)),
            _ => {}
        }
    }
}

fn timers(time: Res<Time>, mut game: ResMut<Game>) {
    let dt = time.delta_secs();
    game.chest_opened += dt;
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
    if !game.paused && !game.choosing() {
        for t in &mut game.toasts {
            t.1 -= dt;
        }
        game.toasts.retain(|t| t.1 > 0.0);
    }
}
