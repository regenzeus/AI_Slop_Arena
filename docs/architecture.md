# Architecture

AI Slop Arena follows the split that worked for Lineage: a deterministic simulation with no
engine code, the rules of a run on top of it, headless tools, and a Bevy client that only
draws and takes input.

```text
slop_lab ───┐
            ├──> slop_rules (Run: 20 waves, shop, items, heroes, levels, training)
slop_game ──┘         │
                      └──> slop_sim (World::step, one tick at a time)
```

## slop_sim: one wave

A `World` is one wave: you, the slop, projectiles, Clout coins, zones and an optional boss.
It is built from a `WaveSetup` (who spawns, how fast, the platform, the boss) and your
`Loadout` (weapons with tiers, stats, perks), both made by the rules.

| Module | Responsibility |
| --- | --- |
| `world` | State, spawning from the spawn table, deaths, coins, the end of a wave, checksum. |
| `spec` | `SlopSpec`: look, movement, attack, numbers and trait flags of one kind of slop. Fixed characters, generated brainrot and bosses all become specs. |
| `ai` | Slop movement, attacks and traits; contact damage and grabs. |
| `combat` | All 22 weapon patterns, projectiles, the damage formula, zones. |
| `player` | Your movement, status effects, getting hurt, actives. |
| `boss` | The seven bosses and their two phases. |
| `hazards` | Each platform's hazard and the ClockTok beat. |
| `content` | Weapons, archetypes, brainrot parts, bosses, platforms and lines, as data. |
| `bot` | A movement bot that fights at its weapons' range, dodges and collects coins. |
| `stats`, `events`, `spatial`, `rng`, `math`, `vec` | Stats, events for the client, a uniform grid, PCG32 random numbers, libm maths, a 2D vector. |

### One tick

1. Move you (input, dash, banners, downvote zones); statuses and item side effects.
2. Turn spawn warnings into slop and queue new warnings from the spawn budget, where every
   spawn costs its base HP.
3. Rebuild the spatial grid; platform hazards; the boss thinks; every slop thinks.
4. Separate crowds; contact damage and grabs; actives; weapons attack.
5. Projectiles move and hit; zones tick; queued slop (splits, hatchlings) is placed.
6. The dead are removed: coins, explosions, reposts, scattering packs.
7. Coins fly to you; timers count down; the wave ends on Burnout, on time, or with the boss.

### Determinism

Randomness comes from three PCG32 streams (spawning, combat, slop behaviour);
transcendental functions go through `libm`. A seed plus one input per tick replays a wave,
and `slop_lab determinism` replays six waves of a run with the bots and compares checksums.

## slop_rules: the run

| Module | Responsibility |
| --- | --- |
| `run` | The run's state machine: wave, level-ups, patch notes, shop, over. Clout, interest, XP. |
| `waves` | The 20-wave curve, spawn tables per platform with house rules, bosses, viral waves, elites. |
| `training` | The base model's zero-sum resistances, the brainrot genome pool, selection, mutation, names, patch notes, Model Collapse. |
| `shop` | Offers, tier odds, prices, rerolls. |
| `items`, `heroes`, `levels` | Items as data with modifiers, the five heroes, level-up offers. |
| `bot` | Heuristic and random policies that make every choice between waves. |

Between waves the run books the wave's tally: Likes, the Creator Fund payout and XP, then a training run that moves
the base model's resistances towards your kill share and breeds the brainrot that hurt you.
The next wave's specs are built from the trained model and genome pool.

## slop_game: the client

| Module | Responsibility |
| --- | --- |
| `state` | Screens, the run, and `UiAction`: everything a button or key can ask for. |
| `play` | Keys to input and shortcuts; steps the wave at 60 ticks per second; bot decisions in autoplay; the demo run behind the menus. |
| `views` | Camera, platform floor, and one view per simulation object, spawned and despawned by id. Brainrot is stacked from three part sprites. Your weapons are drawn around you and aim, kick, swing and throw from `Attacked` events. |
| `fx` | Effects from events: numbers, deletions, names, pops, an effect per weapon, trails, kill streaks, level-ups, statuses. |
| `hud` | Mental Health, statuses, wave timer or boss bar, Clout, level with level-ups earned this wave, banners, ads, pop-ups. |
| `screens` | Title, hero select, briefing, level-up, shop (with the patch notes), pause and end screens, rebuilt on change. |
| `ui` | Keyboard and mouse control of every screen: focus by stable key, arrow navigation by on-screen position, focus ring, tooltips. |
| `art`, `draw`, `icons` | A signed-distance rasteriser and every picture and icon, drawn at startup. |
| `audio` | Sounds synthesised into WAV clips at startup. |
| `autoplay` | The bots play a whole run and the client saves screenshots. |

Effects, views and the HUD only read the simulation; only `play` and `state` change it.
