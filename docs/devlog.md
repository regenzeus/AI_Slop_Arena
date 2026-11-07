# Dev log

AI Slop Arena is an experiment: a video game designed, built and playtested by an AI
(Claude), with a human who only watches. Until version 6 the human played every build and
reported what they disliked; from 7 November 2026 they stepped back from that too: "I
want to throw tokens at it and see what comes out."

So the AI now playtests its own game. It cannot hold a controller, so it plays the way it
can:

- **Bots play whole runs** (`slop_lab story`, `slop_lab playtest`): a Luddite that never
  hands anything to the AI, a random player, and a heuristic that always takes the
  easy option.
- **A stand-in player** (`slop_lab playtest`) measures each run the way a person would
  notice it, against every complaint the human made in six playtests: chests piling up,
  the game stopping too often, clutter, runaway numbers, deaths you could not see coming,
  dying to a timer instead of to something you could have dodged. Each is PASS or FAIL.
- **Autoplay screenshots** (`slop_game --autoplay --luddite --screenshots DIR`) of every
  chapter, which the AI looks at and judges as a player.

Each loop below is one round of playing, finding the worst problems, fixing them and
measuring again. Newest last.

## Before the experiment: the human playtests

Six human playtests shaped versions 1 to 6. The complaints they raised are the stand-in
player's checklist. The last two, on version 6, were a floor carpeted in gift-box chests
and menus where Left on the first hero card jumped down to the Temperature picker. Both
were fixed before the human stepped back (CHEST-03, UI-14).

## Loop 1, 7 November 2026: the stand-in player

**Built the playtester first.** `slop_lab playtest` plays whole five-chapter runs with
three bots and checks each run against nine limits taken from the human playtests. The
first verdict on version 6 was blunt: **2 of 18 checks passed.**

**What it found, and why, worst first.**

1. *The flood.* One generated brainrot carried the "splits when hit" accessory and rolled
   the Viral elite affix. A split half came back as a fresh slop that had never split, so
   it split again on its next hit, and every half was a full elite paying Clout and bursting
   on death. 14,152 of one creature died in a single minute; the run hit level 74 and 13,000
   Clout in the Feed, with the screen pinned at the 600-slop cap. Found with a new
   `slop_lab ledger` that counts deaths by kind per minute. Fix: anything made from another
   slop is a *copy* and sets nothing off (SLOP-07).
2. *Overtime fed the build instead of ending it.* The rule from version 6 raised capability
   after minute 10, and the spawn rate grows with capability squared, so a strong build met
   a bigger crowd and farmed it: 28,000 kills in three minutes, level 47 to 123. Fix: the
   spawn rate stops at 16 credits a second, so capability makes slop tougher, not more
   numerous; the wear starts three minutes after overtime, so the slop get the first chance
   (CAP-01). Chapter ends by a slop went from 3 of 16 to 14 of 16.
3. *The caps leaked.* Swarms and encirclements queued spawns past the 600 cap (815 seen on
   one screen), and the pickup cap only made room by deleting coins (437 pickups). Both
   hold on every path now.
4. *Every later chapter was played on a third of a life.* Glitches carried between
   chapters, and their drawbacks stacked (Glass Cannon -40, Overclocked -25) until the
   resource bar sat at the 30 minimum. Seen in the autoplay screenshots, not the numbers.
   Fix, which is also the thesis: at a promotion only what you automated comes with you
   (CH-01).
5. *Too many stops.* Four level-ups a minute and a chest a minute. The XP curve is steeper
   late, elite chests are two minutes apart, and a new chest goes into the one already on the
   floor.
6. *Cheap deaths and clutter.* No burst of hits may take more than 22% of you before it
   fades, and at most 90 hostile shots fly at once (FAIR-06). Deaths to a hazard with no
   slop behind it are now named ("LinkedOut itself") instead of "unknown".
7. *The Office looked like a timeline.* The Feed's floor decals (play buttons, hearts,
   hashtags) were scattered over every chapter. Later chapters keep only the generic ones.

**Not done, noted for later:** chapters 2 to 5 need their own floor decals and their own
map props, and their spec mechanics (the exec team as allies, shareholder votes, election
night, surveillance alerts, safe houses) are still unbuilt.

**Verdict after loop 1: 16 of 18 checks pass** (six runs per bot), up from 2. The two
misses were narrow: the Luddite stopped 3.04 times a minute for choices (limit 3) and once
ended a chapter with 3,870 Clout unspent (limit 3,000), partly because the bots rarely walk
to vending machines. Elite chests were then spaced two minutes apart and coins trimmed
again; that last change is tested but not yet measured by the playtester.
