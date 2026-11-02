//! The screens around the waves (UI-08 to UI-12): title, hero select,
//! level-ups, patch notes, the full-screen shop, and the end of a run.
//! They are rebuilt from the run whenever something changes.

use bevy::prelude::*;
use slop_rules::items::{ItemKind, StatKind, ITEMS};
use slop_rules::shop::{Goods, Note};
use slop_rules::training::Section;
use slop_rules::{Hero, Phase, Run};
use slop_sim::content::{Pattern, WEAPONS};
use slop_sim::DamageType;

use crate::art::Art;
use crate::play::{Simulation, END_PAUSE};
use crate::state::{Game, Screen, UiAction};
use crate::theme;

pub struct ScreensPlugin;

impl Plugin for ScreensPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (clicks, rebuild, hover).chain().after(Simulation));
    }
}

#[derive(Component)]
struct ScreenRoot;

/// A clickable node and what it does.
#[derive(Component, Clone, Copy)]
pub struct Act(pub UiAction);

/// Background colours of a button: normal and hovered.
#[derive(Component, Clone, Copy)]
struct Shade(Color, Color);

pub fn roman(tier: u8) -> &'static str {
    ["I", "II", "III", "IV"][(tier.clamp(1, 4) - 1) as usize]
}

/// Bronze, silver, gold and diamond, as in Lineage.
pub fn tier_color(tier: u8) -> Color {
    match tier {
        1 => Color::srgb(0.72, 0.51, 0.29),
        2 => Color::srgb(0.66, 0.70, 0.75),
        3 => Color::srgb(0.91, 0.71, 0.20),
        _ => Color::srgb(0.44, 0.83, 0.91),
    }
}

fn label(p: &mut ChildSpawnerCommands, words: impl Into<String>, size: f32, color: Color) {
    p.spawn((Text::new(words), TextFont { font_size: size, ..default() }, TextColor(color)));
}

fn button(p: &mut ChildSpawnerCommands, words: impl Into<String>, action: UiAction, color: Color) {
    let hover = color.mix(&Color::WHITE, 0.25);
    p.spawn((
        Button,
        Act(action),
        Shade(color, hover),
        Node { padding: UiRect::axes(Val::Px(18.0), Val::Px(10.0)), justify_content: JustifyContent::Center, ..default() },
        BackgroundColor(color),
    ))
    .with_children(|b| label(b, words, 20.0, theme::PAPER));
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

fn card(color: Color, width: f32) -> (Node, BackgroundColor, BorderColor) {
    (
        Node {
            width: Val::Px(width),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(10.0)),
            row_gap: Val::Px(5.0),
            border: UiRect::all(Val::Px(4.0)),
            ..default()
        },
        BackgroundColor(theme::PANEL),
        BorderColor::all(color),
    )
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

fn hover(mut buttons: Query<(&Interaction, &Shade, &mut BackgroundColor), Changed<Interaction>>) {
    for (i, shade, mut bg) in &mut buttons {
        bg.0 = if *i == Interaction::None { shade.0 } else { shade.1 };
    }
}

/// What is on screen, to know when to rebuild.
#[derive(Default, PartialEq, Clone)]
struct Shown {
    screen: Option<Screen>,
    phase: String,
    settled: bool,
}

fn rebuild(mut commands: Commands, mut game: ResMut<Game>, art: Res<Art>, roots: Query<Entity, With<ScreenRoot>>, mut last: Local<Shown>) {
    let phase = game.run.as_ref().map_or(String::new(), |r| format!("{:?}{}", std::mem::discriminant(&r.phase), r.wave));
    let shown = Shown { screen: Some(game.screen), phase, settled: game.since_end > END_PAUSE };
    if !game.dirty && *last == shown {
        return;
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
        Screen::Run => {
            if let Some(run) = &game.run {
                let settled = game.since_end > END_PAUSE;
                match &run.phase {
                    Phase::LevelUp(offers) if settled => level_up(p, run, offers),
                    Phase::PatchNotes(notes) if settled => patch_notes(p, run, notes),
                    Phase::Shop => shop(p, run, &art),
                    Phase::Over { victory } if settled => over(p, run, *victory),
                    _ => {}
                }
            }
        }
    });
    if let Some((words, _)) = &game.message {
        commands.spawn((ScreenRoot, Node { position_type: PositionType::Absolute, bottom: Val::Px(84.0), width: Val::Percent(100.0), justify_content: JustifyContent::Center, ..default() }, GlobalZIndex(30)))
            .with_children(|p| {
                p.spawn((Node { padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)), ..default() }, BackgroundColor(theme::DANGER))).with_children(|m| label(m, words.clone(), 18.0, theme::PAPER));
            });
    }
}

fn title(p: &mut ChildSpawnerCommands, art: &Art) {
    p.spawn((Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }, BackgroundColor(Color::srgba(0.09, 0.09, 0.13, 0.82))));
    p.spawn((ImageNode::new(art.human[0].clone()), Node { width: Val::Px(110.0), height: Val::Px(110.0), ..default() }));
    label(p, "AI SLOP ARENA", 86.0, theme::PAPER);
    label(p, "One human. Twenty waves of AI slop. Protect your Mental Health.", 24.0, theme::QUIET);
    p.spawn(Node { height: Val::Px(20.0), ..default() });
    for line in ["WASD or arrows: move. Your weapons fire on their own.", "After every wave: level up, read the patch notes, shop.", "The slop trains on how you play. Adapt, or get fine-tuned."] {
        label(p, line, 19.0, theme::PAPER);
    }
    p.spawn(Node { height: Val::Px(20.0), ..default() });
    button(p, "Play  (Space)", UiAction::ToHeroes, theme::PLAYER_RED);
    label(p, "No meta progression. Everything is unlocked. One death ends the run.", 15.0, theme::QUIET);
}

fn heroes(p: &mut ChildSpawnerCommands, game: &Game, art: &Art) {
    p.spawn((Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }, BackgroundColor(Color::srgba(0.09, 0.09, 0.13, 0.88))));
    label(p, "CHOOSE YOUR HUMAN", 48.0, theme::PAPER);
    label(p, "Same human, different baggage. Your hero sets your starting loadout.", 18.0, theme::QUIET);
    p.spawn(Node { column_gap: Val::Px(14.0), ..default() }).with_children(|row| {
        for (i, h) in Hero::ALL.iter().enumerate() {
            let d = h.def();
            let selected = i == game.hero;
            let border = if selected { theme::LIKE } else { theme::QUIET };
            let (node, _, b) = card(border, 270.0);
            row.spawn((Button, Act(UiAction::PickHero(i)), Shade(theme::PANEL, Color::srgb(0.22, 0.21, 0.28)), node, BackgroundColor(theme::PANEL), b)).with_children(|c| {
                c.spawn((ImageNode::new(art.human[if selected { 0 } else { 1 }].clone()), Node { width: Val::Px(64.0), height: Val::Px(64.0), ..default() }));
                label(c, format!("{}  {}", i + 1, d.name), 26.0, theme::PAPER);
                label(c, format!("Starts with {} and {}", d.weapon, d.item), 15.0, theme::type_color(slop_sim::content::WEAPONS[slop_sim::content::weapon_index(d.weapon).unwrap_or(0)].kind));
                label(c, d.stats, 15.0, theme::PAPER);
                label(c, d.quirk, 15.0, theme::QUIET);
            });
        }
    });
    button(p, format!("Start as {}  (Space)", Hero::ALL[game.hero].def().name), UiAction::StartRun, theme::PLAYER_RED);
}

fn level_up(p: &mut ChildSpawnerCommands, run: &Run, offers: &[slop_rules::levels::StatUp]) {
    p.spawn((Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }, BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.6))));
    label(p, format!("LEVEL UP  -  Level {}", run.level - run.pending_levels + 1), 44.0, theme::PAPER);
    label(p, "Pick one. (1 to 4)", 18.0, theme::QUIET);
    p.spawn(Node { column_gap: Val::Px(14.0), ..default() }).with_children(|row| {
        for (i, up) in offers.iter().enumerate() {
            let (node, _, b) = card(tier_color(up.tier), 220.0);
            row.spawn((Button, Act(UiAction::LevelPick(i)), Shade(theme::PANEL, Color::srgb(0.22, 0.21, 0.28)), node, BackgroundColor(theme::PANEL), b)).with_children(|c| {
                label(c, format!("{}", i + 1), 16.0, theme::QUIET);
                label(c, up.describe(), 26.0, theme::PAPER);
                label(c, format!("Now {}", up.stat.read(&run.stats())), 15.0, theme::QUIET);
            });
        }
    });
}

fn patch_notes(p: &mut ChildSpawnerCommands, run: &Run, notes: &slop_rules::training::PatchNotes) {
    p.spawn((Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }, BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.7))));
    p.spawn((Node { width: Val::Px(900.0), flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(24.0)), row_gap: Val::Px(8.0), ..default() }, BackgroundColor(theme::PAPER)))
        .with_children(|page| {
            if let Some(r) = run.reports.last() {
                label(page, format!("Wave {} cleared  -  {} slop deleted  -  +{} Clout  -  +{} interest  -  {:+} income", r.wave, r.kills, r.likes, r.interest, r.income), 16.0, theme::QUIET);
            }
            label(page, format!("{} patch notes", notes.version), 36.0, theme::INK);
            label(page, "The slop has retrained on your last wave.", 16.0, theme::QUIET);
            for section in [Section::Improved, Section::Adjusted, Section::New, Section::BugFixes, Section::Preview] {
                let lines: Vec<_> = notes.lines.iter().filter(|l| l.section == section).collect();
                if lines.is_empty() {
                    continue;
                }
                label(page, section.title(), 20.0, theme::INK);
                for l in lines {
                    label(page, format!("  -  {}", l.text), 17.0, theme::INK);
                }
            }
            label(page, "Base model resistance", 18.0, theme::INK);
            page.spawn(Node { column_gap: Val::Px(18.0), ..default() }).with_children(|row| {
                for t in DamageType::ALL {
                    let r = notes.model[t as usize];
                    label(row, format!("{} {:+.0}%", t.name(), r * 100.0), 17.0, theme::type_color(t));
                }
            });
            page.spawn(Node { height: Val::Px(8.0), ..default() });
            button(page, "To the shop  (Space)", UiAction::ClosePatchNotes, theme::PLAYER_RED);
        });
}

fn weapon_text(def: usize, tier: u8) -> (String, String) {
    let w = &WEAPONS[def];
    let dmg = w.damage * slop_sim::content::TIER_DAMAGE[tier as usize - 1];
    let kind = match w.pattern {
        Pattern::Shot { .. } => "Shot",
        Pattern::Burst { .. } => "Burst",
        Pattern::Lob { .. } => "Lob, splash",
        Pattern::Mark { .. } => "Marks",
        Pattern::Ricochet { .. } => "Ricochet",
        Pattern::Boomerang { .. } => "Comes back",
        Pattern::Arc { .. } => "Melee swing",
        Pattern::Cone { .. } => "Spray",
        Pattern::Ring => "Ring around you",
        Pattern::Aura => "Aura, faster in crowds",
        Pattern::Chain { .. } => "Chains",
        Pattern::Beam { .. } => "Beam, strips filters",
        Pattern::Reflect => "Reflects bubbles",
        Pattern::Orbit { .. } => "Orbits you",
        Pattern::Zone { .. } => "Patch that heals you",
        Pattern::Slam { .. } => "Slam, deletes small slop",
        Pattern::Stun { .. } => "Stuns chatbots",
    };
    (
        format!("{} - {}", w.kind.name(), kind),
        format!("{dmg:.0} damage, every {:.2} s, range {:.0}. {}", w.cooldown, w.range, w.flavour),
    )
}

fn shop(p: &mut ChildSpawnerCommands, run: &Run, art: &Art) {
    p.spawn((Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }, BackgroundColor(Color::srgb(0.11, 0.11, 0.15))));
    p.spawn(Node { width: Val::Percent(96.0), height: Val::Percent(94.0), flex_direction: FlexDirection::Column, row_gap: Val::Px(12.0), ..default() }).with_children(|page| {
        // Header.
        page.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, ..default() }).with_children(|h| {
            h.spawn(Node { flex_direction: FlexDirection::Column, ..default() }).with_children(|t| {
                label(t, "MARKETPLACE", 40.0, theme::PAPER);
                label(t, format!("Next: wave {} on {}  -  Level {}", run.wave, run.platform.name(), run.level), 17.0, theme::QUIET);
            });
            h.spawn(Node { column_gap: Val::Px(10.0), align_items: AlignItems::Center, ..default() }).with_children(|c| {
                c.spawn((ImageNode::new(art.coin.clone()), Node { width: Val::Px(34.0), height: Val::Px(34.0), ..default() }));
                label(c, run.clout.to_string(), 34.0, theme::PAPER);
                let cost = run.shop.reroll_cost();
                button(c, if cost == 0 { "Reroll free  (R)".to_string() } else { format!("Reroll {cost}  (R)") }, UiAction::Reroll, Color::srgb(0.3, 0.3, 0.38));
                button(c, format!("Start wave {}  (Space)", run.wave), UiAction::NextWave, theme::PLAYER_RED);
            });
        });
        // Offers.
        page.spawn(Node { column_gap: Val::Px(10.0), ..default() }).with_children(|row| {
            for (i, o) in run.shop.offers.iter().enumerate() {
                let tier = o.goods.tier();
                let (mut node, _, border) = card(if o.sold { theme::QUIET } else { tier_color(tier) }, 240.0);
                node.min_height = Val::Px(250.0);
                let shade = if o.sold { Color::srgb(0.16, 0.16, 0.2) } else { theme::PANEL };
                row.spawn((Button, Act(UiAction::Buy(i)), Shade(shade, Color::srgb(0.24, 0.23, 0.3)), node, BackgroundColor(shade), border)).with_children(|c| {
                    if let Some(t) = run.upgrades(i).filter(|_| !o.sold) {
                        c.spawn((Node { padding: UiRect::horizontal(Val::Px(6.0)), ..default() }, BackgroundColor(tier_color(3))))
                            .with_children(|r| label(r, format!("UPGRADE {} > {}", roman(t), roman((t + 1).min(4))), 14.0, theme::INK));
                    }
                    match o.note {
                        Some(Note::LootBox) => label(c, "LOOT BOX: free", 14.0, tier_color(3)),
                        Some(Note::Mercy) => label(c, "You look stressed: half price", 14.0, theme::AGREE),
                        None => {}
                    }
                    label(c, format!("{}  {}", i + 1, roman(tier)), 14.0, tier_color(tier));
                    label(c, o.goods.name(), 22.0, theme::PAPER);
                    match o.goods {
                        Goods::Weapon { def, tier } => {
                            let (kind, body) = weapon_text(def, tier);
                            label(c, kind, 14.0, theme::type_color(WEAPONS[def].kind));
                            label(c, body, 14.0, theme::QUIET);
                        }
                        Goods::Item { item } => {
                            let d = &ITEMS[item];
                            let color = match d.kind {
                                ItemKind::Risky => theme::DANGER,
                                ItemKind::Hustle => tier_color(3),
                                ItemKind::Active => theme::type_color(DamageType::Fact),
                                ItemKind::Healthy => theme::AGREE,
                            };
                            label(c, d.kind.name(), 14.0, color);
                            label(c, d.effect, 14.0, theme::PAPER);
                            label(c, d.flavour, 13.0, theme::QUIET);
                        }
                    }
                    c.spawn(Node { flex_grow: 1.0, ..default() });
                    if o.sold {
                        label(c, "Sold", 18.0, theme::QUIET);
                    } else {
                        c.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, ..default() }).with_children(|b| {
                            let afford = o.price <= run.clout;
                            label(b, format!("{} Clout", o.price), 18.0, if afford { theme::PAPER } else { theme::DANGER });
                            if o.note.is_none() {
                                let words = if o.locked { "Locked" } else { "Lock" };
                                b.spawn((Button, Act(UiAction::Lock(i)), Shade(Color::srgb(0.3, 0.3, 0.38), Color::srgb(0.4, 0.4, 0.5)), Node { padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)), ..default() }, BackgroundColor(Color::srgb(0.3, 0.3, 0.38))))
                                    .with_children(|l| label(l, words, 14.0, if o.locked { tier_color(3) } else { theme::PAPER }));
                            }
                        });
                    }
                });
            }
        });
        // Your weapons, items and stats.
        page.spawn(Node { column_gap: Val::Px(16.0), flex_grow: 1.0, ..default() }).with_children(|bottom| {
            bottom.spawn(Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(8.0), width: Val::Percent(68.0), ..default() }).with_children(|left| {
                label(left, format!("YOUR WEAPONS  {} / 6", run.weapons.len()), 18.0, theme::QUIET);
                left.spawn(Node { column_gap: Val::Px(8.0), flex_wrap: FlexWrap::Wrap, row_gap: Val::Px(8.0), ..default() }).with_children(|ws| {
                    for (slot, w) in run.weapons.iter().enumerate() {
                        let def = &WEAPONS[w.def];
                        let (node, _, border) = card(tier_color(w.tier), 170.0);
                        ws.spawn((node, BackgroundColor(theme::PANEL), border)).with_children(|c| {
                            label(c, format!("{} {}", def.name, roman(w.tier)), 16.0, theme::PAPER);
                            label(c, def.kind.name(), 13.0, theme::type_color(def.kind));
                            let dealt = run.weapon_damage[w.def];
                            label(c, format!("{dealt:.0} damage this run"), 12.0, theme::QUIET);
                            c.spawn((Button, Act(UiAction::Sell(slot)), Shade(Color::srgb(0.3, 0.3, 0.38), Color::srgb(0.45, 0.3, 0.3)), Node { padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.3, 0.3, 0.38))))
                                .with_children(|b| label(b, format!("Sell +{}", run.sell_value(slot)), 13.0, theme::PAPER));
                        });
                    }
                });
                label(left, "YOUR ITEMS", 18.0, theme::QUIET);
                left.spawn(Node { column_gap: Val::Px(8.0), row_gap: Val::Px(6.0), flex_wrap: FlexWrap::Wrap, ..default() }).with_children(|is| {
                    for &(item, n) in &run.items {
                        let d = &ITEMS[item];
                        is.spawn((Node { padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)), border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(theme::PANEL), BorderColor::all(tier_color(d.tier))))
                            .with_children(|c| label(c, if n > 1 { format!("{} x{n}", d.name) } else { d.name.to_string() }, 14.0, theme::PAPER));
                    }
                });
            });
            bottom.spawn((Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(3.0), padding: UiRect::all(Val::Px(12.0)), width: Val::Percent(30.0), ..default() }, BackgroundColor(theme::PANEL))).with_children(|stats| {
                label(stats, "STATS", 18.0, theme::QUIET);
                let s = run.stats();
                for k in StatKind::ALL {
                    stats.spawn(Node { justify_content: JustifyContent::SpaceBetween, ..default() }).with_children(|r| {
                        label(r, k.name(), 15.0, theme::PAPER);
                        label(r, k.read(&s), 15.0, theme::PAPER);
                    });
                }
                let model = run.training.model;
                label(stats, "SLOP RESISTANCE", 16.0, theme::QUIET);
                for t in DamageType::ALL {
                    label(stats, format!("{} {:+.0}%", t.name(), model[t as usize] * 100.0), 14.0, theme::type_color(t));
                }
            });
        });
    });
}

fn over(p: &mut ChildSpawnerCommands, run: &Run, victory: bool) {
    p.spawn((Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }, BackgroundColor(Color::srgba(0.07, 0.07, 0.1, 0.9))));
    let (title, sub, color) = if victory {
        ("YOU LOGGED OFF", "Twenty waves of slop, and your Mental Health survived. Touch grass. You earned it.", theme::mental_health_color(1.0))
    } else {
        ("BURNOUT", "Your account has been suspended.", theme::DANGER)
    };
    label(p, title, 80.0, color);
    label(p, sub, 22.0, theme::QUIET);
    let reached = run.reports.last().map_or(1, |r| r.wave);
    let kills = run.kills_by_type();
    let total: u32 = kills.iter().sum::<u32>().max(1);
    let shares: Vec<String> = DamageType::ALL.iter().map(|t| format!("{} {:.0}%", t.name(), 100.0 * kills[*t as usize] as f32 / total as f32)).collect();
    let weapons: Vec<String> = run.weapons.iter().map(|w| format!("{} {}", WEAPONS[w.def].name, roman(w.tier))).collect();
    for line in [
        format!("{} on {}  -  reached wave {} of 20  -  level {}", run.hero.def().name, run.platform.name(), reached, run.level),
        format!("Slop deleted {}  -  kills by type: {}", total, shares.join(", ")),
        format!("Model Collapse {:.0}%", slop_rules::training::collapse(&kills) * 100.0),
        format!("Weapons: {}", weapons.join(", ")),
    ] {
        label(p, line, 18.0, theme::PAPER);
    }
    p.spawn(Node { height: Val::Px(12.0), ..default() });
    p.spawn(Node { column_gap: Val::Px(14.0), ..default() }).with_children(|r| {
        button(r, "New run  (Space)", UiAction::NewRun, theme::PLAYER_RED);
        button(r, "Quit  (Q)", UiAction::Quit, Color::srgb(0.3, 0.3, 0.38));
    });
}
