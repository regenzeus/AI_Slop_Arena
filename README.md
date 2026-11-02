# AI Slop Arena

A roguelike bullet heaven about one ordinary human against endless waves of AI slop.
You move; your weapons fire on their own. Mental Health is your HP.

The full design and the implementation plan live in the
[design document](https://claude.ai/artifact/HArLkFquUixzNAmQXpZDuZ).

![A wave in progress](docs/img/wave.png)

## Status: milestone 1, the arena

One 60-second wave you can play, with:

- **Six weapons**, one of each pattern and all four damage types: Rock, Stick, Pistol
  (Analog), Red Pen (Fact, marks slop with "[citation needed]"), Eye-Roll (Sarcasm,
  ricochets), Garden Hose (Grass, a spray that pushes back).
- **Five slop:** Bubble Bot (fires fans of speech bubbles), Em-Dash Lancer (aims, then
  throws an em-dash), Listicle (numbered packs; kill number 1 first and the rest scatter),
  Yes-Bot (agrees with nearby slop and buffs them), Six-Finger Hand (grabs and slows you).
- Health bars after the first hit, damage numbers in the colour of their type, popping
  bubbles, spawn and aim warnings, screen shake, generated sound effects.
- Every sprite and sound is generated in code; the game ships no asset files.

Coming next (milestone 2): 20 waves with a shop after each, items, levels and heroes.

## Play

```bash
cargo run --release -p slop_game
```

| Key | Action |
| --- | --- |
| WASD or arrows | Move |
| 1 to 6 | Switch a weapon on or off (for testing how each feels) |
| P or Esc | Pause |
| Space | Start, or try again after a wave |
| Q | Quit (from the title, end or pause screen) |

The window can be resized; the arena and HUD scale with it.

## Tools

```bash
cargo run --release -p slop_game -- --autoplay --screenshots shots --exit-on-end
cargo run --release -p slop_lab -- bench
cargo run --release -p slop_lab -- determinism
cargo run --release -p slop_lab -- wave 200
sh tools/check.sh
```

- `--autoplay` lets the bot play a wave and saves screenshots of the title, the wave and
  the end screen. `--seed N` picks the wave.
- `slop_lab bench` times 400 slop and 300 bubbles; `determinism` replays a wave twice;
  `wave` has the bot play wave 1 on many seeds and prints how it went.
- `tools/check.sh` runs clippy and every test, and regenerates the traceability table.

## Layout

| Crate | What it is |
| --- | --- |
| `slop_sim` | The deterministic arena simulation: movement, weapons, slop, damage, waves. No engine code. |
| `slop_lab` | Headless tools: benchmark, determinism check, wave statistics. |
| `slop_game` | The Bevy client: rendering, input, HUD, effects, generated art and sound. |

`slop_rules` (the run, shop, items and heroes) arrives with milestone 2.

## Docs

- [Requirements](docs/requirements.md): what milestone 1 must do, numbered.
- [Architecture](docs/architecture.md): how the crates fit together.
- [Verification](docs/verification.md): how each requirement was checked, with results.
- [Traceability](docs/traceability.md): which test covers which requirement.
