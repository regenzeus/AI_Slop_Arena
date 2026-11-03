//! Headless tools for AI Slop Arena.
//!
//! ```text
//! slop_lab bench                  time a crowded Feed (SIM-04)
//! slop_lab determinism            replay a run twice and compare (SIM-01)
//! slop_lab balance [runs] [min]   bots play whole runs; minutes survived, bosses (BAL-02)
//! slop_lab progress [runs]        level, kills, slop and weapons minute by minute
//! slop_lab deaths [runs]          what hurt the heuristic bot most in its last 20 seconds
//! ```

use std::collections::BTreeMap;
use std::time::Instant;

use slop_rules::bot::{self, Policy};
use slop_rules::{Hero, Phase, Run};
use slop_sim::content::{Archetype, WEAPONS};
use slop_sim::world::{Loadout, Projectile, WaveSetup, World, BUBBLE_RADIUS, MAX_LEVEL};
use slop_sim::{DamageType, Input, Look, V2};

/// Level, kills, slop out, health share and Clout at one minute of one run.
type Sample = (u32, u32, usize, f32, u32);

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let n = |i: usize, default: u64| args.get(i).and_then(|v| v.parse().ok()).unwrap_or(default);
    match args.first().map(String::as_str) {
        Some("bench") => bench(),
        Some("determinism") => determinism(),
        Some("balance") => balance(n(1, 30), n(2, 45) as f32),
        Some("progress") => progress(n(1, 8)),
        Some("deaths") => deaths(n(1, 8)),
        _ => eprintln!("usage: slop_lab <bench|determinism|balance [runs] [minutes]|progress [runs]>"),
    }
}

/// 400 slop, 300 bubbles and six max-level weapons firing fast around you.
fn bench() {
    let picks = ["Rock", "Stapler", "Library Book", "Eye-Roll", "Slow Clap", "Seed Bomb"];
    let weapons: Vec<(usize, u8)> = picks.iter().map(|p| (slop_sim::content::weapon_index(p).unwrap(), MAX_LEVEL)).collect();
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
        w.step(Input::moving(0.3, 0.1));
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
    while !run.is_over() && run.minutes() < 6.0 {
        match run.phase {
            Phase::Playing => {
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
    println!("seed 42 twice, six minutes: {:016x} vs {:016x} over {} ticks -> {}", a.0, b.0, a.1, if a == b { "identical" } else { "DIFFERENT" });
    if a != b {
        std::process::exit(1);
    }
}

fn balance(runs: u64, limit: f32) {
    let start = Instant::now();
    for policy in [Policy::Heuristic, Policy::Random] {
        let mut minutes = Vec::new();
        let mut bosses: BTreeMap<u32, u32> = BTreeMap::new();
        let mut by_hero: BTreeMap<&str, Vec<f32>> = BTreeMap::new();
        let mut model = 0;
        let mut weapon_use: BTreeMap<String, u32> = BTreeMap::new();
        for seed in 1..=runs {
            let hero = Hero::ALL[(seed % Hero::ALL.len() as u64) as usize];
            let s = bot::play(seed, hero, policy, limit);
            minutes.push(s.minutes);
            *bosses.entry(s.bosses).or_default() += 1;
            by_hero.entry(hero.def().name).or_default().push(s.minutes);
            if s.victory {
                model += 1;
            }
            for (name, _) in s.weapons {
                *weapon_use.entry(name).or_default() += 1;
            }
        }
        minutes.sort_by(|a, b| a.total_cmp(b));
        let median = minutes[minutes.len() / 2];
        let mean = minutes.iter().sum::<f32>() / minutes.len() as f32;
        println!("\n{policy:?} bot, {runs} runs (stopped at {limit} min):");
        println!("  minutes survived: median {median:.1}, mean {mean:.1}, shortest {:.1}, longest {:.1}", minutes[0], minutes[minutes.len() - 1]);
        println!("  The Model beaten: {model}/{runs}");
        println!("  bosses beaten: {}", bosses.iter().map(|(b, n)| format!("{b}: {n}")).collect::<Vec<_>>().join(", "));
        for (h, m) in &by_hero {
            println!("  {h}: mean {:.1} min", m.iter().sum::<f32>() / m.len() as f32);
        }
        let mut uses: Vec<_> = weapon_use.into_iter().collect();
        uses.sort_by(|a, b| b.1.cmp(&a.1));
        println!("  weapons at the end: {}", uses.iter().take(12).map(|(n, c)| format!("{n} {c}")).collect::<Vec<_>>().join(", "));
    }
    println!("\n{:.0} s", start.elapsed().as_secs_f32());
}

/// Level, kills, slop on the field and the build, minute by minute.
fn progress(runs: u64) {
    let mut rows: BTreeMap<u32, Vec<Sample>> = BTreeMap::new();
    for seed in 1..=runs {
        let hero = Hero::ALL[(seed % Hero::ALL.len() as u64) as usize];
        let mut run = Run::new(seed, hero);
        let mut rng = slop_sim::rng::Rng::new(seed, 11);
        let mut next = 1.0;
        while !run.is_over() && run.minutes() < 45.0 {
            match run.phase {
                Phase::Playing => {
                    let input = slop_sim::bot::steer(run.world.as_ref().unwrap());
                    run.step(input);
                    if run.minutes() >= next {
                        let w = run.world.as_ref().unwrap();
                        let health = w.player.mental_health / w.player.stats.max_mental_health;
                        rows.entry(next as u32).or_default().push((run.level, run.kills(), w.slop.len(), health, run.clout));
                        next += 1.0;
                    }
                }
                _ => {
                    bot::decide(&mut run, Policy::Heuristic, &mut rng);
                }
            }
        }
        let names: Vec<String> = run.weapons.iter().map(|w| format!("{} {}", WEAPONS[w.def].name, w.level)).collect();
        println!("run {seed} ({}): {:.1} min, level {}, {} bosses; {}", hero.def().name, run.minutes(), run.level, run.bosses_beaten, names.join(", "));
    }
    println!("\nminute | runs alive | level | kills | slop out | health | Clout");
    for (m, v) in rows {
        let k = v.len() as f32;
        let avg = |f: &dyn Fn(&Sample) -> f32| v.iter().map(f).sum::<f32>() / k;
        println!(
            "{m:>6} | {:>10} | {:>5.1} | {:>5.0} | {:>8.0} | {:>5.0}% | {:>5.0}",
            v.len(),
            avg(&|r| r.0 as f32),
            avg(&|r| r.1 as f32),
            avg(&|r| r.2 as f32),
            avg(&|r| r.3) * 100.0,
            avg(&|r| r.4 as f32)
        );
    }
}

/// What dealt the damage in the last 20 seconds of each run that ended in Burnout.
fn deaths(runs: u64) {
    for seed in 1..=runs {
        let hero = Hero::ALL[(seed % Hero::ALL.len() as u64) as usize];
        let mut run = Run::new(seed, hero);
        let mut rng = slop_sim::rng::Rng::new(seed, 11);
        // Damage per spec, sampled every second, so the last 20 s can be told apart.
        let mut history: Vec<Vec<f32>> = Vec::new();
        let mut next = 0.0;
        while !run.is_over() && run.minutes() < 45.0 {
            match run.phase {
                Phase::Playing => {
                    let input = slop_sim::bot::steer(run.world.as_ref().unwrap());
                    run.step(input);
                    if run.seconds() >= next {
                        next += 1.0;
                        history.push(run.world.as_ref().unwrap().spec_tally.iter().map(|t| t.damage).collect());
                    }
                }
                _ => {
                    bot::decide(&mut run, Policy::Heuristic, &mut rng);
                }
            }
        }
        let w = run.world.as_ref().unwrap();
        let now: Vec<f32> = w.spec_tally.iter().map(|t| t.damage).collect();
        let then = history.get(history.len().saturating_sub(20)).cloned().unwrap_or_default();
        let mut blame: Vec<(String, f32)> = now.iter().enumerate().map(|(i, d)| (w.setup.specs[i].name.clone(), d - then.get(i).copied().unwrap_or(0.0))).filter(|b| b.1 > 0.5).collect();
        blame.sort_by(|a, b| b.1.total_cmp(&a.1));
        let unowned = w.tally.damage_taken - w.spec_tally.iter().map(|t| t.damage).sum::<f32>();
        println!(
            "run {seed} ({}) {:.1} min on {}, boss out: {:?}, Mental Health {:.0}: {} | not from slop over the run: {unowned:.0}",
            hero.def().name,
            run.minutes(),
            run.platform.name(),
            run.boss_out,
            w.player.stats.max_mental_health,
            blame.iter().take(5).map(|(n, d)| format!("{n} {d:.0}")).collect::<Vec<_>>().join(", ")
        );
    }
}
