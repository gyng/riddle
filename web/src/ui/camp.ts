import { classIcon } from './class-icons';
import { heroRoster } from "./heroes";
// Camp: lineage strip · set tabs · rule editor · forecast · vault loadout · unlocks · send.
// Cut 6: the strip's gold opens the `gold` sheet (the last 20 movements, newest first, §1); the disabled send reads
// `6/5 · drop one` and a card's buy reads `◆3 · takes a row` on a full set (§4); owned cards and automations stay on the
// shelf as chips that open their rows (§6); `rest 12m` is a chip that answers `send skips rest` (§7).
// Cut 9: an unlock card opens its sheet, the buy is there (§2); the forge sheet shows each kind's ladder (§10).
// Cut 10 §3: the rest chip reads `rest 20m · send skips` permanently; a greyed supply says why under its price (`3/3 slots`,
// the engine's `needs`, `$12 short`); the `+1 row` card is dimmed `fill rows` while free rows exist; a card's reach delta is
// labelled `at end` (a bought card becomes the last row). Cut 10 §4: the camp drone (biome of the next floor) while mounted.
// Cut 12 §1: rows are own rows — `fill rows` and `5/4 · drop one` count them against `max_rows`; a card never takes a row (it
// sits outside the cap) and its reach delta is labelled where it goes (`at R3`, the catalogue's `insert_at`). §6: the unlock
// shelf refetches when a rule edit crosses `max_rows` (`app.onShelf`); a supply line has its own `×`; a free line reads `· kennel`.
// Cut 13 §2: a new heir's trait is chosen — while `Lineage.trait_offer` holds two names the strip shows two chips beside `♟3`
// (`brave | curious`, the chosen one `on`, each with its rule as a small under-label); a tap is `setTrait(name)`; the chips
// vanish once the offer is empty (the send took it).
// Cut 16 §2: beside them, while `Lineage.class_offer` stands, a chip per owned class with its signature verb (`rogue · vanish`;
// one not yet open reads `ranger · mark L7`); the chosen one `on`; a tap is `setClass(name)` (it sticks until changed). The chip
// row stands in for the class button while it is up.
import { conceptCap, conceptIcon } from "./concepts";
import { wallTablet } from "./wall";
import { meterCompare, meterPanel, secs } from "./meters";
import { anyNew, hasCurriculum, sysOpen } from "./systems";
import { lookStud } from "./look";
import type { App, Mounted } from "../app";
import type { CageOption, ForkOption, InvItem, Lineage, StandingOrders, StartOption, SupplyEntry, UnlockInfo } from "../engine/types";
import { h, clear, flash, replace, spanOf, twoTap } from "./dom";
import { heroBinding, renderEditor } from "./editor";
import { renderParty } from "./party";
import { lowOf, renderForecast, renderShaft, share } from "./forecast";
import { renderScene } from "./divergence";
import { gem, metersSlot, portrait, renderBar, renderConsole, stud, tile } from "./frame";
import { revealed, type Step } from "./reveal";
import { openLedger } from "./party";
import { kitAffordable, openForge } from "./forge";
import { afterOf, labelOf, stallLabel, classList, classUnlockReason, deltaClass, deltaLabel, deltaPts, goldAffordable, addCard, isCard, openOwnedSheet, openUnlockSheet, ownedRows, priceLabel, supplyCap, visible, vaultSlots, withRowsGate } from "./unlocks";
import { audio, biomeOf } from "../audio";
import { salvageValue } from "./salvage";
import { CLASS_VERBS } from "../engine/classes";
import { classSkillChip } from "./class-skills";
import { isFreeSupply, ownRowCount, setRefRows } from "./tokens";
import { closeAllSheets, openSheet, openWindow, setPanelEscape } from "./sheet";
import { AUTO, autoDismiss } from "./autodismiss";
import { setBusyHost } from "./progress";
import { icon } from "./skin";
import { biomeAt, routeChips, routeForks, seenForks, withFork } from "./route";
import { openOathBoard, paintOathTab } from "./oaths";
import { anchorPanel, buildingTile, exposeTown, markOpened, openBank, openHero, renderTown, townBuilt } from "./town";
import { onPackages, openPackages, packagesShown, packagesStrip, penOpen } from "./packages";   // Cut 30 §2: the packages, the pen gated late
import { openQuest, questShown } from "./quest";   // Cut 30 §5: the quest board
import { sendMark } from "./works";
import { itemIcon, itemName, itemChip } from "./items";   // run-clear: items in their rarity rims
import { kwHost } from "./tips";

const SET_NAME_MAX = 12;
/** QA 524827b (qaAA): a supply whose name does not say its use — its use under the shop chip (≤ 3 words). */
/* copy:callout */
const SUPPLY_USE: Record<string, string> = { leash: "tames a foe", chalk: "marks a floor", bell: "lures hunters" };
/** Cut 19 §1: the cage's preferences, the picker's order. */
const CAGE_PREFS = ["weapon", "armour", "potion", "scroll"];
/** Cut 19 §1: the last `cageForecast()` and what it was measured for (the set, the preference, the best) — the picker paints it at once. */
let cageMemo: { key: string; opts: CageOption[] } | null = null;
/** Cut 19 §1: an option's headline delta (`+36%`, the bank share's move when either panel banks, else the reach's); none on the
 *  current preference or a move that rounds to 0; `dim` inside its ±. */
/** QA 1a2a4a9 (O, P: "`armour +15%` — % of what?"; "the selected option never has a number"; "`scroll` stayed blank"): each option
 *  names what it measures — `bank +15%` (the bank share, when either panel banks) or `D7 +15%` (the reach at the option's depth);
 *  the current one its own level (`D7 73%`), a move that rounds to 0 `D7 +0%` (dim). */
export function cageDelta(o: Pick<CageOption, "current" | "depth" | "reach" | "bank" | "bank_delta" | "delta" | "pm">): { text: string; cls: string } | null {
  const banks = o.delta === o.bank_delta && (o.bank > 0 || o.bank - o.bank_delta > 0);
  const at = banks ? /* copy:label */ "bank" : `D${o.depth}`;
  const level = `${at} ${Math.round((banks ? o.bank : o.reach) * 100)}%`;
  if (o.current) return { text: level, cls: "cur" };
  const d = Math.round(o.delta * 100);
  const pm = Math.round(o.pm * 100);
  // Cut 22 §4 (AG: "the red `−18%` confused me; is it a delta?"): a move is signed points in the delta look (`bank −18`), never a `%`.
  // QA 0c6e126 (qaZ: `weapon D4 60% · armour D4 +28 · potion D4 −12` — "one absolute, three deltas"): every option reads its own level
  // on one scale (`armour D4 88%`), the move against the current one a mark beside it (`▲28`; dim inside its ±)
  const mark = d === 0 ? "" : ` ${d > 0 ? "▲" : "▼"}${Math.abs(d)}`;   // (inside its ± the option reads dim: `flat`)
  return { text: `${level}${mark}`, cls: `dlt ${d > 0 ? "up" : d < 0 ? "down" : "flat"}${Math.abs(d) <= pm ? " flat" : ""}` };
}
/** Cut 21 §1: the last `startForecast()` and what it was measured for (the set, the start, the lit waystones, the best). */
let startMemo: { key: string; opts: StartOption[] } | null = null;
const forkMemo = new Map<string, ForkOption[]>();   // Cut 26 §2: the fork tablet's measures (per fork, this set's, this lineage's)
/** Cut 21 §1: a start's toll — the engine's, else the contract's `$10 × depth` (0 at D1). */
export const startToll = (_d: number, o?: Pick<StartOption, "toll">): number => o?.toll ?? 0;   // QA 778fa1b (qaV; core): the toll is going — the wire's toll only, no client guess
/** QA a946e04 (T: `start → D5 · $50` at $32, and the run began on D1 with no word): can the purse pay a start's toll now? The engine's
 *  word when it sends one (`StartOption.short`, `Lineage.start_payable` for the current start), else the gold against the toll. */
export function startShort(L: Pick<Lineage, "gold" | "start" | "start_payable" | "start_toll">, st: number, o?: Pick<StartOption, "toll" | "short">): boolean {
  const toll = st === (L.start ?? 1) && L.start_toll !== undefined ? L.start_toll : startToll(st, o);
  if (toll <= 0) return false;
  if (o?.short !== undefined) return o.short;
  if (st === (L.start ?? 1) && L.start_payable !== undefined) return !L.start_payable;
  return L.gold < toll;
}
/** Cut 17 §3: which step carves each console tile (the tile glints on its first appearance). */
const STEP_OF: Record<string, Step> = { edit: "edit", loadout: "loadout", unlocks: "unlocks", vault: "vault", forge: "kit", party: "party", ledger: "heirs", chronicle: "heirs" };
const ALL_KEY = "riddle.unlocks.all";
const unlocksAll = (): boolean => { try { return localStorage.getItem(ALL_KEY) === "1"; } catch { return false; } };
const setUnlocksAll = (on: boolean): void => { try { localStorage.setItem(ALL_KEY, on ? "1" : "0"); } catch { /* a per-viewer convenience */ } };
/** Cut 13 §2: each trait's one-line rule, ≤ 3 words (the core's: cowardly retreats under 50 % hp with foes in view; brave skips a
 *  retreat row vs one foe, once a floor — `holds a retreat` read as its opposite, QA 23ed91f; curious drinks an unknown when clear; greedy steps onto adjacent loot). */
/* copy:callout */
const TRAIT_RULE: Record<string, string> = { cowardly: "flees under 50%", brave: "skips a retreat", curious: "drinks unknowns", greedy: "grabs loot" };
/** QA a946e04 (S, T: `cowardly · flees under 50%` never fled): the trait's rule as the core states it (`Lineage.trait_rules`), else the table's. */
export const traitRule = (L: Pick<Lineage, "trait_rules">, t: string): string | undefined => { const r = L.trait_rules?.[t]; return (r && ruleShort(r)) ?? TRAIT_RULE[t]; };
/** The core's rule within the chip's three words: `backs off once a floor under 50%` → `backs off <50% 1×/floor`, `tries one unknown a
 *  floor` → `tries unknown 1×/floor`; undefined when it will not fit (the table's words stand). */
export function ruleShort(rule: string): string | undefined {
  const perFloor = /\b(once|one \w+) a floor\b/.test(rule);
  const t = rule.replace(/\bonce a floor\b/, "").replace(/\bone (\w+) a floor\b/, "$1").replace(/\bunder (\d+%)/, "<$1").replace(/\s+/g, " ").trim() + (perFloor ? /* copy:none */ " 1×/floor" : "");
  return t.split(/\s+/).filter((w) => /\p{L}/u.test(w)).length <= 3 ? t : undefined;
}
export const setName = (s: { name?: string }, i: number): string => (s.name ?? "").trim().slice(0, SET_NAME_MAX) || `${i + 1}`;

/** Cut 29 §3: the camp's meters (the desktop's column under the shaft): the last two runs compared, else the last run's breakdown. */
function campMeters(app: App): HTMLElement | null {
  const runs = app.lineage.meters?.runs ?? [];
  if (!runs.length) return null;
  const latest = runs[runs.length - 1];
  const body = runs.length >= 2 ? meterCompare(runs[runs.length - 2], latest)
    : meterPanel(latest, app.rules.rows, { title: /* copy:label */ "last run" });
  return h("details", { class: "camp-run-details" },
    h("summary", { class: "btn" }, h("span", null, /* copy:button */ "Run details"), h("small", { class: "dim num" }, secs(latest.seconds))), body);
}
const SEND_ARM_MS = 800;
/** Cut 29 §4: the exit's keep order, in the vault panel's words. */
const KEEP_ORDERS = ["best_weapon", "best_armour", "none"];
/** blind 1fb7786 (core `FORGE_ORDERS`): the apprentice's forge order — half of each haul home, the spare purse, nothing. */
const FORGE_ORDERS = ["half", "all", "off"];
const FORGE_WORD: Record<string, string> = { half: "half haul", all: "all spare", off: "off" };
/* copy:label */
const KEEP_WORD: Record<string, string> = { best_weapon: "weapon", best_armour: "armour", none: "none" };
const SEEN_MS = 1800;   // Cut 29 §2: a new system's glint plays before the core clears its `new`
/** QA 92eb880 (M: "kept `sealed scroll?` … reappear as `summon ally scroll` with no line saying they were identified"): an identified
 *  item's flavour off the lineage's facts (`item:sealed=summon_ally` → `sealed`); undefined while unknown or unmatched. */
export function keptAs(facts: string[], it: { kind: string; known: boolean }): string | undefined {
  if (!it.known) return undefined;
  const f = facts.find((x) => x.startsWith("item:") && x.endsWith(`=${it.kind}`));
  return f ? f.slice(5, f.indexOf("=")).replace(/_/g, " ") : undefined;
}
/** QA 0c6e126 (qaY): the lineage state a forecast's sims start from (the core's `lineage_key`, as the wire shows it) — a memo of a
 *  measure keyed without it survives a purchase, a drop or a cage change and paints a stale number. */
export function simKey(app: Pick<App, "lineage" | "loadout">): string {
  const L = app.lineage;
  return JSON.stringify([L.heir, L.trait, L.class, L.gold, L.facts?.length, L.unlocks, (L.supplies ?? []).map((s) => s.id), (L.vault ?? []).map((v) => v.id), [...(app.loadout ?? [])].sort(), L.kit, L.party, L.keep_pref, L.vault_pref, L.start]);
}
/** QA 0c6e126 (qaY: the shelf read `invisibility pot…`): a shelf line too long for its half-width chip drops its class word
 *  (`invisibility`); the shop's chip keeps the full name. */
export const shelfLabel = (label: string): string => label.length > 14 ? label.replace(/ (potion|scroll)$/, "") : label;
export function renderCamp(app: App, highlight?: number): Mounted {
  setRefRows(() => app.rules.rows);
  // Cut 17: the frame — the bar (the strip: heir, `$`, `◆`, `★`, best, the stud; the wake's offers under it), the well (the
  // tablets and the depth shaft; the set tabs from the 5th heir; the panels over it), the console (the portrait, the command
  // card as revealed, the gem `send`)
  const bar = renderBar(app, { live: true });
  const strip = bar.el;
  const tabs = h("nav", { class: "tabs" });
  // editing (`app.editing`): the tablets carry the editor (chips, ▲▼, ×, `+`); off, each is one carved tablet
  const editor = renderEditor(heroBinding(app), highlight, {
    compact: () => !app.editing,
    onTablet: (i) => { app.editing = true; closePanel(); editor.refresh(); paintTiles(); flashRow(i); },
  });
  // Cut 19 §1: the cage is a camp decision — its own tablet under the rules (`cage → armour`), revealed once a cage was seen; the
  // tap opens the picker with each preference's forecast delta
  const cageTab = h("button", { class: "row tablet compact cage-tab", hidden: true, onclick: () => openCagePicker() });
  // Cut 21 §1: where the send starts — its own tablet beside the rules and the cage (`start → D9 · $90`), carved when the first
  // waystone lights; the tap opens the picker (D1 and each lit waystone, each with its forecast move and toll)
  const startTab = h("button", { class: "row tablet compact start-tab", hidden: true, onclick: () => openStartPicker() });
  // Cut 26 §2: the set's route — a chip line above the rows (`⑂ D5 fens · D14 crypt`), carved once a hero has stood on two stairs (the
  // fact `fork D5`); the tap opens the fork tablet (both stairs priced for this set)
  const routeTab = h("button", { class: "route-line", hidden: true, onclick: () => openRoutePicker() });
  // Cut 29 §4: the standing orders in one tablet (the exit's keep, the cage's pick, the start, the repeat, insuring) — with a Cut 29
  // core the cage and start tablets fold into it; the tap opens the orders sheet
  const ordersTab = h("button", { class: "row tablet compact orders-tab", hidden: true, onclick: () => openOrders() });
  // Cut 29 §4: a kind a `throw` row names that the repeat lacks — one tap packs it (`+ fire · for throw fire`), the repeat keeps it after
  const repeatAdd = h("div", { class: "chips repeat-add", hidden: true });
  // Cut 29 §1 (E1): the wall's edit the core cached for the day (`Lineage.wall`) as a patch tablet under the rules
  const wallBox = h("div", { class: "wall-host", hidden: true });
  const paintWall = (): void => { const w = penOpen(app.lineage) && app.lineage.wall && JSON.stringify(app.lineage.wall.rules.rows.map((r) => [r.conds, r.verb])) !== JSON.stringify(app.rules.rows.map((r) => [r.conds, r.verb])) ? app.lineage.wall : undefined; wallBox.hidden = !w; if (w) replace(wallBox, wallTablet(app, w, () => undefined)); };
  // Cut 28 §1: the oath board — its own tablet under the start's (`oath → D10 no drink · 34%`, or `oaths 3`), carved when an oath is
  // first affordable; the tap opens the board (three oaths, each its chips, its reward, its price)
  const oathTab: HTMLButtonElement = h("button", { class: "row tablet compact oath-tab", hidden: true, onclick: () => openOathBoard(app, oathTab) });
  const paintOath = (): void => { const R = revealed(app); if (R.has("oaths") && !onPackages(app.lineage)) { paintOathTab(app, oathTab); oathTab.classList.toggle("reveal", R.fresh("oaths")); } else oathTab.hidden = true; };
  const party = renderParty(app);
  // Cut 30 §2: before the pen the tablets' place holds the worn packages (`Steady L3`, its level bar); the rule tablets come with the pen
  const pkgStrip = packagesStrip(app);
  const fc = renderForecast(app);
  const shaft = renderShaft(app, () => togglePanel("forecast"), () => revealed(app).has("gems"));
  const vault = h("section", { class: "vault" });
  const supplies = h("section", { class: "supplies" });
  const unlocks = h("section", { class: "unlocks" });
  // Cut 18 §4 (rater Z: "APPLY also sent the next heir immediately"): the death screen's gem and this one share the console's gem slot —
  // a camp opened on a lit tablet (a patch applied, `highlight`) keeps its send deaf for SEND_ARM_MS, so the tap (or a second one) that
  // applied never lands on `send`; the player sends
  let residentUntil = 0, residentTimer = 0;
  const armedAt = performance.now() + (highlight !== undefined ? SEND_ARM_MS : 0);
  // Cut 30 §3: the send — the gem or the dungeon's mouth; the hero walks from where he is into the mouth, then the watch opens
  // RUNS_UI: with a run under way the gem watches it (he is already down there: no walk to the mouth)
  const doSend = (): void => { if (app.lineage.ended) { app.go({ kind: "ending" }); return; } if (app.lineage.town?.home === false || performance.now() < Math.max(armedAt, residentUntil)) return; if (isLive()) { app.go({ kind: "watch" }); return; } if (!app.overBudget && app.rules.rows.length > 0) town.send(() => { if (el.isConnected) app.go({ kind: "watch" }); }); };
  const isLive = (): boolean => !!app.lineage.live && app.lineage.live.turn > 0;
  const send = gem({ label: /* copy:button */ "send", cls: "send", pulse: true, onclick: () => doSend() });
  // Cut 10 §3: the rest chip says what it means all the time (`rest 20m · send skips`), no tap needed
  const rest = h("span", { class: "rest chip num" });
  // the engine's busy label (`forecast` · `offline`) in its own strip under the header (QA on 50bb162: it drew over `D4 ★0`)
  const busyStrip = h("div", { class: "busy-strip num" });
  // Cut 17 §2: the camp's secondary objects are panels over the well, one at a time, each opened by its console tile (the
  // forecast's by the shaft), each with a close stud; Escape closes the open one
  const PANELS: Record<string, HTMLElement> = { forecast: fc.el, loadout: supplies, unlocks, vault, party: party.el };
  // QA 23ed91f (K: "the SUPPLIES sheet covers R4's verb chip; you have to close it to edit R4"): a tap on the well around the panel
  // closes it (the stud and Escape still do)
  const panelHost = h("div", { class: "panel-host", onclick: (e: Event) => { if (e.target === panelHost) closePanel(); } });
  // a closed panel's content waits in the DOM, unrendered (its engine fetches keep painting it; it is not on screen, so not in the
  // page's text either — a rater's text view reads what the player sees)
  const panelStore = h("div", { class: "panel-store", hidden: true, inert: true }, ...Object.values(PANELS));
  let open: string | null = null;
  function closePanel(): void { if (!open) return; open = null; panelStore.append(...Object.values(PANELS)); panelHost.replaceChildren(); panelHost.classList.remove("open"); paintTiles(); }
  function togglePanel(name: string, anchor?: HTMLElement | null): void {
    if (open === name) { closePanel(); return; }
    closeAllSheets();   // QA 778fa1b (qaV: `vault` tapped under the open CAGE sheet stacked VAULT under CAGE): one sheet or panel at a time
    if (open) panelStore.append(...Object.values(PANELS));
    open = name;
    const pstud = stud(closePanel);
    const panel = h("section", { class: "panel", "data-panel": name }, pstud, h("div", { class: "panel-body" }, PANELS[name]));
    autoDismiss(pstud, { ms: AUTO.panel, scope: panel, yieldToSheets: true, onExpire: closePanel });   // docs/UI.md §7
    panelHost.replaceChildren(panel);
    panelHost.classList.add("open");
    if (anchor) anchorPanel(panel, anchor, panelHost);   // Cut 30 §3: opened from its building, the panel stands over it
    paintTiles();
  }
  setPanelEscape(() => { if (!open) return false; closePanel(); return true; });
  // Cut 30 §3: the town (the camp, then the buildings as the core raises them) is the well's first screen, where the vista stood;
  // every building opens the panel its tile opens, standing over it
  const town = renderTown(app, {
    resident: (ms) => { residentUntil = performance.now() + ms; clearTimeout(residentTimer); residentTimer = window.setTimeout(() => { if (el.isConnected) paintSend(); }, ms); },
    send: () => doSend(),
    // RUNS_UI: the hero's tent keeps his log once he has runs (the lane's `log` stud opens the same); his class and look are the
    // portrait's (the console's well), the hero sheet before the first run
    hero: (a) => openHero(app, a),
    open: (what, a) => { closeAllSheets(); if (open === what) closePanel(); togglePanel(what, a); },
    forge: (a) => { closePanel(); openForge(app, a); },
    quest: (a) => { closePanel(); openQuest(app, a); },
  });
  exposeTown(town);
  const focusHome=():void=>{town.el.scrollIntoView({block:"nearest"});town.view.focusHero();};
  window.addEventListener("riddle:focus-hero",focusHome);
  const well = h("div", { class: "well camp-well" }, busyStrip, town.el, tabs, h("div", { class: "camp-main" }, h("div", { class: "tablets" }, pkgStrip.el, routeTab, editor.el, cageTab, startTab, ordersTab, wallBox, repeatAdd, oathTab), shaft.el, metersSlot(campMeters(app))));
  // QA 0c6e126 (qaZ: `heir rests 20m · send skips rest` half under the console on every camp — the well's last line, cut by its scroll):
  // the rest line sits under the well, outside the scroll (the well-wrap's third row), always whole
  // RUNS_UI (docs/RUNS_UI.md §2): the run lanes take the rest line's place — one row per hero (live · rests · waits), the log at its end
  const lanes = heroRoster(app, { focus:()=>{town.el.scrollIntoView({block:"nearest"});town.view.focusHero();}, rules:()=>{app.editing=true;editor.refresh();paintTiles();} });
  const restLine = h("div", { class: "rest-line lanes-line heroes-col" }, lanes.el, rest);
  const face = portrait(app, { label: "" });
  const cons = renderConsole({ portrait: face.el, tiles: [], gem: send });
  // Cut 27 §2: the edit as a scene — over the well's foot after an edit's refine (before · after on the renderer), then its line under `vs sent`
  const scene = renderScene(app);
  // gfx raters ("the edit scene covers the rule list mid-row"): when the scene rises over the tablets its top edge moves to the nearest
  // gap between two tablets (it shrinks to clear a row it would cut, or grows over it when shrinking would leave it too short)
  const fitScene = (): void => {
    const s0 = scene.el; if (s0.hidden) { s0.style.height = ""; return; }
    s0.style.height = "";
    const box = s0.getBoundingClientRect(), top = box.top;
    const cut = [...el.querySelectorAll<HTMLElement>(".camp-main .tablets .row.tablet, .camp-main .tablets .orders-tab")].map((r) => r.getBoundingClientRect()).find((r) => r.height > 0 && r.top < top && r.bottom > top);
    if (!cut) return;
    const shrink = box.bottom - (cut.bottom + 4), grow = box.bottom - (cut.top - 4);
    s0.style.height = `${Math.round(shrink >= 140 ? shrink : grow)}px`;
  };
  new MutationObserver(() => fitScene()).observe(scene.el, { attributes: true, attributeFilter: ["hidden"] });
  // the scene fills the well's first screen (the rows, the shaft and the rest scroll under it)
  const fitTown = (): void => { const hgt = well.clientHeight; if (hgt > 0) town.el.style.setProperty("--town-h", `${hgt}px`); };
  const wellRo = typeof ResizeObserver !== "undefined" ? new ResizeObserver(fitTown) : null;
  wellRo?.observe(well);
  const el = h("main", { class: "camp frame" }, strip, h("div", { class: "well-wrap" }, well, scene.el, shaft.vsEl, scene.line, restLine, panelHost, panelStore), cons.el);
  setBusyHost(busyStrip);
  function flashRow(i: number): void { const r = editor.el.querySelector<HTMLElement>(`.row[data-i="${i}"]`); if (r) { flash(r, "hl", 1600); r.scrollIntoView({ block: "center" }); } }

  // QA 1a2a4a9 (P: "nothing on the camp shows what's packed — only opening SUPPLIES does"): the loadout tile carries the pack's count
  const withPack = (el: HTMLElement): HTMLElement => { const n = app.lineage.supplies?.length ?? 0; if (n) el.appendChild(h("span", { class: "pack-n num" }, `${n}/${app.lineage.supply_cap ?? supplyCap(app.lineage.unlocks)}`)); return el; };
  /** Cut 23 §1: the forge tile's badge — the kit steps the purse can buy now (`2`), none when there are none. */
  const kitBadge = (): HTMLElement | null => { const n = kitAffordable(app.lineage); return n ? h("span", { class: "kit-n num", "data-n": n }, `${n}`) : null; };
  const withBadge = (el: HTMLElement, badge: HTMLElement | null): HTMLElement => { if (badge) { el.appendChild(badge); el.classList.add("badged"); } return el; };
  /** Cut 30 (the Reveal): a system the core opened since the camp last looked glints once (its `new`). */
  const freshSys = (...ids: string[]): boolean => !!app.lineage.systems?.some((x) => ids.includes(x.id) && x.new);
  /** Cut 30 §2: the pen gates the editor (and the forecast's detail: one headline before it). */
  function paintPen(): void {
    const pen = penOpen(app.lineage);
    // Legacy/literal wires already expose the pen; keep their editor reachable.
    const fresh = onPackages(app.lineage) && app.lineage.best_depth === 0 && !(app.lineage.runs?.length);
    el.dataset.first = fresh ? "1" : "0";
    const main = well.querySelector<HTMLElement>(".camp-main"); if (main) main.hidden = fresh;
    editor.el.hidden = !pen;
    el.classList.toggle("prepen", !pen);
    pkgStrip.paint();
  }
  /** Cut 17 §1/§3: the command card as revealed — edit · loadout · unlocks · vault · forge · party · ledger · chronicle.
   *  Cut 30 §3: the building bar — one tile per building standing, in build order (the forge, the vault, the kennel, the bank: tile and
   *  building open the same panel, the badge shared), then the tiles with no building (the pen's edit, the pack, unlocks, the heirs'). */
  function paintTiles(): void {
    el.classList.toggle("hero-editing",app.editing);
    paintTabs();
    const R = revealed(app);
    const t = (id: string, label: string, ico: string, onclick: () => void, on = false): HTMLElement => tile({ id, label, icon: ico, onclick: () => { closeAllSheets(); onclick(); }, on, fresh: R.fresh(STEP_OF[id]) || (id === "forge" && R.fresh("forge")) });
    const built = townBuilt(app.lineage);
    if (built) {
      // a sheet opened from a tile stands over its building only while the building is on screen (the well may be scrolled to its rows)
      const anchor = (id: string): HTMLElement | null => { const x = town.anchorOf(id); if (!x || x.hidden) return null; const r = x.getBoundingClientRect(), w = well.getBoundingClientRect(); return r.bottom > w.top + 8 && r.top < w.bottom - 8 ? x : null; };
      const bt: Record<string, () => HTMLElement> = {
        blacksmith: () => withBadge(buildingTile("blacksmith", { fresh: R.fresh("forge") || R.fresh("kit"), onclick: () => { closeAllSheets(); closePanel(); markOpened(app.lineage, "blacksmith"); openForge(app, anchor("blacksmith")); } }), kitBadge()),
        storehouse: () => buildingTile("storehouse", { on: open === "vault", fresh: R.fresh("vault"), onclick: () => { closeAllSheets(); markOpened(app.lineage, "storehouse"); togglePanel("vault"); } }),
        kennel: () => buildingTile("kennel", { on: open === "party", fresh: R.fresh("party"), onclick: () => { closeAllSheets(); markOpened(app.lineage, "kennel"); togglePanel("party"); } }),
        bank: () => buildingTile("bank", { onclick: () => { closeAllSheets(); closePanel(); markOpened(app.lineage, "bank"); openBank(app, anchor("bank")); } }),
      };
      cons.setTiles([
        ...built.map((b) => bt[b]?.()),
        // Cut 30 §2/§5: the packages and the quest board (the board by the mouth opens it too)
        packagesShown(app.lineage) && tile({ id: "packages", label: /* copy:button */ "tactics", icon: "unlocks", glyph: "✦", fresh: freshSys("stances", "tactics", "tactic2", "temperament"), onclick: (e: Event) => { closeAllSheets(); openPackages(app, e.currentTarget as HTMLElement); } }),
        questShown(app.lineage) && tile({ id: "quest", label: /* copy:button */ "quest", icon: "renown", glyph: "✠", fresh: freshSys("quests"), onclick: () => { closeAllSheets(); openQuest(app, anchor("board")); } }),
        R.has("edit") && penOpen(app.lineage) && t("edit", /* copy:button */ "edit", "edit", () => { closePanel(); if (!app.editing) { app.editing = true; editor.refresh(); } paintTiles(); }, app.editing),
        R.has("loadout") && withBadge(withPack(t("loadout", /* copy:button */ "supplies", "loadout", () => togglePanel("loadout"), open === "loadout")), repeatBadge()),
        R.has("unlocks") && t("unlocks", /* copy:button */ "unlocks", "unlocks", () => togglePanel("unlocks"), open === "unlocks"),
        R.has("heirs") && t("ledger", /* copy:button */ "enemy guide", "ledger", () => openLedger(app)),
        // RUNS_UI: the chronicle is the log's `heirs` (the lane's log stud) — its tile gave its place to the run lanes' log
      ]);
      shaft.el.classList.toggle("on", open === "forecast");
      return;
    }
    cons.setTiles([
      // QA 92eb880 (N: "the `edit` tile toggles: tapping it while editing closes the editor (I lost the next tap twice)"): it turns
      // editing on and stays lit; a second tap closes the open panel, never the editor
      R.has("edit") && penOpen(app.lineage) && t("edit", /* copy:button */ "edit", "edit", () => { closePanel(); if (!app.editing) { app.editing = true; editor.refresh(); } paintTiles(); }, app.editing),
      R.has("loadout") && withBadge(withPack(t("loadout", /* copy:button */ "supplies", "loadout", () => togglePanel("loadout"), open === "loadout")), repeatBadge()),
      R.has("unlocks") && t("unlocks", /* copy:button */ "unlocks", "unlocks", () => togglePanel("unlocks"), open === "unlocks"),
      R.has("vault") && t("vault", /* copy:button */ "stored gear", "vault", () => togglePanel("vault"), open === "vault"),
      // Cut 23 §1: the forge sells the heir's kit — carved at the first salvage or the first kit step the purse can buy; its badge counts
      // the steps affordable now
      (R.has("forge") || R.has("kit")) && withBadge(t("forge", /* copy:button */ "forge", "forge", () => openForge(app)), kitBadge()),
      R.has("party") && t("party", /* copy:button */ "companions", "party", () => togglePanel("party"), open === "party"),
      // Cut 30 §2/§5: the packages (from the second stance) and the quest board (from the Warlord slain), each glinting once as it comes
      packagesShown(app.lineage) && tile({ id: "packages", label: /* copy:button */ "tactics", icon: "unlocks", glyph: "✦", fresh: freshSys("stances", "tactics", "tactic2", "temperament"), onclick: (e: Event) => { closeAllSheets(); openPackages(app, e.currentTarget as HTMLElement); } }),
      questShown(app.lineage) && tile({ id: "quest", label: /* copy:button */ "quest", icon: "renown", glyph: "✠", fresh: freshSys("quests"), onclick: (e: Event) => { closeAllSheets(); openQuest(app, e.currentTarget as HTMLElement); } }),
      R.has("heirs") && t("ledger", /* copy:button */ "enemy guide", "ledger", () => openLedger(app)),
      // RUNS_UI: the chronicle is the log's `heirs` (the lane's log stud)
    ]);
    shaft.el.classList.toggle("on", open === "forecast");
  }

  function paintStrip(): void {
    const L = app.lineage; const lvl: { level: number; xp: number; next?: number } = L.classes?.[L.class] ?? { level: 1, xp: 0 };
    bar.paint();
    // Legacy trait offers remain here; class offers belong to Hero Details.
    // Cut 30 §2: on packages the old two-trait chips leave the wake (the temperaments, from heir 3, are the packages panel's cards)
    const traits = (L.trait_offer?.length ?? 0) >= 2 && !onPackages(L)
      // QA 524827b (qaAA: a bare `✓` over the first chip, "the trait is picked for you"): the row says what it picks (`trait`) — the ✓ is
      // the heir's own, the other chip the swap
      ? h("span", { class: "chips traits" }, h("small", { class: "dim offer-label" }, /* copy:label */ "trait"), ...L.trait_offer!.map((t) => h("button", { class: `chip trait${t === L.trait ? " on" : ""}`, disabled: t === L.trait, onclick: () => void pickTrait(t), "aria-pressed": t === L.trait ? "true" : "false" },
          // QA 1a2a4a9 (O: "the ✓ is only visual; the text shows no selection"): the mark is text, not a CSS `::before`
          t === L.trait ? h("b", { class: "tick" }, "✓ ") : "", h("span", null, t), traitRule(L, t) ? h("small", { class: "rule dim" }, traitRule(L, t)) : "")))
      : "";
    // Class choices belong to selected Hero Details, including inherited offers.
    replace(bar.offers, traits);
    bar.offers.hidden = !traits;
    // The portrait keeps the class, level and XP, with access to the class picker.
    const R = revealed(app);
    // The portrait remains available when classes can be changed.
    const picker = R.has("edit") || R.has("unlocks") || (onPackages(L) && sysOpen(L, "class"));   // from the first death (a second heir may take another class)
    const next = portrait(app, { hp: 1, cls: picker ? "cls" : "", onclick: picker ? () => pickClass() : undefined,
      label: h("span", { class: "plabel-in" }, h("span", null, L.class, " ", h("b", { class: "num" }, `L${lvl.level}`)),
        // QA 92eb880: the bar is the core's own ladder (`classes[c].next`, the XP the next level costs; 0 at the top: full)
        h("span", { class: "xp" }, h("span", { class: "fill", style: `width:${Math.round(Math.min(1, lvl.next ? lvl.xp / lvl.next : lvl.next === 0 ? 1 : 0) * 100)}%` }))) });
    if (R.has("edit") || (onPackages(L) && L.best_depth > 0)) next.el.appendChild(lookStud(app));   // Cut 30: before the pen, from the first run (a hero rarely dies on Steady)   // hero looks: the stud opens the look sheet — from the first death, like the picker (a fresh camp stays ≤ 8 controls)
    face.el.replaceWith(next.el); face.el = next.el;
    paintRest();
  }
  /** Cut 13 §2: the tap picks the heir's trait (`setTrait`); an engine without it keeps the default (the first offered). */
  async function pickTrait(name: string): Promise<void> {
    if (!app.engine.setTrait) return;
    await app.mutate(() => app.engine.setTrait!(name));
  }
  // Cut 2 §1: camp rest remaining; `send` skips it, so the number just disappears
  function paintRest(): void {
    const restS = app.lineage.rest_left_s ?? 0;
    // Cut 30.5: before the scout the hero is home and waits for a SEND (no rest runs out into a send)
    const waits = !!app.lineage.tree?.waits;
    // QA 912e135 (qaW: "`rest 20m · send skips` — no screen says what rests or what `send skips` means"): who rests, and what the send skips
    replace(rest, waits ? /* copy:callout */ "heir waits" : /* copy:callout */ `departs ${spanOf(restS)}`);   // docs/COPY.md pass 2: `send skips rest` read as a cost; nothing punishes a send
    // RUNS_UI: the lane says it now (`rests 18m` · `waits ▸ send` · `live D3`); the old chip stays for screen readers only
    rest.hidden = !waits && restS <= 0; rest.classList.add("vh"); rest.classList.toggle("waits", waits);
    restLine.hidden = false; lanes.paint();
  }
  // Cut 2 §4: whatever the lineage and the unlock catalogue provide (fighter · rogue · ranger · caster).
  // Cut 5 §6: each row carries the class's verb ladder as chips (`L1 shield bash · L3 cleave · …`), reached rungs lit.
  // Cut 8B §2: a class not yet owned shows its door (`◆0 · bank once`); once open it is taken from here (the rogue is
  // free at the first bank), so a second class is one tap from the strip.
  function pickClass(): void {
    openSheet((close) => {
      const L = app.lineage; const grid = h("div", { class: "classes" });
      const paint = (cat: Parameters<typeof classList>[1]): void => {
        clear(grid);
        for (const { cls, owned, level } of classList(L, cat)) {
          const ladder = Object.entries(CLASS_VERBS[cls] ?? {}).flatMap(([l, vs]) => vs.map((v) => classSkillChip(v, Number(l), Number(l) <= level && owned)));
          const u = owned ? undefined : cat?.find((x) => x.id === cls);
          const door = u ? h("small", { class: "num dim door" }, u.cost ? ` ◆${u.cost}` : "") : "";   // no `◆0` (QA 23ed91f); QA 0c6e126 (qaY: `rogue L1 · bank once` — no hint it is a condition): a gate still shut carries the lock mark, `⊘ bank once`
          const take = async (): Promise<void> => { if (u && !(await app.buy(cls))) return; void app.setClass(cls); close(); };
          grid.appendChild(h("div", { class: "class-row" },
            h("button", { class: `chip verb${cls === L.class ? " on" : ""}${owned || u?.available ? "" : " off"}`, disabled: !(owned || u?.available), onclick: () => void take() }, classIcon(cls), cls, " ", h("b", { class: "num" }, `L${level}`), door), classUnlockReason(u),
            ladder.length ? h("div", { class: "chips ladder" }, ...ladder) : ""));
        }
      };
      paint(unlockCat);
      if (!unlockCat) void app.engine.unlocks().then((cat) => paint(cat)).catch(() => { /* ladders only */ });
      return h("div", { class: "sheet-body" }, h("div", { class: "label row-label" }, /* copy:label */ "class"), grid);
    });
  }
  // Cut 5 §6: sets carry a player-typed name (≤ 12 chars, the game's only free text; default `1 · 2 · 3`); ✎ on the active tab renames.
  // An unnamed set's tab reads `set 2 · 0` — the word, then the row count small (QA on 50bb162, the fourth reader of `2 0` as a
  // party or class count); a named one keeps `fighter 2`
  function paintTabs(): void {
    clear(tabs);
    // Cut 17 §3: the set tabs are carved with the 5th heir (a fresh lineage writes one set)
    const canReturn = app.editing && !!app.lineage.hero_slots?.length;
    const setsShown = revealed(app).has("heirs") || app.active !== 0;
    tabs.hidden = !canReturn && !setsShown;
    if (tabs.hidden) return;
    if (canReturn) tabs.appendChild(h("button", { class: "chip hero-return game-control", onclick: () => { app.editing = false; editor.refresh(); paintTiles(); } }, /* copy:button */ "Heroes"));
    if (!setsShown) return;
    tabs.appendChild(h("span", { class: "saved-set-label label" }, /* copy:label */ "Saved setups"));
    app.sets.forEach((s, i) => {
      const named = !!(s.name ?? "").trim();
      tabs.appendChild(h("button", { class: `tab num game-control${i === app.active ? " on" : ""}`, onclick: () => app.selectSet(i) },
        named ? setName(s, i) : /* copy:label */ `set ${i + 1}`, h("small", { class: "dim" }, ownRowCount(s.rows) ? /* copy:callout */ ` · ${ownRowCount(s.rows)} rules` : "")));   // `fighter · 3` like `set 2 · 0` (QA on 56f2a1d: `fighter 3` read as a hero number); own rows, the editor's `6/6` (QA on 3d71c33: `fighter · 8` beside `6/6 · 3 cards`)
      if (i === app.active) tabs.appendChild(h("button", { class: "tab edit game-control", "aria-label": "rename", onclick: () => renameSet(i) }, /* copy:button */ "rename"));
    });
  }
  function renameSet(i: number): void {
    openWindow((close) => {
      const input = h("input", { class: "name-input", type: "text", maxlength: SET_NAME_MAX, autocomplete: "off", spellcheck: "false", value: app.sets[i].name ?? "", placeholder: `${i + 1}` });
      const commit = (): void => { app.renameSet(i, input.value); close(); };
      input.addEventListener("keydown", (e) => { if (e.key === "Enter") { e.preventDefault(); commit(); } });
      setTimeout(() => input.focus(), 0);
      return h("div", { class: "sheet-body" }, h("div", { class: "label row-label" }, /* copy:label */ "name"), input, h("button", { class: "btn primary wide", onclick: commit }, /* copy:button */ "ok"));
    });
  }
  function paintVault(): void {
    const L = app.lineage; const slots = vaultSlots(L.unlocks);
    clear(vault);
    vault.appendChild(h("div", { class: "label row-label" }, /* copy:label */ "stored gear", " ", h("span", { class: "num dim" }, `${L.vault.length}/${slots}`)));   // Cut 17: `forge` is its console tile
    const chips = h("div", { class: "chips" });
    for (const it of L.vault) {
      const on = app.loadout.includes(it.id);
      chips.appendChild(kwHost(h("button", { class: `chip item${on ? " on risk" : ""}`, "data-rarity": it.rarity ?? "common", onclick: () => {
        app.setLoadout(on ? app.loadout.filter((x) => x !== it.id) : [...app.loadout, it.id]);
      } }, on ? "⚠ " : "", itemIcon(it, { size: "s" }), itemName(it), keptAs(L.facts, it) ? h("small", { class: "dim flav" }, ` · ${keptAs(L.facts, it)}`) : ""), "rarity"));   // run-clear: its icon in its rarity rim, its name tinted
      if (on) {
        const ins = (L.insured ?? []).includes(it.id);
        const price = Math.ceil(salvageValue(it.kind, "bank") * 10 / 4);
        // QA e75ec29 (R: "an unlabelled `$75` chip; one tap charged $75"): `insure $75`, a second tap pays
        chips.appendChild(ins ? h("button", { class: "chip mini on", disabled: true }, /* copy:label */ "insured")
          : twoTap(/* copy:button */ `insure $${price}`, /* copy:button */ `ok $${price}`, () => void app.mutate(() => app.engine.insure(it.id)), { class: "chip mini insure", disabled: L.gold < price }));
      }
    }
    // an empty slot is a plain marker, never a tap target (QA on 952e306: "vault slot '·' tap: nothing happened")
    for (let i = L.vault.length; i < slots; i++) chips.appendChild(h("span", { class: "chip empty", "aria-hidden": "true" }, ""));   // an empty slot is an empty chip (QA on 56f2a1d: `·` read as a chip that says `·`)
    vault.appendChild(chips);
    // QA 308f045 (qaAC: `home armour` answered `vault full · axe stays` and nothing took the axe out): a full vault offers each item out —
    // `sell axe`, a second tap salvages it at a bank's share (the gold sheet's `salvage` line), and the slot is free for the preference
    if (app.engine.sellVault && slots > 0 && L.vault.length >= slots) vault.appendChild(h("div", { class: "chips vault-sell" }, ...L.vault.map((it) =>
      twoTap([h("span", null, /* copy:button */ "sell "), itemChip(it, it.label.replace(/_/g, " "))], /* copy:button */ "ok", () => void app.mutate(() => app.engine.sellVault!(it.id), /* copy:callout */ "sold"), { class: "chip mini sell", key: `sell:${it.id}`, armedContent: () => [h("span", null, /* copy:button */ "ok "), itemChip(it)] }))));
    // keep preference for offline exits
    // QA 23ed91f: two rows that cannot be confused — `home` (what an unwatched exit keeps for the vault) and `cage` (what an
    // unanswered cage in the dungeon takes); K set `vault potion` as "what the home vault keeps"
    const prefs = h("div", { class: "chips prefs home" }, h("span", { class: "dim" }, conceptIcon("vault"), /* copy:callout */ "keep for heirs", conceptCap("vault")),   // docs/COPY.md pass 3: `keep weapon` read as "keep it as a weapon"
      /* copy:label */ ...[["best_weapon", "weapon"], ["best_armour", "armour"], ["none", "none"]].map(([id, lbl]) =>
        h("button", { class: `chip${(L.keep_pref ?? "best_weapon") === id ? " on" : ""}`, onclick: () => void app.mutate(() => app.engine.setKeepPref(id)) }, lbl)));
    // what the home pref does at an unwatched exit, on its row (the core's `keep_auto`, in order: `keeps armour · weapon`)
    const auto = L.keep_auto;
    // QA a946e04 (S: `VAULT 1/1 · axe · keeps armour`, the axe stayed through 17 runs): a keep replaces only a weaker item of its own
    // kind — a vault full of other kinds takes none, and the line says so (`keeps armour · vault full`)
    const CAT: Record<string, RegExp> = { weapon: /^(dagger|sword|axe|bow|spear|mace)$/, armour: /^(leather|mail|plate|scale)$/ };   // core defs.rs
    const blocked = !!auto?.length && L.vault.length >= slots && auto.every((k) => !L.vault.some((v) => CAT[k]?.test(v.kind) ?? v.kind === k));
    // QA 912e135 (qaX: `vault full` shown under `keeps weapon` and gone under `keeps armour` with the vault still 1/1): `vault full` whenever
    // it is; that the preference then keeps nothing new is its own word (`none kept`)
    const full = L.vault.length >= slots && slots > 0;
    // QA 524827b (qaAB: `keeps weapon · vault full · summon ally scroll stays`, later `keeps armour · weapon · vault full` — "two keeps, and
    // the preference does nothing"): the kinds read as one list (`keeps armour + weapon`); a full vault of other kinds keeps none of them,
    // and then the line says only that (`vault full · scroll stays`), never a `keeps` that will not happen
    const stays = blocked ? (L.vault.length === 1 ? (L.vault[0].label ?? L.vault[0].kind).replace(/_/g, " ") : "all") : "";
    if (auto) prefs.appendChild(h("small", { class: "keep-auto dim num" }, blocked ? "" : auto.length ? /* copy:callout */ `keeps ${auto.join(" + ")}` : /* copy:callout */ "keeps nothing",
      full ? h("b", { class: `${blocked ? "warn " : ""}vault-full` }, blocked ? /* copy:callout */ "storage full" : /* copy:callout */ " · storage full", blocked ? /* copy:callout */ ` · ${stays} stays` : "") : ""));   // QA 0c6e126 (qaY: `vault full · none kept` beside a vault holding mail — "the vault holds armour, keeps weapon"): the line names what stays
    vault.appendChild(prefs);
    // Cut 19 §1: the cage's preference left this panel for its own tablet beside the rules (`cage → armour`)
    // QA 912e135 (qaW: `home: armour` set here, `cage → weapon` on the tablet — "one preference, one name"): they are two settings, and
    // this panel shows both — the cage's row under the home row, its chip opening the same picker as the tablet
    if (revealed(app).has("cage")) vault.appendChild(h("div", { class: "chips prefs cage" }, h("span", { class: "dim" }, /* copy:rule_token */ "from cages"),
      h("button", { class: "chip on cage-pref", onclick: () => openCagePicker() }, L.vault_pref ?? "weapon")));   // docs/COPY.md: the tablet's own words (`cage pick → weapon`)
  }
  function paintCage(): void {
    const on = revealed(app).has("cage");
    cageTab.hidden = !on || ordersOn();
    if (!on) return;
    replace(cageTab, h("span", { class: "rn num" }, conceptIcon("cage")),
      // QA 524827b (qaAA: `cage → weapon` after the first death, "no source" for the D4 cage's "take one, leave two"): the tablet names
      // what it sets — the pick at a cage (`cage pick → weapon`)
      h("span", { class: "rtext" }, /* copy:rule_token */ "from cages", h("span", { class: "arrow" }, " → "), app.lineage.vault_pref ?? "weapon", conceptCap("cage")));
  }
  /** Cut 19 §1: the picker — the four preferences, each with its forecast delta against the current one (`armour +36%`); the tap sets it.
   *  The deltas are `cageForecast()` (three extra camp panels, memoised by the core; seconds in wasm): the last measure paints at once
   *  when it is this set's, `…` until the fresh one lands. */
  function openCagePicker(anchor: HTMLElement = cageTab): void {
    // QA 0c6e126 (qaY: `weapon D5 72%` on the sheet beside the camp's D5 66% — the memo was measured before a purchase): the memo is
    // this set's under this lineage — what a sim starts from (`simKey`: the purse, the shelf, the vault, the kit, the facts…)
    // QA 524827b (qaAA: `weapon D6 6%` on the sheet under the camp's refined `D6 11%`; `armour 60% ▲54`, picked: `57% · cage +46`): the
    // options are measured on the pass the camp shows (`refined`), so the current one is the camp's number and a pick lands where it said
    const refined = app.lastForecast?.refined === true;
    const key = (): string => JSON.stringify([app.rules.rows, (app.rules as { route?: number[] }).route ?? [], app.lineage.vault_pref ?? "weapon", app.lineage.best_depth, simKey(app), refined]);
    openSheet((close) => {
      const list = h("div", { class: "chips cage-opts" });
      const paint = (opts: CageOption[] | null, pending: boolean): void => {
        const cur = app.lineage.vault_pref ?? "weapon";
        replace(list, ...CAGE_PREFS.map((p) => {
          const o = opts?.find((x) => x.pref === p); const d = o ? cageDelta(o) : null;
          return h("button", { class: `chip cage-opt${p === cur ? " on" : ""}`, "data-pref": p, onclick: async () => { close(); if (p !== cur) await app.mutate(() => app.engine.setVaultPref(p), /* copy:callout */ "loot choice"); } },
            h("span", null, p), d ? h("b", { class: `num ${d.cls === "cur" ? "level cur" : `delta ${d.cls}`}` }, ` ${d.text}`) : pending && p !== cur ? h("small", { class: "num dim" }, " …") : "");
        }));
      };
      const k = key(), memo = cageMemo?.key === k ? cageMemo.opts : null;
      paint(memo, !memo && !!app.engine.cageForecast);
      if (!memo && app.engine.cageForecast) void app.engine.cageForecast(refined).then((opts) => { cageMemo = { key: k, opts }; if (list.isConnected) paint(opts, false); }).catch((e) => { console.warn("cageForecast", e); if (list.isConnected) paint(null, false); });
      return h("div", { class: "sheet-body cage-picker" }, h("div", { class: "label row-label" }, /* copy:label */ "loot preference"), list);
    }, { anchor: ordersOn() ? ordersTab : anchor });   // Cut 23 §4: beside the tablet it sets, never over it
  }
  function paintStart(): void {
    const L = app.lineage, on = revealed(app).has("start");
    startTab.hidden = !on || ordersOn();
    if (!on) return;
    const st = L.start ?? 1, o = startMemo?.opts.find((x) => x.start === st && x.current), toll = L.start_toll ?? startToll(st, o);
    // QA a946e04 (T): a toll the purse cannot pay says so on the tablet — that send starts on D1 (`start → D5 · $50 short`)
    // QA a946e04 (core: the toll buys a pass for the night): a held pass reads `pass`, not a toll the next send will not pay
    const pass = st > 1 && (L.start_pass === true || o?.pass === true);
    const short = !pass && startShort(L, st, o);
    startTab.classList.toggle("short", short);
    replace(startTab, h("span", { class: "rn num" }, conceptIcon("waystone")),
      h("span", { class: "rtext" }, /* copy:rule_token */ "start", h("span", { class: "arrow" }, " → "), h("span", { class: "num" }, `D${st}`),
        st > 1 && seenForks(L).length && laneOf(st, o) ? h("span", { class: "lane" }, ` ${laneOf(st, o)}`) : "",
        pass ? h("small", { class: "num toll pass dim" }, /* copy:rule_token */ " · pass")
          : toll > 0 ? h("small", { class: `num toll${short ? " short warn" : " dim"}` }, short ? /* copy:rule_token */ ` · $${toll} short` : ` · $${toll}`) : "", conceptCap("waystone")));
  }
  /** Cut 21 §1: the start picker — D1 and each lit waystone, each with its forecast move against the current start (`bank +12%`, the
   *  cage picker's measure) and its toll (`D9 · bank +12% · $90`); the tap is `setStart`. The moves are `startForecast()` (extra camp
   *  panels, memoised by the core; seconds in wasm): the last measure paints at once when it is this set's, `…` until one lands. */
  /** The current start's option with the camp forecast's own numbers (`app.lastForecast`, the shaft's and the panel's) when it is
   *  this set's: its bank share, its reach at the option's depth. */
  function currentFromPanel(o: StartOption): StartOption {
    const f = app.lastForecast; if (!f?.ends || (f.start ?? app.lineage.start ?? 1) !== o.start) return o;
    const r = f.depths.find((x) => x.depth === o.depth)?.reach;
    return { ...o, bank: f.ends.bank, reach: r ?? o.reach, pm: f.ends.pm ?? o.pm, death: f.ends.death, gold: f.ends.gold, net: o.net !== undefined ? f.ends.gold - (o.toll ?? 0) : undefined };
  }
  function openStartPicker(): void {
    const L0 = app.lineage;
    // QA 308f045 (qaAD: `D1 · bank 90% · death 10%` beside the shaft's `bank 92% · death 8%`): the starts on the pass the camp shows
    const refined = app.lastForecast?.refined === true;
    const key = (): string => JSON.stringify([app.rules.rows, (app.rules as { route?: number[] }).route ?? [], app.lineage.start ?? 1, app.lineage.waystones ?? [], app.lineage.best_depth, simKey(app), refined]);
    openSheet((close) => {
      const list = h("div", { class: "chips start-opts" });
      const paint = (opts: StartOption[] | null, pending: boolean): void => {
        const cur = app.lineage.start ?? 1;
        // QA 308f045 (qaAD: `Waystone D5 lit.` on the burrows route, then the start sheet's `D5 burrows` greyed `other-lane`): which lit
        // pairs are this set's is read against the set's route as the camp holds it now (`onRoute`), never a lineage read from before the
        // route edit reached the core
        const lanes = app.lineage.lanes;
        const starts = [...new Set([1, ...(lanes?.length ? lanes.filter((x) => onRoute(x)).map((x) => x.depth) : (app.lineage.waystones ?? L0.waystones ?? []))])].sort((a, b) => a - b);
        // Cut 26 §2: a waystone is lit per route prefix — once a fork was seen each start names its lane (`D9 crypt`: the pair lit)
        const laneOn = seenForks(app.lineage).length > 0;   // (no fork seen: every lane is the base order's — nothing to name)
        replace(list, ...starts.map((st) => {
          const o = opts?.find((x) => x.start === st); const toll = startToll(st, o);
          // QA a946e04 (T: `START D1 · bank 86%` beside the panel's `bank 85%`): the current start's own level is the camp forecast's —
          // the number the shaft and the panel show — never a second measure of the same set
          const oo = o ? (o.current ? currentFromPanel(o) : o) : undefined;
          // QA 778fa1b (qaV: `D1 · bank 100%` beside `D5 · bank −10 · death 10% · $25` — "I read D5 as −10 gold"): every row in one
          // form, absolute — `D5 · bank 40% · death 10% · ~$25` (the bank share, or the reach at the option's depth when no start banks;
          // the gold a send brings home net of the toll), the current start marked by its lit chip
          const banks = (opts ?? []).some((x) => x.bank > 0);
          const lo = oo?.low ?? lowOf(app.lastForecast);
          const d = oo ? { text: banks ? `${/* copy:label */ "bank"} ${share(oo.bank, lo)}` : `D${oo.depth} ${share(oo.reach, lo)}`, cls: "level" } : null, death = oo?.death;
          const net = oo ? Math.round(oo.net ?? oo.gold - (oo.toll ?? 0)) : undefined;
          // QA a946e04 (T): a toll the purse cannot pay dims its option and says so (`D5 · $50 short`); the current one stays lit
          const pass = st > 1 && (o?.pass === true || (st === cur && app.lineage.start_pass === true));
          const short = !pass && startShort(app.lineage, st, o);
          return h("button", { class: `chip start-opt${st === cur ? " on" : ""}${short ? " off short" : ""}`, "data-start": st, disabled: short && st !== cur,
            onclick: async () => { close(); if (st !== cur && app.engine.setStart) await app.mutate(() => app.engine.setStart!(st)); } },
            h("span", { class: "num" }, `D${st}`), st > 1 && laneOn && laneOf(st, o) ? h("span", { class: "lane", "data-biome": laneOf(st, o) }, ` ${laneOf(st, o)}`) : "",
            d ? h("b", { class: `num level${st === cur ? " cur" : ""}` }, ` · ${d.text}`) : pending && st !== cur ? h("small", { class: "num dim" }, " …") : "",
            // Cut 22 §4 (AG: "`D9 · bank +3% · $90` — but the shaft then says death 61%"): the start's death share beside its bank
            d && death !== undefined ? h("span", { class: `num start-death${death >= 0.5 ? " warn" : ""}` }, /* copy:callout */ ` · death ${share(death, lo)}`) : "",
            d && net !== undefined ? h("span", { class: `num start-gold gold${net <= 0 ? " warn" : ""}` }, ` · ~$${net}`) : "",
            // Cut 27 §1: a waystone start the set clears the floors above ≥ 95 % is paid their gold at the start (`+$84 passage`, in `~$`)
            oo?.passage ? h("small", { class: "num passage gold" }, /* copy:callout */ ` · +$${oo.passage} passage`) : "",
            pass ? h("small", { class: "num toll pass" }, /* copy:callout */ " · pass") : toll > 0 ? h("small", { class: `num toll${short ? " warn" : ""}` }, short ? /* copy:callout */ ` · $${toll} short` : ` · $${toll}`) : "");
        }),
        // Cut 26 §2: the (lane, depth) pairs lit on another route (`Lineage.lanes`, not `current`) — shown dim, not a start for this set
        ...(app.lineage.lanes ?? []).filter((x) => !onRoute(x) && x.depth > 1).map((x) => h("span", { class: "chip start-opt other-lane off dim", "data-start": x.depth, "data-biome": x.lane, "aria-disabled": "true" },
          h("span", { class: "num" }, `D${x.depth}`), h("span", { class: "lane" }, ` ${x.lane}`), x.route?.length ? h("small", { class: "num dim" }, ` · fork ${x.route.map((f) => `D${f}`).join(" ")}`) : "")));
      };
      const k = key(), memo = startMemo?.key === k ? startMemo.opts : null;
      paint(memo, !memo && !!app.engine.startForecast);
      if (!memo && app.engine.startForecast) void app.engine.startForecast(refined).then((opts) => { startMemo = { key: k, opts }; if (list.isConnected) paint(opts, false); paintStart(); }).catch((e) => { console.warn("startForecast", e); if (list.isConnected) paint(null, false); });
      return h("div", { class: "sheet-body start-picker" }, h("div", { class: "label row-label" }, /* copy:label */ "start"), list);
    }, { anchor: ordersOn() ? ordersTab : startTab });
  }
  /** Cut 29 §4: the orders tablet stands for the cage's and the start's when the core sends the orders with its curriculum. */
  function apprenticeOn(): boolean { return !!app.lineage.tree?.nodes.some((n) => n.id === "apprentice" && n.state === "done"); }
  function ordersOn(): boolean { return !!app.lineage.orders && hasCurriculum(app.lineage) && !!app.engine.setOrders; }
  function setOrder(patch: Partial<StandingOrders>, move: string): Promise<boolean> {
    const cur = app.lineage.orders!;
    return app.mutate(() => app.engine.setOrders!({ ...cur, ...patch }), move);
  }
  function paintOrders(): void {
    const L = app.lineage, R = revealed(app);
    const on = ordersOn() && (R.has("loadout") || R.has("vault") || R.has("cage") || R.has("start"));
    ordersTab.hidden = !on;
    if (!on) return;
    const o = L.orders!;
    const bits = [
      /* copy:callout */ `keep ${KEEP_WORD[o.keep] ?? o.keep}`,
      R.has("cage") ? /* copy:callout */ `loot → ${o.cage}` : "",
      R.has("start") && o.start > 1 ? /* copy:callout */ `start D${o.start}` : "",
      o.repeat ? "" : /* copy:callout */ "restock off",
      o.insure ? "" : /* copy:callout */ "insurance off",
      apprenticeOn() && o.forge && o.forge !== "half" ? /* copy:callout */ `forge ${o.forge}` : "",
    ].filter(Boolean);
    replace(ordersTab, h("span", { class: "rn num" }, icon("ledger", "☰")), h("span", { class: "rtext" }, h("b", { class: "orders-head" }, /* copy:label */ "run setup"), " ", h("span", { class: "orders-sum dim num" }, bits.join(" · "))));
  }
  function openOrders(): void {
    openSheet((close) => {
      const body = h("div", { class: "sheet-body orders-sheet" }, h("div", { class: "label row-label" }, /* copy:label */ "Run setup"));
      const paint = (): void => {
        const row = (label: string, ...chips: HTMLElement[]): HTMLElement => h("div", { class: "order-row" }, h("span", { class: "olab" }, label), " ", h("span", { class: "chips" }, ...chips.flatMap((c, i) => [i ? " " : "", c])));
        const pick = <T,>(cur: T, v: T, text: string, set: () => void): HTMLElement => h("button", { class: `chip order${cur === v ? " on" : ""}`, "aria-pressed": cur === v ? "true" : "false", onclick: () => { if (cur !== v) set(); } }, text);
        const act = (p: Partial<StandingOrders>, move: string) => (): void => { void setOrder(p, move).then(() => { if (body.isConnected) { replace(rows, ...build()); paintOrders(); } }); };
        const build = (): HTMLElement[] => {
          const o = app.lineage.orders!, R = revealed(app), packed = refundableSupplies();
          const repeatOff = o.repeat && packed.length
            ? twoTap(/* copy:button */ "off", /* copy:callout */ `refund ${packed.length}?`, act({ repeat: false }, /* copy:callout */ "repeat"),
                { class: "chip order", key: `repeat-refund:${packed.map(x => x.id).join(",")}` })
            : pick(o.repeat, false, /* copy:button */ "off", act({ repeat: false }, /* copy:callout */ "repeat"));
          repeatOff.setAttribute("aria-pressed", String(!o.repeat));
          return [
          row(/* copy:callout */ "keep for heirs", ...KEEP_ORDERS.map((k) => pick(o.keep, k, KEEP_WORD[k], act({ keep: k }, /* copy:callout */ "keep")))),
          R.has("cage") ? row(/* copy:label */ "loot preference", h("button", { class: "chip order on", onclick: () => { close(); openCagePicker(ordersTab); } }, o.cage, h("small", { class: "dim" }, " ▸"))) : null,
          R.has("start") ? row(/* copy:label */ "start", h("button", { class: "chip order on", onclick: () => { close(); openStartPicker(); } }, `D${o.start}`, h("small", { class: "dim" }, " ▸"))) : null,
          row(/* copy:label */ "Auto restock", pick(o.repeat, true, /* copy:button */ "on", act({ repeat: true }, /* copy:callout */ "repeat")), repeatOff),
          row(/* copy:label */ "insure kit", pick(o.insure, true, /* copy:button */ "on", act({ insure: true }, /* copy:callout */ "insure")), pick(o.insure, false, /* copy:button */ "off", act({ insure: false }, /* copy:callout */ "insure"))),
          // blind 1fb7786 (A, B: "the apprentice spent my gold without asking"): what he may forge with — half of each haul (default), all, off
          apprenticeOn() && o.forge ? row(/* copy:label */ "apprentice forges", ...FORGE_ORDERS.map((f) => pick(o.forge, f, FORGE_WORD[f], act({ forge: f }, /* copy:callout */ "forge")))) : null,
        ].filter((x): x is HTMLElement => !!x); };
        const rows = h("div", { class: "order-rows" }, ...build());
        body.appendChild(rows);
      };
      paint();
      return body;
    }, { anchor: ordersTab, center: true });
  }
  /** Cut 26 §2: the lane a start sits in for this set — the core's lit pair on this set's route (`onRoute`), the option's own biome, else
   *  the base order's table while the set takes every near stair */
  function laneOf(st: number, o?: StartOption): string | undefined {
    return app.lineage.lanes?.find((x) => x.depth === st && onRoute(x))?.lane ?? o?.biome ?? (!routeForks(app.rules).length && !app.lineage.forks?.length ? biomeAt([], st) : undefined);
  }
  /** QA 308f045 (qaAD): a lit (lane, depth) pair is this set's when its route prefix — the far stairs taken above its floor — is the set's
   *  own prefix to that floor (the core's `Route::prefix`), read off the set as it is now. */
  function onRoute(x: { depth: number; route?: number[] }): boolean {
    const mine = routeForks(app.rules).filter((f) => f <= x.depth).sort((a, b) => a - b), theirs = [...(x.route ?? [])].sort((a, b) => a - b);
    return mine.length === theirs.length && mine.every((f, i) => f === theirs[i]);
  }
  function paintRoute(): void {
    const chips = routeChips(app.rules, app.lineage);
    routeTab.hidden = !chips.length || !sysOpen(app.lineage, "route");   // Cut 29 §2: the route opens with the D5 fork seen twice
    if (!chips.length) return;
    // a fork the stair above closed (its biome deferred here) is no choice: its chip is dim
    // docs/COPY.md pass 3: the `⑂` read "no idea" 8/8, the chip `D5 burrows` as "on floor 5" — the tablet says what it sets: the route taken
    // at each fork (`route · D5 burrows`)
    replace(routeTab, h("span", { class: "fork-glyph dim" }, /* copy:label */ "route"), " ",
      ...chips.flatMap((c, i) => [i ? h("span", { class: "sep dim" }, " · ") : "", h("span", { class: `route-chip${c.isFar ? " far" : ""}${c.open ? "" : " closed dim"}`, "data-fork": c.depth, "data-biome": c.taken }, h("span", { class: "num" }, `D${c.depth}`), " ", c.taken)]));
  }
  /** Cut 26 §2: the fork tablet — per open fork, both stairs for the current set: the lane and its reach at the band's end (`fens D8 61%
   *  · burrows D8 34%`), the core's `forkForecast(fork)` (first-pass sims on the camp's seeds, memoised; seconds in wasm) painting `…`
   *  until it lands; the stair the set takes lit. A tap writes the route — an edit of the set (the forecast reprices; `vs sent` reads it). */
  function openRoutePicker(): void {
    // QA 308f045 (qaAC: the sheet's `fens D8 12%`, picked, the shaft's `D8 16%`): both stairs on the pass the camp shows (`refined`), so the
    // stair taken is the shaft's number and a pick lands where the sheet said (the cage sheet's rule)
    const refined = app.lastForecast?.refined === true;
    const key = (fork: number): string => JSON.stringify([fork, app.rules.rows, routeForks(app.rules), app.lineage.best_depth, simKey(app), app.lineage.facts?.length, refined]);
    openSheet((close) => {
      const list = h("div", { class: "route-opts" });
      const got = new Map<number, ForkOption[] | null>();   // per fork: the options landed (null: pending or none)
      const paint = (): void => {
        const r = routeForks(app.rules), lo = lowOf(app.lastForecast);
        replace(list, ...routeChips(app.rules, app.lineage).filter((c) => c.open).map((c) => {
          const opts = got.get(c.depth) ?? null, pending = !!app.engine.forkForecast && !opts;
          const stair = (far: boolean): HTMLElement => {
            const biome = far ? c.far : c.near, cur = c.isFar === far;
            const o = opts?.find((x) => x.far === far);
            // the current stair's level is the camp forecast's own (the shaft's number) when it is this set's, at the option's bar
            const at = o?.depth, said = cur && at !== undefined && app.lastForecast && app.lastForecast.known_to >= at ? app.lastForecast.depths.find((x) => x.depth === at)?.reach : undefined;
            const reach = said ?? o?.reach, low = o?.low || lo;
            return h("button", { class: `chip route-opt${cur ? " on" : ""}`, "data-fork": c.depth, "data-far": far ? "1" : "0", "data-biome": biome,
              onclick: () => { close(); if (!cur) setRoute(o?.route ?? withFork(r, c.depth, far, app.lineage)); } },
              // QA 308f045 (qaAC: the sheet opened on `burrows … · fens …` numbers that moved ~8 s later): a first-pass number reads as one
              // (`…` after it, dim) until the camp's refine lands — the forecast's own mark
              h("span", null, biome), reach !== undefined && at !== undefined ? h("b", { class: `num level${cur ? " cur" : ""}${refined ? "" : " rough"}` }, ` D${at} ${share(reach, low)}`, refined ? "" : h("small", { class: "dim" }, "…")) : pending ? h("small", { class: "num dim" }, " …") : "");
          };
          return h("div", { class: "route-fork", "data-fork": c.depth }, h("span", { class: "num fork-at" }, /* copy:callout */ `fork D${c.depth}`), stair(false), stair(true));
        }));
      };
      paint();
      for (const c of routeChips(app.rules, app.lineage).filter((x) => x.open)) {
        const k = key(c.depth), memo = forkMemo.get(k);
        if (memo) { got.set(c.depth, memo); continue; }
        if (app.engine.forkForecast) void app.engine.forkForecast(c.depth, refined).then((opts) => { forkMemo.set(k, opts); got.set(c.depth, opts); if (list.isConnected) paint(); })
          .catch((e) => { console.warn("forkForecast", e); got.set(c.depth, []); if (list.isConnected) paint(); });
      }
      if (forkMemo.size > 32) forkMemo.clear();
      paint();
      return h("div", { class: "sheet-body route-picker" }, h("div", { class: "label row-label" }, /* copy:label */ "route"), list);
    }, { anchor: routeTab });
  }
  function setRoute(route: number[]): void {
    const set = app.rules;
    if (route.length) set.route = route; else delete set.route;
    app.rulesChanged(); paintRoute();
  }
  function refundableSupplies(): InvItem[] {
    return (app.lineage.supplies ?? []).filter(x => !isFreeSupply(app.lineage, x) && !x.found);
  }
  /** Cut 19 §3: the loadout repeats by default — the tile carries `repeat · $120` (the kinds the next send re-packs, at the shelf's
   *  price); a tap on it clears the repeat (`setRestock(false)`, the shelf refunded), `repeat off` a tap turns it back on. */
  function repeatBadge(): HTMLElement | null {
    const L = app.lineage;
    if (!app.engine.setRestock || L.repeat === undefined) return null;
    const on = L.repeat !== false;
    if (on && !(L.repeat_kinds?.length)) return null;   // nothing to re-pack
    // QA 778fa1b (qaU: `repeat on · $26` over a `1/3` shelf the absence's capped re-pack left empty): the core's read of the next send —
    // kinds it cannot pay (`repeat_unpaid`: the purse after the toll, or the shelf's cap) read `repeat short`; kinds it buys at the send
    // (`repeat_due`) read `+heal at send`; else the price the repeat charges
    const unpaid = on && !!L.repeat_unpaid?.length, due = on && !unpaid ? L.repeat_due ?? [] : [];
    const short = on && (unpaid || (L.repeat_unpaid === undefined && !!L.repeat_short?.length));
    const dueText = due.length === 1 ? `+${due[0].replace(/_/g, " ")}` : `+${due.length}`;

    // QA 308f045 (qaAD: one tap on `repeat on · held ≤$24` → `repeat off`, and four packed supplies refunded, `loadout 5/5` → `1/5`): turning
    // the repeat off with bought supplies on the shelf unpacks them — the first tap says so (`refund 4`), only the second does it
    const packed = on ? refundableSupplies().length : 0;
    let armed = false;
    const badge: HTMLElement = h("span", { class: `repeat-badge num${on ? " on" : ""}${short ? " short" : ""}${due.length ? " due" : ""}`, role: "switch", "aria-checked": on ? "true" : "false", "data-repeat": on ? "1" : "0",
      onclick: (e: Event) => {
        e.stopPropagation();
        if (packed > 0 && !armed) { armed = true; badge.classList.add("armed"); replace(badge, /* copy:callout */ `refund ${packed}?`); return; }
        void app.mutate(() => app.engine.setRestock!(!on), /* copy:callout */ "restock");
      } },
      // QA 1a2a4a9 (P: "the restock was skipped with no word"): a re-pack the purse could not pay reads so on the tile
      // QA a946e04 (T: "`repeat · $40` reads like a price to pay"; its tap refunded $40): the badge is a switch and reads as one —
      // `repeat on · $40` (the tap turns it off and refunds the re-packed shelf) / `repeat off`
      on ? (short ? /* copy:callout */ "Restock needs gold" : due.length ? /* copy:callout */ `${dueText} at send` : /* copy:callout */ `Restock on · held ≤$${L.repeat_gold ?? 0}`) : /* copy:callout */ "Restock off");
    return badge;   // QA 524827b (qaAB: `≤$24` beside the shop's `heal potion $26` — the repeat pays the quote it showed, held)   // QA 778fa1b (qaV: `repeat on · $104` read as a per-send cost; nothing was charged when the supplies came back): the most it re-buys
  }
  function paintSupplies(): void {
    const L = app.lineage; const picks = L.supplies ?? []; const cap = L.supply_cap ?? supplyCap(L.unlocks); const full = picks.length >= cap;   // Cut 29 §6 (AX: `pack 4` bought, the shelf read 3/3): the core's cap (forge pack steps included)
    const adds = (L.repeat_added ?? []).filter((a) => !picks.some((p) => p.kind === a.kind));
    repeatAdd.hidden = !adds.length || !revealed(app).has("loadout");
    replace(repeatAdd, ...adds.map((a) => h("button", { class: "chip repeat-add-chip num", "data-kind": a.kind, disabled: full, onclick: () => void app.mutate(() => app.engine.buySupply(a.kind), /* copy:callout */ "buy") },
      h("b", null, "+ ", itemChip({kind:a.kind, label:a.kind.replace(/_/g, " ")})), h("small", { class: "dim" }, /* copy:callout */ ` · for ${a.row}`))));
    clear(supplies);
    supplies.appendChild(h("div", { class: "label row-label" }, /* copy:label */ "supplies", " ", h("span", { class: "num dim" }, `${picks.length}/${cap}`)));
    // Cut 23 §4 (AJ: "a layout jump bought a confusion potion"): the shelf is `cap` fixed slots (a bought line fills the next empty one)
    // and the shop a fixed grid under it whose chips keep one height (the `why` line always there) — a buy never moves a chip under the finger
    const chips = h("div", { class: "chips packed" });
    // Cut 12 §6: each line carries its own `×` (the header's cleared the whole shelf: "I lost the leash"); a free line reads `· kennel`
    // QA 92eb880 (M, N: "`leash · kennel` — kennel?"; "× drops the free leash at once, no undo; re-buying costs $30"): a free line reads
    // `· free`, and its `×` takes two taps — the first arms it (`drop`), the second drops it. Cut 23 §4 (AI: "the drop confirm undid
    // itself"): armed, it stays armed until the second tap or a tap elsewhere (never a timer; a repaint keeps it); the `×` is ≥ 32 px
    for (const p of picks) {
      const free = isFreeSupply(L, p);
      const x: HTMLButtonElement = free
        ? twoTap("×", /* copy:button */ "drop", () => void app.dropSupply(p.id), { class: "x", key: `drop:${p.id}` })
        : h("button", { class: "x", "aria-label": /* copy:button */ "drop", onclick: () => void app.dropSupply(p.id) }, "×");
      // Cut 21 / Cut86: found supplies were packed free. An excluded repeat kind will not be
      // automatically bought again; this does not prove it cannot be used in the current run.
      const noRow = !free && !p.found && L.repeat !== false && L.repeat_kinds !== undefined && !L.repeat_kinds.includes(p.kind);
      chips.appendChild(h("span", { class: `chip item on${noRow ? " no-row" : ""}` }, h("span", { class: "item-l", title: SUPPLY_USE[p.kind] ? `${p.label} · ${SUPPLY_USE[p.kind]}` : p.label }, itemIcon(p, { size: "s" }), itemName(p, shelfLabel(p.label)), free ? h("small", { class: "dim found" }, /* copy:callout */ " · free") : p.found ? h("small", { class: "dim found shelf" }, /* copy:callout */ " · found") : "",
        noRow ? h("small", { class: "dim no-row", title: /* copy:tooltip */ "Not included in automatic restocking" }, /* copy:callout */ " · No restock") : ""), x));
    }
    for (let i = picks.length; i < cap; i++) chips.appendChild(h("span", { class: "chip slot empty", "aria-hidden": "true" }, ""));
    supplies.appendChild(chips);
    const shopEl = h("div", { class: "chips shop" });
    supplies.appendChild(shopEl);
    // the shop from the last catalogue at once, the engine's replacing it when it arrives (QA B on 952e306: "while FORECAST
    // shows '…' the SUPPLIES shop chips are gone"); the gold and the slots are read live either way
    const shop = (cat: SupplyEntry[]): void => {
      for (const b of [...shopEl.querySelectorAll(".chip.buy")]) b.remove();
      for (const e of cat) {
        const can = !full && !e.needs && L.gold >= e.price;
        // QA 0c6e126 (qaY: `leash · free` on the shelf and `leash $30` in the shop at once — "which is it?"): the shelf's is the kennel's
        // (free while nothing is tamed); the shop's is a second one, and says so
        const second = e.kind === "leash" && picks.some((p) => p.kind === "leash");
        // Cut 10 §3: a greyed supply says why under its price — the slots, the engine's gate, or the gold missing
        // QA 524827b (qaAA: `leash · free`, `leash $30 · 2nd leash` — "nothing says what a leash does"): a line whose name does not say its
        // use carries it (`tames a foe`: the leash is `tame`'s, a foe under a quarter hp); the buy stays one tap — its `×` refunds it
        const use = SUPPLY_USE[e.kind];
        // QA 524827b (qaAB: `summon ally scroll $33` beside the vault's own — read as packing the kept one; it bought a new one): a kind the
        // vault holds says the shop's is another (`new · 1 in vault`; the vault's chip packs the kept one)
        const vaulted = (L.vault ?? []).filter((v) => v.kind === e.kind).length;
        const why = full ? /* copy:callout */ `${picks.length}/${cap} slots` : e.needs ? e.needs.replace(/_/g, " ") : L.gold < e.price ? /* copy:callout */ `$${e.price - L.gold} short` : second ? /* copy:callout */ "2nd · tames foe" : vaulted ? /* copy:callout */ `new · ${vaulted} stored` : use ?? "";
        shopEl.appendChild(h("button", { class: `chip buy${can ? "" : " off"}`, disabled: !can, onclick: () => void app.mutate(() => app.engine.buySupply(e.kind), /* copy:callout */ "buy") },
          h("span", { class: "buy-main" }, h("span", {class:"buy-title"}, itemChip({kind:e.kind, label:e.label}), " ", h("b", { class: "num gold" }, e.price > 0 ? `$${e.price}` : /* copy:label */ "free")), h("small", { class: "why num dim" }, why || "\u00a0"))));   // QA 912e135: the kennel's leash, taken back
      }
    };
    if (app.supplyCat.length) shop(app.supplyCat);
    const gen = ++supplyGen;
    void app.engine.supplyCatalogue().then((cat) => {
      if (gen !== supplyGen) return;
      app.supplyCat = cat; shop(cat);
    }).catch((e) => console.warn("catalogue", e));
  }
  let supplyGen = 0, unlockGen = 0;
  let cellIds: string[] = [], cellHeights: number[] = [];   // Cut 25 §6: the unlock grid's fixed cells (this camp's)
  let unlockCat: Parameters<typeof classList>[1];
  function paintUnlocks(): void {
    const gen = ++unlockGen;
    // the shelf from the last catalogue at once (QA B on 952e306: "the whole UNLOCKS list is gone"); the engine's replaces it
    if (!unlockCat && app.unlockCat.length) { unlockCat = app.unlockCat; paintFrom(unlockCat); }
    void app.engine.unlocks().then((fresh) => {
      if (gen !== unlockGen) return;
      // QA 92eb880 (M: "the reach line vanishes after buying +1 row"; N: "the tile list reshuffles twice within ~4 s"): until the new
      // deltas land, each card keeps its last measured delta (the shelf's order and lines hold still); prices and gates are the fresh ones
      const prev = new Map((unlockCat ?? []).filter((u) => u.delta !== undefined).map((u) => [u.id, u]));
      const cat = fresh.some((u) => u.delta !== undefined) ? fresh : fresh.map((u) => { const p = prev.get(u.id); return p && !u.owned ? { ...u, delta: p.delta, pm: p.pm, stall: u.stall ?? p.stall, insert_at: p.insert_at ?? u.insert_at, situation: u.situation ?? p.situation, auto_insert: u.auto_insert ?? p.auto_insert } : u; });   // QA a946e04: the measured place rides with its measure (the sheet read `joins at R4`, the buy went in at R2)
      unlockCat = cat; app.unlockCat = cat;
      if (cat.some((u) => u.owned && u.rows?.length)) editor.refresh();   // Cut 6 §6: `[card]` chips open their rows once the catalogue is here
      // forecast deltas arrive later (0.3–2 s of sims); repaint once with them, never blocking the shelf
      if (!fresh.some((u) => u.delta !== undefined)) {
        // N (QA 92eb880: a tile read `$450` while its sheet charged `$562` after a gold buy): the deltas' catalogue keeps its deltas only —
        // the price, the gate and ownership are always the fresh catalogue's
        const byId = new Map(fresh.map((u) => [u.id, u]));
        void app.engine.unlockDeltas().then((withDeltas) => { if (gen === unlockGen && withDeltas.some((u) => u.delta)) {
          const merged = withDeltas.map((u) => { const f = byId.get(u.id); return f ? { ...f, delta: u.delta, pm: u.pm, stall: u.stall ?? f.stall, insert_at: u.insert_at ?? f.insert_at, situation: u.situation ?? f.situation, auto_insert: u.auto_insert ?? f.auto_insert } : u; });   // QA a946e04: the measured `auto_insert` rides the merge (it was dropped: every card inserted)
          unlockGen++; unlockCat = merged; app.unlockCat = merged; paintFrom(merged);
        } }).catch(() => { /* deltas are optional */ });
      }
      paintFrom(cat);
    }).catch((e) => console.warn("unlocks", e));
  }
  function paintFrom(cat: UnlockInfo[]): void {
    keepScroll(() => paintShelf(cat));
  }
  function paintShelf(cat: UnlockInfo[]): void {
    {
      // Cut 25 §6: the cells' heights as painted (a hidden panel measures 0: nothing recorded)
      const was = [...unlocks.querySelectorAll<HTMLElement>(".cards > .card")].map((el) => el.getBoundingClientRect().height);
      was.forEach((x, i) => { if (x > (cellHeights[i] ?? 0)) cellHeights[i] = x; });
      clear(unlocks);
      // Cut 10 §3: `+1 row` waits for the rows to fill (a client-side gate; the core may send the same `needs`)
      const list = visible(cat, app.lineage).map((u) => withRowsGate(u, app.ownRows(), app.vocab.max_rows));   // Cut 12 §1: own rows
      // Cut 6 §6: owned cards and automations stay on the shelf as chips that open their rows
      const owned = ownedRows(cat);
      if (!list.length && !owned.length) return;
      unlocks.appendChild(h("div", { class: "label" }, /* copy:label */ "unlocks"));
      const grid = h("div", { class: "cards" });
      // Cut 17 §3: a short list, not a wall — the next three (affordable first, then gated, then short of marks; the larger reach
      // gain, then the cheaper, then the catalogue's order); `more` opens the whole catalogue (remembered for this viewer)
      // Cut 18 §5: a tile the gold buys ranks with the ones the marks buy (and glows like them)
      const buyable = (u: typeof list[number]): boolean => u.available || goldAffordable(u, app.lineage.gold);
      const rank = (u: typeof list[number]): number => (buyable(u) ? 0 : u.gated ? 1 : 2);
      let next = list.map((u, i) => ({ u, i })).sort((a, b) => rank(a.u) - rank(b.u) || (b.u.delta ?? 0) - (a.u.delta ?? 0) || a.u.cost - b.u.cost || a.i - b.i).slice(0, 3).map((x) => x.u);
      // Cut 19 §3 (AA: "`+1 row` vanished"): a pinned unlock (the next `+1 row`) is always on the short list — it takes the last place
      const pins = list.filter((u) => u.pinned);
      if (pins.some((u) => !next.includes(u))) next = [...next.filter((u) => !u.pinned).slice(0, Math.max(0, 3 - pins.length)), ...pins];
      // QA 1a2a4a9 (O, P: "offers stay put between two opens"; the report's PENDING and the shelf differed): the core's short list
      // (`UnlockInfo.short`, from the lineage alone) when it sends one — the same three here and in the report
      if (list.some((u) => u.short !== undefined)) next = list.filter((u) => u.short);
      const picked = unlocksAll() || list.length <= 3 ? list : list.filter((u) => next.includes(u));
      // Cut 25 §6 (AN: "the grid reflowed under my finger after a buy — bought path: bones by accident"): fixed cells — while the camp is
      // up each card keeps the cell it was first painted in; a bought step's successor takes its cell (`+1 row` → the next `+1 row`), any
      // other bought card leaves its cell as `✓` (a tap there buys nothing); a new card takes the next free cell; a cell never shrinks
      // (a card that left the short list unbought frees its cell for the next one picked: the short list stays three)
      const byId = new Map(picked.map((u) => [u.id, u]));
      const cells: (typeof list[number] | string | null)[] = [];
      const placed = new Set<string>();
      for (const id of cellIds) {
        const succ = list.find((u) => afterOf(u.id) === id && !placed.has(u.id));
        const u = byId.get(id) ?? succ;
        if (u && !placed.has(u.id)) { cells.push(u); placed.add(u.id); }
        else if (cat.some((x) => x.id === id && x.owned)) cells.push(id);
        else cells.push(null);
      }
      for (const u of picked) if (!placed.has(u.id)) { const free = cells.indexOf(null); if (free >= 0) cells[free] = u; else cells.push(u); placed.add(u.id); }
      while (cells.length && cells[cells.length - 1] === null) cells.pop();
      cellIds = cells.map((c) => c === null ? "" : typeof c === "string" ? c : c.id);
      const shown = cells.filter((c): c is typeof list[number] => !!c && typeof c !== "string");
      for (const c of cells) {
        if (c === null) { grid.appendChild(h("div", { class: "card done empty", "aria-hidden": "true" })); continue; }
        if (typeof c === "string") { grid.appendChild(h("div", { class: "card done", "data-id": c, "aria-hidden": "true" }, h("span", { class: "card-main" }, h("span", null, labelOf(c))), h("span", { class: "num cost" }, "✓"))); continue; }
        const u = c;
        // `available` = prerequisite + fact gate + affordable (engine truth). Two dims: gated (the `needs` line
        // is what is missing, marks are there) and unaffordable.
        // Cut 4 §9: the forecast delta of buying (tactic cards), only when the catalogue carries one and it is not 0
        const d = deltaPts(u);   // Cut 13 §5: a delta within its ± paints as `reach ~0`
        // Cut 9 §2: the tap opens the sheet (rows, cost, needs, reach); the buy is on the sheet. A gated or unaffordable card
        // still opens it (the `needs` line is the answer), so nothing on the shelf is disabled.
        // Cut 12 §1: a card's delta is measured where it goes — `at R3` (the catalogue's `insert_at`), else `at end`
        const byGold = !u.available && goldAffordable(u, app.lineage.gold);
        grid.appendChild(h("button", { class: `card${u.available ? " buyable" : byGold ? " buyable gold-ok" : u.gated ? " gated" : " off"}`, onclick: () => openUnlockSheet(app, u) },
          // QA 92eb880 (N: "`AUTO: RESTOCK · ⊘ ◆1 more` while `$ buy` is enabled"): a marks shortfall the gold covers carries no `⊘`
          h("span", { class: "card-main" }, h("span", null, u.label), u.carries ? h("small", { class: "carries dim" }, u.carries) : "", u.needs ? h("small", { class: "needs dim" }, u.gated && !byGold && !/^\$\d+ more$/.test(u.needs) ? "⊘ " : "", u.needs.replace(/_/g, " ")) : "",
            d ? h("small", { class: `num delta ${deltaClass(u, d)}` }, deltaLabel(u, d, app.rules.rows.length)) : "",
            stallLabel(u) ? h("small", { class: "num delta down stall-risk" }, stallLabel(u)) : ""),   // QA 92eb880: the stall risk before buying
          h("span", { class: "num cost" }, priceLabel(u))));
        (grid.lastElementChild as HTMLElement).dataset.id = u.id;   // QA 23ed91f: a free door reads no `◆0`; Cut 18 §5: both prices, `◆3 · $450`
      }
      unlocks.appendChild(grid);
      // a cell keeps the height it had (a delta line landing or a card's successor never pulls the cells under it up)
      [...grid.children].forEach((el, i) => { const hgt = cellHeights[i]; if (hgt) (el as HTMLElement).style.minHeight = `${hgt}px`; });
      if (shown.length < list.length) unlocks.appendChild(h("button", { class: "mini more", onclick: () => { setUnlocksAll(true); cellIds = []; cellHeights = []; paintFrom(cat); } }, /* copy:button */ "more"));
      // an owned chip reads `card: thief guard · owned` (QA on 952e306: "bought card appears at the end with no cost"); its sheet
      // carries the title and, for a card whose row was dropped, `insert`
      // QA e75ec29: a card owned but off the set carries its own `add` (the buy put it in only where the core measured it helps)
      if (owned.length) unlocks.appendChild(h("div", { class: "chips owned" }, ...owned.flatMap((u) => [h("button", { class: "chip mini owned", onclick: () => openOwnedSheet(app, u) }, u.label, h("small", { class: "dim" }, /* copy:callout */ " · owned")),
        isCard(u) && !app.holdsCard(u.id) ? h("button", { class: "chip mini add-card", "data-card": u.id, onclick: () => addCard(app, u) }, /* copy:button */ "add") : ""])));
    }
  }
  // Cut 4 §1: `send` waits while the set is over budget (the editor shows which row to drop). Cut 6 §4: it says so: `6/5 · drop one`.
  function paintSend(): void {
    // QA 308f045 (qaAD: the empty `set 2 · 0` tab tapped, the camp read `death >98% · ~$0` with SEND armed): a set with no rows is no send
    const empty = app.rules.rows.length === 0;
    const homeless = app.lineage.town?.home === false;
    const arriving = performance.now() < residentUntil;
    el.classList.toggle("awaiting-home", homeless);
    cons.el.hidden = homeless;
    restLine.hidden = homeless;
    const campMain = well.querySelector<HTMLElement>(".camp-main"); if (campMain) campMain.hidden = homeless || (onPackages(app.lineage) && app.lineage.best_depth === 0 && !(app.lineage.runs?.length));
    const cleared = app.lineage.ended;
    send.disabled = !cleared && (homeless || arriving || app.overBudget || empty);
    send.classList.toggle("pulse", !cleared && !app.overBudget && !empty);
    send.classList.toggle("small", !cleared && (app.overBudget || empty));
    // Speed is chosen once, in the watch. The send gem only sends.
    send.dataset.mode = app.watchMode;
    replace(send, cleared ? h("span", { class: "send-l" }, /* copy:button */ "Next descent") : arriving ? /* copy:callout */ "Hero arriving" : empty ? /* copy:callout */ "no rules" : app.overBudget ? /* copy:callout */ `${app.ownRows()}/${app.vocab.max_rows} · drop one`
      : h("span", { class: "send-l" }, isLive() ? /* copy:button */ "watch" : /* copy:button */ "send", sendMark(app.lineage)));   // Cut 12 §1: own rows; RUNS_UI: a run under way is watched
    send.dataset.live = isLive() ? "1" : "0";
    paintTabs();
    if (unlockCat) paintFrom(unlockCat);   // `+1 row` reads `⊘ fill rows` only while a free own row exists
  }
  /** Cut 23 §4: a repaint rebuilds the open panel's content — its scroll (and the well's) is kept, so nothing moves under the finger. */
  function keepScroll(fn: () => void): void {
    const boxes = [panelHost.querySelector<HTMLElement>(".panel-body"), panelHost.querySelector<HTMLElement>(".panel"), well];
    const tops = boxes.map((b) => b?.scrollTop ?? 0);
    fn();
    boxes.forEach((b, i) => { if (b && b.scrollTop !== tops[i]) b.scrollTop = tops[i]; });
  }
  function paintAll(): void { keepScroll(() => { town.paint(); paintPen(); paintStrip(); paintTiles(); paintTabs(); paintVault(); paintCage(); paintStart(); paintOrders(); paintWall(); paintRoute(); paintOath(); paintSupplies(); paintUnlocks(); party.refresh(); editor.refresh(); paintSend(); }); audio.drone(biomeOf(app.lineage.best_depth + 1)); }
  paintAll();
  // Cut 12 §6: `+1 row ⊘ fill rows` is the engine's read of its own set — refetched once an edit crossed `max_rows`
  const off = app.onChange(paintAll), offRules = app.onRules(paintSend), offShelf = app.onShelf(paintUnlocks);
  const offShadow = app.onForecast(() => { editor.paintShadow(); paintOath(); });
  // Cut 29 §2: the systems the core opened since the camp last looked glint on this paint (reveal.ts reads `new`); once shown the core
  // forgets them (`seenSystems`), quietly — the next paint is an ordinary one
  // (asked as the camp is left, never mid-edit: a mutating call re-syncs the forecast lanes, and one landing in a burst of edits cost the
  // next edit's first pass ~1 s in wasm — clarity:paint)
  const shownAt = anyNew(app.lineage) && app.engine.seenSystems ? performance.now() : -1;
  const seen = (): void => { if (shownAt >= 0 && performance.now() - shownAt >= SEEN_MS) app.seenPending = true; };   // the send clears them (watch.ts)
  const offLive = app.onLive(() => { paintRest(); if (String(isLive() ? 1 : 0) !== send.dataset.live) paintSend(); });
  return { el, dispose: () => { window.removeEventListener("riddle:focus-hero",focusHome); offLive(); lanes.dispose(); town.dispose(); exposeTown(null); wellRo?.disconnect(); off(); offRules(); offShelf(); offShadow(); clearTimeout(residentTimer); seen(); fc.dispose(); shaft.dispose(); scene.dispose(); bar.dispose(); setPanelEscape(null); audio.drone(null); setBusyHost(null); } };
}
