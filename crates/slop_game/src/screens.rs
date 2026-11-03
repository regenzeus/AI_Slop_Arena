//! The screens around the waves (UI-08 to UI-13): title, hero select, the
//! run's briefing, level-ups, the shop, pause with your whole build, and the
//! end of a run. They are rebuilt from the run whenever something changes.
//! Every one works by keyboard (`ui`).

use bevy::prelude::*;
use slop_rules::items::{ItemKind, StatKind, ITEMS};
use slop_rules::shop::{Goods, Note};
use slop_rules::{Hero, Phase, Run};
use slop_sim::content::{Archetype, Pattern, Platform, TIER_DAMAGE, WEAPONS};
use slop_sim::DamageType;

use crate::art::Art;
use crate::play::{Simulation, END_PAUSE};
use crate::state::{Game, Screen, UiAction};
use crate::theme;
use crate::ui::{focus_ring, Act, Alt, Continue, DefaultFocus, Focusable, Nav, Shade, Tip, TipDock};

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
    pub const LEVEL: u32 = 20;
    pub const OFFER: u32 = 100;
    pub const WEAPON: u32 = 200;
    pub const ITEM: u32 = 300;
    pub const STAT: u32 = 500;
    pub const RESIST: u32 = 520;
}

pub fn roman(tier: u8) -> &'static str {
    ["I", "II", "III", "IV"][(tier.clamp(1, 4) - 1) as usize]
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

const CARD: Color = Color::srgb(0.15, 0.14, 0.2);
const BUTTON: Color = Color::srgb(0.30, 0.30, 0.38);
const BACKDROP: Color = Color::srgba(0.06, 0.06, 0.09, 0.8);

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

/// Keys and what they do, in the hint bar at the bottom of a screen.
fn hints(p: &mut ChildSpawnerCommands, keys: &[(&str, &str)]) {
    p.spawn(Node { position_type: PositionType::Absolute, bottom: Val::Px(10.0), width: Val::Percent(100.0), justify_content: JustifyContent::Center, column_gap: Val::Px(18.0), ..default() })
        .with_children(|r| {
            for (k, what) in keys {
                r.spawn(row(6.0)).with_children(|h| {
                    h.spawn((Node { padding: UiRect::axes(Val::Px(7.0), Val::Px(2.0)), border: UiRect::all(Val::Px(1.0)), ..default() }, BorderColor::all(theme::QUIET), BackgroundColor(Color::srgb(0.2, 0.2, 0.26))))
                        .with_children(|c| label(c, *k, 14.0, theme::PAPER));
                    label(h, *what, 14.0, theme::QUIET);
                });
            }
        });
}

fn clicks(
    mut game: ResMut<Game>,
    buttons: Query<(&Interaction, &Act), Changed<Interaction>>,
    alts: Query<(&Interaction, &Alt)>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut exit: MessageWriter<AppExit>,
) {
    if game.bot {
        return;
    }
    for (interaction, act) in &buttons {
        if *interaction == Interaction::Pressed && !game.act(act.0) {
            exit.write(AppExit::Success);
        }
    }
    // A right click is the other action: lock an offer, sell a weapon.
    if mouse.just_pressed(MouseButton::Right) {
        if let Some((_, alt)) = alts.iter().find(|(i, _)| **i == Interaction::Hovered) {
            game.act(alt.0);
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
}

fn rebuild(mut commands: Commands, mut game: ResMut<Game>, mut nav: ResMut<Nav>, art: Res<Art>, roots: Query<Entity, With<ScreenRoot>>, mut last: Local<Shown>) {
    let phase = game.run.as_ref().map_or(String::new(), |r| format!("{:?}{}", std::mem::discriminant(&r.phase), r.wave));
    let shown = Shown { screen: Some(game.screen), phase, settled: game.since_end > END_PAUSE, paused: game.paused };
    if !game.dirty && *last == shown {
        return;
    }
    if *last != shown {
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
    let root = commands.spawn((ScreenRoot, full(), GlobalZIndex(20))).id();
    commands.entity(root).with_children(|p| match game.screen {
        Screen::Title => title(p, &art),
        Screen::Heroes => heroes(p, game, &art),
        Screen::Briefing => {
            if let Some(run) = &game.run {
                briefing(p, run, &art);
            }
        }
        Screen::Run => {
            if let Some(run) = &game.run {
                let settled = game.since_end > END_PAUSE;
                match &run.phase {
                    Phase::Wave if game.paused => pause(p, run, &art),
                    Phase::LevelUp(offers) if settled => level_up(p, run, offers, &art),
                    Phase::Shop => shop(p, run, &art),
                    Phase::Over { victory } if settled => over(p, run, *victory, &art),
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

fn title(p: &mut ChildSpawnerCommands, art: &Art) {
    backdrop(p, Color::srgba(0.07, 0.07, 0.11, 0.55));
    p.spawn(row(18.0)).with_children(|r| {
        for (i, h) in Hero::ALL.iter().enumerate() {
            icon(r, &art.human[h][if i == 2 { 2 } else { 0 }], 84.0);
        }
    });
    label(p, "AI SLOP ARENA", 92.0, theme::PAPER);
    label(p, "One human. Twenty waves of AI slop. Protect your Mental Health.", 24.0, theme::QUIET);
    p.spawn(Node { height: Val::Px(16.0), ..default() });
    for line in ["Move with WASD or the arrows. Your weapons fire on their own.", "After every wave: level up, then shop. Bosses at waves 10 and 20.", "The slop retrains on how you kill it. Adapt, or get fine-tuned."] {
        label(p, line, 19.0, theme::PAPER);
    }
    p.spawn(Node { height: Val::Px(20.0), ..default() });
    p.spawn(row(14.0)).with_children(|r| {
        button(r, "Play", UiAction::ToHeroes, theme::PLAYER_RED, key::BUTTON).insert((Continue, DefaultFocus));
        button(r, "Quit", UiAction::Quit, BUTTON, key::BUTTON + 1);
    });
    label(p, "No meta progression. Everything is unlocked. One Burnout ends the run.", 15.0, theme::QUIET);
    hints(p, &[("Arrows/WASD", "choose"), ("Enter", "press"), ("Space", "play"), ("Q", "quit")]);
}

fn weapon_tip(def: usize, tier: u8, owned: Option<u8>) -> Tip {
    let w = &WEAPONS[def];
    let dmg = |t: u8| w.damage * TIER_DAMAGE[t as usize - 1];
    let mut body = format!(
        "{} damage: {}.\n{:.0} damage every {:.2} s, range {:.0}.\nAbout {:.0} damage per second on one target.\n\"{}\"",
        w.kind.name(),
        pattern_words(w.pattern),
        dmg(tier),
        w.cooldown,
        w.range,
        dmg(tier) / w.cooldown,
        w.flavour
    );
    if let Some(have) = owned {
        let to = (have + 1).max(tier).min(4);
        body = format!("UPGRADE: tier {} -> {}, {:.0} -> {:.0} damage per hit.\n\n{body}", roman(have), roman(to), dmg(have), dmg(to));
    }
    Tip::new(format!("{} {}", w.name, roman(tier)), body, theme::type_color(w.kind))
}

fn pattern_words(p: Pattern) -> &'static str {
    match p {
        Pattern::Shot { .. } => "a shot at the nearest slop",
        Pattern::Burst { .. } => "a fan of shots",
        Pattern::Lob { .. } => "lobbed; splashes where it lands",
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

fn item_color(kind: ItemKind) -> Color {
    match kind {
        ItemKind::Risky => theme::DANGER,
        ItemKind::Hustle => tier_color(3),
        ItemKind::Active => theme::type_color(DamageType::Fact),
        ItemKind::Healthy => theme::AGREE,
    }
}

fn item_tip(item: usize, count: u32) -> Tip {
    let d = &ITEMS[item];
    let have = if count > 1 { format!("\nYou would have {count}; they stack.") } else { String::new() };
    Tip::new(d.name, format!("{}, tier {}.\n{}.\n\"{}\"{have}", d.kind.name(), roman(d.tier), d.effect, d.flavour), item_color(d.kind))
}

fn heroes(p: &mut ChildSpawnerCommands, game: &Game, art: &Art) {
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
    hints(p, &[("Arrows", "choose"), ("1-5", "jump to a hero"), ("Enter/Space", "start as this hero"), ("Q", "quit")]);
}

/// A slop on the briefing: picture, name and what it does, all visible at once.
fn slop_card(p: &mut ChildSpawnerCommands, image: &Handle<Image>, name: &str, blurb: &str) {
    p.spawn((Node { width: Val::Px(338.0), min_height: Val::Px(96.0), column_gap: Val::Px(10.0), align_items: AlignItems::Center, padding: UiRect::all(Val::Px(8.0)), border: UiRect::all(Val::Px(2.0)), ..default() }, BorderColor::all(Color::srgb(0.35, 0.35, 0.42)), BackgroundColor(CARD)))
        .with_children(|c| {
            icon(c, image, 64.0);
            c.spawn(Node { flex_shrink: 1.0, ..column(3.0) }).with_children(|t| {
                label(t, name, 16.0, theme::PAPER);
                label(t, blurb, 13.0, theme::QUIET);
            });
        });
}

/// Once per run, before wave 1: where you are and what is coming.
fn briefing(p: &mut ChildSpawnerCommands, run: &Run, art: &Art) {
    backdrop(p, Color::srgba(0.06, 0.06, 0.09, 0.95));
    let platform = run.platform;
    label(p, format!("TODAY ON {}", platform.name().to_uppercase()), 46.0, theme::PAPER);
    label(p, format!("{}  {}", platform.hazard(), platform.house_rule()), 19.0, theme::QUIET);
    label(p, "WHO YOU WILL MEET", 16.0, theme::QUIET);
    let mut slop: Vec<(&Handle<Image>, &str, &str)> = Vec::new();
    if platform != Platform::ClockTok {
        for a in [Archetype::ChadGpt, Archetype::Clawed, Archetype::YesBot, Archetype::Hallucinator, Archetype::ListicleDrone].iter().chain(platform.locals()) {
            slop.push((&art.archetypes[a].0, a.def().name, a.def().blurb));
        }
    }
    let g = run.training.best();
    slop.push((&art.bodies[&g.body].0, "Generated brainrot", "Body, object and accessory, each with an ability. The ones that hurt you breed. From wave 3 (ClockTok: wave 1)."));
    // Fixed rows of four: wrapping rows inside a centred column overlap what follows.
    for line in slop.chunks(4) {
        p.spawn(row(12.0)).with_children(|r| {
            for (image, name, blurb) in line {
                slop_card(r, image, name, blurb);
            }
        });
    }
    p.spawn(Node { height: Val::Px(6.0), ..default() });
    p.spawn(row(30.0)).with_children(|r| {
        for (wave, boss) in [(10, platform.boss()), (20, slop_sim::BossKind::TheModel)] {
            r.spawn((card(560.0), BorderColor::all(theme::DANGER), BackgroundColor(CARD))).with_children(|c| {
                c.spawn(row(14.0)).with_children(|l| {
                    icon(l, &art.bosses[&boss].0, 86.0);
                    l.spawn(column(4.0)).with_children(|t| {
                        label(t, format!("WAVE {wave} BOSS"), 14.0, theme::DANGER);
                        label(t, boss.name(), 22.0, theme::PAPER);
                        label(t, boss.puzzle(), 15.0, theme::QUIET);
                    });
                });
            });
        }
    });
    p.spawn(Node { max_width: Val::Px(1100.0), ..column(4.0) }).with_children(|c| {
        label(c, "THE SLOP LEARNS", 16.0, theme::QUIET);
        label(c, "After every wave it retrains on what killed it: it resists the damage types you kill with most.", 16.0, theme::PAPER);
        label(c, "Resistance is zero-sum, so countering you opens a weakness elsewhere. Watch SLOP RESISTANCE in the shop.", 16.0, theme::PAPER);
    });
    button(p, "Start wave 1", UiAction::BeginRun, theme::PLAYER_RED, key::BUTTON).insert((Continue, DefaultFocus));
    hints(p, &[("Space/Enter", "start wave 1"), ("Esc", "back to heroes")]);
}

/// Your ten stats and the slop's resistances, each with a tooltip (UI-13).
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
                // What the slop said about it after the last wave.
                let said: String = run.notes.iter().flat_map(|n| &n.lines).filter(|l| l.text.starts_with(t.name())).map(|l| format!("\n\nLast wave: {}", l.text)).collect();
                let tip = Tip::new(
                    format!("{} resistance {:+.0}%", t.name(), r * 100.0),
                    format!(
                        "Chatbots take {:.0}% damage from {} weapons; brainrot has its own genes on top.\nThe slop learns to resist what kills it. Resistance is zero-sum: when one rises, the others fall.{said}",
                        (1.0 - r) * 100.0,
                        t.name()
                    ),
                    theme::type_color(t),
                );
                col.spawn((focusable(key::RESIST + t as u32, Color::NONE), Node { padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)), column_gap: Val::Px(10.0), align_items: AlignItems::Center, ..default() }, tip))
                    .with_children(|line| {
                        icon(line, &art.type_icons[t as usize], 24.0);
                        line.spawn((Text::new(t.name()), TextFont { font_size: 16.0, ..default() }, TextColor(theme::type_color(t)), Node { flex_grow: 1.0, ..default() }));
                        label(line, format!("{:+.0}%", r * 100.0), 16.0, if r > 0.01 { theme::DANGER } else if r < -0.01 { theme::AGREE } else { theme::PAPER });
                    });
            }
        });
}

fn level_up(p: &mut ChildSpawnerCommands, run: &Run, offers: &[slop_rules::levels::StatUp], art: &Art) {
    backdrop(p, Color::srgba(0.05, 0.05, 0.08, 0.75));
    p.spawn(Node { width: Val::Percent(100.0), height: Val::Percent(100.0), column_gap: Val::Px(20.0), padding: UiRect { left: Val::Px(24.0), right: Val::Px(0.0), top: Val::Px(0.0), bottom: Val::Px(40.0) }, ..default() }).with_children(|page| {
        page.spawn(Node { flex_grow: 1.0, flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center, align_items: AlignItems::Center, row_gap: Val::Px(14.0), ..default() }).with_children(|c| {
            c.spawn(row(14.0)).with_children(|t| {
                icon(t, &art.star, 64.0);
                label(t, "LEVEL UP!", 56.0, tier_color(3));
                icon(t, &art.star, 64.0);
            });
            let more = run.pending_levels.saturating_sub(1);
            let reached = run.level - run.pending_levels + 1;
            label(c, if more > 0 { format!("Level {reached}. {more} more to pick after this one.") } else { format!("Level {reached}. Pick one.") }, 19.0, theme::QUIET);
            c.spawn(Node { column_gap: Val::Px(16.0), margin: UiRect::top(Val::Px(10.0)), ..default() }).with_children(|r| {
                let now = run.stats();
                for (i, up) in offers.iter().enumerate() {
                    let mut after = now;
                    up.stat.apply(&mut after, up.amount);
                    let grade = ["", "GOOD", "GREAT"][up.tier as usize - 1];
                    let color = tier_color(up.tier + 1);
                    let tip = Tip::new(up.describe(), format!("{}\n\nAfterwards: {}", up.stat.explain(&now), up.stat.explain(&after)), color);
                    let mut e = r.spawn((focusable(key::LEVEL + i as u32, CARD), Act(UiAction::LevelPick(i)), card(220.0), BorderColor::all(color), tip));
                    if i == 0 {
                        e.insert(DefaultFocus);
                    }
                    e.with_children(|c| {
                        c.spawn(Node { justify_content: JustifyContent::SpaceBetween, ..default() }).with_children(|h| {
                            label(h, format!("{}", i + 1), 16.0, theme::QUIET);
                            label(h, grade, 16.0, color);
                        });
                        c.spawn(Node { justify_content: JustifyContent::Center, ..default() }).with_children(|m| icon(m, &art.stat_icons[&up.stat], 84.0));
                        label(c, up.describe(), 23.0, theme::PAPER);
                        label(c, format!("{} -> {}", up.stat.read(&now), up.stat.read(&after)), 16.0, theme::QUIET);
                    });
                }
            });
            if let Some(r) = run.reports.last() {
                label(c, format!("Wave {} cleared: {} slop deleted, +{} Likes, +{} Creator Fund.", r.wave, r.kills, r.likes, r.payout), 16.0, theme::QUIET);
            }
        });
        stats_panel(page, run, art);
    });
    hints(p, &[("Arrows", "choose"), ("1-4", "pick"), ("Enter/Space", "pick the highlighted one")]);
}

/// Offer `i`, shown at position `n`: its number key and its focus key, so
/// focus stays in place when the card before it is bought.
fn offer_card(c: &mut ChildSpawnerCommands, run: &Run, i: usize, n: usize, art: &Art) {
    let o = &run.shop.offers[i];
    let tier = o.goods.tier();
    let upgrade = run.upgrades(i);
    let afford = o.price <= run.clout;
    let mut tip = match o.goods {
        Goods::Weapon { def, tier } => weapon_tip(def, tier, upgrade),
        Goods::Item { item } => item_tip(item, run.items.iter().find(|(it, _)| *it == item).map_or(0, |(_, n)| *n + 1)),
    };
    tip.body = format!("{}\n\nEnter or {}: buy for {} Clout.{}", tip.body, n + 1, o.price, if o.note.is_none() { " L or right-click: lock it for later." } else { "" });
    let border = if o.locked { tier_color(3) } else { tier_color(tier) };
    let node = Node { width: Val::Auto, flex_grow: 1.0, flex_basis: Val::Px(0.0), max_width: Val::Px(250.0), ..card(0.0) };
    let mut e = c.spawn((focusable(key::OFFER + n as u32, CARD), Act(UiAction::Buy(i)), node, BorderColor::all(border), tip));
    if o.note.is_none() {
        e.insert(Alt(UiAction::Lock(i)));
    }
    if n == 0 {
        e.insert(DefaultFocus);
    }
    e.with_children(|c| {
        c.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, min_height: Val::Px(24.0), ..default() }).with_children(|h| {
            label(h, format!("{}  {}", n + 1, roman(tier)), 15.0, tier_color(tier));
            if o.locked {
                icon(h, &art.lock, 24.0);
            }
        });
        if let Some(t) = upgrade {
            c.spawn((Node { padding: UiRect::horizontal(Val::Px(6.0)), ..default() }, BackgroundColor(tier_color(3)))).with_children(|r| label(r, format!("UPGRADE {} -> {}", roman(t), roman((t + 1).max(tier).min(4))), 14.0, theme::INK));
        }
        match o.note {
            Some(Note::LootBox) => label(c, "LOOT BOX: free", 14.0, tier_color(3)),
            Some(Note::Mercy) => label(c, "You look stressed: half price", 14.0, theme::AGREE),
            None => {}
        }
        let image = match o.goods {
            Goods::Weapon { def, .. } => &art.weapon_icons[def],
            Goods::Item { item } => &art.item_icons[item],
        };
        c.spawn(Node { justify_content: JustifyContent::Center, padding: UiRect::vertical(Val::Px(4.0)), ..default() }).with_children(|m| icon(m, image, 76.0));
        label(c, o.goods.name(), 20.0, theme::PAPER);
        match o.goods {
            Goods::Weapon { def, tier } => {
                let w = &WEAPONS[def];
                c.spawn(row(6.0)).with_children(|l| {
                    icon(l, &art.type_icons[w.kind as usize], 20.0);
                    label(l, format!("{} weapon", w.kind.name()), 14.0, theme::type_color(w.kind));
                });
                label(c, format!("{:.0} dmg every {:.2} s", w.damage * TIER_DAMAGE[tier as usize - 1], w.cooldown), 14.0, theme::PAPER);
                label(c, pattern_words(w.pattern), 13.0, theme::QUIET);
            }
            Goods::Item { item } => {
                let d = &ITEMS[item];
                label(c, d.kind.name(), 14.0, item_color(d.kind));
                label(c, d.effect, 14.0, theme::PAPER);
            }
        }
        c.spawn(Node { flex_grow: 1.0, ..default() });
        c.spawn(row(6.0)).with_children(|b| {
            icon(b, &art.coin, 22.0);
            label(b, o.price.to_string(), 22.0, if afford { theme::PAPER } else { theme::DANGER });
        });
    });
}

fn weapons_row(p: &mut ChildSpawnerCommands, run: &Run, art: &Art, sell: bool) {
    p.spawn(Node { column_gap: Val::Px(8.0), flex_wrap: FlexWrap::Wrap, row_gap: Val::Px(8.0), ..default() }).with_children(|ws| {
        for slot in 0..slop_rules::run::WEAPON_SLOTS {
            let Some(w) = run.weapons.get(slot) else {
                ws.spawn((Node { width: Val::Px(100.0), height: Val::Px(120.0), border: UiRect::all(Val::Px(2.0)), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BorderColor::all(Color::srgb(0.25, 0.25, 0.3))))
                    .with_children(|e| label(e, "empty", 14.0, Color::srgb(0.35, 0.35, 0.4)));
                continue;
            };
            let def = &WEAPONS[w.def];
            let mut tip = weapon_tip(w.def, w.tier, None);
            tip.body = format!("{:.0} damage dealt this run.\n{}", run.weapon_damage[w.def], tip.body);
            if sell {
                tip.body = format!("{}\n\nSell for {} Clout: Enter twice, X, or right-click twice.", tip.body, run.sell_value(slot));
            }
            let mut e = ws.spawn((
                focusable(key::WEAPON + slot as u32, CARD),
                Node { width: Val::Px(100.0), height: Val::Px(120.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, padding: UiRect::all(Val::Px(6.0)), row_gap: Val::Px(2.0), border: UiRect::all(Val::Px(3.0)), ..default() },
                BorderColor::all(tier_color(w.tier)),
                tip,
            ));
            if sell {
                e.insert((Act(UiAction::SellAsk(slot)), Alt(UiAction::SellAsk(slot))));
            }
            e.with_children(|c| {
                icon(c, &art.weapon_icons[w.def], 56.0);
                label(c, format!("{} {}", short(def.name), roman(w.tier)), 13.0, theme::PAPER);
                if sell {
                    label(c, format!("sell {}", run.sell_value(slot)), 12.0, theme::QUIET);
                } else {
                    label(c, format!("{:.0} dmg", run.weapon_damage[w.def]), 12.0, theme::QUIET);
                }
            });
        }
    });
}

/// Long names shortened for small cards.
fn short(name: &str) -> &str {
    match name {
        "Mechanical Keyboard" => "Keyboard",
        "Fact-Check Flashlight" => "Flashlight",
        "Uno Reverse Card" => "Uno",
        "Newspaper Roll" => "Newspaper",
        "CAPTCHA Turret" => "CAPTCHA",
        _ => name,
    }
}

fn items_grid(p: &mut ChildSpawnerCommands, run: &Run, art: &Art) {
    p.spawn(Node { column_gap: Val::Px(6.0), row_gap: Val::Px(6.0), flex_wrap: FlexWrap::Wrap, ..default() }).with_children(|is| {
        if run.items.is_empty() {
            label(is, "None yet.", 14.0, theme::QUIET);
        }
        for (n, &(item, count)) in run.items.iter().enumerate() {
            let d = &ITEMS[item];
            is.spawn((focusable(key::ITEM + n as u32, CARD), Node { width: Val::Px(62.0), height: Val::Px(62.0), border: UiRect::all(Val::Px(2.0)), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BorderColor::all(item_color(d.kind)), item_tip(item, count)))
                .with_children(|c| {
                    icon(c, &art.item_icons[item], 50.0);
                    if count > 1 {
                        c.spawn((Node { position_type: PositionType::Absolute, right: Val::Px(2.0), bottom: Val::Px(0.0), padding: UiRect::horizontal(Val::Px(4.0)), ..default() }, BackgroundColor(theme::INK)))
                            .with_children(|b| label(b, format!("x{count}"), 13.0, theme::PAPER));
                    }
                });
        }
    });
}

/// The offers still for sale, in the order they are shown and numbered.
pub fn shown_offers(run: &Run) -> Vec<usize> {
    (0..run.shop.offers.len()).filter(|&i| !run.shop.offers[i].sold).collect()
}

/// Where the shop shows details of the highlighted thing, instead of a
/// floating tooltip over the cards.
fn details(p: &mut ChildSpawnerCommands) {
    p.spawn((TipDock, Node { flex_grow: 1.0, flex_basis: Val::Px(0.0), padding: UiRect::all(Val::Px(14.0)), border: UiRect::all(Val::Px(2.0)), ..default() }, BorderColor::all(Color::srgb(0.22, 0.22, 0.28))));
}

fn shop(p: &mut ChildSpawnerCommands, run: &Run, art: &Art) {
    backdrop(p, Color::srgb(0.10, 0.10, 0.14));
    p.spawn(Node { width: Val::Percent(100.0), height: Val::Percent(100.0), column_gap: Val::Px(16.0), padding: UiRect { left: Val::Px(20.0), right: Val::Px(0.0), top: Val::Px(16.0), bottom: Val::Px(40.0) }, ..default() }).with_children(|page| {
        page.spawn(Node { flex_grow: 1.0, flex_direction: FlexDirection::Column, row_gap: Val::Px(12.0), ..default() }).with_children(|left| {
            // Header: title, what the last wave paid, Clout, reroll and go.
            left.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, ..default() }).with_children(|h| {
                h.spawn(column(2.0)).with_children(|t| {
                    label(t, "MARKETPLACE", 40.0, theme::PAPER);
                    let paid = run.reports.last().map_or(String::new(), |r| {
                        let mut s = format!("Wave {}: +{} Likes  +{} Creator Fund  +{} interest", r.wave, r.likes, r.payout, r.interest);
                        if r.income != 0 {
                            s += &format!("  {:+} side hustles", r.income);
                        }
                        s
                    });
                    label(t, paid, 15.0, theme::QUIET);
                });
                h.spawn(row(12.0)).with_children(|c| {
                    icon(c, &art.coin, 40.0);
                    label(c, run.clout.to_string(), 40.0, theme::PAPER);
                    c.spawn(Node { width: Val::Px(10.0), ..default() });
                    let cost = run.reroll_cost();
                    let mut b = button(c, if cost == 0 { "Reroll free".to_string() } else { format!("Reroll {cost}") }, UiAction::Reroll, BUTTON, key::BUTTON);
                    b.insert(Tip::new("Reroll (R)", "New offers. Locked offers stay. Each reroll in the same shop costs a little more; the price resets next shop.", theme::PAPER));
                    b.with_children(|r| icon(r, &art.reroll, 26.0));
                    button(c, format!("Start wave {}", run.wave), UiAction::NextWave, theme::PLAYER_RED, key::BUTTON + 1).insert(Continue);
                });
            });
            // Offers; bought ones leave the row.
            left.spawn(Node { column_gap: Val::Px(10.0), min_height: Val::Px(330.0), align_items: AlignItems::Stretch, ..default() }).with_children(|r| {
                let shown = shown_offers(run);
                if shown.is_empty() {
                    r.spawn(Node { flex_grow: 1.0, justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() })
                        .with_children(|e| label(e, "Sold out. Reroll for new offers, or start the wave.", 20.0, theme::QUIET));
                }
                for (n, &i) in shown.iter().enumerate() {
                    offer_card(r, run, i, n, art);
                }
            });
            // Your weapons and items; details of what is highlighted.
            left.spawn(Node { column_gap: Val::Px(16.0), flex_grow: 1.0, ..default() }).with_children(|bottom| {
                bottom.spawn(Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(8.0), flex_grow: 1.3, flex_basis: Val::Px(0.0), ..default() }).with_children(|c| {
                    label(c, format!("YOUR WEAPONS  {} / 6", run.weapons.len()), 16.0, theme::QUIET);
                    weapons_row(c, run, art, true);
                    label(c, "YOUR ITEMS", 16.0, theme::QUIET);
                    items_grid(c, run, art);
                });
                details(bottom);
            });
        });
        stats_panel(page, run, art);
    });
    hints(p, &[("Arrows", "choose"), ("Enter", "buy / sell"), ("1-8", "buy"), ("L", "lock"), ("R", "reroll"), ("Space", "start wave")]);
}

fn pause(p: &mut ChildSpawnerCommands, run: &Run, art: &Art) {
    backdrop(p, Color::srgba(0.04, 0.04, 0.07, 0.78));
    p.spawn(Node { width: Val::Percent(100.0), height: Val::Percent(100.0), column_gap: Val::Px(16.0), padding: UiRect { left: Val::Px(40.0), right: Val::Px(0.0), top: Val::Px(30.0), bottom: Val::Px(40.0) }, ..default() }).with_children(|page| {
        page.spawn(Node { flex_grow: 1.0, flex_direction: FlexDirection::Column, row_gap: Val::Px(12.0), ..default() }).with_children(|c| {
            label(c, "PAUSED", 52.0, theme::PAPER);
            if let Some(w) = &run.world {
                let left = if w.boss.is_some() { "boss wave".to_string() } else { format!("{:.0} s left", (w.setup.seconds - w.elapsed()).max(0.0)) };
                label(c, format!("Wave {} on {}  -  {}", run.wave, run.platform.name(), left), 18.0, theme::QUIET);
            }
            c.spawn(row(12.0)).with_children(|r| {
                button(r, "Resume", UiAction::Resume, theme::PLAYER_RED, key::BUTTON).insert((Continue, DefaultFocus));
                button(r, "Quit game", UiAction::Quit, BUTTON, key::BUTTON + 1);
            });
            c.spawn(Node { height: Val::Px(10.0), ..default() });
            label(c, format!("WEAPONS  {} / 6", run.weapons.len()), 16.0, theme::QUIET);
            weapons_row(c, run, art, false);
            label(c, "ITEMS", 16.0, theme::QUIET);
            items_grid(c, run, art);
        });
        stats_panel(page, run, art);
    });
    hints(p, &[("Arrows", "inspect"), ("Space/P/Esc", "resume"), ("Q", "quit")]);
}

fn over(p: &mut ChildSpawnerCommands, run: &Run, victory: bool, art: &Art) {
    backdrop(p, Color::srgba(0.07, 0.07, 0.1, 0.9));
    let (title, sub, color) = if victory {
        ("YOU LOGGED OFF", "Twenty waves of slop, and your Mental Health survived. Touch grass. You earned it.", theme::mental_health_color(1.0))
    } else {
        ("BURNOUT", "Your account has been suspended.", theme::DANGER)
    };
    icon(p, &art.human[&run.hero][if victory { 0 } else { 3 }], 110.0);
    label(p, title, 80.0, color);
    label(p, sub, 22.0, theme::QUIET);
    let reached = run.reports.last().map_or(1, |r| r.wave);
    let kills = run.kills_by_type();
    let total: u32 = kills.iter().sum::<u32>().max(1);
    label(p, format!("{} on {}  -  reached wave {} of 20  -  level {}  -  {} slop deleted", run.hero.def().name, run.platform.name(), reached, run.level, total), 18.0, theme::PAPER);
    p.spawn(row(18.0)).with_children(|r| {
        for t in DamageType::ALL {
            r.spawn(row(6.0)).with_children(|k| {
                icon(k, &art.type_icons[t as usize], 26.0);
                label(k, format!("{} {:.0}%", t.name(), 100.0 * kills[t as usize] as f32 / total as f32), 17.0, theme::type_color(t));
            });
        }
        label(r, format!("Model Collapse {:.0}%", slop_rules::training::collapse(&kills) * 100.0), 17.0, theme::PAPER);
    });
    weapons_row(p, run, art, false);
    p.spawn(Node { height: Val::Px(8.0), ..default() });
    p.spawn(row(14.0)).with_children(|r| {
        button(r, "New run", UiAction::NewRun, theme::PLAYER_RED, key::BUTTON).insert((Continue, DefaultFocus));
        button(r, "Quit", UiAction::Quit, BUTTON, key::BUTTON + 1);
    });
    hints(p, &[("Arrows", "inspect"), ("Space", "new run"), ("Q", "quit")]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use slop_rules::shop::Offer;

    // REQ: UI-11
    #[test]
    fn bought_offers_leave_the_row_and_numbers_count_what_is_left() {
        let mut run = Run::new(3, Hero::Teacher);
        let offer = |item| Offer { goods: Goods::Item { item }, price: 10, locked: false, sold: false, note: None };
        run.shop.offers = (0..4).map(offer).collect();
        run.shop.offers[1].sold = true;
        assert_eq!(shown_offers(&run), vec![0, 2, 3], "card 2 on screen is offer 3, so key 2 buys offer 3");
        for o in &mut run.shop.offers {
            o.sold = true;
        }
        assert!(shown_offers(&run).is_empty());
    }
}
