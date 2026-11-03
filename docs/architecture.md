# Architecture

AI Slop Arena follows the split that worked for Lineage: a deterministic simulation with no
engine code, the rules of a run on top of it, headless tools, and a Bevy client that only
draws and takes input. Version 3 made the run endless: one `World` lives for the whole run.
Version 4 added dashes, defence, legendaries, glitches, biomes and the director's events.

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
| `world` | State, spawning around you, moving stragglers back in front, walls, deaths and drops, Likes and pickups (the vacuum), using props, the biome you are in, swarms, rings and livestream arenas, checksum. |
| `map` | Chunks made from the seed and their coordinates: walls, maintenance fences, props; biome squares; collision and line-of-sight queries. |
| `spec` | `SlopSpec`: look, movement, attack, numbers and trait flags of one kind of slop, including elite affixes. |
| `ai` | Slop movement (rush, kite, charge, blink, sweep...), attacks (bubbles, bombs, bonks, needles, stomps, peels) and traits; damage-type effects on movement; contact damage, grabs and thorns. |
| `combat` | All weapon patterns with levels, barriers, projectiles, the damage formula with damage-type effects, legendary and glitch effects, zones (fire and brainrot zones too). |
| `player` | Your movement and dash, trails, status effects, Boundaries and getting hurt, timed defence items, actives. |
| `boss` | The seven bosses and their two phases; a defeated boss drops a chest. |
| `hazards` | Each platform's hazard in shapes, the ClockTok beat, maintenance. |
| `content` | Weapons, merges, archetypes, brainrot parts, bosses, platforms and lines, as data. |
| `bot` | A movement bot that fights at its weapons' range, dodges, avoids walls and collects. |
| `stats`, `events`, `spatial`, `rng`, `math`, `vec` | Stats, events for the client, a grid that follows you, PCG32 random numbers, libm maths, a 2D vector. |

### One tick

1. Remember positions for drawing; make the map around you; fences count down; note the
   biome you are in; the livestream counts down.
2. Move you and dash; statuses, Boundaries and timed items.
3. Turn spawn warnings into slop and queue new ones from the spawn budget; move stragglers
   back in front of you.
4. Rebuild the grid around you; hazards; the boss and every slop think (unless frozen).
5. Separate crowds; push everyone out of walls (and keep you in a livestream); contact
   damage and thorns; actives; dash hits and burning trails; weapons attack, with echoes
   and the Alt Account.
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
| `run` | The run: platforms and bosses, the biome you are in, the director's events, experience and level-ups, valid choices with rarities, rerolls, chests and merges, glitches, vending, gacha, shrines, livestreams, per-minute income and Hustle, retraining, syncing your loadout into the world. |
| `waves` | The director: difficulty by minute, spawn tables per platform with house rules, elites with affixes, bosses for the time they arrive. |
| `training` | The base model's zero-sum resistances, the brainrot genome pool, selection, mutation, names, patch notes, Model Collapse. |
| `items`, `heroes`, `levels` | Items (with levels and rarities), blessings and glitches as data with modifiers, the eight heroes and their rules, the level curve and stat-ups. |
| `bot` | Heuristic and random policies for every choice the Feed asks for. |

## slop_game: the client

| Module | Responsibility |
| --- | --- |
| `controls` | One input layer: keyboard and gamepads into movement, menu directions and presses. |
| `state` | Screens, the run, and `UiAction`: everything a button or key can ask for. |
| `play` | Steps the Feed at 60 ticks per second and records how far between ticks the frame is; shortcuts; bot decisions in autoplay; the demo run behind the menus; toasts. |
| `views` | The camera locked on you; a floor per biome square with seams; one view per simulation object, drawn between ticks, slop animated; barriers, the Alt Account and the livestream ring; walls, fences, props with rings and meters, pickups with coin sizes and light pillars, and feed scraps per chunk. |
| `fx` | Effects from events: numbers, deletions, names, pops, an effect per weapon, trails, kill streaks, level-ups, pickups, props, statuses. |
| `hud` | Mental Health and Boundaries, dash charges, statuses and why your controls are reversed, the run clock with the biome and the boss countdown or its health, Clout, level and kills, the experience bar, the livestream clock, arrows to chests off screen, toasts, banners, ads, pop-ups. |
| `screens` | Title, hero select, and over the Feed: level-up, chest (the slot-reel reveal) and merge, glitches, vending, shrine, pause and end screens, rebuilt on change. |
| `ui` | Focus by stable key, arrow navigation by on-screen position, focus ring, tooltips or a details panel. |
| `display` | Fullscreen at start, Alt+Enter or F11 to switch to a window, F12 or Print Screen screenshots. |
| `art`, `draw`, `icons`, `feed_art` | A signed-distance rasteriser and every picture and icon, drawn at startup. |
| `audio` | Sounds synthesised into WAV clips at startup. |
| `autoplay` | The bots play the Feed and the client saves screenshots. |

Effects, views and the HUD only read the simulation; only `play` and `state` change it.
