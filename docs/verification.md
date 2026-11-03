# Verification: version 2

Checked on 2026-11-03 with Rust 1.88 and Bevy 0.17.3 on Windows 10.

## Summary

| Check | Result |
| --- | --- |
| `cargo clippy --workspace --all-targets -- -D warnings` | Clean |
| `cargo test --workspace` | 68 passed, 0 failed (39 simulation, 23 rules, 6 client), 1 ignored (the icon sheet) |
| `slop_lab determinism` | Six waves of a run, played twice by the bots: identical checksums over 15,300 ticks |
| `slop_lab bench` | 400 slop, up to 360 projectiles, six tier IV weapons: 0.43 ms per tick (budget 2 ms) |
| `slop_lab balance 100` | Heuristic bot wins 38% (target 20% to 40%); random bot wins 4% (target under 10%) |
| `slop_lab economy 30` | About one level per wave; one to two purchases per shop in waves 1 to 3, three or more from wave 5 |
| Autoplay (seeds 4 and 7) | Title with the demo behind it, hero select, briefing, waves, level-up, pause, shops and the end screen render as intended |

Requirement-to-test mapping: [traceability.md](traceability.md).

## What changed from version 1, and why

From the first playtest:

| Feedback | Change |
| --- | --- |
| Patch notes after every wave make no sense | No patch-notes screen. A briefing before wave 1 explains how the slop learns; each wave's patch notes sit in the shop. |
| The screen warps when you change direction | The camera no longer looks ahead of your movement; it follows you tightly. |
| The background has more contrast when paused | Floors are dark-mode versions of each platform, so bright characters and pink danger stand out while you play. |
| The game must work by keyboard; the shop looks bad | Every screen works by arrows/WASD, Enter and Space, with a focus ring and tooltips. The shop was rebuilt around icon cards, with the stats down the whole right side. |
| Icons for everything | Every weapon, item, stat, damage type and level-up has an icon, drawn in code. |
| Stronger economy, cheaper rerolls | A Creator Fund payout after every wave, cheaper items, and rerolls from 2 Clout. |
| About one level per wave | Likes and the payout are XP, on a new curve. |
| No items at the bottom during waves | The weapon bar is gone; pause shows your weapons, items and stats. |
| Every stat needs a tooltip | Each stat's tooltip explains its mechanics with your current numbers. |
| Weapons need animation | Weapons are drawn around you and aim, kick, swing and throw; each weapon has its own effect. |
| Better chat lines | New lines for ChadGPT, Clawed (while it aims), Yes-Bots and Thought Leaders, and varied removal notices. |

Beyond the list: heroes look different, the bots play behind the title screen, kill streaks
and level-ups are announced, projectiles leave trails, bricks fly in an arc, and there are
sounds for buying, rerolling, levelling up and moving the focus.

## Balance: 100 runs per bot

| Measure | Heuristic bot | Random bot |
| --- | --- | --- |
| Runs won | 38% | 4% |
| Most common wave of Burnout | 10 (the platform boss) | 10 |
| Runs won by platform | MeTube 10/17, FaceSlop 8/20, LinkedOut 6/13, Thredd 6/17, InstaSlop 5/15, ClockTok 3/18 | 1 in 13 or fewer everywhere |

The richer economy made the bots much stronger (81% wins at version 1's difficulty), so slop
HP now grows as 1 + 0.22w + 0.04w² (was 1 + 0.13w + 0.009w²) and slop damage as 1 + 0.11w
(was 0.07w), where w is the wave minus one. The platform spread is narrower than in
version 1; ClockTok is still the hardest.

## Economy: 30 runs of the heuristic bot

| Wave | Likes earned | Clout at the shop | Level after the wave | Things bought |
| --- | --- | --- | --- | --- |
| 1 | 14 | 25 | 1.0 | 1.0 |
| 3 | 20 | 44 | 3.5 | 1.8 |
| 5 | 70 | 104 | 6.4 | 3.6 |
| 10 | 44 | 110 | 11.6 | 2.4 |
| 15 | 217 | 354 | 17.2 | 4.6 |
| 19 | 180 | 390 | 21.5 | 4.7 |

## Demonstrated in the client

From autoplay screenshots (`slop_game --autoplay --seed 7 --screenshots shots --exit-on-end`):

| Requirement | What the screenshots show |
| --- | --- |
| UI-02, ART-03 | Dark platform floors; characters, coins and bubbles stand out. |
| UI-05 | Mental Health, the wave bar, Clout and level with "+1 LEVEL UP"; no weapon bar. |
| UI-10 | The briefing: platform, eight slop cards, both bosses, how the slop learns. |
| UI-09, ART-04 | Level-up cards with stat icons and values before and after, and the stats panel. |
| UI-11, EVO-03 | The shop: icon cards with tiers, upgrade ribbons and prices; weapons with sell values; items; the patch notes; stats and resistances down the right side. |
| UI-13 | Pause: weapons with damage dealt, items with counts, the stats panel. |
| UI-15 | The title with the bots' demo wave behind it. |
| WPN-09 | Weapons around you, keyboard rings, brick debris, trails. |
| ART-05 | Each hero's own look on the title, hero cards and in the arena. |

## Not yet verified by a person

- How the new economy and difficulty feel over a whole run.
- Keyboard navigation and tooltips in a real session (autoplay does not press keys).
- Whether the weapon animations read well at full speed, and the new sounds.
- The platform spread: ClockTok remains the hardest for the bots.
