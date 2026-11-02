//! The HUD over the arena, plus the title and end screens (UI-05).
//!
//! Mental Health top left, the wave timer as a video progress bar top
//! centre, Clout top right, weapons along the bottom.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use slop_sim::content::WEAPONS;
use slop_sim::world::Outcome;
use slop_sim::{DamageType, Event};

use crate::art::Art;
use crate::play::Simulation;
use crate::state::{Game, Screen};
use crate::theme;
use crate::views::VIEW;

const BAR_W: f32 = 280.0;
const PROGRESS_W: f32 = 560.0;
/// Seconds the "how to play" hint stays at the start of a wave.
const HINT_SECONDS: f32 = 7.0;
/// Seconds after the wave ends before the end screen appears.
const END_DELAY: f32 = 1.2;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, setup).add_systems(Update, (update.after(Simulation), scale_ui));
    }
}

/// The HUD is laid out for 1600 x 900 and scales with the window (UI-07).
fn scale_ui(windows: Query<&Window, With<PrimaryWindow>>, mut scale: ResMut<UiScale>) {
    if let Ok(w) = windows.single() {
        let k = (w.width() / VIEW.x).min(w.height() / VIEW.y).max(0.5);
        if (scale.0 - k).abs() > 0.01 {
            scale.0 = k;
        }
    }
}

#[derive(Component)]
struct Hud;
#[derive(Component)]
struct MhFill;
#[derive(Component)]
struct MhText;
#[derive(Component)]
struct WaveText;
#[derive(Component)]
struct Progress;
#[derive(Component)]
struct TimeText;
#[derive(Component)]
struct CloutText;
#[derive(Component)]
struct Chip(usize);
#[derive(Component)]
struct Hint;
#[derive(Component)]
struct Paused;
#[derive(Component)]
struct Vignette;
#[derive(Component)]
struct Title;
#[derive(Component)]
struct End;
#[derive(Component)]
struct EndHeading;
#[derive(Component)]
struct EndSub;
#[derive(Component)]
struct EndStats;

fn label(p: &mut ChildSpawnerCommands, words: impl Into<String>, size: f32, color: Color) {
    p.spawn((Text::new(words), TextFont { font_size: size, ..default() }, TextColor(color)));
}

fn overlay() -> Node {
    Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        row_gap: Val::Px(14.0),
        ..default()
    }
}

fn setup(mut commands: Commands, art: Res<Art>) {
    // The red flash when you are hurt sits under everything else.
    commands.spawn((Vignette, overlay(), BackgroundColor(Color::NONE), GlobalZIndex(1)));

    commands
        .spawn((Hud, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }, GlobalZIndex(2)))
        .with_children(|root| {
            // Mental Health.
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(20.0),
                    top: Val::Px(16.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(theme::PANEL),
            ))
            .with_children(|p| {
                label(p, "MENTAL HEALTH", 14.0, theme::QUIET);
                p.spawn((Node { width: Val::Px(BAR_W), height: Val::Px(16.0), ..default() }, BackgroundColor(Color::srgb(0.25, 0.25, 0.3))))
                    .with_children(|b| {
                        b.spawn((MhFill, Node { width: Val::Px(BAR_W), height: Val::Percent(100.0), ..default() }, BackgroundColor(theme::mental_health_color(1.0))));
                    });
                p.spawn((MhText, Text::new("100 / 100"), TextFont { font_size: 16.0, ..default() }, TextColor(theme::PAPER)));
            });
            // The wave timer, as a video progress bar.
            root.spawn(Node {
                position_type: PositionType::Absolute,
                top: Val::Px(16.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(6.0),
                        padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(theme::PANEL),
                ))
                .with_children(|p| {
                    p.spawn((WaveText, Text::new("WAVE 1"), TextFont { font_size: 22.0, ..default() }, TextColor(theme::PAPER)));
                    p.spawn((Node { width: Val::Px(PROGRESS_W), height: Val::Px(8.0), ..default() }, BackgroundColor(Color::srgb(0.32, 0.32, 0.36))))
                        .with_children(|b| {
                            b.spawn((Progress, Node { width: Val::Px(0.0), height: Val::Percent(100.0), ..default() }, BackgroundColor(theme::PLAYER_RED)));
                        });
                    p.spawn((TimeText, Text::new("0:00 / 1:00"), TextFont { font_size: 15.0, ..default() }, TextColor(theme::PAPER)));
                });
            });
            // Clout.
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(20.0),
                    top: Val::Px(16.0),
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(theme::PANEL),
            ))
            .with_children(|p| {
                p.spawn((ImageNode::new(art.like.clone()), Node { width: Val::Px(26.0), height: Val::Px(26.0), ..default() }));
                p.spawn((CloutText, Text::new("0"), TextFont { font_size: 24.0, ..default() }, TextColor(theme::PAPER)));
                label(p, "Clout", 14.0, theme::QUIET);
            });
            // Weapons.
            root.spawn(Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(16.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|p| {
                for (i, w) in WEAPONS.iter().enumerate() {
                    p.spawn((
                        Chip(i),
                        Node {
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                            border: UiRect::all(Val::Px(3.0)),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(theme::PANEL),
                        BorderColor::all(theme::type_color(w.kind)),
                    ))
                    .with_children(|c| {
                        label(c, format!("{}  {}", i + 1, w.name), 16.0, theme::PAPER);
                        label(c, w.kind.name(), 12.0, theme::type_color(w.kind));
                    });
                }
            });
            // How to play, fading after a few seconds.
            root.spawn((Hint, Node { position_type: PositionType::Absolute, top: Val::Px(110.0), width: Val::Percent(100.0), justify_content: JustifyContent::Center, ..default() }))
                .with_children(|p| {
                    p.spawn((Text::new("WASD or arrows to move. Your weapons fire on their own. Survive the wave."), TextFont { font_size: 20.0, ..default() }, TextColor(theme::INK)));
                });
        });

    commands.spawn((Paused, overlay(), BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)), GlobalZIndex(5), Visibility::Hidden)).with_children(|p| {
        label(p, "PAUSED", 48.0, theme::PAPER);
        label(p, "P to resume    1-6 switch weapons    Q to quit", 18.0, theme::QUIET);
    });

    commands.spawn((Title, overlay(), BackgroundColor(Color::srgba(0.09, 0.09, 0.13, 0.86)), GlobalZIndex(10))).with_children(|p| {
        label(p, "AI SLOP ARENA", 84.0, theme::PAPER);
        label(p, "One human. Endless slop. Protect your Mental Health.", 24.0, theme::QUIET);
        p.spawn(Node { height: Val::Px(30.0), ..default() });
        for line in ["WASD or arrows: move", "Your weapons fire on their own", "1-6: switch a weapon on or off", "P: pause"] {
            label(p, line, 20.0, theme::PAPER);
        }
        p.spawn(Node { height: Val::Px(30.0), ..default() });
        label(p, "Press Space to start", 30.0, theme::LIKE);
        label(p, "Milestone 1 prototype: one wave, six weapons, five kinds of slop", 15.0, theme::QUIET);
    });

    commands.spawn((End, overlay(), BackgroundColor(Color::srgba(0.09, 0.09, 0.13, 0.86)), GlobalZIndex(10), Visibility::Hidden)).with_children(|p| {
        p.spawn((EndHeading, Text::new(""), TextFont { font_size: 72.0, ..default() }, TextColor(theme::PAPER)));
        p.spawn((EndSub, Text::new(""), TextFont { font_size: 22.0, ..default() }, TextColor(theme::QUIET)));
        p.spawn(Node { height: Val::Px(16.0), ..default() });
        p.spawn((EndStats, Text::new(""), TextFont { font_size: 20.0, ..default() }, TextColor(theme::PAPER), TextLayout::new_with_justify(Justify::Center)));
        p.spawn(Node { height: Val::Px(16.0), ..default() });
        label(p, "Space: try again    Q: quit", 22.0, theme::LIKE);
    });
}

fn clock(seconds: f32) -> String {
    let s = seconds.max(0.0) as u32;
    format!("{}:{:02}", s / 60, s % 60)
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn update(
    game: Res<Game>,
    time: Res<Time>,
    mut vignette_alpha: Local<f32>,
    mut nodes: ParamSet<(
        Query<(&mut Node, &mut BackgroundColor), With<MhFill>>,
        Query<&mut Node, With<Progress>>,
        Query<(&Chip, &mut BackgroundColor)>,
        Query<&mut BackgroundColor, With<Vignette>>,
    )>,
    mut texts: ParamSet<(
        Query<&mut Text, With<MhText>>,
        Query<&mut Text, With<TimeText>>,
        Query<&mut Text, With<CloutText>>,
        Query<&mut Text, With<EndHeading>>,
        Query<(&mut Text, &mut TextColor), With<EndSub>>,
        Query<&mut Text, With<EndStats>>,
    )>,
    mut shown: ParamSet<(
        Query<&mut Visibility, With<Hud>>,
        Query<&mut Visibility, With<Title>>,
        Query<&mut Visibility, With<End>>,
        Query<&mut Visibility, With<Paused>>,
        Query<&mut Visibility, With<Hint>>,
    )>,
    mut heading_color: Query<&mut TextColor, (With<EndHeading>, Without<EndSub>)>,
) {
    let world = &game.world;
    let p = &world.player;
    let share = (p.mental_health / p.stats.max_mental_health).clamp(0.0, 1.0);
    let vis = |on: bool| if on { Visibility::Inherited } else { Visibility::Hidden };

    if let Ok((mut node, mut bg)) = nodes.p0().single_mut() {
        node.width = Val::Px(BAR_W * share);
        bg.0 = theme::mental_health_color(share);
    }
    if let Ok(mut node) = nodes.p1().single_mut() {
        node.width = Val::Px(PROGRESS_W * world.progress());
    }
    for (chip, mut bg) in &mut nodes.p2() {
        let on = p.weapons.get(chip.0).is_some_and(|w| w.enabled);
        bg.0 = if on { theme::PANEL } else { Color::srgba(0.3, 0.3, 0.3, 0.4) };
    }
    if game.events.iter().any(|e| matches!(e, Event::PlayerHurt { .. })) {
        *vignette_alpha = 0.28;
    }
    *vignette_alpha = (*vignette_alpha - time.delta_secs() * 0.9).max(0.0);
    if let Ok(mut bg) = nodes.p3().single_mut() {
        bg.0 = theme::DANGER.with_alpha(*vignette_alpha);
    }

    if let Ok(mut t) = texts.p0().single_mut() {
        **t = format!("{:.0} / {:.0}", p.mental_health.ceil(), p.stats.max_mental_health);
    }
    if let Ok(mut t) = texts.p1().single_mut() {
        **t = format!("{} / {}", clock(world.elapsed()), clock(world.wave.seconds));
    }
    if let Ok(mut t) = texts.p2().single_mut() {
        **t = world.clout.to_string();
    }

    let ended = game.screen == Screen::End && game.since_end > END_DELAY;
    if ended {
        let cleared = world.outcome == Outcome::Cleared;
        if let Ok(mut t) = texts.p3().single_mut() {
            **t = if cleared { "WAVE CLEARED".into() } else { "BURNOUT".into() };
        }
        if let Ok(mut c) = heading_color.single_mut() {
            c.0 = if cleared { theme::mental_health_color(1.0) } else { theme::DANGER };
        }
        if let Ok((mut t, mut c)) = texts.p4().single_mut() {
            **t = if cleared { "The slop is retraining on your data.".into() } else { "Your account has been suspended.".into() };
            c.0 = theme::QUIET;
        }
        if let Ok(mut t) = texts.p5().single_mut() {
            let tally = &world.tally;
            let total: f32 = tally.damage_dealt.iter().sum::<f32>().max(1.0);
            let shares: Vec<String> = DamageType::ALL
                .iter()
                .map(|d| format!("{} {:.0}%", d.name(), 100.0 * tally.damage_dealt[*d as usize] / total))
                .collect();
            **t = format!(
                "Survived {} of {}\nSlop deleted {}\nClout earned {}\nMental Health left {:.0} / {:.0}\nBubbles popped {}    Dodges {}\nDamage by type: {}",
                clock(world.elapsed()),
                clock(world.wave.seconds),
                tally.kills,
                world.clout,
                p.mental_health.ceil(),
                p.stats.max_mental_health,
                tally.bubbles_popped,
                tally.dodges,
                shares.join(", ")
            );
        }
    }

    let in_wave = game.screen != Screen::Title;
    if let Ok(mut v) = shown.p0().single_mut() {
        *v = vis(in_wave);
    }
    if let Ok(mut v) = shown.p1().single_mut() {
        *v = vis(game.screen == Screen::Title);
    }
    if let Ok(mut v) = shown.p2().single_mut() {
        *v = vis(ended);
    }
    if let Ok(mut v) = shown.p3().single_mut() {
        *v = vis(game.screen == Screen::Wave && game.paused);
    }
    if let Ok(mut v) = shown.p4().single_mut() {
        *v = vis(game.screen == Screen::Wave && world.elapsed() < HINT_SECONDS);
    }
}
