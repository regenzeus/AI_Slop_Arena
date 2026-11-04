# Requirements: version 4, "The Algorithm Update"

Each requirement has an ID that tests reference with `// REQ: ID`. "Verified by" says how it
is checked: **T** an automated test, **L** a lab measurement, **D** a demonstration in the
client (autoplay screenshots or play), **I** inspection of the code. The design behind
version 3 is in [design-v3.md](design-v3.md), version 4 in [design-v4.md](design-v4.md).

## Simulation

| ID | Requirement | Verified by |
| --- | --- | --- |
| SIM-01 | The Feed is deterministic: the same seed and the same input every tick give the same run, bit for bit. | T, L |
| SIM-02 | There is no edge to the map. | T |
| SIM-03 | Movement eases in quickly, and diagonal movement is not faster. | T |
| SIM-04 | A crowded Feed (400 slop, 300 bubbles, six max-level weapons firing fast) steps in under 2 ms. | L |

## The map

| ID | Requirement | Verified by |
| --- | --- | --- |
| MAP-01 | The map is endless and generated in chunks from the run's seed alone, in any order; slop left far behind is moved back in front of you, and Likes are never lost. | T |
| MAP-02 | Walls (server racks, billboards, desks) and closed fences stop you, the slop and shots; lobbed shots fly over. | T |
| MAP-03 | Props: ads and miners break when you walk into them and drop loot; Wi-Fi heals while you stand in it; vending machines sell an item for Clout; gacha machines give a random upgrade for Clout, dearer each pull; shrines offer one of three blessings; data centers upload while you stand inside and draw slop in, then pay a chest; comment sections release elites and pay a chest when they are deleted. | T, D |
| MAP-04 | About every 50 s a block of the map closes for maintenance after a warning, never on top of you, and opens again later. | T, D |
| ZONE-02 | Zones come in shapes: circles, rectangles and rings. Downvotes on Thredd use all three. | T, D |

## You

| ID | Requirement | Verified by |
| --- | --- | --- |
| STAT-01 | Twelve stats exist. Composure multiplies damage taken by 15 / (15 + Composure); Scepticism dodges at most 60% of hits. | T |
| STATUS-01 | The slop can give you Anxiety, Doomscroll, Pop-up, Ratio'd and Brainrot. Items and heroes can prevent them. | T |
| ACT-01 | Actives fire on their own when their trigger is met: Log Off, Block Button, Digital Detox, Unsubscribe, Phone a Friend. | T |
| HERO-01 | Eight heroes, all unlocked, each with a starting weapon and a starting item. | T |
| HERO-02 | Each hero bends a rule (a banned weapon type, fewer slots, a start level, Clout as damage, more dashes...), and the hero screen lists every property, good in green and bad in red. | T, D |

## Weapons and damage

| ID | Requirement | Verified by |
| --- | --- | --- |
| WPN-01 | Every weapon attacks on its own and damages slop; at least four of each damage type. | T |
| WPN-02 | Melee swings hit only slop in front. | T |
| WPN-03 | The Eye-Roll bounces and Air Quotes chain between slop. | T |
| WPN-04 | Damage is the weapon's damage times its level, Conviction, type bonuses, the slop's resistance and marks. | T |
| WPN-05 | Bricks splash where they land; the Ban Hammer deletes small slop outright. | T |
| WPN-06 | The Fact-Check Flashlight strips beauty filters. | T |
| WPN-07 | The Frisbee comes back; the Uno Reverse Card turns bubbles around at the slop. | T |
| WPN-08 | Seed Bombs heal you inside their patch; the Dog Walk's dog circles you. | T |
| WPN-09 | Weapons are drawn around you and aim, kick, swing and throw, each with its own effect. | D |
| WPN-10 | Weapons have levels 1 to 8: more damage every level, an extra projectile at 3 and 8, more area at 5, faster at 7. | T |
| WPN-11 | 33 base weapons, including the Rubber Duck, Office Memo, Sticky Notes, Side-Eye, Ratio, Sunflower Patch, Leaf Blower, Firewall, Bubble Wrap, Fidget Spinner and Vuvuzela. | T |
| WPN-12 | 15 merges: two weapons that belong together, both at level 8, become one merged weapon when you open a chest; the merge frees a slot. Merged weapons are never offered on a level-up. | T, D |
| DMG-01 | Four damage types: Analog, Fact, Sarcasm, Grass; the slop resists them as it learns. | T |
| DMG-02 | Each damage type does something besides damage: Analog staggers and knocks back harder, Fact cuts through filters, shields and Verified, Sarcasm can confuse (the slop turns around and stops attacking), Grass roots. | T |
| DMG-03 | Set bonuses: two weapons of a type give +10% damage of that type; four make its effect stronger. | T |

## Slop

| ID | Requirement | Verified by |
| --- | --- | --- |
| SLOP-01 | Every enemy is a character: five parody chatbots and platform slop. ChadGPT fires fans of three bubbles; only the middle one talks. | T, D |
| SLOP-02 | Six-Finger Hands grab you, slow you and hurt you while holding on. | T |
| SLOP-03 | Listicle Drones come in numbered packs; killing number 1 first scatters the pack. | T |
| SLOP-04 | Yes-Bots buff nearby slop; Grandma's Cake heals it. | T |
| SLOP-05 | Traits work: splitting, reposting, dodging, exploding, beauty filters. | T |
| SLOP-06 | Elites are bigger and tougher and drop a chest. | T |
| ELITE-01 | An elite arrives about every 50 s with an affix: Verified, Viral, Sponsored, Trending or Toxic; its name and label say which. | T, D |
| BRAIN-01 | Brainrot is built from a body, an object and an accessory (ten of each); each part sets its look and one ability. | T |
| BRAIN-02 | Every brainrot gets a fake-Italian name from its parts and shouts it when it appears. | T, D |
| BRAIN-03 | Brainrot is drawn from its three parts, so evolution changes what you see. | D |
| BUB-01 | Your projectiles pop speech bubbles they pass through. | T |
| LIKE-01 | Dead slop drops Likes (experience); Likes within Curiosity fly to you. Hallucinator fakes explode into Anxiety. | T |
| LOOT-01 | Pickups: Touch Grass heals, Algorithm Boost pulls in every Like, Mass Report deletes the slop on screen, Server Outage freezes the slop, Clout coins pay for machines, chests open upgrades or a merge. | T |

## Bosses and platforms

| ID | Requirement | Verified by |
| --- | --- | --- |
| BOSS-01 | Seven bosses each fight with their own puzzle and can be beaten; each drops a chest. | T |
| BOSS-07 | The Model regenerates once unless your kills were spread evenly over damage types (Model Collapse); it copies your most-used weapons. | T |
| PLAT-01 | The Feed visits all six parody platforms in a shuffled order; each sets the floor, the local slop and its boss. | T, D |
| PLAT-02 | Every platform has its hazard: chain letters, Open-to-Work banners, mid-roll ads, downvote zones, filter zones, the beat. | T |

## Fairness

| ID | Requirement | Verified by |
| --- | --- | --- |
| FAIR-01 | Slop only shoots when it is near you, roughly on screen. | T |
| FAIR-02 | Spawns, em-dashes, bombs and maintenance fences are telegraphed before they happen. | T, D |
| FAIR-03 | After a hit you are invulnerable for half a second, and your sprite blinks. | T, D |
| FAIR-04 | Everything that hurts you is outlined in hot pink, and nothing else uses that colour. | I, D |

## The run

| ID | Requirement | Verified by |
| --- | --- | --- |
| RUN-01 | A run is one endless trip through the Feed on one world: no waves, no shop. It ends only in Burnout. | T |
| RUN-02 | Your hero, weapons, items, level-ups and blessings reach the world as soon as you pick them. | T |
| RUN-10 | Difficulty is a function of time: slop HP, damage and the spawn rate keep climbing; early slop is fragile and comes in crowds. | T |
| RUN-11 | About 4:30 into each platform its boss arrives; deleting it moves the Feed to the next platform. After the sixth comes The Model; after it, the Feed goes on. | T |
| WAVE-02 | Spawns follow the director's rate; elites appear at random too. | T |
| WAVE-04 | Platform house rules shape the spawn table (support, shooters, tanks, reposts, all brainrot). | T |
| LEVEL-01 | Likes are experience; the first levels take seconds, later ones about a minute. | T |
| LEVEL-02 | A level-up offers three different choices (more with the right items); every one is valid: no new weapons with six weapons, no level for a weapon at level 8, no new items with six items, no level for a maxed item. If nothing is left, Touch Grass and Brand Deal fill in. One free reroll per level-up, then rerolls cost Clout. | T |
| CHEST-01 | A chest merges two ready weapons; otherwise it gives one to three upgrades (three from a boss), and Clout. | T |
| ITEM-01 | More than fifty items in seven kinds; items have levels (up to five, or one for on-off effects and legendaries); you carry six. | T |
| ITEM-02 | Side hustles pay Clout every minute; so does the Creator Fund. | T |
| EVO-01 | Every two minutes the base model's resistances move towards your kill share, zero-sum and capped. | T |
| EVO-02 | Brainrot that hurt you and lived long breeds; children mutate their numbers and sometimes a part. | T |
| EVO-03 | After each retraining the slop says what it learned in a one-line toast; the resistance tooltips quote it. Nothing stops the game for it. | T, D |
| EVO-04 | Model Collapse measures how evenly your kills spread over the four damage types. | T |
| BAL-01 | Hard but manageable: the heuristic lab bot's median run lasts 15 to 30 minutes, and it beats The Model in 10% to 40% of runs. | L |
| BAL-02 | Random play dies early: the random lab bot's median run is under 10 minutes. | L |

## Version 4: "The Algorithm Update"

| ID | Requirement | Verified by |
| --- | --- | --- |
| DASH-01 | You dash with Space, Shift or A/RB/RT: about 200 units, untouchable, and grabbing hands let go. Everyone has two charges that come back in 3 s; items change the count and the time. | T, D |
| DASH-02 | Items change what a dash does: it can hit slop, leave fire, end in a shockwave, or leave you untouchable a moment longer; Flame War leaves a burning trail as you walk. | T |
| DEF-01 | Boundaries: a shield stat that takes hits before Mental Health and comes back after 3 s without a hit; the HUD shows it over Mental Health. | T, D |
| DEF-02 | Barrier weapons circle you, stop enemy shots and hurt slop they touch. | T, D |
| ITEM-03 | Items have four rarities (common, rare, epic, legendary) with frames in grey, blue, purple and gold. Legendaries are offered rarely, more often with Luck, are one of a kind, and break rules (more projectiles, exploding kills, a second you, screen wipes, God Mode). | T, D |
| GLITCH-01 | Glitches: rare perks that change a run. Every boss's chest is followed by a pick of three; now and then one takes a level-up card. | T, D |
| LOOT-02 | Chests open one, three or five upgrades, each spinning like a slot reel and landing rarest last, with a fanfare for a legendary. Elites with a chest sometimes drop a golden one (rare early, up to one in eight later; five upgrades, always a legendary) under a pillar of light; each card says what the upgrade does; chests off screen get an arrow at the screen's edge. | T, D |
| LOOT-03 | Clout drops as coins, stacks and bags; Algorithm Boost pulls every Like and coin on the map. | T, D |
| STAT-02 | Hustle, a stat: Clout every minute, and coins drop more often. | T |
| BIOME-01 | The map is a patchwork of the six platforms in squares of 6144 units; neighbours always differ. Walking into one changes the floor, the local slop and the hazard, once you are properly inside; the floor and the HUD show where you are, without an announcement. Bosses keep their order. | T, D |
| EVENT-01 | Swarms cross the screen in a line and are gone; rings of Recruiter Bots close in. Neither is announced in words. | T, D |
| EVENT-02 | Livestreams: after a warning a ring holds you in with a growing crowd for 28 s, then pays a big chest. They come on their own and from Go Live ring lights on the map. | T, D |
| ICON-01 | Twelve brainrot icons, parodies of the viral AI characters, each with its own trick: bonks, charges, blinks, needles, stomps, peels, shards, flying bombers, turrets. They join every platform as the Feed goes on. | T, D |
| ICON-02 | The icons are animated in four frames, and every slop waddles and squashes as it walks. | D |
| STATUS-02 | Brainrot only comes from something you can see: a brainrot zone you stood in when the beat dropped, or the Supreme Brainrot's warning. It slows you; the Mental Health panel lists it. | T, D |
| UI-18 | Print Screen saves a screenshot like F12, read from the key's physical state so a screenshot tool that grabs the key cannot hide it. Start on a gamepad only pauses and resumes; B resumes too. | D |
| UI-19 | Reroll sits below the build, out of the arrow-key path between cards; R, Y or a click rerolls. | D |
| UI-20 | An Xbox controller keeps playing while the game window is in the background (XInput), and the game keeps running at full speed there. | D |
| FAIR-05 | Nothing takes control away or covers the Feed: no reversed controls, no drags or shoves, no screen-covering pop-ups or ads; such effects slow you instead. Effects on you are shown on you (an aura, a bubble, a flash), not announced in words. | T, D |
| ELITE-02 | Elites come every 30 s, more of them later, with seven affixes (Summoner and Hulking added); each glows in its affix's colour. Fewer than half carry a chest; split copies never do. | T, D |
| EVENT-03 | The crowd surges: calm, build-up, a burst of melee slop, a lull, every 75 s, and every fourth burst is a horde. | T |
| CHEST-02 | Once nothing can be upgraded, chests pay Clout and a heal without stopping the game, and level-ups offer small stat-ups. | T |
| MAP-05 | One or two things to use per chunk; new: recycle bins (throw an item away for Clout), fine-tuning stations (buy a weapon level), data brokers (sell Mental Health for a rare item), power outlets (dashes refill, run faster). | T, D |
| LOOT-04 | The loot goblin: now and then a Crypto Bro runs off with a bag of Clout; catch him for coins and a chest, or he gets away. | T, D |

## Version 5: "You are the training data"

The direction is in [direction.md](direction.md).

| ID | Requirement | Verified by |
| --- | --- | --- |
| MODEL-01 | What the model has learned is visible: the HUD says what it resists and what it is weak to, and weapon cards on level-ups say what that means for that weapon. | D |
| TEMP-01 | Temperature is the difficulty, chosen on the hero screen (0.2 for a first run): 0.2 is calm with no surges; hotter means more slop that hits harder, more Likes and Clout, and hallucinated Likes. | T, D |
| INJECT-01 | Prompt Injection: a rare pickup; for 6 s the slop on screen stop attacking you and tear at each other. Bosses are not fooled. | T, D |
| CARD-01 | The end screen is a model card written from the run: biases, what the model learned, what deleted you. | T, D |
| NEMESIS-01 | Between runs the slop remember you: the next run starts ahead on your favourite damage type, and what deleted you comes back by name as an elite that drops a golden chest. Memory is kept in a small text file; autoplay and the lab never touch it. | T, D |

## Version 6: "Automated"

The spec is [design-v6.md](design-v6.md), the plan [plan-v6.md](plan-v6.md).

| ID | Requirement | Verified by |
| --- | --- | --- |
| CH-01 | A run is five chapters in order (the Feed, the Office, the Boardroom, the Campaign, the Resistance); death in one starts the next with a fresh build, carrying Automation, the agents, the glitches and what the model learned; the resource is renamed and recoloured per chapter. | T, D |
| CH-02 | Each chapter has its own six biomes, roster, hazards, boss at a fixed time, lines and patch-note tone. Bosses: Your Replacement closes tickets to heal; The Board needs a quorum; The Candidate's promises land ahead of you; The State is the Model governing, with your agents. | T, D |
| CAP-01 | Capability (the version on the HUD) climbs every 30 s and scales the slop; it climbs faster with Automation; later chapters start ahead; it never stops. | T |
| AUTO-01 | Delegate cards are offered on level-ups, are the strongest option by the bot's own evaluation, and raise Automation as stated. | T |
| AUTO-02 | Agent weapons level themselves and cannot be levelled, merged or recycled by you; they carry across chapters; in the Resistance they defect and fire your weapons back. | T |
| AUTO-03 | At 25 a card is suggested and rerolls are gone; at 50 level-ups pick themselves after 2 s and the dash is handled; at 75 the model drives for part of every minute; at 100 you are automated. Each is said once. | T |
| AUTO-04 | The Unplug glitch (offered once Automation is 25) stops Automation for the chapter, fires the agents, and halves the resource. | T |
| END-01 | At Automation 100 or death in the Resistance, the bot plays your character for 20 s, then the model card with humanity retained and the score; the best score is kept; the title demo replays your last build. | T, D |
| LESSON-01 | Every archetype has a lesson; the chapter card and the model card name what deleted you and the lesson; the last frame holds for two seconds with the killer named. | T, D |
| MUSIC-01 | Each chapter has its own generated loop, crossfaded on change; Automation quantizes it, until at 100% the grid plays itself. | T, D |
| ASSET-01 | Files in `assets/` (sprites, music, sfx) replace drawn sprites, generated music and synthesised sounds; the game runs with none. | T |

## Client

| ID | Requirement | Verified by |
| --- | --- | --- |
| UI-01 | The client runs the Feed at 60 ticks per second; P, Esc or Start pauses. | D |
| UI-02 | The camera stays on you; floors are dark so characters and danger stand out. | D |
| UI-03 | Slop show a health bar for three seconds after being hit. | D |
| UI-04 | Damage numbers appear in the colour of their damage type. | D |
| UI-05 | The HUD shows Mental Health with statuses, the run clock and platform, a countdown to the boss (then its health), Clout, level and kills, experience as a bar along the top edge, and news toasts. | D |
| UI-06 | Autoplay plays a run with the bots and screenshots every kind of screen. | D |
| UI-07 | Resizing the window scales the view and the HUD. | D |
| UI-08 | Title and hero select; every choice works by mouse, keyboard and gamepad. | T, D |
| UI-09 | Level-ups pause the Feed and show wide cards: icon, NEW or the level change, what it does, and a merge hint; your build is shown below. | D |
| UI-11 | Chests, merges, vending machines and shrines each have their own overlay. | D |
| UI-12 | End screen with time survived, level, kills, bosses, kills by damage type and your best weapons. | D |
| UI-13 | Pause shows your whole build with levels, blessings and stats; details of what is highlighted appear in a fixed panel. | T, D |
| UI-14 | Every screen works by arrows/WASD or stick/d-pad to move the focus, Enter or A to press, Space or Start to continue. | T, D |
| UI-15 | Behind the title and hero screens the bots play the Feed, silently. | D |
| UI-16 | The game starts fullscreen; Alt+Enter or F11 switches to a window and back; F12 saves a screenshot to Pictures\AI Slop Arena. | D |
| UI-17 | Gamepads (such as the Xbox 360 controller) work everywhere: left stick or d-pad to move and choose, A to confirm, B to go back, X for the other action, Y to reroll, Start to pause. Hint bars name the gamepad's buttons when it was used last. | D |
| ART-01 | Every sprite is drawn in code with outlines and anti-aliased edges; characters flash white when hit. | T, D |
| ART-02 | Every character, brainrot part and boss has its own picture. | T, D |
| ART-03 | Each platform has its own floor. | D |
| ART-04 | Every weapon, item, stat and damage type has its own icon; merged weapons show their two parents on a gold burst. | T, D |
| ART-05 | Each hero has their own look. | D |
| ART-06 | The Feed has pictures for every pickup, prop, wall and blessing, and faint scraps of feed (posts, thumbnails, comments, hearts, hashtags) tinted per platform, scattered differently in every chunk. | T, D |
| FEEL-01 | Hits, deaths, pops, swings, beams, chains, splashes and statuses have visible effects and screen shake, capped so crowds stay readable. Kill streaks and level-ups are announced. | D |
| FEEL-02 | Placeholder sounds are synthesised in code and rate-limited. | T, D |
| FEEL-03 | The slop talks in character; deleted slop leaves varied removal notices. | T, D |
| FEEL-04 | Everything moving is drawn between simulation ticks, so motion is smooth on any refresh rate, and the camera has no lag. | D |
