// Hero looks: the heir's cosmetic look (`Lineage.look`: male | female | cat). The hero entity draws as `hero_<class>_<look>`; the atlas
// falls back to `hero_<class>`, then the procedural silhouette (art never blocks the game). Module state: the UI sets it from the
// lineage before a viewer loads (`setHeroLook`); a replay or a later floor reads the same look.
export const LOOKS = ["male", "female", "cat"] as const;
export type Look = (typeof LOOKS)[number];
let current = "";
export function setHeroLook(look: string | undefined): void { current = (LOOKS as readonly string[]).includes(look ?? "") ? look! : ""; }
export function heroLook(): string { return current; }
/** `hero_<class>` → `hero_<class>_<look>` under the current look; any other kind unchanged. */
export function heroKind(kind: string, look = current): string { return look && /^hero_[a-z]+$/.test(kind) ? `${kind}_${look}` : kind; }
/** `hero_<class>_<look>` → `hero_<class>` (the look's fallback); null for any other kind. */
export function heroBase(kind: string): string | null { const m = /^(hero_[a-z]+)_(male|female|cat)$/.exec(kind); return m ? m[1]! : null; }
