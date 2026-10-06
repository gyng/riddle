# Complete item silhouettes — 2026-10-06

All43 current Rust item definitions must resolve to packaged art in itemIcon,
including gold, bones and trap. Reuse the existing gold silhouette; new bones
and trap silhouettes match the ink/bone/mist moonlit painted style, read at
24–40px and remain unframed. No frame, labels or rectangle baked into images.
Authored transparent masters kept in art/ui/icons, packed with the existing
tools/ui-skin.py workflow. Item identity, rarity, labels and behavior unchanged.

Keep glyph fallback for unknown/future items and failed image loads. No potion
identity spoilers: unidentified potions/scrolls retain their family art. Verify
all Rust definitions, packaged paths, alpha/readability/size, old-wire labels,
rarity colors, silhouette bounds and core item surfaces320/400/1440. Show actual
earned item UI and clearly label any all-item diagnostic gallery. Copy/build
and relevant item/Forge/report/death/unit checks. Push approved; no deploy.
Next: earned King/ending and ascension presentation audit.

Implemented: gold reuses the existing coin art; built-in imagegen produced
transparent bones/skull and open iron trap masters, packed to96px with
tools/ui-skin.py. Existing bones concept uses the same new silhouette; no
duplicate marker. Both final masters remain in art/ui/icons/it_{bones,trap}.png,
client PNGs in web/public/ui/icons/, manifests updated. Full prompt set and
generation mode saved at art/prompts/item_silhouettes_20261006.json. Authored
alpha retained, no frames. Unknown item and failed image loads keep primitive
fallbacks without dropping name/kind/rarity. Family art preserves unknown
potion/scroll identities.

All43 Rust definitions require a real image in the client test, rather than
accepting a glyph as success. Item tests now320/400/1440; final114checks each.
Blocked trap request verifies failed-file fallback metadata, label and unframed
bounds. Initial6 affected item/unit/report/death/training jobs pass7.2s; later
ascension+frame4jobs24.6s and5affected report/QA jobs23.2s pass. Final focused
item/fallback2jobs5.2s pass after shared-concept polish. Build/typecheck/copy1616/diff
clean, existing bundle advisory. No Rust/WASM/gameplay edits or balance claim.

Actual earned headed WASM400/1440 recovered-packs report and labelled43-item
diagnostic gallery inspected/shown. Gallery is a reference, not inventory;
46 loaded pictures including the three highlighted subjects, no errors/overflow,
complete save bytes unchanged. Visual review caught two adjacent skull markers;
reused the shared bones concept instead and recaptured. Pack alpha positive
and transparent,96x96,15.5/15.8KB. Source masters copied into project; generated
originals retained. Artifacts scratchpad/item-icons-20261006/.
