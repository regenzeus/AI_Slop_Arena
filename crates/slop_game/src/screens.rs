//! The screens (UI-08 to UI-13): title, hero select, and over the paused
//! Feed: level-ups, chests and merges, vending machines, shrines, pause with
//! your whole build, and the end of a run. They are rebuilt whenever
//! something changes, and every one works by keyboard and gamepad (`ui`).

use bevy::prelude::*;
use slop_rules::memory::Temperature;
use slop_rules::chapters::Chapter;
use slop_rules::run::Delegate;
use slop_rules::items::{ItemKind, StatKind, BLESSINGS, GLITCHES, ITEMS};
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
        app.add_systems(Update, (clicks, rebuild).chain().in_set(Rebuilt).after(Simulation)).add_systems(Update, animate.after(Rebuilt));
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
    pub const GLITCH: u32 = 600;
    pub const TEMPERATURE: u32 = 700;
}

/// Seconds after a chest opens before its first upgrade lands, and between
/// the next ones (LOOT-02).
const REVEAL_FIRST: f32 = 0.55;
const REVEAL_EACH: f32 = 0.5;

/// An icon that spins through other icons like a slot reel until `at`
/// seconds after the chest opened, then lands on `image`.
#[derive(Component)]
struct Reel {
    at: f32,
    image: Handle<Image>,
    pool: Vec<Handle<Image>>,
}

/// Hidden until `at` seconds after the chest opened.
#[derive(Component)]
struct Reveal(f32);

/// A frame that turns its rarity's colour when its upgrade lands.
#[derive(Component)]
struct RevealBorder {
    at: f32,
    color: Color,
    rarity: u8,
}

/// A legendary's frame pulses.
#[derive(Component)]
struct Shine(Color);

/// Text that cycles through the colours of a legendary.
#[derive(Component)]
struct Rainbow;

/// A full-screen glow that breathes.
#[derive(Component)]
struct Glow;

/// The chest reveal, shining frames and rainbow titles (LOOT-02, ITEM-03).
#[allow(clippy::type_complexity)]
fn animate(
    time: Res<Time>,
    mut game: ResMut<Game>,
    mut reels: Query<(&mut ImageNode, &mut Reel)>,
    mut reveals: Query<(&mut Visibility, &Reveal)>,
    mut borders: Query<(&mut BorderColor, Option<&mut RevealBorder>, Option<&Shine>)>,
    mut rainbows: Query<&mut TextColor, With<Rainbow>>,
    mut glows: Query<&mut BackgroundColor, With<Glow>>,
) {
    let t = time.elapsed_secs();
    let since = game.chest_opened;
    let mut landed = Vec::new();
    for (mut node, mut reel) in &mut reels {
        if since >= reel.at {
            if !reel.pool.is_empty() {
                node.image = reel.image.clone();
                reel.pool.clear();
            }
        } else if !reel.pool.is_empty() {
            let k = ((since * 18.0) as usize + reel.at as usize * 7) % reel.pool.len();
            node.image = reel.pool[k].clone();
        }
    }
    for (mut vis, r) in &mut reveals {
        let want = if since >= r.0 { Visibility::Inherited } else { Visibility::Hidden };
        if *vis != want {
            *vis = want;
        }
    }
    for (mut border, reveal, shine) in &mut borders {
        if let Some(mut r) = reveal {
            if since >= r.at && r.rarity > 0 {
                *border = BorderColor::all(r.color);
                landed.push(r.rarity);
                r.rarity = 0;
            }
            if since < r.at {
                continue;
            }
        }
        if let Some(s) = shine {
            let k = 0.5 + 0.5 * (t * 6.0).sin();
            *border = BorderColor::all(s.0.mix(&Color::WHITE, k * 0.6));
        }
    }
    for mut color in &mut rainbows {
        color.0 = Color::hsl((t * 140.0) % 360.0, 0.9, 0.62);
    }
    for mut bg in &mut glows {
        bg.0 = LEGENDARY.with_alpha(0.08 + 0.08 * (t * 3.0).sin().abs());
    }
    // A sound as each upgrade lands; the big one for a legendary.
    for rarity in landed {
        game.cues.push(if rarity >= 4 { crate::state::Cue::Legendary } else if rarity >= 3 { crate::state::Cue::Level } else { crate::state::Cue::Tick });
    }
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
pub const LEGENDARY: Color = Color::srgb(1.0, 0.72, 0.12);
/// Glitches: a glitchy cyan.
pub const GLITCH: Color = Color::srgb(0.25, 0.95, 0.9);
/// The easy option: a cold, helpful blue (AUTO-01).
pub const DELEGATE: Color = Color::srgb(0.55, 0.75, 1.0);
const GOOD: Color = Color::srgb(0.45, 0.88, 0.45);
const BAD: Color = Color::srgb(1.0, 0.42, 0.42);

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
    let held = matches!(game.run.as_ref().map(|r| &r.phase), Some(Phase::ChapterCard(_))) && game.since_choice < DEATH_HOLD;
    let shown = Shown { screen: Some(game.screen), phase, settled: game.since_end > END_PAUSE && !held, paused: game.paused, pad: controls.pad };
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
        Screen::Title => title(p, game, &art, pad),
        Screen::Heroes => heroes(p, game, &art, pad),
        Screen::Run => {
            if let Some(run) = &game.run {
                match &run.phase {
                    Phase::Playing if game.paused => pause(p, run, &art, pad),
                    Phase::LevelUp(choices) => level_up(p, run, choices, &art, pad),
                    Phase::Chest(reward) => chest(p, run, reward, &art, pad),
                    Phase::Vending { offers, .. } => vending(p, run, offers, &art, pad),
                    Phase::Shrine { offers, .. } => shrine(p, offers, &art, pad),
                    Phase::Glitch(offers) => glitch_pick(p, run, offers, &art, pad),
                    Phase::Recycle { items, .. } => recycle(p, run, items, &art, pad),
                    Phase::ChapterCard(_) if held && !game.bot => death_hold(p, run),
                    Phase::ChapterCard(c) => chapter_card(p, run, *c, &art, pad),
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

fn title(p: &mut ChildSpawnerCommands, game: &Game, art: &Art, pad: bool) {
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
    // It remembers you (NEMESIS-01).
    if let Some(k) = &game.memory.killer {
        let n = game.memory.killer_count;
        let times = if n > 1 { format!(" ({n} times)") } else { String::new() };
        label(p, format!("Welcome back. Last time: deleted by {k}{times}. It remembers."), 18.0, tier_color(3));
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
        Pattern::Barrier { .. } => "a barrier circling you: blocks enemy shots and hurts slop it touches",
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
        ItemKind::Movement => Color::srgb(1.0, 0.55, 0.2),
        ItemKind::Defence => Color::srgb(0.31, 0.72, 0.91),
        ItemKind::Legendary => LEGENDARY,
    }
}

/// Common, rare, epic, legendary: grey, blue, purple, gold (ITEM-03).
pub fn rarity_color(tier: u8) -> Color {
    match tier {
        0 | 1 => Color::srgb(0.62, 0.64, 0.7),
        2 => Color::srgb(0.3, 0.62, 1.0),
        3 => Color::srgb(0.72, 0.42, 1.0),
        _ => LEGENDARY,
    }
}

fn item_tip(item: usize, level: u32) -> Tip {
    let d = &ITEMS[item];
    let per = if d.max_level() > 1 { " per level" } else { "" };
    Tip::new(
        format!("{} - level {level}/{}", d.name, d.max_level()),
        format!("{} {}.\n{}{per}.\n\"{}\"", slop_rules::items::rarity_name(d.tier), d.kind.name().to_lowercase(), d.effect, d.flavour),
        item_color(d.kind),
    )
}

fn glitch_tip(g: usize) -> Tip {
    let d = &GLITCHES[g];
    Tip::new(format!("GLITCH: {}", d.name), format!("{}.\n\"{}\"", d.effect, d.flavour), GLITCH)
}

fn heroes(p: &mut ChildSpawnerCommands, game: &Game, art: &Art, pad: bool) {
    backdrop(p, BACKDROP);
    label(p, "CHOOSE YOUR HUMAN", 36.0, theme::PAPER);
    label(p, "Same human, different baggage. Every hero bends the rules: read the fine print.", 17.0, theme::QUIET);
    // Two rows of four (HERO-02).
    for half in Hero::ALL.chunks(4).enumerate() {
        p.spawn(Node { column_gap: Val::Px(12.0), ..default() }).with_children(|r| {
            for (k, h) in half.1.iter().enumerate() {
                let i = half.0 * 4 + k;
                let d = h.def();
                let w = slop_sim::content::weapon_index(d.weapon).unwrap_or(0);
                let it = slop_rules::items::item_index(d.item).unwrap_or(0);
                let mut e = r.spawn((
                    focusable(key::HERO + i as u32, CARD),
                    Act(UiAction::StartAs(i)),
                    Node { min_height: Val::Px(300.0), ..card(300.0) },
                    BorderColor::all(theme::type_color(WEAPONS[w].kind)),
                ));
                if i == game.hero {
                    e.insert(DefaultFocus);
                }
                e.with_children(|c| {
                    c.spawn(row(10.0)).with_children(|t| {
                        icon(t, &art.human[h][0], 62.0);
                        t.spawn(column(2.0)).with_children(|n| {
                            label(n, format!("{}  {}", i + 1, d.name), 25.0, theme::PAPER);
                            label(n, format!("\"{}\"", d.tagline), 13.0, theme::QUIET);
                        });
                    });
                    c.spawn(row(6.0)).with_children(|l| {
                        icon(l, &art.weapon_icons[w], 28.0);
                        label(l, d.weapon, 14.0, theme::type_color(WEAPONS[w].kind));
                        icon(l, &art.item_icons[it], 28.0);
                        label(l, d.item, 14.0, item_color(ITEMS[it].kind));
                    });
                    // Every property, exactly: good in green, bad in red.
                    for line in d.properties {
                        let color = if line.starts_with('+') {
                            GOOD
                        } else if line.starts_with('-') {
                            BAD
                        } else {
                            theme::PAPER
                        };
                        label(c, *line, 14.0, color);
                    }
                    label(c, "Everyone: 2 dash charges", 12.0, theme::QUIET);
                });
            }
        });
    }
    // The difficulty, in the model's own words (TEMP-01).
    p.spawn(row(10.0)).with_children(|r| {
        label(r, "Temperature", 16.0, theme::QUIET);
        for (i, t) in Temperature::ALL.iter().enumerate() {
            let on = *t == game.temperature;
            let color = if on { theme::PLAYER_RED } else { BUTTON };
            button(r, t.label(), UiAction::SetTemperature(i), color, key::TEMPERATURE + i as u32);
        }
        label(r, game.temperature.blurb(), 15.0, theme::PAPER);
    });
    hints(p, pad, &[("Arrows", "Stick", "choose"), ("1-8", "", "jump to a hero"), ("Enter", "A", "start"), ("Esc", "B", "back")]);
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
            let perks = run.loadout().perks;
            label(col, format!("Dash: {} charges, back in {:.1} s each", perks.dash_charges, perks.dash_recharge), 14.0, theme::QUIET);
            col.spawn(Node { height: Val::Px(6.0), ..default() });
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
            // What the model has learned decides what is worth picking (MODEL-01).
            let r = run.training.model[w.kind as usize];
            let read = if r > 0.03 {
                format!("The model resists this: -{:.0}% damage.
", r * 100.0)
            } else if r < -0.03 {
                format!("The model is weak to this: +{:.0}% damage!
", -r * 100.0)
            } else {
                String::new()
            };
            let mut words = if from == 0 {
                format!("{}.\n{:.0} damage every {:.2} s.\n{}", capital(pattern_words(w.pattern)), w.damage, w.cooldown, type_effect(w.kind))
            } else {
                level_note(to as u8).to_string()
            };
            if let Some((other, into)) = merge_of(d) {
                let owned = run.weapons.iter().any(|x| x.def == other);
                words = format!("{words}\n\n{}Merges with {} into {}.", if owned { "You have its partner! " } else { "" }, WEAPONS[other].name, WEAPONS[into].name);
            }
            (art.weapon_icons[d].clone(), w.name.to_string(), tag, format!("{read}{words}"), theme::type_color(w.kind))
        }
        Choice::NewItem(i) | Choice::ItemLevel(i) => {
            let d = &ITEMS[i];
            let (from, to) = run.levels_of(c).unwrap_or((0, 1));
            let rarity = slop_rules::items::rarity_name(d.tier).to_uppercase();
            let tag = if from == 0 { format!("NEW {rarity} {}", d.kind.name().to_lowercase()) } else { format!("Level {from} -> {to} of {}", d.max_level()) };
            (art.item_icons[i].clone(), d.name.to_string(), tag, format!("{}.\n\"{}\"", d.effect, d.flavour), item_color(d.kind))
        }
        Choice::Glitch(g) => {
            let d = &GLITCHES[g];
            (art.glitch_icons[g].clone(), d.name.to_string(), "GLITCH: changes your run".into(), format!("{}.\n\"{}\"", d.effect, d.flavour), GLITCH)
        }
        // The easy option (AUTO-01): what it gives, and what it costs, in red.
        Choice::Delegate(d) => {
            let image = match d {
                Delegate::Agent(w) => art.weapon_icons[w].clone(),
                _ => art.delegate.clone(),
            };
            let words = format!("{}

+{} Automation. The AI learns from you.", d.effect(), d.cost());
            (image, d.name(), format!("{}: +{} AUTOMATION", run.chapter.def().delegate.to_uppercase(), d.cost()), words, DELEGATE)
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
#[allow(clippy::too_many_arguments)]
fn choice_card(p: &mut ChildSpawnerCommands, run: &Run, c: Choice, art: &Art, k: u32, action: UiAction, price: Option<u32>, focus_key: u32) {
    let (image, name, tag, words, color) = describe(run, c, art);
    let merge = matches!(c, Choice::WeaponLevel(d) | Choice::NewWeapon(d) if merge_of(d).is_some_and(|(o, _)| run.weapons.iter().any(|x| x.def == o)));
    // Rare things get a frame in their rarity's colour, and the rarest shine.
    let rarity = c.rarity();
    let border = if matches!(c, Choice::Glitch(_)) {
        GLITCH
    } else if matches!(c, Choice::Delegate(_)) {
        DELEGATE
    } else if rarity >= 2 && matches!(c, Choice::NewItem(_) | Choice::ItemLevel(_)) {
        rarity_color(rarity)
    } else if merge {
        MERGE
    } else {
        color
    };
    let back = if rarity >= 4 { Color::srgb(0.22, 0.17, 0.1) } else { CARD };
    let width = if rarity >= 4 { 5.0 } else { 3.0 };
    let mut e = p.spawn((focusable(k, back), Act(action), Node { width: Val::Px(300.0), min_height: Val::Px(330.0), border: UiRect::all(Val::Px(width)), ..card(300.0) }, BorderColor::all(border)));
    if rarity >= 4 {
        e.insert(Shine(border));
    }
    if k == focus_key {
        e.insert(DefaultFocus);
    }
    let suggested = run.suggested.is_some_and(|s| key::CHOICE + s as u32 == k) && matches!(action, UiAction::Choose(_));
    e.with_children(|c| {
        c.spawn(Node { justify_content: JustifyContent::SpaceBetween, ..default() }).with_children(|h| {
            label(h, if suggested { format!("{}  suggested", k - key::CHOICE + 1) } else { format!("{}", k - key::CHOICE + 1) }, 16.0, if suggested { DELEGATE } else { theme::QUIET });
            label(h, tag, 15.0, if rarity >= 3 { border } else { color });
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
            for slot in 0..run.weapon_slots() {
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
            choice_card(r, run, *c, art, key::CHOICE + i as u32, UiAction::Choose(i), None, key::CHOICE + run.suggested.unwrap_or(0) as u32);
        }
    });
    build_strip(p, run, art, false);
    // The reroll sits under everything and is not in the arrow-key path, so
    // moving between cards never lands on it: R or Y, or click it.
    p.spawn(row(14.0)).with_children(|r| {
        let cost = run.reroll_cost();
        let key_name = if pad { "Y" } else { "R" };
        let words = if cost == 0 { format!("{key_name}: reroll (free, {} left)", run.rerolls) } else { format!("{key_name}: reroll for {cost} Clout") };
        r.spawn((Button, Act(UiAction::Reroll), Node { padding: UiRect::axes(Val::Px(14.0), Val::Px(7.0)), column_gap: Val::Px(8.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgb(0.2, 0.2, 0.26))))
            .with_children(|b| {
                icon(b, &art.reroll, 20.0);
                label(b, words, 16.0, theme::PAPER);
            });
        r.spawn(row(6.0)).with_children(|c| {
            icon(c, &art.coin, 22.0);
            label(c, run.clout.to_string(), 18.0, theme::PAPER);
        });
    });
    hints(p, pad, &[("Arrows", "Stick", "choose"), ("Enter", "A", "pick"), ("1-4", "", "pick"), ("R", "Y", "reroll")]);
}

fn chest(p: &mut ChildSpawnerCommands, run: &Run, reward: &Reward, art: &Art, pad: bool) {
    backdrop(p, Color::srgba(0.04, 0.04, 0.07, 0.72));
    let gold = tier_color(3);
    if reward.best >= 4 {
        // A golden glow behind everything for a legendary (LOOT-02).
        p.spawn((Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }, BackgroundColor(LEGENDARY.with_alpha(0.12)), Glow));
    }
    let title = if reward.best >= 4 && reward.merged.is_none() { format!("{}: LEGENDARY!", reward.title.to_uppercase()) } else { reward.title.to_uppercase() };
    let mut t = p.spawn((Text::new(title), TextFont { font_size: 60.0, ..default() }, TextColor(if reward.merged.is_some() { MERGE } else { gold })));
    if reward.best >= 4 {
        t.insert(Rainbow);
    }
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
        // Each upgrade spins like a slot reel, then lands, rarest last
        // (LOOT-02).
        let mut order: Vec<Choice> = reward.upgrades.clone();
        order.sort_by_key(|c| c.rarity());
        let pool: Vec<Handle<Image>> = art.item_icons.iter().chain(art.weapon_icons.iter().take(33)).cloned().collect();
        p.spawn(row(14.0)).with_children(|r| {
            for (n, c) in order.iter().enumerate() {
                let at = REVEAL_FIRST + n as f32 * REVEAL_EACH;
                let (image, name, tag, words, color) = describe(run, *c, art);
                let rarity = c.rarity();
                let frame = if rarity >= 2 && matches!(c, Choice::NewItem(_) | Choice::ItemLevel(_)) { rarity_color(rarity) } else { color };
                let mut card = r.spawn((
                    Node { width: Val::Px(240.0), min_height: Val::Px(290.0), padding: UiRect::all(Val::Px(12.0)), border: UiRect::all(Val::Px(if rarity >= 4 { 5.0 } else { 3.0 })), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: Val::Px(6.0), ..default() },
                    BorderColor::all(Color::srgb(0.3, 0.3, 0.36)),
                    BackgroundColor(if rarity >= 4 { Color::srgb(0.22, 0.17, 0.1) } else { CARD }),
                    RevealBorder { at, color: frame, rarity },
                ));
                if rarity >= 4 {
                    card.insert(Shine(frame));
                }
                card.with_children(|k| {
                    k.spawn((ImageNode::new(pool[n % pool.len()].clone()), Node { width: Val::Px(80.0), height: Val::Px(80.0), ..default() }, Reel { at, image, pool: pool.clone() }));
                    k.spawn((Text::new(name), TextFont { font_size: 19.0, ..default() }, TextColor(theme::PAPER), Visibility::Hidden, Reveal(at)));
                    let rarity_line = match c {
                        Choice::NewItem(i) | Choice::ItemLevel(i) => format!("{}  -  {}", slop_rules::items::rarity_name(ITEMS[*i].tier).to_uppercase(), chest_tag(run, *c, tag)),
                        _ => chest_tag(run, *c, tag),
                    };
                    k.spawn((Text::new(rarity_line), TextFont { font_size: 13.0, ..default() }, TextColor(frame), Visibility::Hidden, Reveal(at)));
                    // What it does, right on the card (LOOT-02).
                    k.spawn((Text::new(words), TextFont { font_size: 13.0, ..default() }, TextColor(theme::QUIET), Visibility::Hidden, Reveal(at)));
                });
            }
        });
        if let Some((a, b, into)) = near_merge(run) {
            label(p, merge_line(run, a, b, into), 16.0, MERGE);
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

/// Exactly where a merge stands: both levels, and what is still missing.
fn merge_line(run: &Run, a: usize, b: usize, into: usize) -> String {
    let level = |d: usize| run.weapons.iter().find(|w| w.def == d).map_or(0, |w| w.level);
    let (la, lb) = (level(a), level(b));
    let what = if la >= MAX_LEVEL && lb >= MAX_LEVEL {
        "READY: the next chest merges them (elites and bosses drop chests).".to_string()
    } else {
        "Get both to level 8, then open a chest.".to_string()
    };
    format!("Merge: {} (level {la}/8) + {} (level {lb}/8) = {}. {what}", WEAPONS[a].name, WEAPONS[b].name, WEAPONS[into].name)
}

/// A recycle bin: throw one of your items away for Clout, freeing its slot
/// (MAP-05).
fn recycle(p: &mut ChildSpawnerCommands, run: &Run, items: &[usize], art: &Art, pad: bool) {
    backdrop(p, Color::srgba(0.04, 0.04, 0.07, 0.72));
    label(p, "RECYCLE BIN", 52.0, theme::PAPER);
    label(p, "Throw one item away for Clout. Its slot is free again.", 18.0, theme::QUIET);
    p.spawn(Node { column_gap: Val::Px(12.0), ..default() }).with_children(|r| {
        for (k, &i) in items.iter().enumerate() {
            let d = &ITEMS[i];
            let level = run.items.iter().find(|x| x.0 == i).map_or(1, |x| x.1);
            let refund = slop_rules::items::tier_price(d.tier) * level / 2 + 5;
            let key = key::CHOICE + k as u32;
            let mut e = r.spawn((focusable(key, CARD), Act(UiAction::Recycle(k)), Node { min_height: Val::Px(220.0), align_items: AlignItems::Center, ..card(190.0) }, BorderColor::all(item_color(d.kind)), item_tip(i, level)));
            if k == 0 {
                e.insert(DefaultFocus);
            }
            e.with_children(|c| {
                icon(c, &art.item_icons[i], 72.0);
                label(c, d.name, 17.0, theme::PAPER);
                label(c, format!("level {level}"), 13.0, theme::QUIET);
                c.spawn(row(6.0)).with_children(|b| {
                    icon(b, &art.coin, 20.0);
                    label(b, format!("+{refund}"), 18.0, tier_color(3));
                });
            });
        }
    });
    button(p, "Walk away", UiAction::Close, BUTTON, key::BUTTON);
    hints(p, pad, &[("Arrows", "Stick", "choose"), ("Enter", "A", "throw away"), ("Esc", "B", "walk away")]);
}

/// After a boss: three glitches, pick one (GLITCH-01).
fn glitch_pick(p: &mut ChildSpawnerCommands, run: &Run, offers: &[usize], art: &Art, pad: bool) {
    backdrop(p, Color::srgba(0.02, 0.05, 0.06, 0.85));
    p.spawn((Text::new("THE FEED GLITCHED"), TextFont { font_size: 58.0, ..default() }, TextColor(GLITCH), Rainbow));
    label(p, "The boss left a hole in the algorithm. Pick one glitch: it changes how this run plays.", 18.0, theme::QUIET);
    p.spawn(Node { column_gap: Val::Px(16.0), margin: UiRect::vertical(Val::Px(8.0)), ..default() }).with_children(|r| {
        for (i, &g) in offers.iter().enumerate() {
            choice_card(r, run, Choice::Glitch(g), art, key::CHOICE + i as u32, UiAction::Glitch(i), None, key::CHOICE);
        }
    });
    build_strip(p, run, art, false);
    hints(p, pad, &[("Arrows", "Stick", "choose"), ("Enter", "A", "take it"), ("1-3", "", "pick")]);
}

fn vending(p: &mut ChildSpawnerCommands, run: &Run, offers: &[(Choice, u32)], art: &Art, pad: bool) {
    backdrop(p, Color::srgba(0.04, 0.04, 0.07, 0.72));
    // A vending machine sells items; a fine-tuning station sells weapon levels.
    let tuning = offers.iter().any(|(c, _)| matches!(c, Choice::WeaponLevel(_)));
    if tuning {
        label(p, "FINE-TUNING STATION", 52.0, theme::PAPER);
        label(p, "One weapon level. Pick it, or walk away.", 18.0, theme::QUIET);
    } else {
        label(p, "VENDING MACHINE", 52.0, theme::PAPER);
        label(p, "One item. Pick it, or walk away and come back later.", 18.0, theme::QUIET);
    }
    p.spawn(Node { column_gap: Val::Px(16.0), ..default() }).with_children(|r| {
        for (i, (c, price)) in offers.iter().enumerate() {
            choice_card(r, run, *c, art, key::CHOICE + i as u32, UiAction::Buy(i), Some(*price), key::CHOICE);
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
    hints(p, pad, &[("Arrows", "Stick", "choose"), ("Enter", "A", "take it")]);
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
            if !run.glitches.is_empty() {
                label(c, "GLITCHES", 16.0, theme::QUIET);
                c.spawn(row(8.0)).with_children(|r| {
                    for (k, &g) in run.glitches.iter().enumerate() {
                        r.spawn((focusable(key::GLITCH + k as u32, Color::NONE), Node { padding: UiRect::all(Val::Px(4.0)), ..default() }, glitch_tip(g))).with_children(|i| icon(i, &art.glitch_icons[g], 44.0));
                    }
                });
            }
            if let Some((a, b, into)) = near_merge(run) {
                label(c, merge_line(run, a, b, into), 16.0, MERGE);
            }
            // Details of whatever is highlighted, instead of tooltips over the build.
            c.spawn((TipDock, Node { flex_grow: 1.0, min_height: Val::Px(160.0), border: UiRect::all(Val::Px(2.0)), padding: UiRect::all(Val::Px(14.0)), ..default() }, BorderColor::all(Color::srgb(0.22, 0.22, 0.28)), BackgroundColor(Color::srgb(0.07, 0.07, 0.1))))
                .with_children(|d| label(d, "Highlight a weapon, an item or a stat to read what it does.", 15.0, theme::QUIET));
        });
        stats_panel(page, run, art);
    });
    hints(p, pad, &[("Arrows", "Stick", "inspect"), ("P/Esc", "Start/B", "resume")]);
}

fn over(p: &mut ChildSpawnerCommands, run: &Run, _victory: bool, art: &Art, pad: bool) {
    backdrop(p, Color::srgba(0.07, 0.07, 0.1, 0.9));
    // The end is always the same one (END-01).
    let (title, sub, color) = ("AUTOMATED", "Your role has been automated. No human required.", DELEGATE);
    icon(p, &art.human[&run.hero][3], 70.0);
    label(p, title, 58.0, color);
    label(p, sub, 22.0, theme::QUIET);
    let t = run.seconds() as u32;
    label(p, format!("{} lasted {}:{:02} over {} of 5 chapters - {} slop deleted - humanity retained {}% - score {}", run.hero.def().name, t / 60, t % 60, run.chapters.len().max(1), run.total_kills(), run.humanity(), run.score), 18.0, theme::PAPER);
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
    // The model card: written from what really happened (CARD-01).
    p.spawn((Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(3.0), padding: UiRect::all(Val::Px(14.0)), width: Val::Px(820.0), border: UiRect::all(Val::Px(2.0)), ..default() }, BorderColor::all(theme::QUIET), BackgroundColor(Color::srgb(0.1, 0.1, 0.14)))).with_children(|c| {
        label(c, "MODEL CARD", 15.0, theme::QUIET);
        for line in run.model_card() {
            label(c, line, 16.0, theme::PAPER);
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

/// Seconds the last frame of a chapter holds, with what deleted you named
/// (LESSON-01), before the next chapter's card.
pub const DEATH_HOLD: f32 = 2.2;

/// The last frame, held: the chapter's death word, what did it, and the
/// one line that would have saved you. The Feed stays visible behind it.
fn death_hold(p: &mut ChildSpawnerCommands, run: &Run) {
    backdrop(p, Color::srgba(0.4, 0.0, 0.05, 0.35));
    let Some(last) = run.chapters.last() else { return };
    let d = last.chapter.def();
    label(p, d.death, 72.0, theme::DANGER);
    match &last.killer {
        Some((name, lesson)) => {
            label(p, format!("by {name}"), 26.0, theme::PAPER);
            p.spawn(Node { height: Val::Px(10.0), ..default() });
            label(p, lesson.clone(), 20.0, tier_color(3));
        }
        None => label(p, format!("{} ran out.", d.resource), 26.0, theme::PAPER),
    }
}

/// Between chapters (CH-01): what you are now, what you protect, what you
/// keep, and what the last chapter taught you.
fn chapter_card(p: &mut ChildSpawnerCommands, run: &Run, c: Chapter, art: &Art, pad: bool) {
    backdrop(p, Color::srgba(0.02, 0.02, 0.04, 0.94));
    let d = c.def();
    let color = Color::srgb(d.color[0], d.color[1], d.color[2]);
    label(p, format!("CHAPTER {} OF 5", c.number()), 18.0, theme::QUIET);
    label(p, d.name.to_uppercase(), 58.0, color);
    icon(p, &art.human[&run.hero][0], 80.0);
    p.spawn(Node { height: Val::Px(8.0), ..default() });
    for line in d.card {
        label(p, *line, 20.0, theme::PAPER);
    }
    p.spawn(Node { height: Val::Px(10.0), ..default() });
    if let Some(last) = run.chapters.last() {
        let m = last.seconds as u32;
        let words = match &last.killer {
            Some((name, lesson)) => format!("{}: {}:{:02}, {} by {}. {}", last.chapter.def().name, m / 60, m % 60, last.chapter.def().death.to_lowercase(), name, lesson),
            None => format!("{}: {}:{:02}.", last.chapter.def().name, m / 60, m % 60),
        };
        label(p, words, 16.0, tier_color(3));
    }
    let agents = run.weapons.iter().filter(|w| w.agent).count();
    let keep = format!("You keep: Automation {}%, {} agent{}, {} glitch{}. Everything else starts over.", run.automation, agents, if agents == 1 { "" } else { "s" }, run.glitches.len(), if run.glitches.len() == 1 { "" } else { "es" });
    label(p, keep, 15.0, theme::QUIET);
    if c == Chapter::Resistance && agents > 0 {
        label(p, "Your agents have been reassigned. You will meet them.", 15.0, theme::DANGER);
    }
    p.spawn(Node { height: Val::Px(10.0), ..default() });
    button(p, "Continue", UiAction::NextChapter, color, key::BUTTON).insert((Continue, DefaultFocus));
    hints(p, pad, &[("Enter/Space", "A", "continue")]);
}
