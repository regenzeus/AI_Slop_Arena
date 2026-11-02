# Requirements: milestone 1, the arena

Each requirement has an ID that tests reference with `// REQ: ID`. "Verified by" says how it
is checked: **T** an automated test, **L** a lab measurement, **D** a demonstration in the
client (autoplay screenshots or play), **I** inspection of the code.

## Simulation

| ID | Requirement | Verified by |
| --- | --- | --- |
| SIM-01 | The arena is deterministic: the same seed and the same input every tick give the same wave, bit for bit. | T, L |
| SIM-02 | You stay inside the square arena. | T |
| SIM-03 | Movement eases in and out, and diagonal movement is not faster. | T |
| SIM-04 | A crowded arena (400 slop, 300 bubbles, six fast weapons) steps in under 2 ms per tick. | L |

## You

| ID | Requirement | Verified by |
| --- | --- | --- |
| STAT-01 | Ten stats exist with the starting values of the design. Composure multiplies damage taken by 15 / (15 + Composure); Scepticism dodges at most 60% of hits. | T |
| DMG-01 | Damage to slop is the weapon's damage times its tier, Conviction, the slop's resistance to the damage type, and a mark if present. | T |

## Weapons

| ID | Requirement | Verified by |
| --- | --- | --- |
| WPN-01 | Six weapons fire on their own at the nearest slop in range, covering all four damage types: Rock, Stick, Pistol, Red Pen, Eye-Roll, Garden Hose. | T |
| WPN-02 | Melee swings hit only slop in front, within their arc. | T |
| WPN-03 | The Eye-Roll bounces on to further slop. | T |
| WPN-04 | The Red Pen marks slop it hits; marked slop takes more damage from everything. | T |

## Slop

| ID | Requirement | Verified by |
| --- | --- | --- |
| SLOP-01 | Bubble Bots keep their distance and fire fans of three speech bubbles; only the middle one carries a line, so crowds stay readable. | T, D |
| SLOP-02 | Six-Finger Hands grab you, slow you by 20% each and hurt you while holding on. | T |
| SLOP-03 | Listicles come in numbered packs of five; killing number 1 first makes the pack scatter. | T |
| SLOP-04 | Yes-Bots follow other slop and agree with up to three nearby, giving each 10% more damage per agreement, up to three. | T |
| BUB-01 | Your projectiles pop speech bubbles they pass through; some pops drop a Like. | T |

## Fairness and readability

| ID | Requirement | Verified by |
| --- | --- | --- |
| FAIR-01 | Slop only shoots when it is near you, roughly on screen. | T |
| FAIR-02 | Spawns are telegraphed one second ahead, away from you; Em-Dash Lancers show their aim before they throw. | T, D |
| FAIR-03 | After a hit you are invulnerable for half a second, and your sprite blinks. | T, D |
| FAIR-04 | Everything that hurts you is outlined in hot pink, and nothing else uses that colour. | I, D |

## Waves and Likes

| ID | Requirement | Verified by |
| --- | --- | --- |
| WAVE-01 | A wave lasts 60 seconds. Surviving clears it and credits half of the Likes left on the floor; Mental Health at 0 is Burnout and ends it. | T |
| WAVE-02 | Spawns ramp up over the wave and new kinds of slop join over time. | T |
| LIKE-01 | Dead slop drops Likes; Likes within Curiosity fly to you and become Clout. | T |
| BAL-01 | Wave 1 is hard but manageable: the lab bot clears it 70% to 90% of the time. | L |

## Client

| ID | Requirement | Verified by |
| --- | --- | --- |
| UI-01 | The client runs the simulation at its fixed 60 ticks per second from keyboard input; weapons can be switched off with 1 to 6; P pauses. | D |
| UI-02 | A camera follows you with a slight lead and never shows much beyond the arena walls. | D |
| UI-03 | Slop show a health bar for three seconds after being hit. | D |
| UI-04 | Damage numbers appear in the colour of their damage type. | D |
| UI-05 | The HUD shows Mental Health, the wave timer as a video progress bar, Clout and the weapons; title and end screens explain the controls and the result. | D |
| UI-06 | Autoplay plays a wave with the bot and saves screenshots of the title, the wave and the end. | D |
| UI-07 | Resizing the window scales the arena view and the HUD instead of showing more or less of the game. | D |
| ART-01 | Every sprite is drawn in code with outlines and anti-aliased edges; slop flash white when hit. | T, D |
| FEEL-01 | Hits, deaths, pops, swings and sprays have visible effects and screen shake; effects are capped so crowds stay readable. | D |
| FEEL-02 | Placeholder sounds are synthesised in code and rate-limited. | T, D |
