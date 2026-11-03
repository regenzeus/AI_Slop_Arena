# Design: version 4, "The Algorithm Update"

From the fourth playtest: "I absolutely love it", plus a long list. Version 4 keeps the
survivor-like of version 3 and adds what the genre is known for: a dash, defence, rare
loot that feels rare, run-changing perks, heroes with rules of their own, a map that
changes as you walk, and events that break the rhythm. It also adds the characters the
playtest asked for: the brainrot icons.

## Fixes from the playtest

| Playtest | Fix |
| --- | --- |
| Print Screen takes no screenshot | Windows reports only the release of Print Screen, which never reached Bevy's key state. The game now reads the raw key event; F12 still works. |
| "Why do I get damage when maintenance is up?" | Fences never did damage, but they were drawn in hot pink, the colour reserved for danger. They are now construction yellow with "blocks the way, harmless" on the warning, and a fence waits while you stand where it would close. |
| Start on the controller flickers the menu | Start opened the pause screen and pressed its "Resume" button in the same frame. Start now only pauses and unpauses, and a pause or resume uses up the press. |
| Can't return from the keyboard pause with the controller | B now resumes as well as Start. |
| Reroll sits between the middle cards | Reroll is below your build and out of the arrow-key path: R or Y, or click it. |
| The Sunflower Patch is a green circle | It is drawn as sunflowers. |
| "Ad blocked"? | Breaking an ad now says what it did: "ad smashed: loot!". The Ad Blocker item says what it blocks. |
| Brainrot reverses controls for no visible reason | ClockTok's brainrot is now a zone that pulses to the beat: stand in it when the beat drops and your controls flip. The HUD says "CONTROLS REVERSED" with a countdown, in the middle of the screen. |
| "It says merge in reach" | The merge line now says both levels and what is missing ("Rock (level 8/8) + Slingshot (level 5/8)"), and a ready merge is announced with an elite sent your way. |

## Movement: the dash

- **Dash charges** (Death Must Die). Space, Shift, or A, RB, RT on a controller. Everyone
  has two charges that come back in 3 s. A dash covers about 200 units, you cannot be hurt
  during it, and hands let go.
- **Items that change the dash:** Speedrun Shoes (+1 charge), Cold Brew (faster charges),
  Swipe Left (dashing through slop hits it), Hot Take Spill (dashes leave burning coffee),
  Left on Read (the end of a dash confuses and shoves), Ghosting (untouchable afterwards).
- **Trails:** Flame War leaves a burning trail as you walk; Dragon Energy Drink does both.

## Defence

- **Boundaries**, a new stat: a shield in front of Mental Health that takes hits first and
  comes back after 3 s without one.
- **Barrier weapons**: the Firewall and Bubble Wrap circle you, stop enemy shots and hurt
  slop they touch. They merge into the Great Firewall.
- **Defensive items:** Healthy Boundaries, Bulletproof Hoodie, Privacy Settings (shots hurt
  less), Spiky Opinions (thorns), Do Not Disturb (a shove and a moment untouchable every
  8 s), Therapist on Speed Dial (a heal below half health), Emotional Support Water Bottle.

## Loot that feels rare

- **Rarities:** common, rare, epic, legendary, in grey, blue, purple and gold frames.
  Legendaries are offered rarely; Luck makes them likelier.
- **Legendaries break rules:** Infinite Scroll (+2 projectiles on every weapon), Viral
  Moment (kills explode, and explosions set off more), Alt Account (a second you fires your
  weapons), Golden Ring Light (a Mass Report every 25 s), God Mode Glitch, Ratio Machine,
  The Algorithm's Favourite, Dragon Energy Drink.
- **Chests roll like slot machines.** Each upgrade spins through icons and lands, rarest
  last, with a sound per landing and a fanfare and golden glow for a legendary. A chest
  holds one, three or five upgrades.
- **Golden chests:** one elite in ten drops one, under a pillar of light, with an arrow at
  the screen edge when it is off screen. A golden chest holds five upgrades and always a
  legendary.
- **Coins come in three sizes:** a coin, a stack, a bag.
- **The vacuum:** Algorithm Boost now pulls every Like and coin on the map, faster the
  farther they are.

## Glitches: perks that change a run

Arcana-style rare perks. After every boss, its chest is followed by "THE FEED GLITCHED":
pick one of three. Now and then one also turns up on a level-up card. Examples: Glass
Cannon, Ricochet Brain (every projectile bounces twice), Speed Demon (damage times your
Speed), Hoarder (Clout is damage), Vampire Feed, Echo Chamber (every attack happens again),
Cursed Feed (more slop, more loot), Doom Dash, Plot Armour.

## Heroes bend the rules

Eight heroes. Each starts with a weapon and an item and bends a rule, Brotato style; the
hero screen lists every property, good in green and bad in red.

| Hero | Rules |
| --- | --- |
| Boomer | Chain letters; +20% Analog; a pension; never offered Sarcasm weapons |
| Zoomer | 3 dash charges, faster; immune to Brainrot; -20% Reach |
| Teacher | Marks hurt more; elites drop an extra chest; only 5 weapon slots |
| Programmer | New weapons start at level 2; previews patch notes |
| Artist | +40% against brainrot; one more choice; -25% Clout |
| Influencer (new) | Clout is power: +1% Conviction per 10 Clout held; +25 Hustle |
| Intern (new) | +30% experience; one more choice; machines cost more |
| Moderator (new) | Dashes hit and end in a shockwave; +25 Boundaries; a slow drain |

## Hustle

Brotato's harvesting, as a stat: Clout every minute, and coins drop more often.

## The map: biomes

The map is a patchwork of the six platforms, one per square of 3072 units, laid out like
a checkerboard so neighbours always differ. Walking into a square changes the floor, the
local slop, the hazard, and a banner says where you are. The bosses still come in order;
the HUD says which boss comes next and from which platform.

## Events

- **Trend alerts** (Vampire Survivors' bat swarms): a swarm of Trampoline Bunnies,
  Listicle Drones, Six-Finger Hands, Glass Fruit or Gusellos crosses the screen in a line
  and is gone.
- **Pop-up walls:** a ring of Subscribe Pop-ups appears around you and closes in.
- **Livestreams:** "YOU'RE LIVE": a ring closes around you and holds you in with a growing
  crowd for 28 s, then pays a big chest. They come on their own now and then, and from
  "Go Live" ring lights on the map.

## The brainrot icons

Parodies of the viral AI characters, drawn from scratch in code with four frames of
animation each (legs walk, bats swing, propellers spin), and a trick each:

| Icon | Trick |
| --- | --- |
| Tonk Tonk Tonk Sahur | A log with a bat: bonks the ground around it after a warning |
| Tralalelo Sneakerlala | A shark in sneakers: aims, then sprints in a straight line |
| Bombardilo Crocodilo | A crocodile bomber: flies over walls, marks where bombs land |
| Ballerina Cappuccinella | Pirouettes; the spin hurts up close |
| Lirilì Larilò | A cactus elephant in sandals: stands still, fires rings of needles |
| Brr Brr Patapum | A walking forest: stomps roots that slow you |
| Chimpanzini Bananello | Throws banana peels; a hit makes you slip |
| Cappuccino Assassinello | Marks a spot behind you, then appears there |
| Bombombini Gusello | A goose with jet engines: flies at you and explodes |
| Trampoline Bunny | Security-camera bunnies: they come in swarms |
| Glass Fruit | Bursts into shards that fly at you |
| Ring Light | A tripod that buffs slop near it and fires bubbles |

They join every platform as the Feed goes on; ClockTok has its own from the start. Every
slop also waddles and squashes as it walks.

## AI-made art

The game ships no asset files and everything is drawn in code, so I cannot generate AI
pictures here. What would work: generate sprite sheets (four frames, transparent
background) with an image model, and have the client load PNGs from an `assets` folder in
place of the drawn sprites when they exist. That is a small change to `art.rs` when there
are pictures to load.

## What else from the genre

Candidates for later, from Vampire Survivors and its family: permanent upgrades bought
between runs with banked Clout, unlockable heroes and weapons through achievements, a
bestiary of everything deleted, curses that trade difficulty for loot, limit breaks past
level 8, and an "Arcana" deck chosen at the start.

## Version 4.1: from the fifth playtest

"In general I do not want to obscure the gameplay, and not take control away from the
player. Slow and stun is fine."

- **Nothing takes control or covers the screen.** Brainrot, doomscrolling, Clickbait
  arrows, Thought Leader posts and banana peels now slow you; LinkedOut's banners are slow
  bands; MeTube's ads became "buffering" patches on the floor. The ad overlay and the
  pop-up dialog are gone.
- **Effects are shown, not announced.** No "GOD MODE", "TREND ALERT", "LIVE", "Boundaries
  broken", pickup or status words. You get a gold aura (God Mode), a blue bubble
  (Boundaries), a violet tint (slowed), a spark glow (charged), and coloured rings.
  Speech bubbles mostly say "...", and biome crossings show only on the floor and the HUD.
  Boss banners are a small strip under the clock.
- **Denser, bursty hordes.** About twice the slop at a time, each weaker; the crowd
  surges every 75 s, and every fourth surge is a horde. Surges are melee swarms, not a wall
  of bullets.
- **Fewer, better items.** Levels take more experience; fewer than half of elites carry
  a chest; a Viral elite's copies no longer each drop one; chests mostly hold one upgrade.
  Once everything is maxed, chests pay out without stopping the game.
- **Elites stand out:** every 30 s, more later, each with a glowing aura in its affix's
  colour. Two new affixes: Summoner and Hulking.
- **Interactables:** recycle bins, fine-tuning stations, data brokers and power outlets,
  one or two things to use per chunk.
- **The loot goblin:** a Crypto Bro runs off with a bag of Clout.
- **Golden chests explain themselves:** each card says what the upgrade does.
- **Biomes are twice as big** (6144 units).
- **Controller in the background:** the game reads Xbox controllers through XInput and
  keeps running when unfocused.
- **More animation:** everything that walks has feet that step; brainrot objects swing
  and accessories bob.

### Item slots

Six weapon slots and six item slots stay: the limit is what makes picks matter. A recycle
bin frees a slot when a build changes direction, and a maxed build gets paid instead of
being stopped.
