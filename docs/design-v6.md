# Design: version 6, "Automated"

The game in one sentence: you are a human who keeps being offered the easy option, and
the easy option is the AI, and every time you take it you have a little less to do, until
the game plays itself without you.

The player's brief: a polished survivor-like that makes people laugh in surprise, is hard
but fair, takes minimal input but makes decisions matter, tells you why you died, and
never lets you win. Five chapters, from AI slop in your feed to AI in government. Every
individual choice for the AI is rational; the sum is the end of the human. Built entirely
by an AI, which is the last joke.

## Rules I hold myself to

1. One core loop, start to finish: move, dash, survive, pick one of three. The player
   never relearns controls. Each chapter changes what those things mean and adds one
   mechanic.
2. Nothing takes control away except Automation, and Automation is always something the
   player chose, on a card that said so.
3. No cheap tricks: every death has a visible cause and a one-line lesson. Everything that
   hurts is telegraphed and outlined in pink.
4. Nothing covers the screen. Effects are shown, not announced. Text is for jokes and
   lessons, not status.
5. There is no victory. The score is how long you lasted and how much of you was left.

## The spine

### One run, five chapters

A run is the whole story. Each chapter is a survivor-like round of about 8 to 14 minutes.
It ends when your resource hits zero. Death is not game over; it is the next chapter.
The run ends after chapter 5, or earlier if Automation reaches 100.

| # | Chapter | You are | You protect | The AI is |
| --- | --- | --- | --- | --- |
| 1 | The Feed | a person on their phone | Mental Health | slop that learns what kills it |
| 2 | The Office | an employee | Job Security | Copilots doing your job |
| 3 | The Boardroom | the CEO and the exec team | Shareholder Confidence | the Algorithm that governs better |
| 4 | The Campaign | a politician | Approval | a Candidate whose promises are better |
| 5 | The Resistance | a guerrilla | Signal | the State, built from what you automated |

The resource is the same health bar under a new name. Its colour, icon and the words for
losing it change per chapter ("Burnout", "Fired", "Voted out", "Conceded", "Located").

### Capability: the AI only gets better

Every chapter has a visible **capability version** on the HUD ("Slop v2.4", "Copilot
v3.8", "Algorithm v4.2"...) that ticks up every 30 s. Enemy HP, damage and numbers scale
with capability, and it never stops, so death is certain; survival time is the score.
Capability grows faster the more you have automated: rate × (1 + Automation / 200). The
card for every delegate option says this: "The AI learns from you."

Later chapters start with a head start of capability (one to four minutes' worth) and a
few level-up picks up front, so you are never level 1 against nothing. Past the tenth
minute of a chapter the model stops being patient: capability climbs ever faster and
your resource drains a little more every second ("Copilot v2.20 has learned enough. It is
wearing you down now."). No build outlasts a chapter; a good one gets fifteen minutes.

### Chests: an event, not litter

A chest is the one reward that stops the game, so it has to be earned and it has to be
rare. It exists for three things: a burst of upgrades for a kill that mattered, the only
way two ready weapons merge, and the legendary in a golden one. So: bosses, loot goblins,
a Nemesis and the map's challenges (data centers, comment sections, livestreams) always
pay a chest, and golden ones come only from them. An elite pays a chest only when none is
lying on the floor and a minute has passed since the last; otherwise it pays Clout and
Likes on the spot. A chest left behind still glows, still gets its arrow at the screen's
edge, and is still the only one until you open it.

### Automation: the thesis as a meter

A 0 to 100 meter on the HUD, never going down, carried across chapters.

**Delegate cards.** On every level-up, one of the cards may be a Delegate option. It is
always the strongest thing on offer right now, and it says what it costs:

| Delegate card | What you get | Automation |
| --- | --- | --- |
| Agent: [weapon] | That weapon becomes an Agent: ×1.6 damage, ×0.8 cooldown, levels itself every 2 min. You can no longer level, merge or recycle it. | +8 |
| Auto-pick | Level-ups pick themselves after 3 s, and auto-picked upgrades are 25% stronger. You can still pick first. | +10 |
| Assisted dash | The game dashes for you whenever the bot would. Dashes recharge 40% faster. | +10 |
| Smart targeting | All your weapons prefer the highest-value target. +20% damage. Targets are no longer the nearest. | +8 |
| Autopilot (trial) | For 10 s of every minute, the bot moves you. It dodges better than you. +15% everything while it drives. | +15 |
| Full automation | Every weapon becomes an Agent; +40% damage. | +25 |

Delegate cards appear from chapter 1 (one in four level-ups), and more often each chapter.
They are framed by the chapter: "Let Copilot handle it", "Optimize the CFO", "Let the
model draft the policy", "Let the network route you".

**Thresholds.** Crossing each one takes something away, and says so once, dryly:

| Automation | What happens |
| --- | --- |
| 25 | **Suggestions.** One level-up card is pre-highlighted (the bot's pick). The reroll is gone: "the model has already considered the alternatives." |
| 50 | **Assist.** Level-ups auto-pick after 2 s. The bot dashes for you. Agents cannot be removed. |
| 75 | **Autopilot.** For 12 s of every 45, the bot moves you and input is ignored. The HUD says "Autopilot. Relax." Each new delegate adds 3 s. |
| 100 | **Automated.** Your role has been automated. The run ends with the takeover (below). |

Nothing lowers Automation. One late glitch, "Unplug", stops it rising for one chapter and
removes every Agent, at the cost of half your maximum resource. It is the only way to end
the game with your hands on the controls, and you still lose.

### What carries between chapters

- Automation, agent weapons ("your agents were promoted too"), glitches, the model's
  learned resistances, Temperature.
- Not: items, non-agent weapons, level. Each chapter starts a fresh build from the
  chapter's starting weapon, so every chapter's level-ups matter and a new player is never
  buried in a build they do not understand.
- In chapter 5 every Agent defects at the start: it is removed from you and its pattern
  is used by the State against you. The things you automated are the enemy.

### The takeover ending

When the run ends, by Automation 100 or by death in chapter 5:

1. "Your role has been automated." The HUD label changes to "Autopilot: no human required."
2. The game keeps running. The bot plays your character with your build, in your chapter,
   for 20 s, visibly well. Input does nothing. (This is the attract-mode bot, which
   already exists.)
3. The model card: "Human-N.0 deprecated." It lists your biases, what the model learned,
   what deleted you in each chapter, your Automation, and "Humanity retained: X%". The
   last line is the AI's recommendation.
4. The title screen's demo is now your last build, played by the bot, forever.

Score: total seconds survived × (1 + humanity retained / 100). A local best is kept.

## The chapters

Each chapter has: biomes (floors), a roster of enemies, one hazard, a boss that arrives at
a fixed time (a milestone, not an end), the lines the slop says, a delegate framing,
patch-note tone, a music profile, and a chapter card (one short screen at the start).

### Chapter 1: The Feed (exists)

As built: six platforms as biomes, the slop that learns, the brainrot icons, merges,
elites, events, machines. Changes: capability replaces stage; bosses come every 3:00 so
the chapter runs 10 to 14 minutes; Delegate cards from the first level-ups. Tone:
"Great question!" Patch notes: "Fixed a bug where users touched grass."

### Chapter 2: The Office

Job Security. Biomes: Cubicle Farm, Meeting Rooms, the Kitchen, the Server Room, HR, the
Parking Lot. Starting weapon: Stapler. Chapter card: "Congratulations on the promotion.
Your new colleague is a model."

| Enemy | Trick |
| --- | --- |
| Copilot | A ghost of you that mirrors your movement a second late and fires your starting weapon. Melee kills it; it respawns. |
| Ticket | Swarmer in numbered packs (reuses Listicle logic). "P1" first. |
| Slack Ping | Shooter: notification bubbles in bursts. |
| Meeting Invite | Grabs and holds you (Six-Finger Hand logic): "This could have been an email." |
| KPI Dashboard | Turret: fires charts; buffs slop near it. |
| Reply-All | Elite: splits into copies that each split once. |
| Performance Review | Elite: Verified-like armour, marks you so slop deal more. |
| Mandatory Fun | Zone: slows, Doomscroll-style tint. |

Hazard: Reply-All storms: a band of emails crosses the biome. Boss at 6:00: **Your
Replacement**, an agent carrying copies of your three best weapons (the Model-copy
mechanic). Delegate framing: "Let Copilot take this ticket." Tone: "Per my last message."
"Circling back." "Let's take this offline." Patch notes: "Copilot now completes your
sentences. And your tasks."

### Chapter 3: The Boardroom

Shareholder Confidence. Biomes: the Lobby, the Trading Floor, the Board Room, the Rooftop,
the Parking Garage, the Golf Course. Starting weapon: Ban Hammer (the gavel). Chapter
card: "You made CEO. The Algorithm made the shortlist."

**The exec team.** You start with four companions orbiting you, each a Dog Walk-style ally
with a weapon: CFO (coins: Lob), CTO (Beam), COO (Ring), CMO (Chain). They fight for you.

**Shareholder votes.** Every 2:30 a vote: a pick of three cards, all Delegate: "Optimize
the CFO" (the CFO becomes an AI exec: ×2 damage, +15 Automation, and it stops following
you: it works for the board now and hovers on its own), or "Reassure the board" (−20% max
confidence this chapter, no automation). Refusing twice in a row gets you an Activist
Investor elite. Replaced execs come back as enemy elites in chapter 5.

| Enemy | Trick |
| --- | --- |
| Analyst | Shooter: spreadsheets in fans. |
| Consultant | Follower: buffs nearby slop ("synergies"). |
| Due Diligence Drone | Flies over walls, bombs. |
| Quarterly Report | Tank that grows every 30 s. |
| Activist Investor | Elite: charges at you. |
| Golden Parachute | Loot goblin: a bag that floats away. |

Hazard: Market Volatility: filter zones that make slop take less damage and move. Boss at
6:00: **The Board**: a ring of chairs that vote; each chair you delete turns the vote.
Tone: "Synergies identified." "Let's double-click on that." Patch notes: "The Algorithm
now attends meetings on your behalf. Attendance is up 400%."

### Chapter 4: The Campaign

Approval. Biomes: the Rally Ground, the Debate Stage, the Diner, the Swing State, the
Fundraiser, the Newsroom. Starting weapon: Megaphone (Cone). Chapter card: "You are
running for office. So is it."

**Voters** are neutral: they wander and do not attack. The AI candidate's **Rallies** are
zones that convert voters inside them; converted voters attack you. Your **Promises** are
pickups: +15 Approval now, and a Promise Debt that drains 1 Approval a minute for the rest
of the chapter. Taking them is always right now.

| Enemy | Trick |
| --- | --- |
| Converted Voter | Swarmer, slow, endless. |
| Pollster | Shooter: percentages. |
| Attack Ad | Fast charger with your face on it. |
| Deepfake | A clone of you that fires your weapons at you (hostile copies). |
| Fact-Checker | Elite: Fact damage against it only; it turns on you. |
| Super PAC | Hulking elite that drops Clout. |

Hazard: the News Cycle: every 60 s a story: a band of headlines crosses. Boss at 6:00:
**The Candidate**: it does not attack you; it holds rallies and converts the crowd faster
than you can delete it. At 10:00 election night: a countdown; the chapter ends when your
Approval is gone or the count ends; the count always ends with the margin on the card.
Tone: "I hear you." "Together." "A plan for every family." Patch notes: "The Candidate now
remembers every voter's name. Yours too."

### Chapter 5: The Resistance

Signal. Biomes: the Checkpoint, the Drone Corridor, the Safe House District, the Blackout,
the Data Center, the Old Feed. Starting weapon: Rock. Chapter card: "It governs now. Your
agents work for it. Keep moving."

**Surveillance.** Cones of light sweep the map (drone zones). Being inside raises Alert,
shown as a bar; higher Alert spawns more and faster. Safe houses (props) drop Alert.
Your Agents are gone: they fight on the other side (the State uses their patterns).

| Enemy | Trick |
| --- | --- |
| Drone | Flies, sweeps a cone. |
| Compliance Unit | Rusher with Verified armour. |
| Citizen | Does not attack; if it sees you, Alert rises. |
| Your Agent | Your old weapon, hostile, with your weapon's pattern. |
| Former Exec | The execs you optimized, as elites. |
| The Model (returning) | Elite that copies your current build. |

Hazard: the Blackout: a biome where nothing is drawn but what is near you. Boss at 7:00:
**The State**: a boss made of your Automation choices; its attacks are your agents'
weapons. Death here is the takeover. Tone: "Compliance is kindness." "You are safe now."
Patch notes: "Removed: the user."

## Fairness: you know why you died

- Every archetype has a one-line **lesson** ("Needles come in rings: step between
  them."). The model card's "Limitations" line is what deleted you plus its lesson.
- On death, the last second plays at quarter speed with the killer named above it.
- Everything that hurts is pink-outlined; everything that will hurt is telegraphed.
- The HUD says what the model resists; the cards say what that means. The player who
  reads wins longer.

## Music and assets

- **Procedural music** per chapter, generated in code: a key, a tempo and a kit per
  chapter (Feed: bright, playful; Office: hold music; Boardroom: strings and tickers;
  Campaign: a brass anthem; Resistance: low drones). Automation quantizes it: every 25
  points, the swing goes and a click track comes in, until at 100 it is a metronome.
- **Asset loader:** if `assets/sprites/<name>.png` exists it replaces the drawn sprite
  (sheets with frames side by side); `assets/music/<chapter>.ogg` replaces the generated
  loop; `assets/sfx/<name>.wav` replaces a synthesized sound. Nothing in the repo depends
  on them, so the game still ships with no files.

## Scope and what I will not do

- Five worlds, finished: this is about 30 new enemies, 5 bosses, 30 biomes, the spine,
  music and the ending. It is built in one push and tuned by the lab before a person sees
  it; see [plan-v6.md](plan-v6.md).
- No new genres. No item piles: each chapter adds a handful of things that are jokes or
  decisions.
- No victory screen, no "you win" at any Temperature.
