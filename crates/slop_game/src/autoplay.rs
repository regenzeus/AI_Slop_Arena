//! Autoplay: the bots play a whole run and the client screenshots every
//! kind of screen (UI-06). This is how the client is checked without a
//! human: `slop_game --autoplay --screenshots shots --exit-on-end`.

use std::path::PathBuf;

use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use slop_rules::Phase;

use crate::play::END_PAUSE;
use crate::state::{Game, Screen, UiAction};

/// (wave, tick) moments worth a screenshot during waves.
const WAVE_SHOTS: [(u32, u32); 9] = [(1, 900), (3, 1500), (5, 1800), (8, 2400), (10, 900), (10, 2400), (13, 2400), (17, 3000), (20, 1500)];
/// Shops worth a screenshot, by the wave they sell for.
const SHOP_SHOTS: [u32; 3] = [2, 7, 14];

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
}

impl Autoplay {
    pub fn new(shots: Option<PathBuf>, exit_on_end: bool, speed: u32) -> Autoplay {
        Autoplay { shots, exit_on_end, speed: speed.max(1), frames: 0, wait: 0, taken: Vec::new(), count: 0, holding: None }
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

fn drive(mut commands: Commands, mut game: ResMut<Game>, mut auto: ResMut<Autoplay>, mut exit: MessageWriter<AppExit>) {
    auto.frames += 1;
    if auto.wait > 0 {
        auto.wait -= 1;
        return;
    }
    // Finish a held screenshot: shoot, then let the bots go on.
    if let Some(name) = auto.holding.take() {
        auto.shoot(&mut commands, &name);
        game.paused = false;
        game.frozen = false;
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
                let hero = (game.seed % 5) as usize;
                game.act(UiAction::PickHero(hero));
                auto.holding = Some("heroes".into());
                auto.wait = 10;
            } else {
                game.bot = true;
                game.speed = auto.speed;
                game.act(UiAction::StartRun);
            }
        }
        Screen::Briefing => {
            if auto.wants("briefing") {
                auto.holding = Some("briefing".into());
                auto.wait = 12;
            } else {
                game.act(UiAction::BeginRun);
            }
        }
        Screen::Run => {
            let Some(run) = game.run.as_ref() else { return };
            let wave = run.wave;
            let settled = game.since_end > END_PAUSE + 0.1;
            let name = match &run.phase {
                Phase::Wave if wave == 4 && run.world.as_ref().is_some_and(|w| w.tick >= 900) && auto.wants("pause") => {
                    // The pause screen, with the build so far.
                    game.paused = true;
                    game.dirty = true;
                    auto.holding = Some("pause".into());
                    auto.wait = 12;
                    return;
                }
                Phase::Wave => {
                    let tick = run.world.as_ref().map_or(0, |w| w.tick);
                    WAVE_SHOTS.iter().find(|&&(w, t)| w == wave && tick >= t).map(|(w, t)| format!("wave{w:02}_{:02}s", t / 60))
                }
                Phase::LevelUp(_) if settled => Some("levelup".into()),
                Phase::Shop => SHOP_SHOTS.contains(&wave).then(|| format!("shop_w{wave:02}")),
                Phase::Over { .. } if settled => Some("over".into()),
                _ => None,
            };
            let over = run.is_over();
            if let Some(name) = name.filter(|n| auto.wants(n)) {
                // Hold the screen still while it is captured.
                game.frozen = true;
                game.dirty = true;
                auto.holding = Some(name);
                auto.wait = 12;
                return;
            }
            if over && settled && auto.exit_on_end {
                exit.write(AppExit::Success);
            }
        }
    }
}
