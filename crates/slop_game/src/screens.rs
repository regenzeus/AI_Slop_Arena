//! The screens (UI-08 to UI-13): title, hero select, and over the paused
//! Feed: level-ups, chests and merges, vending machines, shrines, pause with
//! your whole build, and the end of a run. They are rebuilt whenever
//! something changes, and every one works by keyboard and gamepad (`ui`).

use bevy::prelude::*;
use slop_rules::items::{ItemKind, StatKind, BLESSINGS, ITEMS};
use slop_rules::run::{Choice, Reward};
use slop_rules::{Hero, Phase, Run};
use slop_sim::content::{level_note, merge_of, Pattern, LEVEL_DAMAGE, WEAPONS};
use slop_sim::world::MAX_LEVEL;
use slop_sim::DamageType;

use crate::art::Art;
use crate::controls::Controls;
use crate::play::{Simulation, END_PAUSE};
use crate::state::{Game, Screen, UiAction};
use crate::theme;
use crate::ui::{focus_ring, Act, Continue, DefaultFocus, Focusable, Nav, Shade, Tip, TipDock};

pub struct ScreensPlugin;

impl Plugin for ScreensPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (clicks, rebuild).chain().in_set(Rebuilt).after(Simulation));
    }
}

/// Screens are rebuilt in this set; navigation runs after it.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Rebuilt;

#[derive(Component)]
struct ScreenRoot;

/// Focus keys, by kind of element.
pub mod key {
    pub const BUTTON: u32 = 0;
    pub const HERO: u32 = 10;
    pub const CHOICE: u32 = 20;
    pub const WEAPON: u32 = 200;
    pub const ITEM: u32 = 300;
    pub const BLESSING: u32 = 400;
    pub const STAT: u32 = 500;
    pub const RESIST: u32 = 520;
}

/// Bronze, silver, gold and diamond.
pub fn tier_color(tier: u8) -> Color {
    match tier {
        1 => Color::srgb(0.72, 0.51, 0.29),
        2 => Color::srgb(0.66, 0.70, 0.75),
        3 => Color::srgb(0.91, 0.71, 0.20),
        _ => Color::srgb(0.44, 0.83, 0.91),
    }
}

/// A weapon level's frame: bronze early, gold near the top, diamond at max.
pub fn level_color(level: u8) -> Color {
    tier_color(match level {
        0..=2 => 1,
        3..=5 => 2,
        6..=7 => 3,
        _ => 4,
    })
}

const CARD: Color = Color::srgb(0.15, 0.14, 0.2);
const BUTTON: Color = Color::srgb(0.30, 0.30, 0.38);
const BACKDROP: Color = Color::srgba(0.06, 0.06, 0.09, 0.8);
const MERGE: Color = Color::srgb(1.0, 0.78, 0.25);

fn label(p: &mut ChildSpawnerCommands, words: impl Into<String>, size: f32, color: Color) {
    p.spawn((Text::new(words), TextFont { font_size: size, ..default() }, TextColor(color)));
}

fn icon(p: &mut ChildSpawnerCommands, image: &Handle<Image>, size: f32) {
    p.spawn((ImageNode::new(image.clone()), Node { width: Val::Px(size), height: Val::Px(size), flex_shrink: 0.0, ..default() }));
}

fn row(gap: f32) -> Node {
    Node { column_gap: Val::Px(gap), align_items: AlignItems::Center, ..default() }
}

fn column(gap: f32) -> Node {
    Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(gap), ..default() }
}

/// Everything a focusable element needs besides its node and looks.
fn focusable(k: u32, shade: Color) -> (Button, Focusable, Outline, Shade, BackgroundColor) {
    let hover = if shade.alpha() < 0.01 { Color::srgba(1.0, 1.0, 1.0, 0.08) } else { shade.mix(&Color::WHITE, 0.15) };
    (Button, Focusable(k), focus_ring(), Shade(shade, hover), BackgroundColor(shade))
}

fn button<'a>(p: &'a mut ChildSpawnerCommands, words: impl Into<String>, action: UiAction, color: Color, k: u32) -> EntityCommands<'a> {
    let words = words.into();
    let mut e = p.spawn((
        focusable(k, color),
        Act(action),
        Node { padding: UiRect::axes(Val::Px(20.0), Val::Px(11.0)), justify_content: JustifyContent::Center, align_items: AlignItems::Center, column_gap: Val::Px(8.0), ..default() },
    ));
    e.with_children(|b| label(b, words, 20.0, theme::PAPER));
    e
}

fn full() -> Node {
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

fn backdrop(p: &mut ChildSpawnerCommands, color: Color) {
    p.spawn((Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }, BackgroundColor(color)));
}

fn card(width: f32) -> Node {
    Node { width: Val::Px(width), flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(12.0)), row_gap: Val::Px(6.0), border: UiRect::all(Val::Px(3.0)), ..default() }
}

/// Keys and what they do, in the hint bar at the bottom of a screen; the
/// gamepad's buttons when a gamepad was used last.
fn hints(p: &mut ChildSpawnerCommands, pad: bool, keys: &[(&str, &str, &str)]) {
    p.spawn(Node { position_type: PositionType::Absolute, bottom: Val::Px(10.0), width: Val::Percent(100.0), justify_content: JustifyContent::Center, column_gap: Val::Px(18.0), ..default() })
        .with_children(|r| {
            for (k, pk, what) in keys {
                let k = if pad { pk } else { k };
                if k.is_empty() {
                    continue;
                }
                r.spawn(row(6.0)).with_children(|h| {
                    h.spawn((Node { padding: UiRect::axes(Val::Px(7.0), Val::Px(2.0)), border: UiRect::all(Val::Px(1.0)), ..default() }, BorderColor::all(theme::QUIET), BackgroundColor(Color::srgb(0.2, 0.2, 0.26))))
                        .with_children(|c| label(c, *k, 14.0, theme::PAPER));
                    label(h, *what, 14.0, theme::QUIET);
                });
            }
        });
}

fn clicks(mut game: ResMut<Game>, buttons: Query<(&Interaction, &Act), Changed<Interaction>>, mut exit: MessageWriter<AppExit>) {
    if game.bot {
        return;
    }
    for (interaction, act) in &buttons {
        if *interaction == Interaction::Pressed && !game.act(act.0) {
            exit.write(AppExit::Success);
        }
    }
}

/// What is on screen, to know when to rebuild.
#[derive(Default, PartialEq, Clone)]
struct Shown {
    screen: Option<Screen>,
    phase: String,
    settled: bool,
    paused: bool,
    pad: bool,
}

fn rebuild(mut commands: Commands, mut game: ResMut<Game>, mut nav: ResMut<Nav>, controls: Res<Controls>, art: Res<Art>, roots: Query<Entity, With<ScreenRoot>>, mut last: Local<Shown>) {
    let phase = game.run.as_ref().map_or(String::new(), |r| format!("{:?}", std::mem::discriminant(&r.phase)));
    let shown = Shown { screen: Some(game.screen), phase, settled: game.since_end > END_PAUSE, paused: game.paused, pad: controls.pad };
    if !game.dirty && *last == shown {
        return;
    }
    if last.screen != shown.screen || last.phase != shown.phase || last.paused != shown.paused {
        // A different screen starts at its default element.
        nav.key = None;
        nav.at = None;
    }
    *last = shown;
    game.dirty = false;
    for e in &roots {
        commands.entity(e).despawn();
    }
    let game = &*game;
    let pad = controls.pad;
    let root = commands.spawn((ScreenRoot, full(), GlobalZIndex(20))).id();
    commands.entity(root).with_children(|p| match game.screen {
        Screen::Title => title(p, &art, pad),
        Screen::Heroes => heroes(p, game, &art, pad),
        Screen::Run => {
            if let Some(run) = &game.run {
                match &run.phase {
                    Phase::Playing if game.paused => pause(p, run, &art, pad),
                    Phase::LevelUp(choices) => level_up(p, run, choices, &art, pad),
                    Phase::Chest(reward) => chest(p, run, reward, &art, pad),
                    Phase::Vending { offers, .. } => vending(p, run, offers, &art, pad),
                    Phase::Shrine { offers, .. } => shrine(p, offers, &art, pad),
                    Phase::Over { victory } if game.since_end > END_PAUSE => over(p, run, *victory, &art, pad),
                    _ => {}
                }
            }
        }
    });
    if let Some((words, _)) = &game.message {
        commands.spawn((ScreenRoot, Node { position_type: PositionType::Absolute, bottom: Val::Px(48.0), width: Val::Percent(100.0), justify_content: JustifyContent::Center, ..default() }, GlobalZIndex(30)))
            .with_children(|p| {
                p.spawn((Node { padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)), ..default() }, BackgroundColor(theme::DANGER))).with_children(|m| label(m, words.clone(), 18.0, theme::PAPER));
            });
    }
}

fn title(p: &mut ChildSpawnerCommands, art: &Art, pad: bool) {
    backdrop(p, Color::srgba(0.07, 0.07, 0.11, 0.55));
    p.spawn(row(18.0)).with_children(|r| {
        for (i, h) in Hero::ALL.iter().enumerate() {
            icon(r, &art.human[h][if i == 2 { 2 } else { 0 }], 84.0);
        }
    });
    label(p, "AI SLOP ARENA", 92.0, theme::PAPER);
    label(p, "One human. An endless feed of AI slop. Protect your Mental Health.", 24.0, theme::QUIET);
    p.spawn(Node { height: Val::Px(16.0), ..default() });
    for line in [
        "Move. Your weapons fire on their own. Every level: pick one of three.",
        "Six platforms, six bosses, then The Model. The Feed never ends.",
        "Max out two weapons that belong together and a chest merges them.",
        "The slop retrains on how you kill it. Adapt, or get fine-tuned.",
    ] {
        label(p, line, 19.0, theme::PAPER);
    }
    p.spawn(Node { height: Val::Px(20.0), ..default() });
    p.spawn(row(14.0)).with_children(|r| {
        button(r, "Play", UiAction::ToHeroes, theme::PLAYER_RED, key::BUTTON).insert((Continue, DefaultFocus));
        button(r, "Quit", UiAction::Quit, BUTTON, key::BUTTON + 1);
    });
    label(p, "Keyboard or controller. Alt+Enter or F11: fullscreen. F12: screenshot.", 15.0, theme::QUIET);
    hints(p, pad, &[("Arrows/WASD", "Stick/D-pad", "choose"), ("Enter", "A", "press"), ("Space", "Start", "play")]);
}

fn weapon_tip(def: usize, level: u8) -> Tip {
    let w = &WEAPONS[def];
    let dmg = w.damage * LEVEL_DAMAGE[level.clamp(1, MAX_LEVEL) as usize - 1];
    let mut body = format!("{} damage: {}.\n{:.0} damage every {:.2} s, range {:.0}.\n\"{}\"", w.kind.name(), pattern_words(w.pattern), dmg, w.cooldown, w.range, w.flavour);
    if let Some((other, into)) = merge_of(def) {
        body = format!("{body}\n\nMERGE: level 8 with {} at level 8 becomes {}.", WEAPONS[other].name, WEAPONS[into].name);
    }
    if w.evolved {
        body = format!("{body}\n\nA merged weapon.");
    }
    Tip::new(format!("{} - level {level}", w.name), body, theme::type_color(w.kind))
}

fn pattern_words(p: Pattern) -> &'static str {
    match p {
        Pattern::Shot { .. } => "a shot at the nearest slop",
        Pattern::Burst { .. } => "a fan of shots",
        Pattern::Lob { .. } => "lobbed over walls; splashes where it lands",
        Pattern::Mark { .. } => "marks slop, which then takes more damage from everything",
        Pattern::Ricochet { .. } => "bounces from slop to slop",
        Pattern::Boomerang { .. } => "flies out and comes back, hitting both ways",
        Pattern::Arc { .. } => "a melee swing",
        Pattern::Cone { .. } => "a short spray that pushes slop back",
        Pattern::Ring => "hits everything around you",
        Pattern::Aura => "pulses around you, faster the more slop is near",
        Pattern::Chain { .. } => "jumps from slop to slop",
        Pattern::Beam { .. } => "a beam through everything in line; strips beauty filters",
        Pattern::Reflect => "turns speech bubbles around at the slop",
        Pattern::Orbit { .. } => "a dog circles you and bites what it touches",
        Pattern::Zone { .. } => "plants a patch that hurts slop and heals you inside",
        Pattern::Slam { .. } => "a slam; small slop is deleted outright",
        Pattern::Stun { .. } => "stuns chatbots",
    }
}

/// What a damage type does besides damage (DMG-02).
fn type_effect(t: DamageType) -> &'static str {
    match t {
        DamageType::Analog => "Analog staggers and knocks back.",
        DamageType::Fact => "Fact cuts through shields, filters and Verified.",
        DamageType::Sarcasm => "Sarcasm can confuse: confused slop turn around and stop attacking.",
        DamageType::Grass => "Grass roots slop in place.",
    }
}

fn item_color(kind: ItemKind) -> Color {
    match kind {
        ItemKind::Risky => theme::DANGER,
        ItemKind::Hustle => tier_color(3),
        ItemKind::Active => theme::type_color(DamageType::Fact),
        ItemKind::Healthy => theme::AGREE,
    }
}

fn item_tip(item: usize, level: u32) -> Tip {
    let d = &ITEMS[item];
    Tip::new(format!("{} - level {level}/{}", d.name, d.max_level()), format!("{}.\n{} per level.\n\"{}\"", d.kind.name(), d.effect, d.flavour), item_color(d.kind))
}

fn heroes(p: &mut ChildSpawnerCommands, game: &Game, art: &Art, pad: bool) {
    backdrop(p, BACKDROP);
    label(p, "CHOOSE YOUR HUMAN", 50.0, theme::PAPER);
    label(p, "Same human, different baggage. Your hero sets your starting weapon, item and quirk.", 18.0, theme::QUIET);
    p.spawn(Node { column_gap: Val::Px(14.0), ..default() }).with_children(|r| {
        for (i, h) in Hero::ALL.iter().enumerate() {
            let d = h.def();
            let w = slop_sim::content::weapon_index(d.weapon).unwrap_or(0);
            let it = slop_rules::items::item_index(d.item).unwrap_or(0);
            let mut e = r.spawn((focusable(key::HERO + i as u32, CARD), Act(UiAction::StartAs(i)), card(280.0), BorderColor::all(theme::type_color(WEAPONS[w].kind))));
            if i == game.hero {
                e.insert(DefaultFocus);
            }
            e.with_children(|c| {
                c.spawn(Node { justify_content: JustifyContent::Center, ..default() }).with_children(|m| icon(m, &art.human[h][0], 110.0));
                label(c, format!("{}  {}", i + 1, d.name), 28.0, theme::PAPER);
                c.spawn(row(8.0)).with_children(|l| {
                    icon(l, &art.weapon_icons[w], 34.0);
                    label(l, d.weapon, 16.0, theme::type_color(WEAPONS[w].kind));
                });
                label(c, format!("{}: {}", WEAPONS[w].kind.name(), pattern_words(WEAPONS[w].pattern)), 13.0, theme::QUIET);
                c.spawn(row(8.0)).with_children(|l| {
                    icon(l, &art.item_icons[it], 34.0);
                    label(l, d.item, 16.0, item_color(ITEMS[it].kind));
                });
                label(c, d.stats, 15.0, theme::PAPER);
                label(c, d.quirk, 15.0, theme::QUIET);
            });
        }
    });
    hints(p, pad, &[("Arrows", "Stick", "choose"), ("1-5", "", "jump to a hero"), ("Enter/Space", "A", "start"), ("Esc", "B", "back")]);
}

/// Your stats and the slop's resistances, each with a tooltip (UI-13).
fn stats_panel(p: &mut ChildSpawnerCommands, run: &Run, art: &Art) {
    let s = run.stats();
    p.spawn((Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(3.0), padding: UiRect::all(Val::Px(14.0)), width: Val::Px(330.0), height: Val::Percent(100.0), flex_shrink: 0.0, ..default() }, BackgroundColor(theme::PANEL)))
        .with_children(|col| {
            col.spawn(row(10.0)).with_children(|h| {
                icon(h, &art.human[&run.hero][0], 46.0);
                h.spawn(column(2.0)).with_children(|t| {
                    label(t, run.hero.def().name, 22.0, theme::PAPER);
                    label(t, format!("Level {}  -  {}", run.level, run.platform.name()), 14.0, theme::QUIET);
                });
            });
            col.spawn(Node { height: Val::Px(6.0), ..default() });
            label(col, "STATS", 15.0, theme::QUIET);
            for (i, k) in StatKind::ALL.iter().enumerate() {
                col.spawn((focusable(key::STAT + i as u32, Color::NONE), Node { padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)), column_gap: Val::Px(10.0), align_items: AlignItems::Center, ..default() }, Tip::new(k.name(), k.explain(&s), theme::PAPER)))
                    .with_children(|r| {
                        icon(r, &art.stat_icons[k], 26.0);
                        r.spawn((Text::new(k.name()), TextFont { font_size: 16.0, ..default() }, TextColor(theme::PAPER), Node { flex_grow: 1.0, ..default() }));
                        label(r, k.read(&s), 16.0, theme::PAPER);
                    });
            }
            col.spawn(Node { height: Val::Px(8.0), ..default() });
            label(col, "SLOP RESISTANCE", 15.0, theme::QUIET);
            for t in DamageType::ALL {
                let r = run.training.model[t as usize];
                let said: String = run.notes.iter().flat_map(|n| &n.lines).filter(|l| l.text.starts_with(t.name())).map(|l| format!("\n\nLast retraining: {}", l.text)).collect();
                let n = run.weapons.iter().filter(|w| WEAPONS[w.def].kind == t).count();
                let set = match n {
                    0 | 1 => format!("Carry 2 {} weapons: +10% {} damage. Carry 4: a stronger effect.", t.name(), t.name()),
                    2 | 3 => format!("Set bonus: +10% {} damage ({n} weapons). Carry 4: a stronger effect.", t.name()),
                    _ => format!("Set bonus: +10% {} damage and a stronger effect ({n} weapons).", t.name()),
                };
                let tip = Tip::new(
                    format!("{} resistance {:+.0}%", t.name(), r * 100.0),
                    format!(
                        "Chatbots take {:.0}% damage from {} weapons; brainrot has its own genes on top.\n{}\n{}\nThe slop learns to resist what kills it. Resistance is zero-sum: when one rises, the others fall.{said}",
                        (1.0 - r) * 100.0,
                        t.name(),
                        type_effect(t),
                        set
                    ),
                    theme::type_color(t),
                );
                col.spawn((focusable(key::RESIST + t as u32, Color::NONE), Node { padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)), column_gap: Val::Px(10.0), align_items: AlignItems::Center, ..default() }, tip))
                    .with_children(|line| {
                        icon(line, &art.type_icons[t as usize], 24.0);
                        line.spawn((Text::new(format!("{}{}", t.name(), if n >= 2 { format!("  x{n}") } else { String::new() })), TextFont { font_size: 16.0, ..default() }, TextColor(theme::type_color(t)), Node { flex_grow: 1.0, ..default() }));
                        label(line, format!("{:+.0}%", r * 100.0), 16.0, if r > 0.01 { theme::DANGER } else if r < -0.01 { theme::AGREE } else { theme::PAPER });
                    });
            }
        });
}

/// The picture, name, tag line and words of a choice.
fn describe(run: &Run, c: Choice, art: &Art) -> (Handle<Image>, String, String, String, Color) {
    match c {
        Choice::NewWeapon(d) | Choice::WeaponLevel(d) => {
            let w = &WEAPONS[d];
            let (from, to) = run.levels_of(c).unwrap_or((0, 1));
            let tag = if from == 0 { format!("NEW {} weapon", w.kind.name()) } else { format!("Level {from} -> {to}") };
            let mut words = if from == 0 {
                format!("{}.\n{:.0} damage every {:.2} s.\n{}", capital(pattern_words(w.pattern)), w.damage, w.cooldown, type_effect(w.kind))
            } else {
                level_note(to as u8).to_string()
            };
            if let Some((other, into)) = merge_of(d) {
                let owned = run.weapons.iter().any(|x| x.def == other);
                words = format!("{words}\n\n{}Merges with {} into {}.", if owned { "You have its partner! " } else { "" }, WEAPONS[other].name, WEAPONS[into].name);
            }
            (art.weapon_icons[d].clone(), w.name.to_string(), tag, words, theme::type_color(w.kind))
        }
        Choice::NewItem(i) | Choice::ItemLevel(i) => {
            let d = &ITEMS[i];
            let (from, to) = run.levels_of(c).unwrap_or((0, 1));
            let tag = if from == 0 { format!("NEW {}", d.kind.name()) } else { format!("Level {from} -> {to} of {}", d.max_level()) };
            (art.item_icons[i].clone(), d.name.to_string(), tag, format!("{}.\n\"{}\"", d.effect, d.flavour), item_color(d.kind))
        }
        Choice::Stat(up) => {
            let grade = ["Stat", "Good stat", "Great stat"][up.tier as usize - 1];
            (art.stat_icons[&up.stat].clone(), up.describe(), grade.to_string(), up.stat.explain(&run.stats()), tier_color(up.tier + 1))
        }
        Choice::Heal => (art.pickups[&slop_sim::world::PickupKind::Food].clone(), "Touch Grass".into(), "Everything is maxed".into(), "Heal to full Mental Health.".into(), theme::AGREE),
        Choice::Clout(n) => (art.coin.clone(), format!("Brand Deal: {n} Clout"), "Everything is maxed".into(), "Money for the machines on the map.".into(), tier_color(3)),
    }
}

fn capital(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

/// A wide card for a choice; `action` takes it.
fn choice_card(p: &mut ChildSpawnerCommands, run: &Run, c: Choice, art: &Art, k: u32, action: UiAction, price: Option<u32>) {
    let (image, name, tag, words, color) = describe(run, c, art);
    let merge = matches!(c, Choice::WeaponLevel(d) | Choice::NewWeapon(d) if merge_of(d).is_some_and(|(o, _)| run.weapons.iter().any(|x| x.def == o)));
    let border = if merge { MERGE } else { color };
    let mut e = p.spawn((focusable(k, CARD), Act(action), Node { width: Val::Px(300.0), min_height: Val::Px(330.0), ..card(300.0) }, BorderColor::all(border)));
    if k == key::CHOICE {
        e.insert(DefaultFocus);
    }
    e.with_children(|c| {
        c.spawn(Node { justify_content: JustifyContent::SpaceBetween, ..default() }).with_children(|h| {
            label(h, format!("{}", k - key::CHOICE + 1), 16.0, theme::QUIET);
            label(h, tag, 15.0, color);
        });
        c.spawn(Node { justify_content: JustifyContent::Center, padding: UiRect::vertical(Val::Px(6.0)), ..default() }).with_children(|m| icon(m, &image, 96.0));
        label(c, name, 24.0, theme::PAPER);
        label(c, words, 15.0, theme::QUIET);
        c.spawn(Node { flex_grow: 1.0, ..default() });
        if let Some(price) = price {
            c.spawn(row(6.0)).with_children(|b| {
                icon(b, &art.coin, 24.0);
                label(b, price.to_string(), 24.0, if price <= run.clout { theme::PAPER } else { theme::DANGER });
            });
        }
    });
}

/// Your weapons and items as a strip of small framed icons with levels.
fn build_strip(p: &mut ChildSpawnerCommands, run: &Run, art: &Art, focus: bool) {
    p.spawn(row(18.0)).with_children(|r| {
        r.spawn(row(6.0)).with_children(|ws| {
            for slot in 0..slop_rules::run::WEAPON_SLOTS {
                let Some(w) = run.weapons.get(slot) else {
                    ws.spawn((Node { width: Val::Px(56.0), height: Val::Px(56.0), border: UiRect::all(Val::Px(2.0)), ..default() }, BorderColor::all(Color::srgb(0.25, 0.25, 0.3))));
                    continue;
                };
                let mut e = ws.spawn((
                    Node { width: Val::Px(56.0), height: Val::Px(56.0), border: UiRect::all(Val::Px(3.0)), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                    BorderColor::all(level_color(w.level)),
                    BackgroundColor(CARD),
                ));
                if focus {
                    e.insert((focusable(key::WEAPON + slot as u32, CARD), weapon_tip(w.def, w.level)));
                }
                e.with_children(|c| {
                    icon(c, &art.weapon_icons[w.def], 42.0);
                    badge(c, if w.level >= MAX_LEVEL { "MAX".to_string() } else { w.level.to_string() });
                });
            }
        });
        r.spawn(row(6.0)).with_children(|is| {
            for slot in 0..slop_rules::run::ITEM_SLOTS {
                let Some(&(item, n)) = run.items.get(slot) else {
                    is.spawn((Node { width: Val::Px(48.0), height: Val::Px(48.0), border: UiRect::all(Val::Px(2.0)), ..default() }, BorderColor::all(Color::srgb(0.25, 0.25, 0.3))));
                    continue;
                };
                let mut e = is.spawn((
                    Node { width: Val::Px(48.0), height: Val::Px(48.0), border: UiRect::all(Val::Px(2.0)), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                    BorderColor::all(item_color(ITEMS[item].kind)),
                    BackgroundColor(CARD),
                ));
                if focus {
                    e.insert((focusable(key::ITEM + slot as u32, CARD), item_tip(item, n)));
                }
                e.with_children(|c| {
                    icon(c, &art.item_icons[item], 38.0);
                    badge(c, if n >= ITEMS[item].max_level() { "MAX".to_string() } else { n.to_string() });
                });
            }
        });
    });
}

fn badge(c: &mut ChildSpawnerCommands, words: String) {
    c.spawn((Node { position_type: PositionType::Absolute, right: Val::Px(1.0), bottom: Val::Px(0.0), padding: UiRect::horizontal(Val::Px(3.0)), ..default() }, BackgroundColor(theme::INK)))
        .with_children(|b| label(b, words, 12.0, theme::PAPER));
}

fn level_up(p: &mut ChildSpawnerCommands, run: &Run, choices: &[Choice], art: &Art, pad: bool) {
    backdrop(p, Color::srgba(0.04, 0.04, 0.07, 0.72));
    p.spawn(row(14.0)).with_children(|t| {
        icon(t, &art.star, 56.0);
        label(t, format!("LEVEL {}", run.level - run.pending_levels + 1), 52.0, tier_color(3));
        icon(t, &art.star, 56.0);
    });
    let more = run.pending_levels.saturating_sub(1);
    label(p, if more > 0 { format!("Pick one. {more} more level-ups after this.") } else { "Pick one.".to_string() }, 18.0, theme::QUIET);
    p.spawn(Node { column_gap: Val::Px(16.0), margin: UiRect::vertical(Val::Px(8.0)), ..default() }).with_children(|r| {
        for (i, c) in choices.iter().enumerate() {
            choice_card(r, run, *c, art, key::CHOICE + i as u32, UiAction::Choose(i), None);
        }
    });
    p.spawn(row(14.0)).with_children(|r| {
        let cost = run.reroll_cost();
        let words = if cost == 0 { format!("Reroll (free, {} left)", run.rerolls) } else { format!("Reroll: {cost} Clout") };
        let mut b = button(r, words, UiAction::Reroll, BUTTON, key::BUTTON);
        b.with_children(|r| icon(r, &art.reroll, 24.0));
        r.spawn(row(6.0)).with_children(|c| {
            icon(c, &art.coin, 26.0);
            label(c, run.clout.to_string(), 22.0, theme::PAPER);
        });
    });
    build_strip(p, run, art, false);
    hints(p, pad, &[("Arrows", "Stick", "choose"), ("Enter/Space", "A", "pick"), ("1-4", "", "pick"), ("R", "Y", "reroll")]);
}

fn chest(p: &mut ChildSpawnerCommands, run: &Run, reward: &Reward, art: &Art, pad: bool) {
    backdrop(p, Color::srgba(0.04, 0.04, 0.07, 0.72));
    let gold = tier_color(3);
    label(p, reward.title.to_uppercase(), 60.0, if reward.merged.is_some() { MERGE } else { gold });
    if let Some((a, b, into)) = reward.merged {
        p.spawn(row(24.0)).with_children(|r| {
            for (d, size) in [(a, 90.0), (b, 90.0)] {
                r.spawn(column(4.0)).with_children(|c| {
                    icon(c, &art.weapon_icons[d], size);
                    label(c, WEAPONS[d].name, 16.0, theme::QUIET);
                });
            }
            label(r, "->", 48.0, MERGE);
            r.spawn((BorderColor::all(MERGE), Node { border: UiRect::all(Val::Px(4.0)), padding: UiRect::all(Val::Px(12.0)), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, ..default() }, BackgroundColor(CARD)))
                .with_children(|c| {
                    icon(c, &art.weapon_icons[into], 150.0);
                    label(c, WEAPONS[into].name, 30.0, theme::PAPER);
                    label(c, format!("\"{}\"", WEAPONS[into].flavour), 16.0, theme::QUIET);
                });
        });
        label(p, "Two weapons became one. A slot is free again.", 18.0, theme::PAPER);
    } else {
        p.spawn(row(16.0)).with_children(|r| {
            for c in &reward.upgrades {
                let (image, name, tag, _, color) = describe(run, *c, art);
                r.spawn((Node { width: Val::Px(220.0), padding: UiRect::all(Val::Px(12.0)), border: UiRect::all(Val::Px(3.0)), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: Val::Px(6.0), ..default() }, BorderColor::all(color), BackgroundColor(CARD)))
                    .with_children(|k| {
                        icon(k, &image, 80.0);
                        label(k, name, 20.0, theme::PAPER);
                        label(k, chest_tag(run, *c, tag), 14.0, color);
                    });
            }
        });
        if let Some((a, b, into)) = near_merge(run) {
            label(p, format!("Level {} and {} to 8, then open a chest: {}.", WEAPONS[a].name, WEAPONS[b].name, WEAPONS[into].name), 16.0, MERGE);
        }
    }
    p.spawn(row(8.0)).with_children(|c| {
        icon(c, &art.coin, 28.0);
        label(c, format!("+{} Clout", reward.clout), 22.0, gold);
    });
    button(p, "Continue", UiAction::Close, theme::PLAYER_RED, key::BUTTON).insert((Continue, DefaultFocus));
    build_strip(p, run, art, false);
    hints(p, pad, &[("Enter/Space", "A", "continue")]);
}

/// After a chest the level shown is the level reached.
fn chest_tag(run: &Run, c: Choice, tag: String) -> String {
    match c {
        Choice::WeaponLevel(d) => run.weapons.iter().find(|w| w.def == d).map_or(tag, |w| format!("now level {}", w.level)),
        Choice::ItemLevel(i) | Choice::NewItem(i) => run.items.iter().find(|x| x.0 == i).map_or(tag, |x| format!("now level {}", x.1)),
        Choice::NewWeapon(_) => "new!".into(),
        _ => tag,
    }
}

/// A merge you could make: both weapons owned, not both maxed yet.
fn near_merge(run: &Run) -> Option<(usize, usize, usize)> {
    run.weapons.iter().find_map(|w| {
        let (other, into) = merge_of(w.def)?;
        run.weapons.iter().any(|x| x.def == other).then_some((w.def, other, into))
    })
}

fn vending(p: &mut ChildSpawnerCommands, run: &Run, offers: &[(Choice, u32)], art: &Art, pad: bool) {
    backdrop(p, Color::srgba(0.04, 0.04, 0.07, 0.72));
    label(p, "VENDING MACHINE", 52.0, theme::PAPER);
    label(p, "One item. Pick it, or walk away and come back later.", 18.0, theme::QUIET);
    p.spawn(Node { column_gap: Val::Px(16.0), ..default() }).with_children(|r| {
        for (i, (c, price)) in offers.iter().enumerate() {
            choice_card(r, run, *c, art, key::CHOICE + i as u32, UiAction::Buy(i), Some(*price));
        }
    });
    p.spawn(row(14.0)).with_children(|r| {
        r.spawn(row(6.0)).with_children(|c| {
            icon(c, &art.coin, 26.0);
            label(c, format!("You have {}", run.clout), 22.0, theme::PAPER);
        });
        button(r, "Walk away", UiAction::Close, BUTTON, key::BUTTON);
    });
    build_strip(p, run, art, false);
    hints(p, pad, &[("Arrows", "Stick", "choose"), ("Enter", "A", "buy"), ("Esc", "B", "walk away")]);
}

fn shrine(p: &mut ChildSpawnerCommands, offers: &[usize], art: &Art, pad: bool) {
    backdrop(p, Color::srgba(0.04, 0.04, 0.07, 0.72));
    label(p, "SHRINE OF THE ALGORITHM", 48.0, tier_color(3));
    label(p, "The Algorithm offers a blessing. It lasts the whole run.", 18.0, theme::QUIET);
    p.spawn(Node { column_gap: Val::Px(16.0), ..default() }).with_children(|r| {
        for (i, &b) in offers.iter().enumerate() {
            let d = &BLESSINGS[b];
            let k = key::CHOICE + i as u32;
            let mut e = r.spawn((focusable(k, CARD), Act(UiAction::Bless(i)), Node { min_height: Val::Px(260.0), align_items: AlignItems::Center, ..card(280.0) }, BorderColor::all(tier_color(3))));
            if i == 0 {
                e.insert(DefaultFocus);
            }
            e.with_children(|c| {
                icon(c, &art.blessings[b], 96.0);
                label(c, d.name, 22.0, theme::PAPER);
                label(c, d.effect, 17.0, tier_color(3));
                label(c, format!("\"{}\"", d.flavour), 14.0, theme::QUIET);
            });
        }
    });
    hints(p, pad, &[("Arrows", "Stick", "choose"), ("Enter/Space", "A", "take it")]);
}

fn pause(p: &mut ChildSpawnerCommands, run: &Run, art: &Art, pad: bool) {
    backdrop(p, Color::srgba(0.04, 0.04, 0.07, 0.82));
    p.spawn(Node { width: Val::Percent(100.0), height: Val::Percent(100.0), column_gap: Val::Px(16.0), padding: UiRect { left: Val::Px(40.0), right: Val::Px(0.0), top: Val::Px(30.0), bottom: Val::Px(40.0) }, ..default() }).with_children(|page| {
        page.spawn(Node { flex_grow: 1.0, flex_direction: FlexDirection::Column, row_gap: Val::Px(12.0), ..default() }).with_children(|c| {
            label(c, "PAUSED", 52.0, theme::PAPER);
            let t = run.seconds() as u32;
            label(c, format!("{}:{:02} into the Feed on {}  -  {} slop deleted  -  {} bosses", t / 60, t % 60, run.platform.name(), run.kills(), run.bosses_beaten), 18.0, theme::QUIET);
            c.spawn(row(12.0)).with_children(|r| {
                button(r, "Resume", UiAction::Resume, theme::PLAYER_RED, key::BUTTON).insert((Continue, DefaultFocus));
                button(r, "Quit game", UiAction::Quit, BUTTON, key::BUTTON + 1);
            });
            c.spawn(Node { height: Val::Px(8.0), ..default() });
            label(c, "YOUR BUILD", 16.0, theme::QUIET);
            build_strip(c, run, art, true);
            if !run.blessings.is_empty() {
                label(c, "BLESSINGS", 16.0, theme::QUIET);
                c.spawn(row(8.0)).with_children(|r| {
                    for (k, &b) in run.blessings.iter().enumerate() {
                        let d = &BLESSINGS[b];
                        r.spawn((focusable(key::BLESSING + k as u32, Color::NONE), Node { padding: UiRect::all(Val::Px(4.0)), ..default() }, Tip::new(d.name, d.effect, tier_color(3)))).with_children(|i| icon(i, &art.blessings[b], 44.0));
                    }
                });
            }
            if let Some((a, b, into)) = near_merge(run) {
                label(c, format!("Merge in reach: {} + {} at level 8 -> {}", WEAPONS[a].name, WEAPONS[b].name, WEAPONS[into].name), 16.0, MERGE);
            }
            // Details of whatever is highlighted, instead of tooltips over the build.
            c.spawn((TipDock, Node { flex_grow: 1.0, min_height: Val::Px(160.0), border: UiRect::all(Val::Px(2.0)), ..default() }, BorderColor::all(Color::srgb(0.22, 0.22, 0.28))));
        });
        stats_panel(page, run, art);
    });
    hints(p, pad, &[("Arrows", "Stick", "inspect"), ("Space/P/Esc", "Start", "resume")]);
}

fn over(p: &mut ChildSpawnerCommands, run: &Run, victory: bool, art: &Art, pad: bool) {
    backdrop(p, Color::srgba(0.07, 0.07, 0.1, 0.9));
    let (title, sub, color) = if victory {
        ("YOU BEAT THE FEED", "The Model collapsed. Then the Feed kept going, and so did you, for a while.", theme::mental_health_color(1.0))
    } else {
        ("BURNOUT", "Your account has been suspended.", theme::DANGER)
    };
    icon(p, &art.human[&run.hero][if victory { 0 } else { 3 }], 110.0);
    label(p, title, 80.0, color);
    label(p, sub, 22.0, theme::QUIET);
    let t = run.seconds() as u32;
    label(p, format!("{} survived {}:{:02} - level {} - {} slop deleted - {} bosses", run.hero.def().name, t / 60, t % 60, run.level, run.kills(), run.bosses_beaten), 20.0, theme::PAPER);
    let kills = run.kills_by_type();
    let total: u32 = kills.iter().sum::<u32>().max(1);
    p.spawn(row(18.0)).with_children(|r| {
        for t in DamageType::ALL {
            r.spawn(row(6.0)).with_children(|k| {
                icon(k, &art.type_icons[t as usize], 26.0);
                label(k, format!("{} {:.0}%", t.name(), 100.0 * kills[t as usize] as f32 / total as f32), 17.0, theme::type_color(t));
            });
        }
    });
    let dmg = run.damage_by_weapon();
    let mut top: Vec<usize> = (0..dmg.len()).filter(|&d| dmg[d] > 0.0).collect();
    top.sort_by(|&a, &b| dmg[b].total_cmp(&dmg[a]));
    p.spawn(row(14.0)).with_children(|r| {
        for d in top.into_iter().take(6) {
            r.spawn(column(2.0)).with_children(|c| {
                icon(c, &art.weapon_icons[d], 48.0);
                label(c, format!("{:.0}", dmg[d]), 14.0, theme::QUIET);
            });
        }
    });
    p.spawn(Node { height: Val::Px(8.0), ..default() });
    p.spawn(row(14.0)).with_children(|r| {
        button(r, "New run", UiAction::NewRun, theme::PLAYER_RED, key::BUTTON).insert((Continue, DefaultFocus));
        button(r, "Quit", UiAction::Quit, BUTTON, key::BUTTON + 1);
    });
    hints(p, pad, &[("Space", "A", "new run")]);
}
