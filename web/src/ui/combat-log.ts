import type { Ev } from "../engine/types";

/** Display the known goblin summon plainly; preserve other mechanics verbatim. */
export function telegraphText(kind: string | undefined, what: string): string {
  return what === "rallies" && (kind === "goblin_warlord" || kind === "goblin_captain") ? /* copy:callout */ "calls goblins" : what;
}

/** The overlay describes an unchanged hit once. Rust's hurt amount may differ
 * after counters/shields, so matching the attack alone is insufficient. */
export function combatLogEvents(events: readonly Ev[], heroId: number, kinds: ReadonlyMap<number, string>): Ev[] {
  const hits = new Map<string, number>();
  const rows: Ev[] = [];
  const key = (tick: number, victim: number, damage: number, cause: string): string => `${tick}:${victim}:${damage}:${cause}`;
  for (const ev of events) {
    if (ev.k === "attack" && ev.hit && ev.dmg > 0) {
      const cause = ev.src === heroId ? "hero" : kinds.get(ev.src);
      if (cause) {
        const k = key(ev.t, ev.dst, ev.dmg, cause);
        hits.set(k, (hits.get(k) ?? 0) + 1);
      }
    } else if (ev.k === "hurt" && ev.dmg > 0) {
      const k = key(ev.t, ev.id, ev.dmg, ev.cause), count = hits.get(k) ?? 0;
      if (count > 0) { hits.set(k, count - 1); continue; }
    }
    if (["attack", "hurt", "heal", "telegraph", "die", "use", "pickup", "rule", "descend", "exit"].includes(ev.k)) rows.push(ev);
  }
  return rows;
}
