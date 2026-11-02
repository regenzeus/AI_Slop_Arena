# Architecture

AI Slop Arena follows the split that worked for Lineage: a deterministic simulation with no
engine code, headless tools on top of it, and a thin Bevy client that only draws the
simulation and feeds it input.

```text
slop_lab ──┐
           ├──> slop_sim  (World::step, one tick at a time)
slop_game ─┘
```

`slop_rules` joins in milestone 2 and owns everything between waves: the run, the shop,
items, levels and heroes. The simulation will stay unaware of all of that; a wave is
started from a `WaveConfig` and the player's loadout.

## slop_sim

| Module | Responsibility |
| --- | --- |
| `world` | `World`: you, the slop, projectiles, Likes, spawn warnings, and `step(input)`. |
| `content` | Weapons, slop and their lines as data tables. |
| `stats` | The ten stats, the four damage types, Composure and Scepticism formulas. |
| `bot` | A movement bot: kites, dodges bubbles, avoids walls, collects Likes. |
| `events` | What happened in a tick, for the client's effects and sounds. |
| `spatial` | A uniform grid over the arena for neighbour queries. |
| `rng`, `math`, `vec` | PCG32 random numbers, libm maths and a small 2D vector, all deterministic. |

### One tick

`World::step` runs these stages in order:

1. Move you, easing towards the input direction; heal by Self-Care.
2. Count down spawn warnings into slop, and queue new warnings from the spawn budget.
3. Rebuild the spatial grid.
4. Each slop thinks and moves: Listicles rush or scatter, Bubble Bots keep their distance
   and fire, Lancers aim and throw, Yes-Bots follow and agree, Hands chase or hold on.
5. Separate overlapping slop.
6. Contact damage and grabs.
7. Weapons whose cooldown is up attack the nearest slop in range.
8. Projectiles move, hit, pierce, bounce and pop bubbles.
9. Dead slop are removed and drop Likes; Listicle packs scatter.
10. Likes within Curiosity fly to you.
11. Timers count down; the wave ends on Burnout or when its time is up.

### Determinism

Randomness comes from three PCG32 streams (spawning, combat, slop behaviour), so adding a
roll to one system does not shift the others. Transcendental functions go through `libm`.
`World::checksum` fingerprints the state; the lab and a test replay a wave twice and
compare. The client sends one input per tick, so a seed plus the input log replays a wave.

Ids are stable: every slop, projectile and Like keeps its id for life, which is how the
client matches its sprites to the simulation.

## slop_game

| Module | Responsibility |
| --- | --- |
| `play` | Keys to input; steps the simulation at 60 ticks per second; collects the events. |
| `views` | Camera, floor and walls; one view per simulation object, spawned and despawned by id. |
| `fx` | Effects from events: damage numbers, deletions, pops, swings, warnings, shake. |
| `hud` | Mental Health, wave timer, Clout, weapons; title, pause and end screens; UI scaling. |
| `art` | A small signed-distance rasteriser that draws every sprite at startup. |
| `audio` | Sound effects synthesised into WAV clips at startup, rate-limited when played. |
| `autoplay` | The bot plays and the client saves screenshots, for checking without a human. |

Systems run in a fixed order each frame: `Simulation` (keys, step) first, then views,
effects, HUD and sound, which only read the simulation. Effects never change the wave.

The camera uses an `AutoMin` projection: every window shows at least 1600 x 900 of the
arena, and the HUD's `UiScale` follows the window size, so a larger window shows the same
game, larger.
