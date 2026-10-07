# Cut 83 — numbered pack recommendation icons

Earned seed27 QA after the eight-hour return: the D12 ogre death recommends
pack 4, but its item icon is a generic diamond. The existing loadout artwork
already represents packs; numbered display labels missed that mapping.

Normalize numbered pack labels for icon lookup only. Preserve the displayed
capacity, label, rarity and primitive fallback. No core, price or save changes.

Gates: existing catalogue icon checks cover pack, pack 4 and pack 12 at
320/400/1440; all prior catalogue, fallback and layout assertions remain.
TypeScript, copy lint and production build pass. No deployment or fun score.
