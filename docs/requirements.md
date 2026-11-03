# Requirements: version 1

Each requirement has an ID that tests reference with `// REQ: ID`. "Verified by" says how it
is checked: **T** an automated test, **L** a lab measurement, **D** a demonstration in the
client (autoplay screenshots or play), **I** inspection of the code.

## Simulation

| ID | Requirement | Verified by |
| --- | --- | --- |
| SIM-01 | The arena is deterministic: the same seed and the same input every tick give the same wave and the same run, bit for bit. | T, L |
| SIM-02 | You stay inside the square arena. | T |
| SIM-03 | Movement eases in and out, and diagonal movement is not faster. | T |
| SIM-04 | A crowded arena (400 slop, 300 bubbles, six tier IV weapons firing fast) steps in under 2 ms. | L |

## You

| ID | Requirement | Verified by |
| --- | --- | --- |
| STAT-01 | Ten stats exist. Composure multiplies damage taken by 15 / (15 + Composure); Scepticism dodges at most 60% of hits. | T |
| STATUS-01 | The slop can give you Anxiety (damage over time), Doomscroll (dragged forward), Pop-up, Ratio'd (-20% damage) and Brainrot (reversed controls after a warning). Items and heroes can prevent them. | T |
| ACT-01 | Actives fire on their own when their trigger is met: Log Off, Block Button, Digital Detox, Unsubscribe, Phone a Friend. | T |
| HERO-01 | Five heroes, all unlocked, each with a starting weapon, a starting item, stat changes and a quirk. | T |

## Weapons

| ID | Requirement | Verified by |
| --- | --- | --- |
| WPN-01 | All 22 weapons of the design attack on their own and damage slop; at least four of each damage type. | T |
| WPN-02 | Melee swings hit only slop in front. | T |
| WPN-03 | The Eye-Roll bounces and Air Quotes chain between slop. | T |
| WPN-04 | Damage is the weapon's damage times its tier, Conviction, type bonuses, the slop's resistance and marks. | T |
| WPN-05 | Bricks splash where they land; the Ban Hammer deletes small slop outright. | T |
| WPN-06 | The Fact-Check Flashlight strips beauty filters. | T |
| WPN-07 | The Frisbee comes back; the Uno Reverse Card turns bubbles around at the slop. | T |
| WPN-08 | Seed Bombs heal you inside their patch; the Dog Walk's dog circles you. | T |
| WPN-09 | Your weapons are drawn around you: they aim at what they attack, guns kick back, melee weapons sweep through their arc, thrown ones lunge, and every weapon has its own attack effect (muzzle flash, spray, flying keys, zigzag quotes, falling hammer, brick debris, trails). | D |

## Slop

| ID | Requirement | Verified by |
| --- | --- | --- |
| SLOP-01 | Every enemy is a character: five parody chatbots (ChadGPT, Clawed, Yes-Bot, Hallucinator, Listicle Drones) and platform slop. ChadGPT fires fans of three bubbles; only the middle one talks. | T, D |
| SLOP-02 | Six-Finger Hands grab you, slow you by 20% each and hurt you while holding on. | T |
| SLOP-03 | Listicle Drones come in numbered packs; killing number 1 first scatters the pack. | T |
| SLOP-04 | Yes-Bots buff nearby slop; Grandma's Cake heals it. | T |
| SLOP-05 | Traits work: Recruiter Bots split, Repost Bots come back once, cats dodge their first hit, party hats explode, beauty filters block frontal damage. | T |
| SLOP-06 | Elites ("Trending") are bigger and tougher and drop a Loot Box that opens free in the next shop. | T |
| BRAIN-01 | Brainrot is built from a body, an object and an accessory; each part sets its look and one ability. | T |
| BRAIN-02 | Every brainrot gets a fake-Italian name from its parts and shouts it when it appears. | T, D |
| BRAIN-03 | Brainrot is drawn from its three parts, so evolution changes what you see. | D |
| BUB-01 | Your projectiles pop speech bubbles they pass through. | T |
| LIKE-01 | Dead slop drops Clout coins; coins within Curiosity fly to you. Hallucinator fakes explode into Anxiety. | T |

## Bosses and platforms

| ID | Requirement | Verified by |
| --- | --- | --- |
| BOSS-01 | Seven bosses each fight with their own puzzle and can be beaten: Crustacean Saviour, Hustle Guru, Content Farm, Repost Hydra, The Influencer Who Doesn't Exist, The Supreme Brainrot, The Model. | T |
| BOSS-07 | The Model regenerates once unless your kills were spread evenly over damage types (Model Collapse). | T |
| PLAT-01 | Each run takes place on one of six parody platforms, which sets the floor, the local slop and the wave-10 boss. | T, D |
| PLAT-02 | Every platform has its hazard: chain letters, Open-to-Work banners, mid-roll ads, downvote zones, filter zones, the beat. | T |

## Fairness

| ID | Requirement | Verified by |
| --- | --- | --- |
| FAIR-01 | Slop only shoots when it is near you, roughly on screen. | T |
| FAIR-02 | Spawns, em-dashes and bombs are telegraphed before they happen. | T, D |
| FAIR-03 | After a hit you are invulnerable for half a second, and your sprite blinks. | T, D |
| FAIR-04 | Everything that hurts you is outlined in hot pink, and nothing else uses that colour. | I, D |

## The run

| ID | Requirement | Verified by |
| --- | --- | --- |
| RUN-01 | A run is 20 waves. After every wave: level-ups, then the shop; the first shop comes after wave 1. | T |
| RUN-02 | Your hero, items and level-ups make up the loadout you take into a wave. | T |
| RUN-03 | Clearing a wave pays its Likes, a Creator Fund payout (8 + 2 per wave), interest and side-hustle income; Likes and the payout are also XP. Burnout ends the run. | T |
| WAVE-01 | A timed wave is cleared when its time runs out and credits half of the coins left on the floor. | T |
| WAVE-02 | Spawns ramp up within a wave; elites appear from wave 6. | T |
| WAVE-03 | Waves get longer and harder; waves 10 and 20 are boss waves that end when the boss dies. | T |
| WAVE-04 | Platform house rules shape the spawn table (support, shooters, tanks, reposts, all brainrot). | T |
| WAVE-05 | Wave 10 brings the platform's boss and wave 20 The Model, which copies your most-used weapons. | T |
| LEVEL-01 | Each level offers four different stat upgrades; Luck makes better ones likelier. | T |
| LEVEL-02 | XP per level is tuned so a wave is worth about one level: the heuristic lab bot is at level 18 to 24 after wave 19. | L |
| ITEM-01 | More than thirty items in four kinds (healthy habits, risky habits, side hustles, actives); risky items carry a drawback. | T |
| SHOP-01 | The shop has five offers, at least one in a damage type you use. Buying a weapon you own upgrades it; six weapon slots; selling refunds half. | T |
| SHOP-02 | Offer tiers get better with waves and Luck. | T |
| SHOP-03 | Prices rise 4% per wave. A reroll costs 2 + wave / 4 and gets dearer within a shop. | T |
| SHOP-04 | Locked offers survive rerolls and carry into the next shop. | T |
| SHOP-05 | The economy lets you buy: the heuristic lab bot buys one to two things per shop in waves 1 to 3 and three or more from wave 5. | L |
| EVO-01 | After every wave the base model's resistances move towards your kill share, zero-sum and capped. | T |
| EVO-02 | Brainrot that hurt you and lived long breeds; children mutate their numbers and sometimes a part. | T |
| EVO-03 | Patch notes after each wave say what the slop learned; they are shown in the shop. | T, D |
| EVO-04 | Model Collapse measures how evenly your kills spread over the four damage types. | T |
| BAL-01 | Hard but manageable: the heuristic lab bot wins 20% to 40% of runs. | L |
| BAL-02 | Random play rarely wins: the random lab bot wins under 10% of runs. | L |

## Client

| ID | Requirement | Verified by |
| --- | --- | --- |
| UI-01 | The client runs waves at 60 ticks per second from keyboard input; P pauses. | D |
| UI-02 | A camera follows you without looking ahead, so turning around does not swing the view, and never shows much beyond the walls. Floors are dark so characters and danger stand out. | D |
| UI-03 | Slop show a health bar for three seconds after being hit. | D |
| UI-04 | Damage numbers appear in the colour of their damage type. | D |
| UI-05 | The HUD shows Mental Health (a brain) with statuses, the wave timer or the boss's health, Clout (a coin with a thumbs-up, never a heart) and level, with level-ups earned so far in the wave. Weapons and items are not shown during a wave; pause shows them. | D |
| UI-06 | Autoplay plays a whole run with the bots and screenshots every kind of screen. | D |
| UI-07 | Resizing the window scales the arena view and the HUD. | D |
| UI-08 | Title and hero select; every choice works by mouse and by keyboard. | T, D |
| UI-09 | Level-up screen with four cards, each with the stat's icon and its value before and after. | D |
| UI-10 | Once per run, before wave 1, a briefing shows the platform, its slop, both bosses and how the slop learns. | D |
| UI-11 | Full-screen shop: offer cards with icons, tier frames and upgrade ribbons, lock, reroll, your weapons and items, the patch notes, and the stats down the whole right side. | D |
| UI-12 | End screen for victory or Burnout with the run's summary. | D |
| UI-13 | Pause shows your whole build: weapons, items and stats. Every stat, item, weapon and resistance has a tooltip; a stat's tooltip explains its mechanics with your current numbers. | T, D |
| UI-14 | Every screen works by keyboard alone: arrows or WASD move the focus to the nearest element in that direction, Enter presses it, Space continues, L or X locks or sells; the mouse focuses what it hovers. | T, D |
| UI-15 | Behind the title and hero screens the bots play a mid-run wave, silently. | D |
| ART-01 | Every sprite is drawn in code with outlines and anti-aliased edges; characters flash white when hit. | T, D |
| ART-02 | Every character, brainrot part and boss has its own picture. | T, D |
| ART-03 | Each platform has its own floor. | D |
| ART-04 | Every weapon, item, stat and damage type has its own icon, drawn in code. | T, D |
| ART-05 | Each hero has their own look, in the menus and in the arena. | D |
| FEEL-01 | Hits, deaths, pops, swings, beams, chains, splashes and statuses have visible effects and screen shake, capped so crowds stay readable. Kill streaks and level-ups are announced. | D |
| FEEL-03 | The slop talks in character: ChadGPT in assistant clichés, Clawed while it aims, Yes-Bots agreeing, Thought Leaders posting; deleted slop leaves varied removal notices. | T, D |
| FEEL-02 | Placeholder sounds are synthesised in code and rate-limited. | T, D |
