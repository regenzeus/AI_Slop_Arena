# Verification: version 1

Checked on 2026-11-02 with Rust 1.88 and Bevy 0.17.3 on Windows 10.

## Summary

| Check | Result |
| --- | --- |
| `cargo clippy --workspace --all-targets -- -D warnings` | Clean |
| `cargo test --workspace` | 65 passed, 0 failed (39 simulation, 22 rules, 4 client) |
| `slop_lab determinism` | Six waves of a run, played twice by the bots: identical checksums over 15,300 ticks |
| `slop_lab bench` | 400 slop, up to 360 projectiles, six tier IV weapons: 0.46 ms per tick (budget 2 ms) |
| `slop_lab balance 100` | Heuristic bot wins 34% (target 20% to 40%); random bot wins 1% (target under 10%) |
| Autoplay (seed 4) | Title, hero select, waves 1 to 12, level-up, patch notes, shops and the Burnout screen render as intended |

Requirement-to-test mapping: [traceability.md](traceability.md).

## Balance: 100 runs per bot

| Measure | Heuristic bot | Random bot |
| --- | --- | --- |
| Runs won | 34% | 1% |
| Most common wave of Burnout | 10 (the platform boss) | 10 |
| Runs won by platform | LinkedOut 9/13, FaceSlop 8/20, MeTube 8/17, Thredd 5/17, InstaSlop 2/15, ClockTok 2/18 | under 1 in 10 everywhere |

The bots only move by simple rules and buy by a fixed heuristic; a person who flanks, kites
on purpose and kills the Amen Totems first should do better. Wave 10 is deliberately the
biggest check of a run.

### Tuning done in this version

- **Spawns cost their HP.** With a flat cost per spawn, a Listicle Drone pack and an Elephant
  cost about the same, and wave 1 threw about 50 slop at one starting weapon. A spawn credit
  now buys 20 base HP.
- **Wave-10 bosses at 55% HP**, and a steeper late HP curve: before, the wave-10 boss stopped
  most runs and everything after it was easy.
- **The movement bot fights at its weapons' range** and closes in when nothing is in reach;
  it used to kite melee builds into doing no damage and never walked up to the Content Farm.
- **Training pressure halved**: one wave with a single Analog weapon gave the slop +30%
  Analog resistance; it now gives about +16%.
- **Beauty filters** block 60% of frontal damage instead of 75%; **Amen Totems** heal their
  boss half as fast; **Content Farm** lays eggs less often; **ClockTok** has a softer beat and
  20% fewer spawns.

## Demonstrated in the client

From autoplay screenshots (`slop_game --autoplay --seed 4 --screenshots shots --exit-on-end`):

| Requirement | What the screenshots show |
| --- | --- |
| UI-05 | Mental Health with a brain, the wave bar or the boss's name and health, Clout with a gold thumbs-up coin, level and XP, weapon chips with tiers. |
| UI-08 | Hero select with five heroes, their loadouts and quirks. |
| UI-09, UI-10 | The level-up cards; the patch notes as a changelog with sections and the model's resistances. |
| UI-11 | The full-screen shop: tier frames, type labels, prices, lock buttons, weapons with sell values, items, stats and the slop's resistance. |
| UI-12 | The Burnout screen with hero, platform, wave reached, kills by type, Model Collapse and weapons. |
| SLOP-01, ART-02 | ChadGPT robots, Clawed starbursts, Yes-Bots, numbered Listicle Drones, Filter Influencers, Six-Finger Hands. |
| BRAIN-02, BRAIN-03 | Brainrot built from parts (sharks with crowns, shrimp with traffic cones), shouting names like "Zing Zing Zing Birillini!". |
| BOSS-01 | The Influencer Who Doesn't Exist with her clones and the "Phase 2" banner. |
| FAIR-02, FAIR-04 | Pink bomb markers, aim lines and spawn warnings; only threats are pink. |

## Not yet verified by a person

- How a whole run **feels**: pacing over 40 minutes, how readable boss waves are, whether the
  shop gives enough interesting choices.
- Keyboard and mouse control in a real session (autoplay plays with the bots).
- The sounds, which autoplay cannot hear.
- Bosses other than the Influencer on screen; all seven are covered by a test that fights
  each one to the end.
- The platform spread: ClockTok and InstaSlop are much harder for the bots than LinkedOut.
