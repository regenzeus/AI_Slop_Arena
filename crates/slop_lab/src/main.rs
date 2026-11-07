//! Headless tools for AI Slop Arena.
//!
//! ```text
//! slop_lab bench                  time a crowded Feed (SIM-04)
//! slop_lab determinism            replay a run twice and compare (SIM-01)
//! slop_lab balance [runs] [min]   bots play whole runs; minutes survived, bosses (BAL-02)
//! slop_lab progress [runs]        level, kills, slop and weapons minute by minute
//! slop_lab deaths [runs]          what hurt the heuristic bot most in its last 20 seconds
//! slop_lab playtest [runs]        the stand-in player: past playtest complaints, measured, PASS or FAIL
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
        Some("balance") => balance(n(1, 30), n(2, 45) as f32, slop_rules::memory::Temperature::ALL[(n(3, 1) as usize).min(3)]),
        Some("progress") => progress(n(1, 8)),
        Some("deaths") => deaths(n(1, 8)),
        Some("story") => story(n(1, 6), slop_rules::memory::Temperature::ALL[(n(2, 1) as usize).min(3)]),
        Some("playtest") => playtest(n(1, 4)),
        Some("ledger") => ledger(n(1, 1), if n(2, 0) == 1 { Policy::Random } else { Policy::Luddite }),
        _ => eprintln!("usage: slop_lab <bench|determinism|balance [runs] [minutes] [temperature 0-3]|story [runs] [temperature]|progress [runs]>"),
    }
}

/// 600 slop, 300 bubbles and six max-level weapons firing fast around you.
fn bench() {
    let picks = ["Rock", "Stapler", "Library Book", "Eye-Roll", "Slow Clap", "Seed Bomb"];
    let weapons: Vec<(usize, u8)> = picks.iter().map(|p| (slop_sim::content::weapon_index(p).unwrap(), MAX_LEVEL)).collect();
    let mut loadout = Loadout { weapons, agents: Vec::new(), stats: Default::default(), perks: Default::default() };
    loadout.stats.focus = 3.0;
    loadout.stats.max_mental_health = 1e9;
    let mut w = World::new(1, WaveSetup::sandbox(), &loadout);
    let centre = w.player.pos;
    for i in 0..600 {
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

fn balance(runs: u64, limit: f32, temperature: slop_rules::memory::Temperature) {
    println!("Temperature {}", temperature.label());
    let start = Instant::now();
    for policy in [Policy::Heuristic, Policy::Random] {
        let mut minutes = Vec::new();
        let mut bosses: BTreeMap<u32, u32> = BTreeMap::new();
        let mut by_hero: BTreeMap<&str, Vec<f32>> = BTreeMap::new();
        let mut model = 0;
        let mut weapon_use: BTreeMap<String, u32> = BTreeMap::new();
        for seed in 1..=runs {
            let hero = Hero::ALL[(seed % Hero::ALL.len() as u64) as usize];
            let s = bot::play_at(seed, hero, policy, limit, temperature);
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
                Phase::Playing | Phase::Takeover { .. } => {
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
                Phase::Playing | Phase::Takeover { .. } => {
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

/// Whole runs, chapters and all (CH-01, AUTO-01, END-01): how long each
/// chapter lasts, where Automation ends, what deleted you, for a bot that
/// always takes the easy option, one that never does, and a random one.
fn story(runs: u64, temperature: slop_rules::memory::Temperature) {
    let start = Instant::now();
    println!("Temperature {}", temperature.label());
    for policy in [Policy::Heuristic, Policy::Luddite, Policy::Random] {
        let mut per_chapter: BTreeMap<usize, Vec<f32>> = BTreeMap::new();
        let mut autos = Vec::new();
        let mut scores = Vec::new();
        let mut reached: BTreeMap<usize, u32> = BTreeMap::new();
        let mut killers: BTreeMap<String, u32> = BTreeMap::new();
        for seed in 1..=runs {
            let hero = Hero::ALL[(seed % Hero::ALL.len() as u64) as usize];
            let run = bot::play_run(seed, hero, policy, 90.0, temperature);
            for c in &run.chapters {
                per_chapter.entry(c.chapter.number()).or_default().push(c.seconds / 60.0);
                if let Some((name, _)) = &c.killer {
                    *killers.entry(name.clone()).or_default() += 1;
                }
            }
            *reached.entry(run.chapter.number()).or_default() += 1;
            autos.push(run.automation);
            scores.push(run.score);
        }
        let k = runs as f32;
        println!("\n{policy:?} bot, {runs} runs:");
        for (ch, mins) in &per_chapter {
            let mean = mins.iter().sum::<f32>() / mins.len() as f32;
            println!("  chapter {ch}: {} runs, mean {mean:.1} min", mins.len());
        }
        println!("  furthest chapter reached: {reached:?}");
        println!("  automation at the end: mean {:.0}%, max {}%", autos.iter().sum::<u32>() as f32 / k, autos.iter().max().unwrap_or(&0));
        println!("  score: mean {:.0}", scores.iter().sum::<u32>() as f32 / k);
        let mut top: Vec<(String, u32)> = killers.into_iter().collect();
        top.sort_by(|a, b| b.1.cmp(&a.1));
        let top: Vec<String> = top.into_iter().take(6).map(|(n, c)| format!("{n} {c}")).collect();
        println!("  deleted by: {}", top.join(", "));
    }
    println!("\n{} s", start.elapsed().as_secs());
}

/// What one run felt like, measured the way a player would notice it.
#[derive(Default)]
struct Feel {
    minutes: f32,
    /// Times the game stopped for a choice, by kind.
    stops: BTreeMap<&'static str, u32>,
    chests_on_floor: usize,
    pickups_on_floor: usize,
    /// Slop inside one screen around you: sum over samples, and peak.
    slop_seen: (f64, usize),
    shots_seen: usize,
    samples: u32,
    /// Per chapter: minutes, what ended it, level and unspent Clout at the end.
    chapters: Vec<(f32, String, u32, u32)>,
    /// Chapter ends by a slop where you were above 70% three seconds before.
    spikes: u32,
    by_slop: u32,
}

/// Half of one screen around you, in world units (UI-07).
const SCREEN: V2 = V2 { x: 720.0, y: 405.0 };

/// Plays one run and measures it (`slop_lab playtest`).
fn feel_run(seed: u64, hero: Hero, policy: Policy, temperature: slop_rules::memory::Temperature) -> Feel {
    let mut run = Run::with(seed, hero, temperature, slop_rules::memory::Memory::default());
    let mut rng = slop_sim::rng::Rng::new(seed, 11);
    let mut f = Feel::default();
    // Health share one, two and three seconds ago.
    let mut health = [1.0f32; 3];
    let mut was_playing = true;
    let mut chapters_seen = 0;
    loop {
        match &run.phase {
            Phase::Over { .. } => break,
            Phase::Playing | Phase::Takeover { .. } => {
                if run.minutes() >= 90.0 {
                    break;
                }
                was_playing = true;
                let input = slop_sim::bot::steer(run.world.as_ref().expect("the Feed runs"));
                run.step(input);
                if let Some(w) = run.world.as_ref() {
                    if w.tick % 60 == 0 && matches!(run.phase, Phase::Playing) {
                        let me = w.player.pos;
                        let near = |p: V2| (p.x - me.x).abs() < SCREEN.x && (p.y - me.y).abs() < SCREEN.y;
                        let seen = w.slop.iter().filter(|s| near(s.pos)).count();
                        f.slop_seen.0 += seen as f64;
                        f.slop_seen.1 = f.slop_seen.1.max(seen);
                        f.shots_seen = f.shots_seen.max(w.projectiles.iter().filter(|p| p.hostile && near(p.pos)).count());
                        f.chests_on_floor = f.chests_on_floor.max(w.pickups.iter().filter(|p| p.kind == slop_sim::world::PickupKind::Chest).count());
                        f.pickups_on_floor = f.pickups_on_floor.max(w.pickups.len());
                        f.samples += 1;
                        health = [w.player.mental_health / w.player.stats.max_mental_health.max(1.0), health[0], health[1]];
                    }
                }
            }
            phase => {
                if was_playing {
                    let kind = match phase {
                        Phase::LevelUp(_) => "level-up",
                        Phase::Chest(_) => "chest",
                        Phase::Glitch(_) => "glitch",
                        Phase::Vending { .. } => "vending",
                        Phase::Shrine { .. } => "shrine",
                        Phase::Recycle { .. } => "recycle",
                        _ => "chapter",
                    };
                    *f.stops.entry(kind).or_default() += 1;
                }
                was_playing = false;
                if run.chapters.len() > chapters_seen {
                    // A chapter just ended: what ended it, and how.
                    for c in &run.chapters[chapters_seen..] {
                        let cause = c.killer.as_ref().map_or("unknown".to_string(), |k| k.0.clone());
                        let wear = c.killer.as_ref().is_some_and(|k| k.1 == slop_rules::run::WEAR_LESSON);
                        if !wear && cause != "Automation" {
                            f.by_slop += 1;
                            if health[2] > 0.7 {
                                f.spikes += 1;
                            }
                        }
                        f.chapters.push((c.seconds / 60.0, cause, run.level, run.clout));
                    }
                    chapters_seen = run.chapters.len();
                    health = [1.0; 3];
                }
                bot::decide(&mut run, policy, &mut rng);
            }
        }
    }
    f.minutes = run.minutes();
    f
}

/// The stand-in player (`slop_lab playtest [runs]`): every complaint from the
/// human playtests, measured over whole runs and checked against a limit.
fn playtest(runs: u64) {
    let start = Instant::now();
    let temperature = slop_rules::memory::Temperature::Default;
    let mut verdicts: Vec<(bool, String)> = Vec::new();
    for policy in [Policy::Luddite, Policy::Random, Policy::Heuristic] {
        let feels: Vec<Feel> = (1..=runs).map(|seed| feel_run(seed, Hero::ALL[(seed % Hero::ALL.len() as u64) as usize], policy, temperature)).collect();
        let k = runs as f32;
        let minutes: f32 = feels.iter().map(|f| f.minutes).sum::<f32>() / k;
        let mut stops: BTreeMap<&str, f32> = BTreeMap::new();
        for f in &feels {
            for (kind, n) in &f.stops {
                *stops.entry(kind).or_default() += *n as f32 / f.minutes.max(0.1) / k;
            }
        }
        let per_minute: f32 = stops.iter().filter(|(kind, _)| **kind != "chapter").map(|(_, v)| v).sum();
        let peak = |g: &dyn Fn(&Feel) -> usize| feels.iter().map(g).max().unwrap_or(0);
        let chests = peak(&|f| f.chests_on_floor);
        let pickups = peak(&|f| f.pickups_on_floor);
        let shots = peak(&|f| f.shots_seen);
        let slop_mean = feels.iter().map(|f| f.slop_seen.0 / f.samples.max(1) as f64).sum::<f64>() / runs as f64;
        let slop_peak = peak(&|f| f.slop_seen.1);
        let ends: Vec<&(f32, String, u32, u32)> = feels.iter().flat_map(|f| f.chapters.iter()).collect();
        let max_level = ends.iter().map(|c| c.2).max().unwrap_or(0);
        let max_clout = ends.iter().map(|c| c.3).max().unwrap_or(0);
        let by_slop: u32 = feels.iter().map(|f| f.by_slop).sum();
        let spikes: u32 = feels.iter().map(|f| f.spikes).sum();
        let deaths = ends.iter().filter(|c| c.1 != "Automation").count().max(1);
        let mut causes: BTreeMap<&str, u32> = BTreeMap::new();
        for c in &ends {
            *causes.entry(c.1.as_str()).or_default() += 1;
        }
        let mut causes: Vec<_> = causes.into_iter().collect();
        causes.sort_by(|a, b| b.1.cmp(&a.1));
        println!("\n{policy:?} bot, {runs} runs, {minutes:.1} min on average");
        println!("  stops per minute: {}", stops.iter().map(|(k, v)| format!("{k} {v:.2}")).collect::<Vec<_>>().join(", "));
        println!("  peaks: {chests} chests on the floor, {pickups} pickups, {shots} hostile shots on screen, {slop_peak} slop on screen (mean {slop_mean:.0})");
        println!("  chapter ends: {by_slop} by slop, {spikes} cheap (above 70% three seconds before), highest level {max_level}, most unspent Clout {max_clout}");
        println!("  ended by: {}", causes.iter().take(6).map(|(n, c)| format!("{n} {c}")).collect::<Vec<_>>().join(", "));
        // The limits are what the human playtests asked for (memory: game-feel-feedback).
        if policy != Policy::Heuristic {
            let tag = format!("{policy:?}");
            verdicts.push((chests <= 2, format!("{tag}: at most 2 chests on the floor at once (got {chests})")));
            verdicts.push((per_minute <= 3.0, format!("{tag}: at most 3 stops a minute for choices (got {per_minute:.2})")));
            verdicts.push((pickups <= 150, format!("{tag}: at most 150 pickups lying around (got {pickups})")));
            verdicts.push((shots <= 120, format!("{tag}: at most 120 hostile shots on screen (got {shots})")));
            verdicts.push((slop_mean >= 25.0, format!("{tag}: a dense crowd, 25+ slop on screen on average (got {slop_mean:.0})")));
            verdicts.push((max_level <= 90, format!("{tag}: levels stay readable, at most 90 at a chapter end (got {max_level})")));
            verdicts.push((max_clout <= 3000, format!("{tag}: Clout stays worth spending, at most 3000 unspent (got {max_clout})")));
            verdicts.push((by_slop as f32 >= deaths as f32 * 0.6, format!("{tag}: fail and know why, 60%+ of chapter ends by a slop (got {by_slop} of {deaths})")));
            verdicts.push(((spikes as f32) <= by_slop as f32 * 0.2, format!("{tag}: no cheap deaths, at most 20% from above 70% health (got {spikes} of {by_slop})")));
        }
    }
    println!("\nVerdict, as the player:");
    for (ok, line) in &verdicts {
        println!("  {} {line}", if *ok { "PASS" } else { "FAIL" });
    }
    println!("\n{} of {} pass, {} s", verdicts.iter().filter(|v| v.0).count(), verdicts.len(), start.elapsed().as_secs());
}

/// Where the Clout and the levels come from (`slop_lab ledger [seed]`): one
/// Luddite run, minute by minute.
fn ledger(seed: u64, policy: Policy) {
    let mut run = Run::with(seed, Hero::ALL[(seed % Hero::ALL.len() as u64) as usize], slop_rules::memory::Temperature::Default, slop_rules::memory::Memory::default());
    let mut rng = slop_sim::rng::Rng::new(seed, 11);
    let mut next = 1.0;
    let mut spent_before = 0u32;
    let mut picks: BTreeMap<String, u32> = BTreeMap::new();
    let mut died: BTreeMap<String, u32> = BTreeMap::new();
    println!("min  ch  lvl   clout  coins-picked  kills   hustle  slop  items");
    loop {
        match &run.phase {
            Phase::Over { .. } => break,
            Phase::Playing | Phase::Takeover { .. } => {
                if run.minutes() >= 60.0 {
                    break;
                }
                let input = slop_sim::bot::steer(run.world.as_ref().expect("the Feed runs"));
                run.step(input);
                if let Some(w) = run.world.as_mut() {
                    for e in w.drain_events() {
                        if let slop_sim::Event::SlopDied { spec, .. } = e {
                            let name = w.setup.specs.get(spec as usize).map_or("?".to_string(), |s| s.name.clone());
                            *died.entry(name).or_default() += 1;
                        }
                    }
                }
                if run.minutes() >= next {
                    next += 1.0;
                    let w = run.world.as_ref().unwrap();
                    let items: Vec<String> = run.items.iter().map(|(i, n)| format!("{}:{n}", slop_rules::items::ITEMS[*i].name)).collect();
                    println!("{:>3} {:>3} {:>4} {:>7} {:>13} {:>6} {:>8.0} {:>5}  {}", run.minutes() as u32, run.chapter.number(), run.level, run.clout, w.clout, w.tally.kills, run.stats().hustle, w.slop.len(), items.join(", "));
                    let mut top: Vec<(String, u32)> = std::mem::take(&mut died).into_iter().collect();
                    top.sort_by(|a, b| b.1.cmp(&a.1));
                    println!("      died: {}", top.iter().take(4).map(|(n, c)| format!("{n} {c}")).collect::<Vec<_>>().join(", "));
                }
            }
            phase => {
                if let Phase::LevelUp(choices) = phase {
                    for c in choices {
                        if let slop_rules::run::Choice::Clout(n) = c {
                            *picks.entry("clout card offered".into()).or_default() += n;
                        }
                    }
                }
                let before = run.clout;
                bot::decide(&mut run, policy, &mut rng);
                if run.clout > before {
                    *picks.entry(format!("{:?}", std::mem::discriminant(&run.phase))).or_default() += run.clout - before;
                } else {
                    spent_before += before - run.clout;
                }
            }
        }
    }
    println!("Clout from choices, by the phase after: {picks:?}; spent: {spent_before}");
}
