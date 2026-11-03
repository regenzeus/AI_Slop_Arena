# Design: version 3, "The Feed"

From the third playtest: waves and the shop break the flow. Version 3 turns AI Slop Arena
from a Brotato-style wave game into a Vampire Survivors-style survivor-like: one endless
run on an endless map, power picked up while moving, never in a menu between rounds.

## What stays

The deterministic simulation, every weapon, slop, brainrot genome and boss, the four damage
types, the slop that learns, the heroes, keyboard navigation, icons and tooltips.

## The run

- **One run, no waves, no shop.** You scroll through the Feed: an endless map whose
  platform changes as you go. A run starts on a random platform; the order of the other
  five is shuffled.
- **Platforms are stages.** Each platform has its floor, hazard, local slop and house rule.
  Around 4:30 on a platform its boss arrives. Deleting it opens a chest and moves the Feed
  on to the next platform. After the sixth boss comes The Model. After The Model the run
  stays endless: platforms repeat as "remixes" and the slop keeps scaling until you burn
  out. The score is time survived and slop deleted.
- **Difficulty only goes up.** HP, damage and spawn rate grow with time, not with the
  platform. Every two minutes the slop **retrains** on what killed it (zero-sum
  resistances, brainrot breeding) and says so in a one-line toast. It never stops the game.
- **Elites** spawn on a timer with an affix: Verified (shielded), Viral (splits), Sponsored
  (drops Clout), Trending (fast), Toxic (leaves a slow trail). Elites drop chests.

## Power

- **Likes are XP.** Slop drops Likes; levels come fast early and slow later.
- **Level-ups: pick 1 of 3** (4 with Luck). Every offer is valid: a new weapon only while
  you have a free slot (6), a weapon level only below level 8, a new item only while you
  have a free item slot (6), an item level only below its maximum, a stat-up always. If the
  pool runs dry, "Touch Grass" (heal) and "Brand Deal" (Clout) fill in.
- **Weapons have 8 levels.** Levels add damage, and at levels 3, 5 and 7 they add a
  projectile, area or a faster cooldown.
- **Merges.** Two specific weapons, both at level 8, merge into an evolved weapon when you
  open a chest. The merge frees a slot. Recipes are hinted on the weapons' tooltips. Some
  weapons evolve with an item instead (Vampire Survivors style).
- **Damage types do something on their own.** Analog staggers and knocks back. Fact
  pierces and breaks shields. Sarcasm confuses: hit slop may turn on other slop. Grass
  roots slop and heals you on kills. Having 2 and 4 weapons of one type gives a set bonus.
- **Clout is money.** It drops from elites, breakables and Sponsored slop, and is spent on
  the map: vending machines, gacha, shrines, rerolls at level-ups.

## The map

- **Endless in every direction.** It is generated in chunks from the run's seed, so the
  same seed gives the same map. Slop that falls far behind is moved ahead of you.
- **Dummy content as background.** Each platform's floor is a feed: post cards, thumbnails,
  comment threads, ads, profile pictures and reaction icons, drawn in code and scattered
  per chunk, so no two screens look alike.
- **Walls and obstacles.** Server racks, billboard walls and desks block you and the slop.
- **Hazards in shapes.** Zones can be circles, rectangles, rings or bands:
  - Downvote zones slow you.
  - Filter zones shield slop.
  - **Under maintenance** areas fence off a rectangle of the map for a while, with a
    countdown. Nobody gets in.
- **Things to do (Death Must Die style):**
  - **Wi-Fi hotspot**: stand inside to heal.
  - **Vending machine**: spend Clout on an item.
  - **Gacha machine**: spend Clout on a random upgrade.
  - **Shrine of the Algorithm**: pick 1 of 3 run-long blessings.
  - **Data center**: stand inside while it uploads; the slop rushes in, then it pays a chest.
  - **Comment section**: a cursed challenge; survive a horde of elites for a big reward.
  - **Breakable billboards and pop-up ads**: they drop food, magnets, bombs, freezes and Clout.
- **Pickups:**
  - Touch Grass (heal).
  - Algorithm Boost (pulls in every Like).
  - Mass Report (deletes everything on screen).
  - Server Outage (freezes the slop).
  - Chests.

## Controls

Keyboard and Xbox controller: left stick or the d-pad to move and to choose, A to confirm,
B to go back, X to lock or banish, Y to reroll, Start to pause. Movement is drawn smoothly
between simulation ticks, and the camera stays locked on you.

## Borrowed from the genre, translated

| Idea | From | Here |
| --- | --- | --- |
| Endless map, timed bosses, chests that evolve | Vampire Survivors | The Feed, platform bosses, merges |
| Weapon + weapon combinations ("collabs") | HoloCure | Merges |
| Map encounters and shrines with choices | Death Must Die | Shrines, vending, gacha, challenges |
| Elites with affixes | Halls of Torment, Diablo | Verified, Viral, Sponsored, Trending, Toxic |
| Stand-in-zone objectives | Deep Rock Galactic: Survivor | Data centers |
| Level-up rerolls and banishes | Vampire Survivors | Clout-priced rerolls, banish |
