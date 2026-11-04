//! The HUD over the Feed (UI-05): Mental Health with your statuses, the run
//! clock with the platform and its boss (a video progress bar counting down
//! to it, then the boss's health), Clout, level and experience, news
//! toasts, banners, MeTube's ad breaks and pop-ups.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use slop_rules::Phase;
use slop_sim::content::BossKind;
use slop_sim::Event;

use crate::art::Art;
use crate::play::Simulation;
use crate::state::{Game, Screen};
use crate::theme;
/// The size the HUD and screens are laid out for.
const LAYOUT: Vec2 = Vec2::new(1600.0, 900.0);

const BAR_W: f32 = 260.0;
const PROGRESS_W: f32 = 560.0;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, setup).add_systems(Update, (update.after(Simulation), extras.after(Simulation), scale_ui));
    }
}

/// The HUD is laid out for 1600 x 900 and scales with the window (UI-07).
fn scale_ui(windows: Query<&Window, With<PrimaryWindow>>, mut scale: ResMut<UiScale>) {
    if let Ok(w) = windows.single() {
        let k = (w.width() / LAYOUT.x).min(w.height() / LAYOUT.y).max(0.5);
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
struct Statuses;
#[derive(Component)]
struct WaveText;
#[derive(Component)]
struct Progress;
#[derive(Component)]
struct TimeText;
#[derive(Component)]
struct CloutText;
#[derive(Component)]
struct LevelText;
#[derive(Component)]
struct XpFill;
#[derive(Component)]
struct Vignette;
#[derive(Component)]
struct Ad;
#[derive(Component)]
struct Popup;
#[derive(Component)]
struct PopupClose;
#[derive(Component)]
struct Banner;
#[derive(Component)]
struct BannerTitle;
#[derive(Component)]
struct BannerLine;
#[derive(Component)]
struct Toasts;
#[derive(Component)]
struct ShieldBack;
#[derive(Component)]
struct ShieldFill;
#[derive(Component)]
struct DashText;
#[derive(Component)]
struct KeysHint;
#[derive(Component)]
struct Reversed;
#[derive(Component)]
struct LiveText;
#[derive(Component)]
struct ChestArrow;
#[derive(Component)]
struct ModelText;
#[derive(Component)]
struct CapText;
#[derive(Component)]
struct ResourceLabel;

const SHIELD: Color = Color::srgb(0.31, 0.72, 0.91);
/// Off-screen chests shown at the screen's edge, at most.
const CHEST_ARROWS: usize = 4;

/// The new parts of the HUD: Boundaries, dash charges, why your controls
/// are reversed, the livestream clock, and arrows to chests off screen.
#[allow(clippy::type_complexity)]
fn extras(
    game: Res<Game>,
    controls: Res<crate::controls::Controls>,
    art: Res<Art>,
    mut bars: Query<&mut Node, (With<ShieldFill>, Without<ShieldBack>, Without<ChestArrow>)>,
    mut backs: Query<&mut Visibility, (With<ShieldBack>, Without<ShieldFill>, Without<ChestArrow>)>,
    mut texts: ParamSet<(Query<&mut Text, With<DashText>>, Query<&mut Text, With<KeysHint>>, Query<&mut Text, With<Reversed>>, Query<&mut Text, With<LiveText>>, Query<&mut Text, With<ModelText>>, Query<&mut Text, With<CapText>>, Query<&mut Text, With<ResourceLabel>>)>,
    mut arrows: Query<(&mut Node, &mut Visibility, &mut ImageNode), (With<ChestArrow>, Without<ShieldFill>, Without<ShieldBack>)>,
) {
    let Some(run) = game.run.as_ref().filter(|_| game.screen == Screen::Run) else { return };
    let Some(world) = run.world.as_ref() else { return };
    let p = &world.player;
    let max = p.stats.boundaries.max(0.0);
    if let Ok(mut v) = backs.single_mut() {
        *v = if max > 0.0 { Visibility::Inherited } else { Visibility::Hidden };
    }
    if let Ok(mut node) = bars.single_mut() {
        node.width = Val::Px(BAR_W * if max > 0.0 { (p.shield / max).clamp(0.0, 1.0) } else { 0.0 });
    }
    let key = if controls.pad { "A" } else { "Space" };
    if let Ok(mut t) = texts.p0().single_mut() {
        let charges = p.perks.dash_charges as usize;
        let ready = p.dashes as usize;
        let pips: String = (0..charges).map(|k| if k < ready { "[#]" } else { "[ ]" }).collect::<Vec<_>>().join(" ");
        **t = if max > 0.0 { format!("DASH {pips}   Boundaries {:.0}/{max:.0}", p.shield) } else { format!("DASH {pips}  ({key})") };
    }
    if let Ok(mut t) = texts.p1().single_mut() {
        **t = if controls.pad { "Start: pause, build and stats   A/RB/RT: dash".into() } else { "P: pause, build and stats   Space/Shift: dash".into() };
    }
    if let Ok(mut t) = texts.p2().single_mut() {
        // Statuses only slow you now; the panel lists them, nothing shouts.
        **t = String::new();
    }
    if let Ok(mut t) = texts.p3().single_mut() {
        **t = match world.arena {
            Some(a) if a.closed() => format!("LIVE  {:.0} s left: stay in the ring", a.ticks as f32 / 60.0),
            Some(_) => "GOING LIVE...".into(),
            None => String::new(),
        };
    }
    if let Ok(mut t) = texts.p4().single_mut() {
        let m = run.training.model;
        let high = (0..4).max_by(|&a, &b| m[a].total_cmp(&m[b])).unwrap_or(0);
        let low = (0..4).min_by(|&a, &b| m[a].total_cmp(&m[b])).unwrap_or(0);
        let name = |k: usize| slop_sim::DamageType::ALL[k].name();
        **t = if m[high] > 0.02 { format!("Model resists {} {:.0}%, weak to {}", name(high), m[high] * 100.0, name(low)) } else { "Model: still learning you".into() };
    }
    if let Ok(mut t) = texts.p5().single_mut() {
        let driving = if run.autopilot_now() { "   AUTOPILOT: the model is driving" } else { "" };
        let worn = if run.worn { "   WEARING YOU DOWN" } else { "" };
        **t = format!("{}   Automation {}%{driving}{worn}", run.version(), run.automation);
    }
    if let Ok(mut t) = texts.p6().single_mut() {
        let want = run.chapter.def().resource.to_uppercase();
        if **t != want {
            **t = want;
        }
    }
    // Arrows to chests off screen, nearest first.
    let me = p.pos;
    let mut chests: Vec<&slop_sim::world::Pickup> = world.pickups.iter().filter(|k| k.kind == slop_sim::world::PickupKind::Chest).collect();
    chests.sort_by(|a, b| a.pos.distance_sq(me).total_cmp(&b.pos.distance_sq(me)));
    let half = Vec2::new(720.0, 405.0);
    let mut shown = chests.into_iter().filter(|k| {
        let d = k.pos - me;
        d.x.abs() > half.x - 40.0 || d.y.abs() > half.y - 40.0
    });
    for (mut node, mut vis, mut image) in &mut arrows {
        match shown.next() {
            Some(k) => {
                let d = Vec2::new(k.pos.x - me.x, -(k.pos.y - me.y));
                let s = (half.x - 60.0) / d.x.abs().max(1.0);
                let s = s.min((half.y - 60.0) / d.y.abs().max(1.0));
                let at = Vec2::new(800.0, 450.0) + d * s;
                node.left = Val::Px(at.x - 20.0);
                node.top = Val::Px(at.y - 20.0);
                image.image = if k.value >= 3 { art.golden_chest.clone() } else { art.pickups[&slop_sim::world::PickupKind::Chest].clone() };
                *vis = Visibility::Inherited;
            }
            None => *vis = Visibility::Hidden,
        }
    }
}

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

fn panel() -> (Node, BackgroundColor) {
    (Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(6.0), padding: UiRect::all(Val::Px(10.0)), ..default() }, BackgroundColor(theme::PANEL))
}

fn setup(mut commands: Commands, art: Res<Art>) {
    commands.spawn((Vignette, overlay(), BackgroundColor(Color::NONE), GlobalZIndex(1)));
    commands.spawn((Ad, overlay(), BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)), GlobalZIndex(3), Visibility::Hidden)).with_children(|p| {
        label(p, "AD", 40.0, theme::PAPER);
        label(p, "Your Mental Health will resume after this message.", 18.0, theme::QUIET);
    });
    commands
        .spawn((
            Popup,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(560.0),
                top: Val::Px(150.0),
                width: Val::Px(480.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(8.0),
                border: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(theme::PAPER),
            BorderColor::all(theme::DANGER),
            GlobalZIndex(4),
            Visibility::Hidden,
        ))
        .with_children(|p| {
            label(p, "Before you go!", 22.0, theme::INK);
            label(p, "Subscribe to our newsletter for 10 daily AI tips.", 16.0, theme::INK);
            p.spawn((PopupClose, Node { position_type: PositionType::Absolute, right: Val::Px(8.0), top: Val::Px(4.0), ..default() }))
                .with_children(|c| label(c, "x", 22.0, theme::QUIET));
        });

    commands
        .spawn((Hud, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }, GlobalZIndex(2), Visibility::Hidden))
        .with_children(|root| {
            // Mental Health and statuses.
            root.spawn((Node { position_type: PositionType::Absolute, left: Val::Px(20.0), top: Val::Px(16.0), ..default() },)).with_children(|w| {
                let (node, bg) = panel();
                w.spawn((node, bg)).with_children(|p| {
                    p.spawn(Node { column_gap: Val::Px(8.0), align_items: AlignItems::Center, ..default() }).with_children(|r| {
                        r.spawn((ImageNode::new(art.brain.clone()), Node { width: Val::Px(24.0), height: Val::Px(24.0), ..default() }));
                        r.spawn((ResourceLabel, Text::new("MENTAL HEALTH"), TextFont { font_size: 14.0, ..default() }, TextColor(theme::QUIET)));
                    });
                    p.spawn((Node { width: Val::Px(BAR_W), height: Val::Px(16.0), ..default() }, BackgroundColor(Color::srgb(0.25, 0.25, 0.3))))
                        .with_children(|b| {
                            b.spawn((MhFill, Node { width: Val::Px(BAR_W), height: Val::Percent(100.0), ..default() }, BackgroundColor(theme::mental_health_color(1.0))));
                        });
                    // Boundaries: a shield over Mental Health (DEF-01).
                    p.spawn((ShieldBack, Node { width: Val::Px(BAR_W), height: Val::Px(6.0), ..default() }, BackgroundColor(Color::srgb(0.18, 0.22, 0.28)), Visibility::Hidden))
                        .with_children(|b| {
                            b.spawn((ShieldFill, Node { width: Val::Px(BAR_W), height: Val::Percent(100.0), ..default() }, BackgroundColor(SHIELD)));
                        });
                    p.spawn((MhText, Text::new(""), TextFont { font_size: 16.0, ..default() }, TextColor(theme::PAPER)));
                    // Dash charges (DASH-01).
                    p.spawn((DashText, Text::new(""), TextFont { font_size: 15.0, ..default() }, TextColor(Color::srgb(0.6, 0.85, 1.0))));
                    p.spawn((Statuses, Text::new(""), TextFont { font_size: 14.0, ..default() }, TextColor(theme::DANGER)));
                });
            });
            // The wave timer, as a video progress bar; or the boss's health.
            root.spawn(Node { position_type: PositionType::Absolute, top: Val::Px(16.0), width: Val::Percent(100.0), justify_content: JustifyContent::Center, ..default() })
                .with_children(|row| {
                    let (mut node, bg) = panel();
                    node.align_items = AlignItems::Center;
                    row.spawn((node, bg)).with_children(|p| {
                        p.spawn((WaveText, Text::new(""), TextFont { font_size: 22.0, ..default() }, TextColor(theme::PAPER)));
                        p.spawn((Node { width: Val::Px(PROGRESS_W), height: Val::Px(10.0), ..default() }, BackgroundColor(Color::srgb(0.32, 0.32, 0.36))))
                            .with_children(|b| {
                                b.spawn((Progress, Node { width: Val::Px(0.0), height: Val::Percent(100.0), ..default() }, BackgroundColor(theme::PLAYER_RED)));
                            });
                        p.spawn((TimeText, Text::new(""), TextFont { font_size: 15.0, ..default() }, TextColor(theme::PAPER)));
                        // How good the AI is, and how much of you is left (CAP-01, AUTO-01).
                        p.spawn((CapText, Text::new(""), TextFont { font_size: 13.0, ..default() }, TextColor(Color::srgb(0.55, 0.75, 1.0))));
                    });
                });
            // Clout and level.
            root.spawn(Node { position_type: PositionType::Absolute, right: Val::Px(20.0), top: Val::Px(16.0), ..default() }).with_children(|w| {
                let (node, bg) = panel();
                w.spawn((node, bg)).with_children(|p| {
                    p.spawn(Node { column_gap: Val::Px(8.0), align_items: AlignItems::Center, ..default() }).with_children(|r| {
                        r.spawn((ImageNode::new(art.coin.clone()), Node { width: Val::Px(26.0), height: Val::Px(26.0), ..default() }));
                        r.spawn((CloutText, Text::new("0"), TextFont { font_size: 24.0, ..default() }, TextColor(theme::PAPER)));
                        label(r, "Clout", 14.0, theme::QUIET);
                    });
                    p.spawn((LevelText, Text::new(""), TextFont { font_size: 14.0, ..default() }, TextColor(theme::QUIET)));
                    // What the model has learned about you (MODEL-01).
                    p.spawn((ModelText, Text::new(""), TextFont { font_size: 14.0, ..default() }, TextColor(Color::srgb(0.95, 0.75, 0.45))));
                });
            });
            // Experience: a bar across the whole top edge.
            root.spawn((Node { position_type: PositionType::Absolute, top: Val::Px(0.0), left: Val::Px(0.0), width: Val::Percent(100.0), height: Val::Px(8.0), ..default() }, BackgroundColor(Color::srgba(0.1, 0.1, 0.14, 0.9))))
                .with_children(|b| {
                    b.spawn((XpFill, Node { width: Val::Percent(0.0), height: Val::Percent(100.0), ..default() }, BackgroundColor(Color::srgb(0.38, 0.75, 1.0))));
                });
            // News: patch notes, the next platform.
            root.spawn((Toasts, Text::new(""), TextFont { font_size: 14.0, ..default() }, TextColor(theme::QUIET), TextLayout::new_with_justify(Justify::Center), Node { position_type: PositionType::Absolute, bottom: Val::Px(34.0), width: Val::Percent(100.0), ..default() }));
            // The way to your build: pause shows everything.
            root.spawn((Node { position_type: PositionType::Absolute, left: Val::Px(20.0), bottom: Val::Px(14.0), ..default() }, BackgroundColor(Color::NONE)))
                .with_children(|h| {
                    h.spawn((KeysHint, Text::new(""), TextFont { font_size: 14.0, ..default() }, TextColor(Color::srgba(1.0, 1.0, 1.0, 0.45))));
                });
            // Why your controls are reversed, right where you look (STATUS-02).
            root.spawn((Node { position_type: PositionType::Absolute, top: Val::Px(150.0), width: Val::Percent(100.0), justify_content: JustifyContent::Center, ..default() },)).with_children(|r| {
                r.spawn((Reversed, Text::new(""), TextFont { font_size: 18.0, ..default() }, TextColor(Color::srgb(0.75, 0.55, 1.0)), TextLayout::new_with_justify(Justify::Center)));
            });
            // A livestream's clock (EVENT-02).
            root.spawn((Node { position_type: PositionType::Absolute, top: Val::Px(196.0), width: Val::Percent(100.0), justify_content: JustifyContent::Center, ..default() },)).with_children(|r| {
                r.spawn((LiveText, Text::new(""), TextFont { font_size: 16.0, ..default() }, TextColor(Color::srgb(1.0, 0.3, 0.3))));
            });
            // Chests off screen: an icon at the edge pointing the way (LOOT-02).
            for _ in 0..CHEST_ARROWS {
                root.spawn((ChestArrow, ImageNode::new(art.golden_chest.clone()), Node { position_type: PositionType::Absolute, width: Val::Px(40.0), height: Val::Px(40.0), ..default() }, Visibility::Hidden));
            }
        });

    // Banners (a boss, its defeat) are a strip under the clock, never over
    // the middle of the Feed (FAIR-05).
    commands
        .spawn((Banner, Node { position_type: PositionType::Absolute, top: Val::Px(122.0), width: Val::Percent(100.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: Val::Px(2.0), ..default() }, GlobalZIndex(6), Visibility::Hidden))
        .with_children(|b| {
            b.spawn((BannerTitle, Text::new(""), TextFont { font_size: 24.0, ..default() }, TextColor(theme::PAPER)));
            b.spawn((BannerLine, Text::new(""), TextFont { font_size: 15.0, ..default() }, TextColor(theme::QUIET)));
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
    mut vignette: Local<f32>,
    mut nodes: ParamSet<(
        Query<(&mut Node, &mut BackgroundColor), With<MhFill>>,
        Query<&mut Node, With<Progress>>,
        Query<&mut Node, With<XpFill>>,
        Query<&mut BackgroundColor, With<Vignette>>,
        Query<&mut Node, With<PopupClose>>,
    )>,
    mut texts: ParamSet<(
        Query<&mut Text, With<MhText>>,
        Query<&mut Text, With<Statuses>>,
        Query<&mut Text, With<WaveText>>,
        Query<&mut Text, With<TimeText>>,
        Query<&mut Text, With<CloutText>>,
        Query<&mut Text, With<LevelText>>,
        Query<&mut Text, With<BannerTitle>>,
    )>,
    mut banner_line: Query<&mut Text, (With<BannerLine>, Without<BannerTitle>, Without<MhText>, Without<Statuses>, Without<WaveText>, Without<TimeText>, Without<CloutText>, Without<LevelText>, Without<Toasts>)>,
    mut toasts: Query<&mut Text, (With<Toasts>, Without<BannerLine>, Without<BannerTitle>, Without<MhText>, Without<Statuses>, Without<WaveText>, Without<TimeText>, Without<CloutText>, Without<LevelText>)>,
    mut shown: ParamSet<(
        Query<&mut Visibility, With<Hud>>,
        Query<&mut Visibility, With<Ad>>,
        Query<&mut Visibility, With<Popup>>,
        Query<&mut Visibility, With<Banner>>,
    )>,
) {
    let vis = |on: bool| if on { Visibility::Inherited } else { Visibility::Hidden };
    let run_screen = game.screen == Screen::Run;
    let between = game.run.as_ref().is_some_and(|r| r.is_over() && game.since_end > crate::play::END_PAUSE);
    let Some(run) = game.run.as_ref().filter(|_| run_screen) else {
        if let Ok(mut v) = shown.p0().single_mut() {
            *v = Visibility::Hidden;
        }
        return;
    };
    let Some(world) = run.world.as_ref() else { return };
    let p = &world.player;
    let share = (p.mental_health / p.stats.max_mental_health).clamp(0.0, 1.0);
    let in_wave = matches!(run.phase, Phase::Playing);

    if let Ok((mut node, mut bg)) = nodes.p0().single_mut() {
        node.width = Val::Px(BAR_W * share);
        // The chapter's colour, towards red as it drains (CH-01).
        let c = run.chapter.def().color;
        bg.0 = Color::srgb(c[0], c[1], c[2]).mix(&theme::DANGER, (1.0 - share) * 0.8);
    }
    let boss = world.boss.as_ref().and_then(|b| world.boss_health().map(|h| (b.kind, h)));
    if let Ok(mut node) = nodes.p1().single_mut() {
        let to_boss = run.boss_in().map_or(1.0, |s| 1.0 - s / slop_rules::waves::BOSS_AFTER);
        node.width = Val::Px(PROGRESS_W * boss.map_or(to_boss.clamp(0.0, 1.0), |(_, h)| h));
    }
    let progress = run.xp as f32 / slop_rules::levels::xp_for(run.level + 1) as f32;
    if let Ok(mut node) = nodes.p2().single_mut() {
        node.width = Val::Percent(100.0 * progress.min(1.0));
    }
    if game.events.iter().any(|e| matches!(e, Event::PlayerHurt { .. })) {
        *vignette = 0.28;
    }
    *vignette = (*vignette - time.delta_secs() * 0.9).max(0.0);
    if let Ok(mut bg) = nodes.p3().single_mut() {
        bg.0 = theme::DANGER.with_alpha(*vignette);
    }
    if let Ok(mut node) = nodes.p4().single_mut() {
        // The close button runs away from your mouse. And from you.
        node.right = Val::Px(8.0 + 60.0 * (time.elapsed_secs() * 3.0).sin().abs());
    }

    if let Ok(mut t) = texts.p0().single_mut() {
        **t = format!("{:.0} / {:.0}", p.mental_health.ceil(), p.stats.max_mental_health);
    }
    if let Ok(mut t) = texts.p1().single_mut() {
        let s = &p.status;
        let mut list = Vec::new();
        if s.anxiety > 0 {
            list.push(format!("Anxiety x{}", s.anxiety));
        }
        if s.doomscroll > 0 {
            list.push("Doomscroll (slowed)".into());
        }
        if s.ratio > 0 {
            list.push("Ratio'd".into());
        }
        if s.brainrot > 0 {
            list.push("Brainrot (slowed)".into());
        }
        if p.grabbed > 0 {
            list.push(format!("Grabbed x{}", p.grabbed));
        }
        **t = list.join("  ");
    }
    if let Ok(mut t) = texts.p2().single_mut() {
        // Where you are: the biome under your feet (BIOME-01).
        **t = format!("{}   IN {}", clock(run.seconds()), world.biome.name().to_uppercase());
    }
    if let Ok(mut t) = texts.p3().single_mut() {
        **t = match boss {
            Some((BossKind::TheModel, h)) => format!("Model Collapse {:.0}%    {:.0}% left", world.setup.model.collapse * 100.0, h * 100.0),
            Some((kind, h)) => format!("{}: {:.0}% left", kind.name(), h * 100.0),
            None if run.stage >= 6 => format!("{} in {}", run.next_boss().name(), clock(run.boss_in().unwrap_or(0.0))),
            None => format!("Boss {}/6: {} ({}) in {}", run.stage + 1, run.next_boss().name(), run.platform.name(), clock(run.boss_in().unwrap_or(0.0))),
        };
    }
    if let Ok(mut t) = texts.p4().single_mut() {
        **t = run.clout.to_string();
    }
    if let Ok(mut t) = texts.p5().single_mut() {
        **t = format!("Level {}  -  {} deleted", run.level, run.kills());
    }
    if let Ok(mut t) = toasts.single_mut() {
        // Overlays have the screen; the news waits.
        let words: Vec<&str> = if game.choosing() { Vec::new() } else { game.toasts.iter().rev().take(3).map(|(w, _)| w.as_str()).collect() };
        **t = words.join("
");
    }
    let banner = game.banner.as_ref().filter(|_| !between);
    if let (Some((title, line, _)), Ok(mut t)) = (banner, texts.p6().single_mut()) {
        **t = title.clone();
        if let Ok(mut l) = banner_line.single_mut() {
            **l = line.clone();
        }
    }

    if let Ok(mut v) = shown.p0().single_mut() {
        *v = vis(!between && !game.paused);
    }
    if let Ok(mut v) = shown.p1().single_mut() {
        // Nothing covers the Feed (FAIR-05): no ad overlay.
        *v = Visibility::Hidden;
        let _ = in_wave;
    }
    if let Ok(mut v) = shown.p2().single_mut() {
        // No pop-up dialog either.
        *v = Visibility::Hidden;
    }
    if let Ok(mut v) = shown.p3().single_mut() {
        *v = vis(banner.is_some() && !game.paused);
    }
}
