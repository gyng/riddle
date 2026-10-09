// Cut 117 §5 (blind 8cf9050 A: "each check-in needed 5 to 10 sheet taps — forge, legacy, tactics, workers, supplies, unlocks"): the
// routine part of a return folds into one camp chip. Routine is what has no alternative to weigh: a forge ladder's next step with no
// branch to choose (the gun backup aside) and a worker whose hire is lit and paid for. Decisions stay where they are: a forge tier's
// two branches, Legacy, unlocks, a tactic or its level, the return's pick. Two taps (`take 3` → `ok $940`); nothing on its own.
import type { App } from "../app";
import type { Lineage } from "../engine/types";
import { h, replace, toast, twoTap } from "./dom";
import { audio } from "../audio";

export type Routine = { kind: "kit" | "hire"; id: string; price: number; label: string };

/** The routine buys the purse covers now (kit steps first, then hires — each priced, in the order they are taken). */
export function routineItems(L: Pick<Lineage, "kit" | "tree" | "gold" | "live">): Routine[] {
  if (L.live) return [];
  const kit: Routine[] = (L.kit ?? []).filter((k) => k.slot !== "gun_sidearm" && k.next?.affordable && !k.branches?.length)
    .map((k) => ({ kind: "kit", id: k.slot, price: k.next!.price, label: k.next!.label }));
  const hires: Routine[] = (L.tree?.nodes ?? []).filter((n) => n.kind === "worker" && n.state === "lit" && n.affordable !== false)
    .map((n) => ({ kind: "hire", id: n.id, price: n.price ?? 0, label: n.name }));
  // (what the purse covers after the kit: the chest pays a hire's rest, the core refuses what it cannot pay)
  return [...kit, ...hires];
}

/** Takes the routine buys in order (each re-read: an earlier buy may have spent what the next needed); how many landed. Cut 118 §6: the
 *  report's `collect & send` takes the same batch. */
export async function takeRoutine(app: App): Promise<number> {
  let n = 0;
  for (const it of routineItems(app.lineage)) {
    const now = routineItems(app.lineage).find((x) => x.kind === it.kind && x.id === it.id);
    if (!now) continue;
    const ok = it.kind === "kit" ? !!app.engine.buyKit && await app.mutate(() => app.engine.buyKit!(it.id), undefined)
      : !!app.engine.hire && await app.mutate(() => app.engine.hire!(it.id));
    if (ok) n++;
  }
  return n;
}

/** The camp's chip: shown while two or more routine buys wait; its second tap takes them in order and says how many landed. */
export function checkinBatch(app: App): { el: HTMLElement; dispose: () => void } {
  const el = h("div", { class: "checkin-batch", "data-checkin": "" });
  let busy = false;
  const take = async (): Promise<void> => {
    if (busy) return;
    busy = true;
    let n = 0;
    try { n = await takeRoutine(app); } finally { busy = false; }
    if (n) audio.cue("unlock");
    toast(n ? /* copy:callout */ `${n} taken` : /* copy:callout */ "nothing taken");
    paint();
  };
  const paint = (): void => {
    const items = routineItems(app.lineage);
    const show = items.length >= 2 && (!!app.engine.buyKit || !!app.engine.hire);
    el.hidden = !show;
    el.dataset.checkin = String(show ? items.length : 0);
    if (!show) { replace(el); return; }
    const total = items.reduce((s, x) => s + x.price, 0);
    const chip = twoTap(/* copy:button */ `take ${items.length}`, /* copy:button */ `ok $${total}`, () => void take(), { class: "chip checkin-take num", key: "checkin-take" });
    chip.title = items.map((x) => `${x.label} $${x.price}`).join(" · ");
    replace(el, chip);
  };
  paint();
  const off = app.onChange(paint);
  return { el, dispose: off };
}
