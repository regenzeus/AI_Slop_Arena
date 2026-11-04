# Verification: version 5, "You are the training data"

Checked on 2026-11-03 with Rust 1.88 and Bevy 0.17.3 on Windows 10.

## Summary

| Check | Result |
| --- | --- |
| `cargo clippy --workspace --all-targets -- -D warnings` | Clean |
| `cargo test --workspace` | 111 passed, 0 failed (67 simulation, 37 rules, 7 client), 1 ignored (the icon sheet) |
| `slop_lab determinism` | Six minutes of a run, played twice by the bots: identical checksums over 21,600 ticks |
| `slop_lab bench` | 600 slop, up to 409 projectiles, six max-level weapons: 1.18 ms per tick (budget 2 ms) |
| `slop_lab balance 16 40 [temperature]` | Temperature 0.7: heuristic bot median 8.6 minutes (mean 15.1), The Model beaten 3 of 16; random bot median 8.8. Temperature 0.2: heuristic bot median 26.0, The Model beaten 4 of 16; random bot median 9.3 |
| Autoplay at 1600 x 900 (seed 6) and 1920 x 1080 (seed 11) | Title, the eight heroes, the Feed from 0:30 to 27:00 with biome seams, barriers and icons, level-up, the chest reveal, a merge, the glitch pick, pause and the end screen render without overlap |

Requirement-to-test mapping: [traceability.md](traceability.md).

## What changed from version 3, and why

From the fourth playtest:

| Feedback | Change |
| --- | --- |
| Print Screen takes no screenshot | Windows reports only the release of Print Screen, and a release without a press never reached the key state. The raw key event is read now (UI-18). |
| Damage when maintenance is up | Fences never hurt, but they were striped and labelled in the danger pink. They are construction yellow now, the warning says "blocks the way, harmless", and a fence waits while you stand where it would close (MAP-04). |
| Start flickers the pause menu | Start opened the pause screen and pressed its Resume in the same frame. Start now only pauses and resumes, and the press is used up (UI-18). |
| Can't return with the controller | B resumes too. |
| A magnet that pulls everything | Algorithm Boost pulls every Like and coin on the map, faster the farther away (LOOT-03). |
| More cool items, defensive items, shields and barriers | 25 new items, Boundaries (a shield stat), barrier weapons that stop shots (DEF-01, DEF-02). |
| Brainrot reverses controls for no clear reason | Brainrot on ClockTok is a visible zone that drops on the beat; the HUD says "CONTROLS REVERSED" with a countdown (STATUS-02). |
| Heroes should bend the rules and list their properties | Eight heroes with rules; the hero screen lists every property (HERO-02). |
| Biome crossing | The map is a checkerboard of the six platforms; walking into a square changes floor, slop and hazard (BIOME-01). |
| Broken rare items, a dopamine rush | Rarities, eight legendaries, golden chests under a pillar of light, a slot-reel chest reveal with a fanfare (ITEM-03, LOOT-02). |
| Rare perks that change the run | Glitches after every boss, and now and then on a level-up (GLITCH-01). |
| The Sunflower Patch is a green circle | Drawn as sunflowers. |
| What does "ad blocked" do? | Breaking an ad says what dropped; the Ad Blocker says what it blocks. |
| A harvest stat | Hustle (STAT-02). |
| Stationary enemies, more brainrot, the viral brainrot characters, animations | Twelve brainrot icons, parodies with their own tricks, three of them stationary; four frames of animation each; every slop waddles (ICON-01, ICON-02). |
| Coin sizes | Coin, stack, bag (LOOT-03). |
| Reroll between the middle cards | Below the build, out of the arrow path (UI-19). |
| Dash with charges, items on the dash, fire trails | DASH-01, DASH-02. |
| "Merge in reach" is unclear | The line gives both levels and what is missing; a ready merge is announced and an elite is sent. |
| Swarm events and lock-in areas | Trend alerts, pop-up walls, livestreams (EVENT-01, EVENT-02). |

## Balance: 16 runs per bot, stopped at 40 minutes

| Measure | Heuristic bot | Random bot |
| --- | --- | --- |
| Minutes survived | median 16.5, mean 21.3 | median 21.1, mean 21.4 |
| The Model beaten | 6 of 16 | 3 of 16 |
| Mean minutes by hero | Influencer 40.0, Moderator 30.9, Boomer 27.6, Programmer 23.0, Teacher 22.0, Artist 11.6, Zoomer 8.5, Intern 6.9 | Boomer 33.6, Teacher 28.4, Artist 27.9, Intern 22.5, Influencer 22.0, Zoomer 19.6, Programmer 11.4, Moderator 5.6 |

BAL-02 is not met: random picks now survive as long as the heuristic's, because the
heuristic bot's pick values were written for version 3 and know nothing of legendaries,
glitches, barriers or Hustle. That is a bot problem as much as a balance one; it is the
next thing to tune. The difficulty curve itself was raised for version 4: slop HP is
0.3 + 0.14w + 0.065w² + 0.004w³ and damage 1 + 0.12w + 0.008w² (w = 0.62 a minute).

The heuristic and random bots share the movement bot, which dashes out of crowds; the
random one only picks at random. Movement and dashing now matter more than picks, which
narrows the gap between them.

## Not yet verified by a person

- Everything in version 4 has only been played by the bots.
- Gamepad: the dash on A, RB and RT, and Start and B on the pause screen, were not tried
  with a real controller.
- Print Screen: the fix follows how Windows reports the key; it was not pressed on a real
  keyboard here. If ShareX is running, it takes Print Screen for itself; F12 always works.
- How the dash, the chest reveal and the new icons feel in the hand.

## Version 4.1 (fifth playtest)

What changed is in [design-v4.md](design-v4.md#version-41-from-the-fifth-playtest). With
twice the slop on screen in surges, the bots now die earlier: median 9.2 minutes for the
heuristic bot (16 runs), mostly around the first boss at 4:30. Weak starts (Artist,
Programmer) suffer most; strong builds still reach 40 minutes. BAL-01 (15 to 30 minutes)
is not met: this is deliberately on the hard side after "too easy, too few enemies", and
is the first thing to tune from the next playtest.

Not verified by a person: XInput in the background and the Print Screen fix were built
against the Windows API without a controller or a key press here.

## Version 5

The direction is in [direction.md](direction.md). At Temperature 0.2 the heuristic bot now
meets BAL-01's range and picks matter (median 26.0 minutes against 9.3 for random). At the
default 0.7, BAL-01 and BAL-02 are not met: runs split between about 8 minutes and very
long ones, and picks barely matter. Tuning 0.7 is the next job.

Not verified by a person: everything in this version. The Nemesis across real runs and
the memory file were tested in code only, because autoplay never touches the file.
