# Verification: version 3, "The Feed"

Checked on 2026-11-03 with Rust 1.88 and Bevy 0.17.3 on Windows 10.

## Summary

| Check | Result |
| --- | --- |
| `cargo clippy --workspace --all-targets -- -D warnings` | Clean |
| `cargo test --workspace` | 78 passed, 0 failed (47 simulation, 24 rules, 7 client), 1 ignored (the icon sheet) |
| `slop_lab determinism` | Six minutes of a run, played twice by the bots: identical checksums over 21,600 ticks |
| `slop_lab bench` | 400 slop, up to 383 projectiles, six max-level weapons: 0.82 ms per tick (budget 2 ms) |
| `slop_lab balance 16 40` | Heuristic bot: median 20.9 minutes, The Model beaten in 6 of 16 runs. Random bot: median 8.0 minutes |
| Autoplay (seeds 4, 6 and 11) | Title with the demo behind it, hero select, the Feed from 0:30 to 27:00, level-up, chest, merge, pause and the end screen render as intended |

Requirement-to-test mapping: [traceability.md](traceability.md).

## What changed from version 2, and why

From the third playtest ("the waves and the shop break the flow; make it a straight
survivor-like"):

| Feedback | Change |
| --- | --- |
| Waves and the shop break the flow | One endless run on one world. A director retunes the spawns as minutes pass; each platform ends with its boss, and the Feed moves on. |
| Pick one of three, and every choice must be valid | Level-ups offer three different valid choices: no new weapons with six, no level for a weapon at 8, no new items with six, no level for a maxed item. |
| Elites, bosses, loot drops | Elites with five affixes drop chests; bosses drop big chests; slop and breakables drop food, magnets, Mass Reports, Server Outages and Clout. |
| Weapon merging like Vampire Survivors | 13 merges: two weapons at level 8 become one when you open a chest. |
| More weapons | 7 new base weapons (29 in all) and 13 merged ones. |
| The weapon classes do nothing | Analog staggers, Fact cuts through shields and Verified, Sarcasm confuses, Grass roots; two and four weapons of a type give set bonuses. |
| More hazards like the downvote zone, not just circles | Zones in circles, rectangles and rings; maintenance fences close blocks of the map for a while. |
| A more diverse, endless map | The map is endless, made per chunk from the seed: walls (server racks, billboards, desks), feed scraps tinted per platform, props. |
| Objects that do things (Death Must Die) | Wi-Fi hotspots, vending and gacha machines, shrines with blessings, data centers, comment sections, ads, crypto miners, lost phones. |
| More brainrot | Three new bodies, objects and accessories (ten of each), each with an ability. |
| Controller support (Xbox 360) | One input layer for keyboard and gamepads; hint bars switch to the gamepad's buttons. |
| The controls feel a bit off | Movement is drawn between simulation ticks, the camera is locked on you with no lag, and movement reaches full speed faster. |

Found while balancing, with the new `slop_lab deaths` command:

- Item drawbacks stacked with item levels (Energy Drink at level 5 took 75 Mental Health).
  A drawback now counts once, and maximum Mental Health never drops below 30.
- Six-Finger Hands could hold on four at a time forever; now three at most, for 3 s each.
- Doomscrolling drained health whenever slop was on screen, which in a horde is always;
  it now drains half as much.

## Balance: 16 runs per bot, stopped at 40 minutes

| Measure | Heuristic bot | Random bot |
| --- | --- | --- |
| Minutes survived | median 20.9, mean 21.1 | median 8.0, mean 16.5 |
| The Model beaten | 6 of 16 | 2 of 16 |
| Mean minutes by hero | Zoomer 32.5, Boomer 22.7, Artist 17.3, Programmer 15.5, Teacher 13.8 | Boomer 38.8, Zoomer 16.3, Artist 16.9, Teacher 7.8, Programmer 3.1 |

Runs are swingy, as survivor-likes are: a build that comes together survives a long time.
The bots never use the map's machines, so a person has more tools than they do.

## Demonstrated in the client

From autoplay screenshots (`slop_game --autoplay --seed 6 --screenshots shots --exit-on-end`):

| Requirement | What the screenshots show |
| --- | --- |
| RUN-01, UI-05, ART-06 | The Feed at 0:30, 2, 5, 9, 14, 20 and 27 minutes: the clock, the platform, the boss countdown, the experience bar, toasts, feed scraps, walls, props and pickups. |
| RUN-11 | A boss with its health bar at 5:00; the next platform after it. |
| UI-09, LEVEL-02 | Level-up cards with icons, NEW or the level change, and merge hints; the build below. |
| UI-11, WPN-12 | A chest with its upgrades; a merge (Eye-Roll + Uno Reverse into Infinite Uno). |
| UI-13 | Pause with the build, levels, the next merge in reach, and stats. |
| UI-15 | The title with the bots' run behind it. |

## Not yet verified by a person

- Everything in version 3 has only been played by the bots.
- Gamepad support: built on Bevy's gamepad input, which handles Xbox controllers on
  Windows; no controller was connected while testing.
- Whether the machines on the map feel worth the detour, and how the merges feel.
- Whether level-ups come too often early on.
