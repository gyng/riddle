// Cut 119 (docs/CUT119_COMPANIONS.md): the client half of the core's `pets.rs` — a pet's role first on its card (a small chunky
// glyph), its name with its generation (`Rook III`, the core's), its level and an XP bar, the L3/L5 signatures on hover, `lame N`,
// the heirs it served and the old hound's mark, its grudge; the death's `Rook brought $47`; the report's and the crier's pet lines,
// condensed (one fetch line a pet, one old-hound line); the kennel keeper's order words. All truth is the core's (`Companion.life`,
// `Lineage.pets`, `ReturnReport.feats`); research/PETS_2026-10.md: attachment is a name, deeds and one quirk — so the card's face
// stays short and the rest is a tooltip. Nothing here asks for a tap.
import "../pets.css";
import type { Companion, Death, FeatNews, Lineage, ReturnReport } from "../engine/types";
import { h } from "./dom";
import { detailHost } from "./tips";
import { bossName } from "./report-bosses";

export type Role = "fetcher" | "guard" | "scout" | "mender";
/** The roles' signatures at L3 and L5 (display words; the core's `Role::signatures`). */
/* copy:callout */
const SIGS: Record<string, [string, string]> = { fetcher: ["carry more", "bring back"], guard: ["taunt", "bulwark"], scout: ["map", "warn"], mender: ["patch", "revive once"] };
/** What each role does, for its tooltip. */
/* copy:tooltip */
const ROLE_GLOSS: Record<string, string> = { fetcher: "brings loot in reach", guard: "draws blows aimed at him", scout: "sees the floor ahead", mender: "heals him between fights" };
/** The xp each level starts at, L1 … L7 (display only: the core's `pets::LEVEL_XP`; the core owns the level itself). */
const LEVEL_XP = [0, 500, 1200, 2400, 4000, 6000, 9000];
const CAP = 7;

/** The pet's role, the core's (`life.role`); absent on a pre-Cut 119 pet. */
export const roleOf = (c: Companion): string => c.life?.role ?? "";

/** A role's glyph: a 7×7 chunky pixel primitive (no sprite to fail) — the sack, the shield, the eye, the heart. */
const GLYPH: Record<string, string> = /* copy:none */ {
  fetcher: '<rect x="2" y="0" width="3" height="1" fill="#8a6a3a"/><rect x="3" y="1" width="1" height="1" fill="#5a4426"/><rect x="1" y="2" width="5" height="4" fill="#b08850"/><rect x="0" y="3" width="1" height="2" fill="#b08850"/><rect x="6" y="3" width="1" height="2" fill="#8a6a3a"/><rect x="2" y="6" width="3" height="1" fill="#8a6a3a"/><rect x="3" y="3" width="1" height="2" fill="#f0cf6a"/>',
  guard: '<rect x="0" y="0" width="7" height="1" fill="#9fb2cf"/><rect x="0" y="1" width="7" height="3" fill="#6f86ab"/><rect x="1" y="4" width="5" height="1" fill="#6f86ab"/><rect x="2" y="5" width="3" height="1" fill="#55698c"/><rect x="3" y="6" width="1" height="1" fill="#55698c"/><rect x="3" y="1" width="1" height="4" fill="#dbe4f2"/>',
  scout: '<rect x="2" y="1" width="3" height="1" fill="#eadfc5"/><rect x="1" y="2" width="5" height="3" fill="#eadfc5"/><rect x="0" y="3" width="7" height="1" fill="#eadfc5"/><rect x="2" y="5" width="3" height="1" fill="#eadfc5"/><rect x="3" y="2" width="1" height="3" fill="#3a6a9a"/><rect x="2" y="3" width="3" height="1" fill="#3a6a9a"/><rect x="3" y="3" width="1" height="1" fill="#141c2b"/>',
  tame: '<rect x="2" y="0" width="3" height="1" fill="#8a6a3a"/><rect x="1" y="1" width="1" height="1" fill="#8a6a3a"/><rect x="5" y="1" width="1" height="1" fill="#8a6a3a"/><rect x="0" y="2" width="1" height="3" fill="#8a6a3a"/><rect x="6" y="2" width="1" height="3" fill="#8a6a3a"/><rect x="1" y="5" width="1" height="1" fill="#8a6a3a"/><rect x="5" y="5" width="1" height="1" fill="#8a6a3a"/><rect x="2" y="6" width="3" height="1" fill="#8a6a3a"/><rect x="3" y="5" width="1" height="2" fill="#f0cf6a"/>',
  mender: '<rect x="1" y="1" width="2" height="1" fill="#e07a8c"/><rect x="4" y="1" width="2" height="1" fill="#e07a8c"/><rect x="0" y="2" width="7" height="2" fill="#e07a8c"/><rect x="1" y="4" width="5" height="1" fill="#c75468"/><rect x="2" y="5" width="3" height="1" fill="#c75468"/><rect x="3" y="6" width="1" height="1" fill="#a33f52"/><rect x="1" y="2" width="1" height="1" fill="#f6c2cc"/>',
};
export function roleGlyph(role: string): HTMLElement {
  const g = h("span", { class: "pet-role-glyph", "aria-hidden": "true" });
  const body = GLYPH[role];
  if (body) g.innerHTML = /* copy:none */ `<svg viewBox="0 0 7 7" shape-rendering="crispEdges" xmlns="http://www.w3.org/2000/svg">${body}</svg>`;
  return g;
}

/** The role, first on the card: its glyph and word (`guard`); nothing for a pre-Cut 119 pet. */
export function roleBadge(c: Companion): HTMLElement | null {
  const r = roleOf(c);
  if (!r) return null;
  return h("span", { class: `pet-role role-${r}`, "data-role": r }, roleGlyph(r), r);
}

/** A level's progress, 0..1 (full at the cap). */
export function xpFrac(c: Companion): number {
  const xp = c.life?.xp ?? 0, lv = Math.max(1, Math.min(CAP, c.level));
  if (lv >= CAP) return 1;
  const lo = LEVEL_XP[lv - 1], hi = LEVEL_XP[lv];
  return Math.max(0, Math.min(1, (xp - lo) / (hi - lo)));
}
/** The XP bar under the name: thin, quiet; `xp 974/1200` on hover. */
export function xpBar(c: Companion): HTMLElement | null {
  if (!c.life?.role) return null;
  const lv = Math.max(1, Math.min(CAP, c.level)), xp = c.life.xp ?? 0;
  const bar = h("span", { class: "pet-xp", "data-xp": xp, title: lv >= CAP ? /* copy:tooltip */ `xp ${xp} · max level` : /* copy:tooltip */ `xp ${xp}/${LEVEL_XP[lv]} · L${lv + 1}` },
    h("span", { class: "pet-xp-fill", style: `width:${Math.round(xpFrac(c) * 100)}%` }));
  return bar;
}

/** The line's old hound (served three heirs): the core's chronicle line names it, else the heirs it served. */
export const oldHound = (L: Pick<Lineage, "pets">, c: Companion): string | undefined =>
  L.pets?.old_hounds?.find((l) => l.startsWith(`${c.name} ·`)) ?? ((c.life?.heirs?.length ?? 0) >= 3 ? /* copy:callout */ `${c.name} · old hound` : undefined);

/** A boss's short title for the grudge (`King`): the siege's, the wall's, else his kind. */
export function bossTitle(L: Pick<Lineage, "feats" | "walls">, boss: string): string {
  return L.feats?.siege.find((s) => s.boss === boss)?.title ?? L.walls?.find((w) => w.boss === boss)?.title ?? bossName(boss);
}

/** The card's quiet meta line: `lame 2 · 4 heirs · grudge King` (each only when it says something). */
export function petMeta(L: Pick<Lineage, "feats" | "walls" | "pets">, c: Companion): HTMLElement | null {
  const life = c.life;
  if (!life?.role) return null;
  const heirs = life.heirs?.length ?? 0, hound = oldHound(L, c);
  const bits: HTMLElement[] = [];
  if (life.lame) bits.push(h("span", { class: "pet-lame", "data-lame": life.lame, title: /* copy:tooltip */ "sits out a few runs · level kept" }, /* copy:callout */ `lame ${life.lame}`));
  if (heirs > 0) bits.push(h("span", { class: `pet-tenure${hound ? " hound" : ""}`, "data-heirs": heirs, title: hound }, hound ? /* copy:callout */ "old hound" : heirs === 1 ? /* copy:callout */ "1 heir" : /* copy:callout */ `${heirs} heirs`));
  if (life.grudge) bits.push(h("span", { class: "pet-grudge", "data-grudge": life.grudge, title: /* copy:tooltip */ "bites harder on him until he falls" }, /* copy:callout */ `grudge ${bossTitle(L, life.grudge)}`));
  if (!bits.length) return null;
  return h("span", { class: "pet-meta num" }, ...bits.flatMap((b, i) => (i ? [h("span", { class: "dim" }, " · "), b] : [b])));
}

/** The card's tip: the role's gloss, the signatures (reached lit), the fetched gold, bred or wild. */
export function petTip(el: HTMLElement, L: Pick<Lineage, "feats" | "walls" | "pets">, c: Companion): HTMLElement {
  const r = roleOf(c);
  if (!r) return el;
  return detailHost(el, () => {
    const sig = SIGS[r] ?? ["", ""], life = c.life ?? {};
    return [h("div", { class: "kw-tip-head" }, roleGlyph(r), " ", h("b", null, c.name), h("small", { class: "dim" }, ` · ${c.kind.replace(/_/g, " ")}`)),
      h("div", { class: "num" }, r, ` · ${ROLE_GLOSS[r] ?? ""}`),
      h("div", { class: "num pet-sigs" }, ...[3, 5].flatMap((lv, i) => [i ? " · " : "", h("span", { class: c.level >= lv ? "pet-sig on" : "pet-sig dim", "data-sig": lv }, `L${lv} ${sig[i]}`)])),
      ...(life.fetched ? [h("div", { class: "num" }, /* copy:callout */ `brought $${life.fetched}`)] : []),
      ...(oldHound(L, c) ? [h("div", { class: "num" }, oldHound(L, c)!)] : []),
      h("div", { class: "kw-tip-gloss" }, life.bred ? /* copy:tooltip */ `bred · gen ${c.gen}` : /* copy:tooltip */ "tamed wild")];
  });
}

// ---------------------------------------------------------------- the death: the pet that fetched

const FETCH_RE = /^(.+?) brought (.+?)'s pack(?: · \$(\d+))?\.?$/;
/** The pet that brought the dead heir's pack home, and the gold: from the absence's feats (`Rook brought Ada's pack · $47`), else the
 *  run's own last notes (`Rook brought Ada's pack.`). Null when no pet fetched for this heir. */
export function fetchedFor(d: Pick<Death, "hero" | "notes" | "memorial">, r: Pick<ReturnReport, "feats"> | undefined): { pet: string; gold?: number } | null {
  const heir = d.hero?.name ?? d.memorial?.name;
  if (!heir) return null;
  for (const f of [...(r?.feats ?? [])].reverse()) {
    if (f.k !== "fetched") continue;
    const m = FETCH_RE.exec(f.text);
    if (m && m[2] === heir) return { pet: m[1], gold: m[3] ? Number(m[3]) : undefined };
  }
  for (const n of d.notes ?? []) { const m = FETCH_RE.exec(n); if (m && m[2] === heir) return { pet: m[1] }; }
  return null;
}
/** `Rook brought $47` under the death's epitaph — quiet; the grave keeps the rest. */
export function fetchedLine(d: Pick<Death, "hero" | "notes" | "memorial">, r: Pick<ReturnReport, "feats"> | undefined): HTMLElement | null {
  const f = fetchedFor(d, r);
  if (!f) return null;
  return h("div", { class: "pet-fetched num", "data-pet": f.pet, title: /* copy:tooltip */ "his pack carried home · the grave keeps the rest" },
    roleGlyph("fetcher"), " ", h("b", null, f.pet), f.gold ? /* copy:callout */ ` brought $${f.gold}` : /* copy:callout */ " brought his pack");
}

// ---------------------------------------------------------------- the report and the crier: pet feats, condensed

export const PET_KINDS = new Set(["pet", "fetched", "old_hound", "bred", "released", "pet_synergy"]);
/** The order a pet line leads by on the report's card (one line there). */
const PET_RANK = ["fetched", "old_hound", "pet_synergy", "bred", "pet", "released"];
export type PetLine = { k: string; text: string; title?: string; day?: number };

/** The absence's pet feats as few lines: one fetch line a pet (summed), one old-hound line, one bred and one released line, each
 *  synergy, the latest `Rook · guard L4`. Most notable first. */
export function petLines(feats: FeatNews[] | undefined): PetLine[] {
  const fs = (feats ?? []).filter((f) => PET_KINDS.has(f.k));
  const out: PetLine[] = [];
  const fetched = new Map<string, { gold: number; heirs: string[]; day: number }>();
  for (const f of fs.filter((x) => x.k === "fetched")) {
    const m = FETCH_RE.exec(f.text);
    if (!m) { out.push({ k: f.k, text: f.text, day: f.day }); continue; }
    const e = fetched.get(m[1]) ?? { gold: 0, heirs: [], day: f.day };
    e.gold += Number(m[3] ?? 0); e.heirs.push(m[2]); e.day = f.day;
    fetched.set(m[1], e);
  }
  for (const [pet, e] of fetched) out.push({ k: "fetched", text: /* copy:callout */ `${pet} brought $${e.gold}`, title: e.heirs.map((x) => /* copy:callout */ `${x}'s pack`).join(" · "), day: e.day });
  const hounds = fs.filter((x) => x.k === "old_hound");
  if (hounds.length === 1) out.push({ k: "old_hound", text: hounds[0].text, day: hounds[0].day });
  else if (hounds.length > 1) out.push({ k: "old_hound", text: /* copy:callout */ `${hounds.map((x) => x.text.split(" · ")[0]).join(", ")} · old hounds`, title: hounds.map((x) => x.text).join("\n"), day: hounds.at(-1)!.day });
  const seen = new Set<string>();
  for (const f of fs.filter((x) => x.k === "pet_synergy")) if (!seen.has(f.text) && seen.add(f.text)) out.push({ k: f.k, text: f.text, day: f.day });
  const bred = fs.filter((x) => x.k === "bred");
  if (bred.length) out.push({ k: "bred", text: bred.length === 1 ? /* copy:callout */ `egg bred · ${bred[0].text.split(" egg")[0]}` : /* copy:callout */ `${bred.length} eggs bred`, title: bred.map((x) => x.text).join(" · "), day: bred.at(-1)!.day });
  const pet = fs.filter((x) => x.k === "pet").at(-1);
  if (pet) out.push({ k: "pet", text: pet.text, day: pet.day });
  const rel = fs.filter((x) => x.k === "released");
  if (rel.length) out.push({ k: "released", text: rel.length === 1 ? rel[0].text : /* copy:callout */ `${rel.length} pets released`, title: rel.map((x) => x.text).join(" · "), day: rel.at(-1)!.day });
  return out.sort((a, b) => PET_RANK.indexOf(a.k) - PET_RANK.indexOf(b.k));
}

/** The hero's diary in his own words for the pet feats (`short` ≤ 3 words): a fetch, an old hound, a synergy, an egg; a signature
 *  learned (the bests' `Rook L3 · taunt`). The routine `Rook · guard L4` and a release are not cried. */
export function petCries(r: Pick<ReturnReport, "feats" | "bests">): { k: string; short: string; long: string; day?: number }[] {
  const out: { k: string; short: string; long: string; day?: number }[] = [];
  for (const l of petLines(r.feats)) {
    const name = l.text.split(/ · | brought /)[0];
    if (l.k === "fetched") {
      const heirs = (l.title ?? "").split(" · ").filter(Boolean);
      out.push({ k: "fetched", day: l.day, short: l.text, long: heirs.length > 1 ? /* copy:diary_line */ `${name} brought ${heirs.length} packs home` : /* copy:diary_line */ `${name} brought ${heirs[0] ?? "a pack"} home` });
    } else if (l.k === "old_hound") out.push({ k: "old_hound", day: l.day, short: l.title ? /* copy:callout */ "old hounds" : /* copy:callout */ `${name} · old hound`, long: l.title ? /* copy:diary_line */ `${name.split(", ").length} hounds served three heirs` : /* copy:diary_line */ `${name} has served three heirs` });
    else if (l.k === "pet_synergy") out.push({ k: "pet_synergy", day: l.day, short: name, long: l.text });
    else if (l.k === "bred") out.push({ k: "bred", day: l.day, short: /* copy:callout */ "egg bred", long: /* copy:diary_line */ `The kennel keeper bred an egg` });
  }
  for (const b of r.bests ?? []) {
    const m = /^(.+) L([35]) · (.+)$/.exec(b);
    if (m) out.push({ k: "pet_sig", short: /* copy:callout */ `${m[1]} L${m[2]}`, long: /* copy:diary_line */ `${m[1]} learned ${m[3]}` });
  }
  return out;
}

// ---------------------------------------------------------------- the kennel keeper's order

export const KENNEL_ORDERS = ["breed", "best", "off"] as const;
/* copy:button */
export const KENNEL_WORD: Record<string, string> = { breed: "breed", best: "best", off: "off" };
/* copy:tooltip */
export const KENNEL_TIP: Record<string, string> = { breed: "eggs for the wall · a kind the party lacks", best: "eggs from the best pets", off: "no eggs · no release" };
/** The keeper's order is shown once a pet is owned or an egg waits (the core sends the order with the companions' read). */
export const kennelShown = (L: Pick<Lineage, "pets" | "orders" | "party" | "kennel" | "eggs">): boolean =>
  !!L.pets && !!L.orders?.kennel && (L.party.length + L.kennel.length + L.eggs.length > 0 || L.orders.kennel !== "breed");
