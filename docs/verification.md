# Verification: milestone 1

Checked on 2026-11-02 with Rust 1.88 and Bevy 0.17.3 on Windows 10.

## Summary

| Check | Result |
| --- | --- |
| `cargo clippy --workspace --all-targets -- -D warnings` | Clean |
| `cargo test --workspace` | 33 passed, 0 failed |
| `slop_lab determinism` | Seed 42 replayed twice: identical checksum |
| `slop_lab bench` | 400 slop, up to 355 projectiles: 0.28 ms per tick (budget 2 ms) |
| `slop_lab wave 200` | Bot clears wave 1 in 82% of runs (target 70% to 90%) |
| Autoplay screenshots | Title, wave at 8, 30 and 51 seconds, and the end screen all render as intended |

Requirement-to-test mapping: [traceability.md](traceability.md).

## Wave 1 with the bot, 200 seeds

| Measure | Value |
| --- | --- |
| Cleared | 82% |
| Mental Health left when cleared | 71 on average |
| Burnout happens at | 55 s on average, in the final rush |
| Slop deleted | 195 |
| Clout earned | 148 |
| Damage taken | 42 |
| Bubbles popped | 70 |
| Damage by type | Analog 35%, Sarcasm 28%, Grass 26%, Fact 11% |

The bot only moves; it never toggles weapons. Wave 1 currently gives you all six weapons
at tier I, which will change in milestone 2 when heroes start with one or two.

### Tuning done during the milestone

- The Garden Hose first sprayed where you walk. While kiting that is away from the slop,
  and it did 3% of all damage. It now sprays at the nearest slop and does about a quarter.
- With all six weapons the first version was too easy (96% cleared, 73 Mental Health left).
  The spawn rate now ramps from 1.7 to 7.3 spawns per second.
- The first autoplay run showed walls of overlapping speech bubbles. Only the middle bubble
  of each fan now carries a line, Bubble Bots fire a little less often, and Listicle packs
  cost more of the spawn budget.
- The window grew past 1600 x 900 during one run and the camera showed empty space. The
  camera now always shows at least 1600 x 900 of the arena, and the HUD scales with the
  window.

## Demonstrated in the client

From autoplay screenshots (`slop_game --autoplay --seed 3 --screenshots shots --exit-on-end`):

| Requirement | What the screenshots show |
| --- | --- |
| UI-02 | The camera follows you and stops at the walls; the dark wall shows at the arena edge. |
| UI-03 | Red health bars over recently hit slop only. |
| UI-04 | Damage numbers in red (Analog), blue (Fact), yellow (Sarcasm) and green (Grass). |
| UI-05 | Mental Health, the red wave progress bar, Clout with a heart, and the six weapon chips. |
| FAIR-02 | Pink "typing..." spawn warnings and the Lancers' pink aim lines. |
| FAIR-04 | Only bubbles, em-dashes, aim lines and spawn warnings carry hot pink. |
| SLOP-01 | Bubble Bots firing fans of three; one bubble per fan carries a line, the others "...". |
| SLOP-03 | Numbered Listicle packs. |
| ART-01 | Outlined, anti-aliased sprites for you, all five slop, projectiles and Likes. |

## Not yet verified by a person

- How moving, hitting and dodging **feel**: that is the milestone 1 gate, and it needs you
  to play.
- Keyboard control in a real session (autoplay steers with the bot).
- The sounds, which autoplay cannot listen to.
- Pausing, switching weapons with 1 to 6, and restarting with Space.
