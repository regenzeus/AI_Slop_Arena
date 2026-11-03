# Architecture

AI Slop Arena follows the split that worked for Lineage: a deterministic simulation with no
engine code, the rules of a run on top of it, headless tools, and a Bevy client that only
draws and takes input. Version 3 made the run endless: one `World` lives for the whole run.

```text
slop_lab ───┐
            ├──> slop_rules (Run: the director, level-ups, chests, merges, machines, training)
slop_game ──┘         │
                      └──> slop_sim (World::step, one tick at a time, on an endless map)
```

## slop_sim: the Feed

A `World` is the whole run: you, the slop, projectiles, Likes, pickups, zones, an optional
boss, and the endless `Map`. It starts from a `WaveSetup` and your `Loadout`; the rules
then retune what spawns (`retune`), bring in bosses (`summon_boss`) and elites (`summon`),
and change your loadout in place.

| Module | Responsibility |
| --- | --- |
| `world` | State, spawning around you, moving stragglers back in front, walls, deaths and drops, Likes and pickups, using props, checksum. |
| `map` | Chunks made from the seed and their coordinates: walls, maintenance fences, props; collision and line-of-sight queries. |
| `spec` | `SlopSpec`: look, movement, attack, numbers and trait flags of one kind of slop, including elite affixes. |
| `ai` | Slop movement, attacks and traits; damage-type effects on movement; contact damage and grabs. |
| `combat` | All weapon patterns with levels, projectiles, the damage formula with damage-type effects, zones. |
| `player` | Your movement, status effects, getting hurt, actives. |
| `boss` | The seven bosses and their two phases; a defeated boss drops a chest. |
| `hazards` | Each platform's hazard in shapes, the ClockTok beat, maintenance. |
| `content` | Weapons, merges, archetypes, brainrot parts, bosses, platforms and lines, as data. |
| `bot` | A movement bot that fights at its weapons' range, dodges, avoids walls and collects. |
| `stats`, `events`, `spatial`, `rng`, `math`, `vec` | Stats, events for the client, a grid that follows you, PCG32 random numbers, libm maths, a 2D vector. |

### One tick

1. Remember positions for drawing; make the map around you; fences count down.
2. Move you; statuses and item side effects.
3. Turn spawn warnings into slop and queue new ones from the spawn budget; move stragglers
   back in front of you.
4. Rebuild the grid around you; hazards; the boss and every slop think (unless frozen).
5. Separate crowds; push everyone out of walls; contact damage; actives; weapons attack.
6. Projectiles move, hit, and stop at walls; zones tick; queued slop is placed.
7. The dead are removed: Likes, drops, chests, explosions, reposts, viral copies.
8. Likes and pickups fly to you; props are used; timers; Burnout ends the run.

### Determinism

Randomness comes from three PCG32 streams (spawning, combat, slop behaviour) and one per
chunk of map; transcendental functions go through `libm`. A seed plus one input per tick
replays a run, and `slop_lab determinism` replays six minutes with the bots and compares
checksums.

## slop_rules: the run

| Module | Responsibility |
| --- | --- |
| `run` | The run: platforms and bosses, experience and level-ups, valid choices, rerolls, chests and merges, vending, gacha, shrines, comment sections, per-minute income, retraining, syncing your loadout into the world. |
| `waves` | The director: difficulty by minute, spawn tables per platform with house rules, elites with affixes, bosses for the time they arrive. |
| `training` | The base model's zero-sum resistances, the brainrot genome pool, selection, mutation, names, patch notes, Model Collapse. |
| `items`, `heroes`, `levels` | Items (with levels) and blessings as data with modifiers, the five heroes, the level curve and stat-ups. |
| `bot` | Heuristic and random policies for every choice the Feed asks for. |

## slop_game: the client

| Module | Responsibility |
| --- | --- |
| `controls` | One input layer: keyboard and gamepads into movement, menu directions and presses. |
| `state` | Screens, the run, and `UiAction`: everything a button or key can ask for. |
| `play` | Steps the Feed at 60 ticks per second and records how far between ticks the frame is; shortcuts; bot decisions in autoplay; the demo run behind the menus; toasts. |
| `views` | The camera locked on you; the endless floor; one view per simulation object, drawn between ticks; walls, fences, props with rings and meters, pickups, and feed scraps per chunk. |
| `fx` | Effects from events: numbers, deletions, names, pops, an effect per weapon, trails, kill streaks, level-ups, pickups, props, statuses. |
| `hud` | Mental Health, statuses, the run clock with the boss countdown or its health, Clout, level and kills, the experience bar, toasts, banners, ads, pop-ups. |
| `screens` | Title, hero select, and over the Feed: level-up, chest and merge, vending, shrine, pause and end screens, rebuilt on change. |
| `ui` | Focus by stable key, arrow navigation by on-screen position, focus ring, tooltips or a details panel. |
| `display` | Fullscreen at start, Alt+Enter or F11 to switch to a window, F12 screenshots. |
| `art`, `draw`, `icons`, `feed_art` | A signed-distance rasteriser and every picture and icon, drawn at startup. |
| `audio` | Sounds synthesised into WAV clips at startup. |
| `autoplay` | The bots play the Feed and the client saves screenshots. |

Effects, views and the HUD only read the simulation; only `play` and `state` change it.
