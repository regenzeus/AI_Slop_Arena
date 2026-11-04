# Implementation plan: version 6, "Automated"

The spec is [design-v6.md](design-v6.md). Everything is built before a person plays it;
the lab bots and autoplay screenshots stand in for the playtest until then. Order is by
dependency, not by chapter: the spine first, so every chapter is built on it.

## Workstreams

### A. The spine (slop_rules, slop_sim)

1. `Chapter` enum and `ChapterDef` table: name, resource name and colour, death word,
   starting weapon, biome list, roster, hazard, boss and its time, lines, delegate
   framing, patch-note tone, music profile, card text. One data module per chapter.
2. Capability replaces stage: `capability(seconds, automation)`; HUD version string.
3. Automation meter on `Run`; `Mod::Automate(n)`; delegate items as `ItemKind::Delegate`
   with the six cards; agent weapons (`OwnedWeapon.agent`), self-levelling, excluded from
   level, merge and recycle.
4. Thresholds: suggestions (pre-highlighted card, reroll gone), assist (auto-pick timer,
   bot dash), autopilot windows (`Run::autopilot()` tells the client to use bot input),
   automated (ending).
5. Chapter transitions: death in chapter N starts chapter N+1 with the carry rules;
   `Phase::ChapterCard` between them; the model's memory carries.
6. The takeover ending: `Phase::Takeover { seconds }` then `Phase::Over`; the run keeps
   stepping with bot input during it. Score and local best in memory.txt.
7. Lessons: `ArchetypeDef.lesson` for every archetype; the card uses it.

### B. World generalisation (slop_sim)

1. Biomes per chapter: `Platform` becomes `Biome` with chapter-specific sets; floors,
   tints, names, hazards keyed by biome.
2. New movements and attacks as needed: Mirror (Copilot), Wander (voters), Sweep cones
   (drones), Convert zones (rallies), Alert.
3. Allies: `Slop` with `friendly: true` that targets slop and is targeted by it (the exec
   team); orbiting like the dog, with a weapon pattern.
4. Hostile copies of your weapons for Deepfakes, Your Replacement, the State: the Model's
   copy mechanic generalised to any spec.

### C. Chapters (content as data + a few behaviours each)

Office, Boardroom, Campaign, Resistance: rosters, bosses, hazards, props, lines. Each
chapter's boss is a `BossKind` with its own think function.

### D. Client (slop_game)

1. Chapter card screen; the resource bar relabels and recolours; capability and
   Automation on the HUD; the autopilot label; threshold notices as one dry line.
2. Delegate cards styled (a distinct frame, the automation cost in red).
3. The takeover sequence and the final model card; the title demo uses the last build.
4. Art: every new archetype, boss, biome floor, prop and pickup, drawn in code with
   frames where they move.
5. Death slow-motion with the killer named.
6. Procedural music: a loop generator per chapter and automation level, played looped,
   crossfaded on chapter change. Asset loader for sprites, music and sfx overrides.

### E. Verification

- Tests per requirement (CH-, CAP-, AUTO-, END-, LESSON-, MUSIC-, ASSET-).
- Lab: `slop_lab story [runs]` plays whole runs across chapters and prints minutes per
  chapter, automation at the end, and causes of death. Target at Temperature 0.7: the
  heuristic bot lasts 8 to 14 minutes per chapter; the random bot dies sooner in every
  chapter.
- Autoplay screenshots of every chapter and the ending at 1600x900 and 1920x1080.
- Determinism across the whole run; the benchmark under 2 ms.

## Order of work

1. A1 to A4 with chapter 1 only (playable end to end with Automation).
2. A5, A6, D1 to D3, D5: the full run with chapter 1 repeated as a placeholder.
3. B, then C and D4 chapter by chapter: Office, Boardroom, Campaign, Resistance.
4. D6 music and assets.
5. E throughout; a tuning pass at the end.

## Requirements added (IDs for tests)

| ID | Requirement |
| --- | --- |
| CH-01 | A run is five chapters in order; death in one starts the next; the resource is renamed and recoloured per chapter. |
| CH-02 | Each chapter has its own biomes, roster, hazard, boss at a fixed time, lines and patch-note tone. |
| CAP-01 | Capability ticks up every 30 s and scales the slop; it grows faster with Automation; it never stops. |
| AUTO-01 | Delegate cards are offered, are the strongest option by the bot's own evaluation, and raise Automation as stated. |
| AUTO-02 | Agent weapons level themselves and cannot be levelled, merged or recycled by you; they carry across chapters; in chapter 5 they defect. |
| AUTO-03 | Thresholds at 25, 50, 75 and 100 take what the spec says, and say so once. |
| AUTO-04 | Unplug stops Automation for a chapter and removes agents at half the resource. |
| END-01 | At Automation 100 or death in chapter 5, the bot plays your character for 20 s, then the final model card with humanity retained and the score; the title demo replays your build. |
| LESSON-01 | Every archetype has a lesson; the card names what deleted you and the lesson; the last second plays slowly with the killer named. |
| MUSIC-01 | Each chapter has its own generated loop; Automation quantizes it. |
| ASSET-01 | Files in `assets/` replace drawn sprites, generated music and synthesized sounds; the game runs with none. |
