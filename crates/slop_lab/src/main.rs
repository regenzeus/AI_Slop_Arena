//! Headless tools for AI Slop Arena.
//!
//! ```text
//! slop_lab bench                 time a crowded arena (SIM-04)
//! slop_lab determinism           replay a wave twice and compare (SIM-01)
//! slop_lab wave [runs]           the bot plays wave 1 on many seeds (BAL-01)
//! ```

use std::time::Instant;

use slop_sim::bot;
use slop_sim::content::{SlopKind, WEAPONS};
use slop_sim::world::{Outcome, Projectile, WaveConfig, Weapon, World, BUBBLE_RADIUS};
use slop_sim::{DamageType, Input, Look, V2};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("bench") => bench(),
        Some("determinism") => determinism(),
        Some("wave") => wave(args.get(1).and_then(|n| n.parse().ok()).unwrap_or(100)),
        _ => eprintln!("usage: slop_lab <bench|determinism|wave [runs]>"),
    }
}

/// 400 slop, 300 bubbles and six fast-firing weapons around you.
fn bench() {
    let mut w = World::sandbox(1);
    w.player.weapons = (0..WEAPONS.len()).map(|d| Weapon::new(d, 4)).collect();
    w.player.stats.focus = 3.0;
    w.player.stats.max_mental_health = 1e9;
    w.player.mental_health = 1e9;
    let centre = w.player.pos;
    for i in 0..400 {
        let kind = SlopKind::ALL[i % SlopKind::ALL.len()];
        let at = centre + V2::from_angle(i as f32 * 0.37) * (150.0 + (i % 40) as f32 * 14.0);
        w.spawn_slop(kind, at);
    }
    for s in &mut w.slop {
        s.hp = 1e9;
        s.max_hp = 1e9;
    }
    for i in 0..300 {
        let at = centre + V2::from_angle(i as f32 * 0.21) * 500.0;
        w.projectiles.push(Projectile {
            id: 1_000_000 + i,
            pos: at,
            vel: (centre - at).normalized() * 40.0,
            radius: BUBBLE_RADIUS,
            damage: 0.0,
            kind: DamageType::Analog,
            hostile: true,
            pierce: 0,
            bounces: 0,
            mark: None,
            look: Look::Bubble,
            life: u16::MAX,
            line: 0,
            last_hit: 0,
            knockback: 0.0,
        });
    }
    let ticks = 600;
    let start = Instant::now();
    let mut peak_projectiles = 0;
    for _ in 0..ticks {
        w.step(Input { x: 0.3, y: 0.1 });
        w.drain_events();
        peak_projectiles = peak_projectiles.max(w.projectiles.len());
    }
    let per_tick = start.elapsed().as_secs_f64() * 1000.0 / ticks as f64;
    println!(
        "{} slop, up to {} projectiles: {:.3} ms per tick (budget 2 ms) {}",
        w.slop.len(),
        peak_projectiles,
        per_tick,
        if per_tick < 2.0 { "OK" } else { "OVER BUDGET" }
    );
}

fn play(seed: u64) -> World {
    let mut w = World::new(seed, WaveConfig::first());
    while w.outcome == Outcome::Running {
        let input = bot::steer(&w);
        w.step(input);
    }
    w
}

fn determinism() {
    let (a, b) = (play(42), play(42));
    let same = a.checksum() == b.checksum() && a.tick == b.tick;
    println!("seed 42 twice: {:016x} vs {:016x} -> {}", a.checksum(), b.checksum(), if same { "identical" } else { "DIFFERENT" });
    if !same {
        std::process::exit(1);
    }
}

fn wave(runs: u64) {
    let mut cleared = 0;
    let (mut kills, mut clout, mut left, mut taken, mut popped) = (0u64, 0u64, 0.0f64, 0.0f64, 0u64);
    let mut dealt = [0.0f64; 4];
    let mut death_times = Vec::new();
    for seed in 1..=runs {
        let w = play(seed);
        kills += w.tally.kills as u64;
        clout += w.clout as u64;
        taken += w.tally.damage_taken as f64;
        popped += w.tally.bubbles_popped as u64;
        for (d, x) in dealt.iter_mut().zip(w.tally.damage_dealt) {
            *d += x as f64;
        }
        if w.outcome == Outcome::Cleared {
            cleared += 1;
            left += w.player.mental_health as f64;
        } else {
            death_times.push(w.elapsed());
        }
    }
    let n = runs as f64;
    println!("wave 1, {runs} runs with the bot");
    println!("  cleared            {:.0}%", 100.0 * cleared as f64 / n);
    if cleared > 0 {
        println!("  Mental Health left {:.0} on average when cleared", left / cleared as f64);
    }
    if !death_times.is_empty() {
        let avg = death_times.iter().sum::<f32>() / death_times.len() as f32;
        println!("  burnout at         {avg:.0} s on average");
    }
    println!("  kills              {:.0}", kills as f64 / n);
    println!("  clout              {:.0}", clout as f64 / n);
    println!("  damage taken       {:.0}", taken / n);
    println!("  bubbles popped     {:.0}", popped as f64 / n);
    let total: f64 = dealt.iter().sum();
    let shares: Vec<String> =
        DamageType::ALL.iter().zip(dealt).map(|(t, d)| format!("{} {:.0}%", t.name(), 100.0 * d / total.max(1.0))).collect();
    println!("  damage by type     {}", shares.join(", "));
}
