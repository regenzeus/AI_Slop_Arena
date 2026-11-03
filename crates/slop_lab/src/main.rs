//! Headless tools for AI Slop Arena.
//!
//! ```text
//! slop_lab bench                  time a crowded arena (SIM-04)
//! slop_lab determinism            replay a whole run twice and compare (SIM-01)
//! slop_lab balance [runs]         bots play whole runs; win rates by policy, hero and platform (BAL-02)
//! slop_lab economy [runs]         Clout, levels and purchases per wave for the heuristic bot
//! ```

use std::collections::BTreeMap;
use std::time::Instant;

use slop_rules::bot::{self, Policy};
use slop_rules::{Hero, Phase, Run};
use slop_sim::content::{Archetype, Platform, WEAPONS};
use slop_sim::world::{Loadout, Projectile, WaveSetup, World, BUBBLE_RADIUS};
use slop_sim::{DamageType, Input, Look, V2};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let n = |i: usize, default: u64| args.get(i).and_then(|v| v.parse().ok()).unwrap_or(default);
    match args.first().map(String::as_str) {
        Some("bench") => bench(),
        Some("determinism") => determinism(),
        Some("balance") => balance(n(1, 60)),
        Some("economy") => economy(n(1, 30)),
        _ => eprintln!("usage: slop_lab <bench|determinism|balance [runs]|economy [runs]>"),
    }
}

/// 400 slop, 300 bubbles and six tier IV weapons firing fast around you.
fn bench() {
    let picks = ["Rock", "Stapler", "Library Book", "Eye-Roll", "Slow Clap", "Seed Bomb"];
    let weapons: Vec<(usize, u8)> = picks.iter().map(|p| (slop_sim::content::weapon_index(p).unwrap(), 4)).collect();
    let mut loadout = Loadout { weapons, stats: Default::default(), perks: Default::default() };
    loadout.stats.focus = 3.0;
    loadout.stats.max_mental_health = 1e9;
    let mut w = World::new(1, WaveSetup::sandbox(), &loadout);
    let centre = w.player.pos;
    for i in 0..400 {
        let a = Archetype::ALL[i % 5];
        let at = centre + V2::from_angle(i as f32 * 0.37) * (150.0 + (i % 40) as f32 * 14.0);
        w.spawn_archetype(a, at);
    }
    for s in &mut w.slop {
        s.hp = 1e9;
        s.max_hp = 1e9;
    }
    for i in 0..300u32 {
        let at = centre + V2::from_angle(i as f32 * 0.21) * 500.0;
        w.projectiles.push(Projectile::new(1_000_000 + i, at, (centre - at).normalized() * 40.0, BUBBLE_RADIUS, 0.0, DamageType::Analog, true, Look::Bubble, u16::MAX));
    }
    let ticks = 600;
    let start = Instant::now();
    let mut peak = 0;
    for _ in 0..ticks {
        w.step(Input { x: 0.3, y: 0.1 });
        w.drain_events();
        peak = peak.max(w.projectiles.len());
    }
    let per_tick = start.elapsed().as_secs_f64() * 1000.0 / ticks as f64;
    println!(
        "{} slop, up to {} projectiles: {:.3} ms per tick (budget 2 ms) {}",
        w.slop.len(),
        peak,
        per_tick,
        if per_tick < 2.0 { "OK" } else { "OVER BUDGET" }
    );
}

fn fingerprint(seed: u64) -> (u64, u32) {
    let mut run = Run::new(seed, Hero::Teacher);
    let mut rng = slop_sim::rng::Rng::new(seed, 11);
    let mut hash = 0u64;
    let mut ticks = 0u32;
    while !run.is_over() && run.wave <= 6 {
        match run.phase {
            Phase::Wave => {
                let input = slop_sim::bot::steer(run.world.as_ref().unwrap());
                run.step(input);
                ticks += 1;
                if let Some(w) = &run.world {
                    hash = hash.rotate_left(5) ^ w.checksum();
                }
            }
            _ => {
                bot::decide(&mut run, Policy::Heuristic, &mut rng);
            }
        }
    }
    (hash, ticks)
}

fn determinism() {
    let (a, b) = (fingerprint(42), fingerprint(42));
    println!("seed 42 twice, six waves: {:016x} vs {:016x} over {} ticks -> {}", a.0, b.0, a.1, if a == b { "identical" } else { "DIFFERENT" });
    if a != b {
        std::process::exit(1);
    }
}

fn balance(runs: u64) {
    let start = Instant::now();
    for policy in [Policy::Heuristic, Policy::Random] {
        let mut wins = 0;
        let mut deaths: BTreeMap<u32, u32> = BTreeMap::new();
        let mut by_hero: BTreeMap<&str, (u32, u32)> = BTreeMap::new();
        let mut by_platform: BTreeMap<&str, (u32, u32)> = BTreeMap::new();
        let mut weapon_use: BTreeMap<&str, u32> = BTreeMap::new();
        let mut boss_deaths: BTreeMap<&str, u32> = BTreeMap::new();
        for seed in 1..=runs {
            let hero = Hero::ALL[(seed % 5) as usize];
            let run = Run::new(seed, hero);
            let platform = run.platform;
            drop(run);
            let s = bot::play(seed, hero, policy);
            if s.victory {
                wins += 1;
            } else {
                *deaths.entry(s.wave).or_default() += 1;
                if s.wave == 10 {
                    *boss_deaths.entry(platform.boss().name()).or_default() += 1;
                }
            }
            let h = by_hero.entry(hero.def().name).or_default();
            h.0 += s.victory as u32;
            h.1 += 1;
            let p = by_platform.entry(platform.name()).or_default();
            p.0 += s.victory as u32;
            p.1 += 1;
            for (name, _) in &s.weapons {
                let name = WEAPONS.iter().find(|w| w.name == name).map_or("?", |w| w.name);
                *weapon_use.entry(name).or_default() += 1;
            }
        }
        println!("{policy:?}: {} of {runs} won ({:.0}%)", wins, 100.0 * wins as f64 / runs as f64);
        let d: Vec<String> = deaths.iter().map(|(w, n)| format!("w{w}:{n}")).collect();
        println!("  burnouts by wave  {}", d.join(" "));
        let h: Vec<String> = by_hero.iter().map(|(k, (w, n))| format!("{k} {w}/{n}")).collect();
        let b: Vec<String> = boss_deaths.iter().map(|(k, n)| format!("{k} {n}")).collect();
        println!("  lost to boss      {}", b.join(", "));
        println!("  by hero           {}", h.join(", "));
        let p: Vec<String> = by_platform.iter().map(|(k, (w, n))| format!("{k} {w}/{n}")).collect();
        println!("  by platform       {}", p.join(", "));
        if policy == Policy::Heuristic {
            let mut u: Vec<_> = weapon_use.into_iter().collect();
            u.sort_by(|a, b| b.1.cmp(&a.1));
            let u: Vec<String> = u.iter().take(8).map(|(k, n)| format!("{k} {n}")).collect();
            println!("  weapons at the end {}", u.join(", "));
        }
    }
    let _ = Platform::ALL;
    println!("({:.0} s)", start.elapsed().as_secs_f64());
}

/// Per wave, averaged over the runs that reached its shop: Clout earned in
/// the wave, Clout held when the shop opens, levels, and tiers bought.
fn economy(runs: u64) {
    #[derive(Default, Clone, Copy)]
    struct Row {
        runs: u32,
        earned: u32,
        held: u32,
        levels: u32,
        level: u32,
        bought: u32,
        rerolls: u32,
    }
    let mut rows = [Row::default(); 21];
    for seed in 1..=runs {
        let hero = Hero::ALL[(seed % 5) as usize];
        let mut run = Run::new(seed, hero);
        let mut rng = slop_sim::rng::Rng::new(seed, 11);
        loop {
            match run.phase {
                Phase::Over { .. } => break,
                Phase::Wave => {
                    let input = slop_sim::bot::steer(run.world.as_ref().unwrap());
                    run.step(input);
                }
                Phase::Shop => {
                    let owned = |run: &Run| run.items.iter().map(|(_, n)| *n).sum::<u32>() + run.weapons.iter().map(|w| w.tier as u32).sum::<u32>();
                    let before = owned(&run);
                    let r = run.reports.last().unwrap();
                    let row = &mut rows[r.wave as usize];
                    row.runs += 1;
                    row.earned += r.likes;
                    row.held += run.clout;
                    row.levels += r.levels;
                    row.level += run.level;
                    // The bot buys, rerolls and starts the next wave in one decision.
                    let rerolls_before = run.shop.rerolls;
                    bot::decide(&mut run, Policy::Heuristic, &mut rng);
                    row.bought += owned(&run).saturating_sub(before);
                    row.rerolls += run.shop.rerolls.saturating_sub(rerolls_before);
                }
                _ => {
                    bot::decide(&mut run, Policy::Heuristic, &mut rng);
                }
            }
        }
    }
    println!("wave  runs  earned  held@shop  levels  level  bought  rerolls");
    for (w, r) in rows.iter().enumerate().filter(|(_, r)| r.runs > 0) {
        let n = r.runs as f32;
        println!(
            "{:>4}  {:>4}  {:>6.1}  {:>9.1}  {:>6.2}  {:>5.1}  {:>6.2}  {:>7.2}",
            w,
            r.runs,
            r.earned as f32 / n,
            r.held as f32 / n,
            r.levels as f32 / n,
            r.level as f32 / n,
            r.bought as f32 / n,
            r.rerolls as f32 / n
        );
    }
}
