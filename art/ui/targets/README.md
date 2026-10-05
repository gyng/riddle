# Chunky UI targets

Generated with built-in imagegen on 2026-10-05, using screenshots of the current
app as visual references. These are design targets, not runtime assets.

## Tactics — tactics-chunky-v1.png

Prompt brief: create a desktop Tactics target in the existing painted fantasy
art style. Center parchment in an iron frame above the chunky command bar;
use tactile bevels, icon sockets, a Steady shield/sword icon, engraved empty
tactic slots, personality icons and readable consistent typography. Retain
the moonlit town, hero roster and dungeon backdrop. Reference: current 1440px
Tactics screenshot. Use SC2/WC3-inspired construction with Riddle's own art.

## Hero, Forge, Workers, Settings — windows-chunky-v1.png

Prompt brief: create a 2×2 target sheet for Hero, Forge, Workers and Settings.
Use the same SC2/WC3-inspired chunky window module and existing painted art:
framed parchment, iron title plates, inset sockets, beveled controls, consistent
fonts. Vary accents slightly: Hero gilt, Forge bronze, Workers blue. Keep the
current mechanics and content. References: desktop Hero/Forge and phone
Settings screenshots. No new mechanics or baked UI assets for implementation.

Implementation extends existing panel/tablet nine-slice art and skin.ts rather
than embedding mockup text. Standard-width Fira Sans body text is self-hosted;
compact controls retain Fira Condensed and titles retain Grenze.
Fira Sans source: https://github.com/google/fonts/tree/main/ofl/firasans
License: web/public/fonts/OFL-Fira.txt (SIL Open Font License).
