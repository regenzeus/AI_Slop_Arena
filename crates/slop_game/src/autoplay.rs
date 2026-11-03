//! Autoplay: the bots play a whole run and the client screenshots every
//! kind of screen (UI-06). This is how the client is checked without a
//! human: `slop_game --autoplay --screenshots shots --exit-on-end`.

use std::path::PathBuf;

use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use slop_rules::Phase;

use crate::play::END_PAUSE;
use crate::state::{Game, Screen, UiAction};
use crate::ui::Nav;

/// Minutes into the Feed worth a screenshot, latest first.
const FEED_SHOTS: [f32; 7] = [27.0, 20.0, 14.0, 9.0, 5.0, 2.0, 0.5];

#[derive(Resource)]
pub struct Autoplay {
    shots: Option<PathBuf>,
    exit_on_end: bool,
    speed: u32,
    frames: u32,
    /// Frames to wait before the next step.
    wait: u32,
    taken: Vec<String>,
    count: u32,
    /// A screenshot is being taken of a held screen.
    holding: Option<String>,
    /// The held screen was shot; let go once the capture has rendered.
    release: bool,
    /// Minutes after which autoplay stops.
    pub limit: f32,
}

impl Autoplay {
    pub fn new(shots: Option<PathBuf>, exit_on_end: bool, speed: u32) -> Autoplay {
        Autoplay { shots, exit_on_end, speed: speed.max(1), frames: 0, wait: 0, taken: Vec::new(), count: 0, holding: None, release: false, limit: 30.0 }
    }

    fn shoot(&mut self, commands: &mut Commands, name: &str) {
        self.taken.push(name.to_string());
        if let Some(dir) = &self.shots {
            self.count += 1;
            let path = dir.join(format!("{:02}_{name}.png", self.count));
            commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
        }
    }

    fn wants(&self, name: &str) -> bool {
        self.shots.is_some() && !self.taken.iter().any(|t| t == name)
    }
}

pub struct AutoplayPlugin;

impl Plugin for AutoplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, drive);
    }
}

fn drive(mut commands: Commands, mut game: ResMut<Game>, mut auto: ResMut<Autoplay>, mut nav: ResMut<Nav>, mut exit: MessageWriter<AppExit>) {
    auto.frames += 1;
    if auto.wait > 0 {
        auto.wait -= 1;
        return;
    }
    // Finish a held screenshot: shoot, wait for the capture, then let the
    // bots go on. Letting go in the same frame could change the screen first.
    if let Some(name) = auto.holding.take() {
        auto.shoot(&mut commands, &name);
        auto.release = true;
        auto.wait = 4;
        return;
    }
    if auto.release {
        auto.release = false;
        game.paused = false;
        game.frozen = false;
        nav.key = None;
        game.dirty = true;
        auto.wait = 4;
        return;
    }
    match game.screen {
        Screen::Title => {
            if auto.frames < 40 {
                return;
            }
            if auto.wants("title") {
                auto.shoot(&mut commands, "title");
                auto.wait = 10;
            } else {
                game.act(UiAction::ToHeroes);
                auto.wait = 15;
            }
        }
        Screen::Heroes => {
            if auto.wants("heroes") {
                let hero = (game.seed % slop_rules::Hero::ALL.len() as u64) as usize;
                game.act(UiAction::PickHero(hero));
                auto.holding = Some("heroes".into());
                auto.wait = 10;
            } else {
                game.bot = true;
                game.speed = auto.speed;
                game.act(UiAction::StartRun);
            }
        }
        Screen::Run => {
            let Some(run) = game.run.as_ref() else { return };
            let minutes = run.minutes();
            let settled = game.since_end > END_PAUSE + 0.1;
            let name = match &run.phase {
                Phase::Playing if minutes >= 3.0 && auto.wants("pause") => {
                    // The pause screen, with the build so far.
                    game.paused = true;
                    game.dirty = true;
                    auto.holding = Some("pause".into());
                    auto.wait = 12;
                    return;
                }
                Phase::Playing => FEED_SHOTS.iter().find(|&&m| minutes >= m).map(|m| format!("feed_{m:04.1}min")),
                Phase::LevelUp(_) => Some("levelup".into()),
                Phase::Chest(_) if game.chest_opened < 2.8 => None,
                Phase::Chest(r) if r.merged.is_some() => Some("merge".into()),
                Phase::Chest(_) => Some("chest".into()),
                Phase::Vending { .. } => Some("vending".into()),
                Phase::Shrine { .. } => Some("shrine".into()),
                Phase::Glitch(_) => Some("glitch".into()),
                Phase::Recycle { .. } => Some("recycle".into()),
                Phase::Over { .. } if settled => Some("over".into()),
                Phase::Over { .. } => None,
            };
            let over = run.is_over();
            if let Some(name) = name.filter(|n| auto.wants(n) && !auto.taken.iter().any(|t| t.starts_with("feed_") && n.starts_with("feed_") && t >= n)) {
                // Hold the screen still while it is captured.
                game.frozen = true;
                game.dirty = true;
                auto.holding = Some(name);
                auto.wait = 12;
                return;
            }
            if (over && settled || minutes >= auto.limit) && auto.exit_on_end {
                exit.write(AppExit::Success);
            }
        }
    }
}
