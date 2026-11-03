//! Keyboard and mouse control of every screen (UI-14): focusable buttons and
//! cards, arrow-key navigation by position, a focus ring, and tooltips.
//!
//! Screens are rebuilt from scratch when the run changes, so focus is held
//! as a stable key rather than an entity. Enter (or E) presses the focused
//! element; Space presses the screen's `Continue` element if it has one, and
//! the focused element otherwise. The mouse focuses what it hovers.

use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;
use bevy::window::PrimaryWindow;

use crate::play::Simulation;
use crate::state::{Game, UiAction};
use crate::theme;

pub struct UiKitPlugin;

impl Plugin for UiKitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Nav>()
            .add_systems(Startup, setup)
            .add_systems(Update, (hover_focus, navigate, activate, ring, tooltip).chain().after(Simulation).after(crate::screens::Rebuilt));
    }
}

/// What a button or card does when pressed.
#[derive(Component, Clone, Copy)]
pub struct Act(pub UiAction);

/// The other action, on L, X or a right click: lock an offer, sell a weapon.
#[derive(Component, Clone, Copy)]
pub struct Alt(pub UiAction);

/// Background colours: normal, and focused or hovered.
#[derive(Component, Clone, Copy)]
pub struct Shade(pub Color, pub Color);

/// Can hold focus. `key` must be unique on a screen and stable across rebuilds.
#[derive(Component, Clone, Copy)]
pub struct Focusable(pub u32);

/// Focused when a screen opens and nothing else is.
#[derive(Component)]
pub struct DefaultFocus;

/// Pressed by Space.
#[derive(Component)]
pub struct Continue;

/// Shown next to the element while it has focus.
#[derive(Component, Clone)]
pub struct Tip {
    pub title: String,
    pub body: String,
    pub color: Color,
}

impl Tip {
    pub fn new(title: impl Into<String>, body: impl Into<String>, color: Color) -> Tip {
        Tip { title: title.into(), body: body.into(), color }
    }
}

/// Where a screen shows tooltips, filling this node, instead of next to the
/// focused element.
#[derive(Component)]
pub struct TipDock;

/// The focused element's key.
#[derive(Resource, Default)]
pub struct Nav {
    pub key: Option<u32>,
    /// Centre of the focused element; when it vanishes (an offer bought),
    /// focus goes to whatever is now nearest. None on a fresh screen.
    pub at: Option<Vec2>,
    /// Set when focus moved this frame, for the tick sound.
    pub moved: bool,
}

#[derive(Component)]
struct Tooltip;
#[derive(Component)]
struct TooltipTitle;
#[derive(Component)]
struct TooltipBody;

const TOOLTIP_W: f32 = 380.0;
pub const FOCUS: Color = Color::srgb(1.0, 0.86, 0.3);

/// The outline every focusable element carries; only the focused one shows it.
pub fn focus_ring() -> Outline {
    Outline::new(Val::Px(3.0), Val::Px(3.0), Color::NONE)
}

fn setup(mut commands: Commands) {
    commands
        .spawn((
            Tooltip,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(TOOLTIP_W),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(6.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.07, 0.07, 0.1)),
            BorderColor::all(FOCUS),
            GlobalZIndex(60),
            Visibility::Hidden,
        ))
        .with_children(|t| {
            t.spawn((TooltipTitle, Text::new(""), TextFont { font_size: 19.0, ..default() }, TextColor(theme::PAPER)));
            t.spawn((TooltipBody, Text::new(""), TextFont { font_size: 15.0, ..default() }, TextColor(Color::srgb(0.82, 0.82, 0.86))));
        });
}

fn hover_focus(mut nav: ResMut<Nav>, hovered: Query<(&Interaction, &Focusable), Changed<Interaction>>) {
    for (i, f) in &hovered {
        if *i != Interaction::None && nav.key != Some(f.0) {
            nav.key = Some(f.0);
            nav.moved = true;
        }
    }
}

/// Centre and size of a node, in UI pixels.
fn rect(node: &ComputedNode, tf: &UiGlobalTransform) -> (Vec2, Vec2) {
    (tf.translation * node.inverse_scale_factor, node.size() * node.inverse_scale_factor)
}

/// The best element from `from` in direction `dir`: near along the direction,
/// and much nearer across it.
fn pick(from: (Vec2, Vec2), dir: Vec2, all: &[(u32, Vec2, Vec2)]) -> Option<u32> {
    let (c, size) = from;
    all.iter()
        .filter_map(|&(key, p, s)| {
            let d = p - c;
            let along = d.dot(dir);
            // Ahead of us, beyond our own half-size.
            let reach = (size * dir.abs()).length() / 2.0 + (s * dir.abs()).length() / 2.0;
            if along <= 1.0 || along < reach * 0.25 {
                return None;
            }
            let across = (d - dir * along).length();
            let overlap = (size * dir.perp().abs()).length() / 2.0 + (s * dir.perp().abs()).length() / 2.0;
            let across = (across - overlap).max(0.0);
            Some((key, along + across * 3.0))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(k, _)| k)
}

fn navigate(
    keyboard: Res<ButtonInput<KeyCode>>,
    game: Res<Game>,
    mut nav: ResMut<Nav>,
    items: Query<(&Focusable, &ComputedNode, &UiGlobalTransform, Option<&DefaultFocus>)>,
) {
    nav.moved = false;
    if game.bot {
        return;
    }
    let all: Vec<(u32, Vec2, Vec2)> = items.iter().filter(|(_, n, ..)| n.size().x > 0.0).map(|(f, n, t, _)| (f.0, rect(n, t).0, rect(n, t).1)).collect();
    if all.is_empty() {
        return;
    }
    let current = nav.key.and_then(|k| all.iter().find(|a| a.0 == k));
    let Some(&(_, c, s)) = current else {
        nav.key = match nav.at {
            // The focused element is gone: the nearest one takes over.
            Some(at) => all.iter().min_by(|a, b| a.1.distance_squared(at).total_cmp(&b.1.distance_squared(at))).map(|a| a.0),
            // A fresh screen: the default element, or the first one.
            None => items.iter().find(|(.., d)| d.is_some()).map(|(f, ..)| f.0).or(Some(all[0].0)),
        };
        return;
    };
    nav.at = Some(c);
    let pressed = |a: KeyCode, b: KeyCode| keyboard.just_pressed(a) || keyboard.just_pressed(b);
    let dir = if pressed(KeyCode::ArrowLeft, KeyCode::KeyA) {
        Vec2::NEG_X
    } else if pressed(KeyCode::ArrowRight, KeyCode::KeyD) {
        Vec2::X
    } else if pressed(KeyCode::ArrowUp, KeyCode::KeyW) {
        // UI y grows downwards.
        Vec2::NEG_Y
    } else if pressed(KeyCode::ArrowDown, KeyCode::KeyS) {
        Vec2::Y
    } else {
        return;
    };
    if let Some(k) = pick((c, s), dir, &all) {
        nav.key = Some(k);
        nav.moved = true;
    }
}

fn activate(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game: ResMut<Game>,
    nav: Res<Nav>,
    items: Query<(&Focusable, &Act)>,
    alts: Query<(&Focusable, &Alt)>,
    continues: Query<&Act, With<Continue>>,
    mut exit: MessageWriter<AppExit>,
) {
    // Alt+Enter toggles fullscreen and presses nothing.
    if game.bot || keyboard.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]) {
        return;
    }
    if keyboard.just_pressed(KeyCode::KeyL) || keyboard.just_pressed(KeyCode::KeyX) {
        if let Some((_, alt)) = nav.key.and_then(|k| alts.iter().find(|(f, _)| f.0 == k)) {
            game.act(alt.0);
        }
        return;
    }
    let enter = keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::NumpadEnter) || keyboard.just_pressed(KeyCode::KeyE);
    let space = keyboard.just_pressed(KeyCode::Space);
    if !enter && !space {
        return;
    }
    let focused = nav.key.and_then(|k| items.iter().find(|(f, _)| f.0 == k)).map(|(_, a)| a.0);
    let action = if space { continues.iter().next().map(|a| a.0).or(focused) } else { focused };
    if let Some(a) = action {
        if !game.act(a) {
            exit.write(AppExit::Success);
        }
    }
}

#[allow(clippy::type_complexity)]
fn ring(nav: Res<Nav>, mut items: Query<(&Focusable, &mut Outline, Option<(&Shade, &mut BackgroundColor)>)>) {
    for (f, mut o, shade) in &mut items {
        let on = Some(f.0) == nav.key;
        let want = if on { FOCUS } else { Color::NONE };
        if o.color != want {
            o.color = want;
        }
        if let Some((shade, mut bg)) = shade {
            let want = if on { shade.1 } else { shade.0 };
            if bg.0 != want {
                bg.0 = want;
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn tooltip(
    nav: Res<Nav>,
    windows: Query<&Window, With<PrimaryWindow>>,
    items: Query<(&Focusable, &Tip, &ComputedNode, &UiGlobalTransform)>,
    docks: Query<(&ComputedNode, &UiGlobalTransform), With<TipDock>>,
    mut tip: Query<(&mut Node, &mut Visibility, &mut BorderColor, &ComputedNode), With<Tooltip>>,
    mut title: Query<(&mut Text, &mut TextColor), (With<TooltipTitle>, Without<TooltipBody>)>,
    mut body: Query<&mut Text, (With<TooltipBody>, Without<TooltipTitle>)>,
) {
    let Ok((mut node, mut vis, mut border, own)) = tip.single_mut() else { return };
    let focused = nav.key.and_then(|k| items.iter().find(|(f, ..)| f.0 == k));
    let Some((_, t, n, tf)) = focused else {
        *vis = Visibility::Hidden;
        return;
    };
    *vis = Visibility::Visible;
    if let Ok((mut text, mut color)) = title.single_mut() {
        if text.0 != t.title {
            text.0 = t.title.clone();
        }
        color.0 = t.color;
    }
    if let Ok(mut text) = body.single_mut() {
        if text.0 != t.body {
            text.0 = t.body.clone();
        }
    }
    *border = BorderColor::all(t.color);
    if let Some((dn, dtf)) = docks.iter().find(|(dn, _)| dn.size().x > 0.0) {
        let (c, s) = rect(dn, dtf);
        node.left = Val::Px(c.x - s.x / 2.0);
        node.top = Val::Px(c.y - s.y / 2.0);
        node.width = Val::Px(s.x);
        node.min_height = Val::Px(s.y);
        return;
    }
    node.width = Val::Px(TOOLTIP_W);
    node.min_height = Val::Auto;
    let (c, s) = rect(n, tf);
    let screen = windows.single().map_or(Vec2::new(1600.0, 900.0), |w| Vec2::new(w.physical_width() as f32, w.physical_height() as f32) * n.inverse_scale_factor);
    let height = own.size().y * own.inverse_scale_factor;
    let right = c.x + s.x / 2.0 + 14.0;
    let left = if right + TOOLTIP_W < screen.x - 8.0 { right } else { (c.x - s.x / 2.0 - 14.0 - TOOLTIP_W).max(8.0) };
    let top = (c.y - s.y / 2.0).min(screen.y - height - 8.0).max(8.0);
    node.left = Val::Px(left);
    node.top = Val::Px(top);
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: UI-14
    #[test]
    fn arrows_pick_the_nearest_element_in_their_direction() {
        let at = |x: f32, y: f32| (Vec2::new(x, y), Vec2::new(100.0, 60.0));
        let row = [(1, Vec2::new(0.0, 0.0), Vec2::new(100.0, 60.0)), (2, Vec2::new(120.0, 0.0), Vec2::new(100.0, 60.0)), (3, Vec2::new(240.0, 0.0), Vec2::new(100.0, 60.0)), (4, Vec2::new(130.0, 200.0), Vec2::new(300.0, 60.0))];
        assert_eq!(pick(at(0.0, 0.0), Vec2::X, &row), Some(2));
        assert_eq!(pick(at(120.0, 0.0), Vec2::NEG_X, &row), Some(1));
        assert_eq!(pick(at(240.0, 0.0), Vec2::X, &row), None);
        assert_eq!(pick(at(240.0, 0.0), Vec2::Y, &row), Some(4), "a wide element below is reachable from any column");
        assert_eq!(pick(at(130.0, 200.0), Vec2::NEG_Y, &row), Some(2));
    }
}
