// Watch: viewer canvas full-bleed; HUD (hp, depth, alert), speed slow · fast · auto, ▶▶| skip, ⏸, callout ticker, bail.
//
// Pacing (Addendum E): the viewer owns the clock (10 ticks/s × speed). The engine worker is pumped in
// 10-tick batches whenever it is fewer than LEAD ticks ahead of the viewer, so events always arrive
// before the viewer needs them and the engine never runs far ahead (≤ 22 ticks, under the viewer's
// dead-air threshold). HUD hp/depth and the ticker are queued by tick and released at the viewer's
// clock, so what the numbers say matches what the sprites do.
//
// Cut 5 §5 — auto cadence (the default): the clock runs at 8× through dead stretches and drops to 1× while
// anything is near. "Near" is read off the engine, which is ≤ 22 ticks ahead of the viewer: a hostile in view
// or an item within 3 tiles in a step's snapshot, a telegraph / attack / hero hp change in its events. Each
// sighting holds 1× until AUTO_TAIL ticks after it (so hp unchanged for 20 ticks is the fast condition), and the
// viewer slows *before* the foe walks into frame because the engine saw it first. At 8× the pump runs the same
// LEAD with a 50 ms interval and a larger batch, so the engine still never runs dry or far ahead. Bail (§5)
// turns the auto rate into a flat 8× and the stake line reads `returning` until the exit sheet.
//
// Cut 5 §4 — the vault choice: a step whose snapshot carries `vault_choice` opens a sheet with the three items
// as chips and holds the clock at 1×; a tap is `choose(id)`. The engine's 50-tick grace runs watched or not, so
// an unanswered sheet closes on its own when `vault_choice` leaves the snapshot (the lineage's `vault_pref` took).
//
// Cut 7 §4 — rooms are scenes: a step whose `snapshot.room` holds ≥ SCENE_FOES awake hostiles (absent `room`: that many
// hostiles in view) opens a scene, and auto holds 1× until the room is clear or the hero leaves it (the id changes),
// AUTO_TAIL after; corridors and empty rooms run at 8×, telegraphs / attacks / hp loss still hold 1× on their own. A
// hostile in view or an item near no longer slows on its own (that was "30–60 s of pick up"). Ambient callouts keep the
// fast stretches legible: `D3 · 4 rooms` on a floor, `$47` on gold, `alert 3` when the clock ticks — one per 10 s,
// shown at 4× and up (the floor line at any speed). The run's end plays at 1×: a batch carrying the `exit` is held
// back from the viewer until its clock is ENDING_TICKS from the exit (so no dead-air jump swallows the walk-out), and
// auto runs 1× from there (a core `ending` marker sets the same point). At 8× the pump keeps the engine ≥ LEAD_FAST
// ticks ahead so the exit is seen in time.
//
// Cut 8A — the fight frame (docs/PLATEAU.md §A): the viewer cuts to its close fight framing (`setFrame("fight")`) when a
// scene opens, when any hostile stands adjacent to the hero, or when a boss is in view, and cuts back to the map AUTO_TAIL
// ticks after the last of those stops holding. Entry and exit are released at the viewer's clock (the engine runs ahead),
// so the cut lands when the foes are on screen. The fight frame runs at 1× whatever the mode; the map frame keeps the
// cadence above. `data-frame="map|fight"` on the element for tooling.
import type { App, Mounted } from "../app";
import type { Ev, ExitLine, Highlight, InvItem, ReturnReport, Row, Snapshot, StepResult, VaultChoice } from "../engine/types";
import { h, items, replace, spanOf } from "./dom";
import { makeViewer, type Viewer } from "./viewer";
import { verbsAt, xpToNext } from "../engine/classes";
import { openSheet } from "./sheet";
import { salvageValue } from "./salvage";
import { vaultSlots } from "./unlocks";
import { kindGlyph, verbLabel } from "./tokens";

type Tier = "bank" | "return" | "death";
type FrameName = "map" | "fight";
/** Cut 8A: the real renderer's frame cut (web/src/render/index.ts); the placeholder viewer has none. */
type FrameViewer = Viewer & { setFrame?(frame: FrameName, focus?: { x: number; y: number; radius: number }): void };
const INTERESTING = new Set(["hurt", "die", "telegraph", "pickup", "use", "fact", "steal", "ally", "descend", "exit", "spawn", "tame"]);
const LEAD = 12, BATCH = 10;        // ticks: pump when the engine is < LEAD ahead; step BATCH at a time
const BATCH_FAST = 12;              // at 8× the viewer eats 4 ticks per pump; a bigger batch keeps the queue fed through a slow step
const LEAD_FAST = 32;               // Cut 7 §4: at 8× the engine stays ≥ ENDING_TICKS ahead, so an exit is seen before its last 30 ticks
const ENDING_TICKS = 30;            // Cut 7 §4: the last ticks before any exit play at 1×
const SCENE_FOES = 2;               // Cut 7 §4: awake hostiles in the hero's room that make it a scene
const AMBIENT_MS = 10_000, AMBIENT_SHOW_MS = 1500;   // Cut 7 §4: one ambient callout per 10 s, shown 1.5 s whatever the speed
const PUMP_MS = 50;
type Mode = "slow" | "fast" | "auto";
const RATE: Record<Mode, number> = { slow: 1, fast: 4, auto: 8 };
const AUTO_FAST = 8, AUTO_TAIL = 20; // auto: 8× when nothing is near; 1× until AUTO_TAIL ticks after the last sighting / hp change
const EXIT_GRACE_MS = 4000;         // wait for the viewer to drain after an exit, at most this long
const PERSIST_MS = 5000;
const BOSS_BANNER_MS = 3000;        // Cut 2 §7: `boss · counter: known|unknown` on first sight
const REST_BEAT_MS = 1400;          // Cut 2 §1: `rest 12m` after the exit, before the exit flow continues
const CHORE_CALLOUT: Record<string, string> = { descend: /* copy:callout */ "descend", pick_up: /* copy:callout */ "pick up" }; // explore never (Cut 4 §4)
const HURT_MS = 600;                // Cut 4 §4: `−7 archer` in red
/** The cause of a `hurt` as one word: `goblin_archer` → `archer`. */
const oneWord = (cause: string): string => cause.replace(/_/g, " ").trim().split(/\s+/).pop() ?? "";

export function renderWatch(app: App): Mounted {
  const canvas = h("canvas", { class: "view" });
  const hpFill = h("span", { class: "fill" });
  const hpText = h("span", { class: "num" });
  const depth = h("span", { class: "num depth" });
  const alert = h("span", { class: "alert num" });
  const ticker = h("div", { class: "ticker" });
  const stake = h("div", { class: "stake num" });
  const banner = h("div", { class: "banner num" });
  const pause = h("button", { class: "hud-btn", onclick: () => togglePause() }, "⏸");
  const modeBtn: Record<Mode, HTMLButtonElement> = {
    slow: h("button", { class: "hud-btn", onclick: () => setMode("slow") }, /* copy:button */ "slow"),
    fast: h("button", { class: "hud-btn", onclick: () => setMode("fast") }, /* copy:button */ "fast"),
    auto: h("button", { class: "hud-btn on", onclick: () => setMode("auto") }, /* copy:button */ "auto"),
  };
  const skip = h("button", { class: "hud-btn", onclick: () => skipToEvent() }, "▶▶|");
  const bail = h("button", { class: "hud-btn bail", onclick: () => doBail() }, /* copy:button */ "bail");
  const el = h("main", { class: "watch" }, canvas,
    h("div", { class: "hud top" }, h("div", { class: "hp" }, h("span", { class: "track" }, hpFill), hpText), depth, alert, pause, stake),
    banner, ticker,
    h("div", { class: "hud bottom" }, modeBtn.slow, modeBtn.fast, modeBtn.auto, skip, bail));

  let viewer: Viewer | null = null;
  let mode: Mode = "auto", paused = false, slowUntil = -Infinity, lastHp = NaN;
  // Cut 7 §4: the scene's room (null = none) and the tick auto may run fast again after one ends; the ending's first tick;
  // the exit batch held back until the viewer is ENDING_TICKS from the exit; the ambient callout limiter; the last alert
  let scene: number | null = null, sceneUntil = -Infinity, endingFrom = Infinity;
  // Cut 8A: the fight frame — whether the engine's latest snapshot holds it, the viewer ticks it spans, the frame shown
  let fightOn = false, fightFrom = Infinity, fightUntil = -Infinity, frame: FrameName = "map";
  let held: { evs: Ev[]; snap: Snapshot; tier: Tier } | null = null;
  let lastAmbient = -Infinity, ambientUntil = 0, lastAlert = 0;
  let speed = AUTO_FAST, done = false, disposed = false, overridden = false, tickerTimer = 0, bannerTimer = 0, pumpTimer = 0;
  // Cut 2: rest after the exit, bones left (death) / found, bosses already announced
  let restS: number | undefined, restUntil = 0, bonesLeft: number | undefined;
  let exitLine: ExitLine | undefined;   // Cut 6 §1: the exit's ledger line (exit sheet, report, death)
  const bonesFound: string[] = []; const bossSeen = new Set<number>();
  let counters = app.lineage.counters ?? [];   // Cut 6 §5: bosses with a named counter row, re-read on a sighting
  let snap: Snapshot | null = null;
  let runId = -1, engineTick = 0, startTick = 0, inflight = false, lastPersist = performance.now();
  let pendingLoad: { snap: Snapshot; rest: Ev[] } | null = null;
  let exitTier: Tier | null = null, exitAt = 0;
  let pendingExit: { items: InvItem[]; tier: string } | undefined;
  // Cut 5 §4: the open vault sheet's close, and the cage it was opened for (a dismissed sheet is not reopened)
  let vaultClose: (() => void) | null = null, vaultKey = "";
  let prepended = false;              // bail fell back to the row prepend (an engine without `bail`)
  const cls = app.lineage.class;
  const before = { best: app.lineage.best_depth, marks: app.lineage.marks, level: app.lineage.classes?.[cls]?.level ?? 1, xp: app.lineage.classes?.[cls]?.xp ?? 0, renown: app.lineage.renown ?? 0, rank: app.lineage.rank ?? 0 };
  const learned: string[] = [], found: InvItem[] = [], notes: Highlight[] = [], tamed: string[] = [], lost: string[] = [];
  const kinds = new Map<number, string>(), names = new Map<number, string>();
  const partyAtStart = (app.lineage.party ?? []).map((c) => `${c.kind} · ${c.name}`);
  const tamedIds: number[] = [], lostIds: number[] = [];
  const compLabel = (id: number): string => { const n = names.get(id); return `${kinds.get(id) ?? "?"}${n ? ` · ${n}` : ""}`; };
  const note_ = (e: { id: number; kind: string; name?: string }): void => { kinds.set(e.id, e.kind); if (e.name) names.set(e.id, e.name); };
  // HUD updates released at the viewer's clock
  const hud = { hp: 0, maxHp: 1, depth: 1 };
  const timed: { t: number; f: () => void }[] = [];
  let lastRuleText = "", lastRuleAt = 0, lastShown = "";
  // placeholder viewer (no clock): a wall clock at 10 ticks/s × speed stands in
  let fbTick = 0, fbAt = performance.now();
  function viewerTick(): number {
    if (viewer?.tick) return viewer.tick();
    const now = performance.now(); fbTick += ((now - fbAt) / 1000) * 10 * speed; fbAt = now;
    return Math.min(fbTick, engineTick);
  }
  const viewerIdle = (): boolean => viewer?.idle ? viewer.idle() : true;

  function paintHud(): void {
    const p = hud.maxHp ? hud.hp / hud.maxHp : 0;
    hpFill.style.width = `${Math.round(Math.max(0, p) * 100)}%`;
    hpFill.classList.toggle("low", p < 0.3);
    stake.classList.toggle("warn", p < 0.4);
    replace(hpText, `${Math.max(0, hud.hp)}/${hud.maxHp}`);
    replace(depth, `D${hud.depth}`);
    if (snap) replace(alert, "!".repeat(Math.max(0, Math.min(5, snap.alert))));
  }
  function hudFrom(s: Snapshot): void { hud.hp = s.hero.hp; hud.maxHp = s.hero.max_hp; hud.depth = s.depth; paintHud(); paintStake(s); }
  // Cut 2 §7: `$47 · sword⚠ · return at D4`; `death: lose all` when no row would bank or return
  function paintStake(s: Snapshot): void {
    const st = s.stake;
    stake.hidden = !st;
    if (!st) return;
    const parts: (string | HTMLElement)[] = [`$${st.loot}`];
    // Cut 6 §1: the kept number while a return/bank row exists (`$84 · keeps $50`)
    if (st.kept !== undefined && !overridden) parts.push(" · ", h("span", { class: "kept" }, /* copy:callout */ `keeps $${st.kept}`));
    for (const b of st.brought) parts.push(" · ", h("span", { class: b.insured ? "" : "risk" }, b.label, b.insured ? "" : "⚠"));
    if (overridden) parts.push(" · ", h("span", { class: "returning" }, /* copy:callout */ "returning"));
    else if (st.return_row === undefined) parts.push(" · ", h("span", { class: "lose" }, /* copy:callout */ "death: lose all"));
    else parts.push(" · ", returnAt(app.rules.rows[st.return_row], st.return_row));
    replace(stake, ...parts);
  }
  function returnAt(row: Row | undefined, i: number): string {
    const v = verbLabel({ v: row?.verb.v ?? "return" });
    const d = row?.conds.find((c) => c.k === "depth>=" && c.n !== undefined); if (d) return /* copy:callout */ `${v} at D${d.n}`;
    const hp = row?.conds.find((c) => c.k === "hp<" && c.n !== undefined); if (hp) return /* copy:callout */ `${v} at ${hp.n}%`;
    return `${v} R${i + 1}`;
  }
  function showBanner(text: string, ms: number, cls = ""): void {
    replace(banner, text); banner.className = `banner num show ${cls}`;
    clearTimeout(bannerTimer); bannerTimer = window.setTimeout(() => banner.classList.remove("show"), ms);
  }
  function bossSighted(s: Snapshot): void {
    for (const e of s.entities) {
      if (!e.tags.includes("boss") || bossSeen.has(e.id) || !s.visible[e.y * s.w + e.x]) continue;
      bossSeen.add(e.id);
      const isFact = (f: string): boolean => f === `boss:${e.kind}:counter` || f.startsWith(`boss:${e.kind}:counter=`);   // Cut 6: the fact may carry the row
      const known = (): boolean => app.lineage.facts.some(isFact) || learned.some(isFact);
      // Cut 6 §5: the counter named as a row (`boss · counter: attack boss`). The fact lands on the sighting step itself (the
      // core learns it on the boss's first telegraph), so the lineage is re-read as the banner is released at the viewer's clock.
      at(s.turn, () => {
        const show = (): void => {
          const named = counters.find((c) => c.boss === e.kind)?.text;
          showBanner(named ? /* copy:callout */ `boss · counter: ${named}` : known() ? /* copy:callout */ "boss · counter: known" : /* copy:callout */ "boss · counter: unknown", BOSS_BANNER_MS, "boss");
        };
        app.engine.lineage().then((L) => { counters = L.counters ?? counters; }).catch(() => { /* the mounted lineage's counters stand */ }).finally(() => { if (!disposed) show(); });
      });
    }
  }
  function callout(text: string, cls = "", ms = 1800 / Math.max(1, speed)): void {
    if (cls !== "ambient" && cls !== "hurt" && performance.now() < ambientUntil) return;   // Cut 7 §4: an ambient keeps the ticker for its 1.5 s
    lastShown = text;
    replace(ticker, text); ticker.className = `ticker show ${cls}`;
    clearTimeout(tickerTimer); tickerTimer = window.setTimeout(() => ticker.classList.remove("show"), ms);
  }
  function ruleCallout(ev: Extract<Ev, { k: "rule" }>): string | null {
    if (ev.row >= 0) return `R${ev.row + 1} · ${verbLabel(ev.verb)}`;
    if (ev.row === -1) return ev.text;                       // trait deviation, e.g. "cowardly → retreat"
    return CHORE_CALLOUT[ev.verb.v] ?? null;                  // chores are silent (pillar 2)
  }
  // Cut 7 §4: ambient callouts — at most one per AMBIENT_MS, only while the clock runs fast; the floor line shows at any
  // speed and is never skipped for an earlier one (it still starts the 10 s)
  function ambient(text: string, always = false): void {
    const now = performance.now();
    if (!always && (speed < RATE.fast || now - lastAmbient < AMBIENT_MS)) return;
    lastAmbient = now; ambientUntil = now + AMBIENT_SHOW_MS;
    callout(text, "ambient", AMBIENT_SHOW_MS);
  }
  function at(t: number, f: () => void): void { timed.push({ t, f }); }
  function release(upTo: number): void {
    if (!timed.length) return;
    const keep: typeof timed = [];
    for (const x of timed) { if (x.t <= upTo) x.f(); else keep.push(x); }
    timed.length = 0; timed.push(...keep);
  }
  function absorb(evs: Ev[], s: Snapshot): Tier | null {
    let exit: Tier | null = null;
    const heroId = s.hero.id;
    for (const ev of evs) {
      if (ev.k === "telegraph" || ev.k === "attack" || ev.k === "use" || (ev.k === "hurt" && ev.id === heroId)) near(ev.t);   // Cut 5 §5: always at 1×
      switch (ev.k) {
        case "callout": if (ev.text !== "explore") at(ev.t, () => callout(ev.text)); break;
        case "rule": {
          const text = ruleCallout(ev);
          // a chore (`pick up`) shows once per streak: not again until another callout intervened
          if (text) at(ev.t, () => { const now = performance.now(); if (ev.row === -2 ? text !== lastShown : text !== lastRuleText || now - lastRuleAt > 4000) callout(text); lastRuleText = text; lastRuleAt = now; });
          break;
        }
        case "hurt": if (ev.id === heroId) at(ev.t, () => { hud.hp = ev.hp; paintHud(); if (ev.dmg > 0) callout(`−${ev.dmg} ${oneWord(ev.cause)}`, "hurt", HURT_MS); }); break;
        case "descend": {
          const rooms = s.depth === ev.depth ? s.rooms : undefined;   // Cut 7 §4: `D3 · 4 rooms` when the snapshot counts them
          at(ev.t, () => { hud.depth = ev.depth; paintHud(); ambient(rooms ? /* copy:callout */ `D${ev.depth} · ${rooms} rooms` : `D${ev.depth}`, true); });
          break;
        }
        case "fact": {
          learned.push(ev.fact);
          // Cut 6 §5: the counter learned mid-fight (the boss's first telegraph) names itself: `boss · counter: attack boss`
          const m = /^boss:([a-z_]+):counter(?:=|$)/.exec(ev.fact);
          if (m) at(ev.t, () => { app.engine.lineage().then((L) => { counters = L.counters ?? counters; }).catch(() => { /* keep */ }).finally(() => {
            const named = counters.find((c) => c.boss === m[1])?.text; if (named && !disposed) showBanner(/* copy:callout */ `boss · counter: ${named}`, BOSS_BANNER_MS, "boss");
          }); });
          break;
        }
        case "pickup": {
          if (ev.id === heroId) found.push({ id: ev.id, kind: ev.item, known: true, label: ev.item });
          const gold = /^gold\b\D*(\d+)/.exec(ev.item);   // Cut 7 §4: `$47` on a gold pickup (`gold (47)` core, `gold 47` fake)
          if (gold) at(ev.t, () => ambient(`$${gold[1]}`));
          break;
        }
        case "note": notes.push({ pattern: "note", score: 0, t: ev.t, run_id: runId, text: ev.text }); break;
        case "exit": exit = ev.tier; exitLine = ev.line ?? exitLine; endingFrom = Math.min(endingFrom, ev.t - ENDING_TICKS); break;   // Cut 7 §4
        case "ending": endingFrom = Math.min(endingFrom, ev.t); break;                                                                // Cut 7 §4: the core's marker
        case "tame": if (ev.ok) { tamedIds.push(ev.id); kinds.set(ev.id, ev.kind); } break;
        case "ally": if (ev.state === "lost") lostIds.push(ev.id); break;
        case "spawn": note_(ev.e); break;
        case "level": for (const v of verbsAt(ev.class, ev.level)) learned.push(`verb:${v}`); at(ev.t, () => callout(`${ev.class} L${ev.level}`)); break;
        case "rank": at(ev.t, () => callout(`★${ev.rank}`)); break;
        case "rest": restS = ev.seconds; break;
        case "bones":
          if (ev.heir === s.run.heir) bonesLeft = ev.items;                                        // this heir's kit, left on death
          else { bonesFound.push(/* copy:callout */ `D${s.depth} · ${items(ev.items)}`); at(ev.t, () => callout(`♟${ev.heir} · ${ev.items}`)); }
          break;
        default: break;
      }
    }
    return exit;
  }
  // Cut 5 §5 / Cut 7 §4: what holds auto at 1× — a scene (a room with SCENE_FOES awake hostiles), the hero's hp moving
  const hostile = (e: { ally?: boolean; kind: string; tags: string[] }): boolean => !e.ally && e.kind !== "bones" && e.kind !== "captive" && !e.tags.includes("captive") && !e.tags.includes("ally");
  /** Cut 7 §4: the hero's room and its awake hostiles; without `room` in the wire, the hostiles in view stand in (one "room"). */
  function roomOf(s: Snapshot): { id: number; hostiles: number } {
    return s.room ?? { id: -1, hostiles: s.entities.filter((e) => hostile(e) && !e.remembered && s.visible[e.y * s.w + e.x]).length };
  }
  function sceneFrom(s: Snapshot): void {
    const r = roomOf(s);
    const open = r.hostiles >= SCENE_FOES || (scene !== null && r.id === scene && r.hostiles > 0);
    if (open) { scene = r.id; sceneUntil = Infinity; }
    else if (scene !== null) { scene = null; sceneUntil = s.turn + AUTO_TAIL; }
    el.dataset.scene = scene === null ? "0" : "1";   // dev: tools sample the cadence off the DOM
    fightFrom_(s, open);
  }
  /** Cut 8A: the fight frame holds while a scene is open, a hostile is adjacent to the hero, or a boss is in view; it lets
   *  go AUTO_TAIL ticks after. Ticks are the engine's; `applyFrame` cuts at the viewer's clock. */
  function fightFrom_(s: Snapshot, sceneOpen: boolean): void {
    const hx = s.hero.x, hy = s.hero.y;
    const seen = (e: { x: number; y: number; remembered?: boolean }): boolean => !e.remembered && !!s.visible[e.y * s.w + e.x];
    const adjacent = s.entities.some((e) => hostile(e) && seen(e) && Math.max(Math.abs(e.x - hx), Math.abs(e.y - hy)) <= 1);
    const boss = s.entities.some((e) => e.tags.includes("boss") && hostile(e) && seen(e));
    const on = sceneOpen || adjacent || boss;
    if (on) { if (!fightOn && viewerTick() >= fightUntil) fightFrom = s.turn; fightUntil = Infinity; }
    else if (fightOn) fightUntil = s.turn + AUTO_TAIL;
    fightOn = on;
  }
  function applyFrame(): void {
    const v = viewerTick();
    const want: FrameName = v >= fightFrom && v < fightUntil ? "fight" : "map";
    if (want === frame) return;
    frame = want; (viewer as FrameViewer | null)?.setFrame?.(want);
    el.dataset.frame = want;
  }
  function near(t: number): void { slowUntil = Math.max(slowUntil, t + AUTO_TAIL); }
  function handle(r: StepResult): void {
    const s = r.snapshot;
    engineTick = s.turn;
    for (const e of s.entities) note_(e);
    sceneFrom(s);
    if (s.hero.hp < lastHp) near(s.turn);   // hp lost by any means; a rest's +1 per turn is a dead stretch, a drink is a `use` event
    lastHp = s.hero.hp;
    if (s.alert > lastAlert) { const n = s.alert; at(s.turn, () => ambient(/* copy:callout */ `alert ${n}`)); }   // Cut 7 §4
    lastAlert = s.alert;
    const exit = absorb(r.events, s);
    snap = s;
    hud.maxHp = s.hero.max_hp; paintHud(); paintStake(s); bossSighted(s);
    vaultFrom(s);
    if (r.exit_pending) pendingExit = r.exit_pending;
    // Cut 7 §4: the exit batch waits (pump) until the viewer is ENDING_TICKS from the exit, then plays at 1×
    if (exit) { held = { evs: r.events, tier: exit, snap: s }; el.dataset.ending = "1"; return; }
    feed(r.events, s);
    if (performance.now() - lastPersist > PERSIST_MS) { lastPersist = performance.now(); app.persist(); }
  }
  function feed(evs: Ev[], s: Snapshot): void {
    const di = evs.findIndex((e) => e.k === "descend");
    if (viewer && di >= 0) { viewer.apply(evs.slice(0, di + 1)); pendingLoad = { snap: s, rest: evs.slice(di + 1) }; }
    else { viewer?.apply(evs); if (viewer?.sync) { const v = viewer; at(s.turn, () => v.sync!(s)); } } // Cut 4 §3: remembered foes
  }
  function pump(): void {
    if (done || disposed || !viewer || !snap) return;
    if (vaultClose && !document.querySelector(".vault-choice")) vaultClose = null;   // dismissed by backdrop / Escape: the engine's grace decides
    applyFrame();
    applySpeed();
    const now = viewerTick();
    el.dataset.tick = String(now);            // dev: tools sample the cadence off the DOM
    release(now);
    if (pendingLoad) {
      if (viewerIdle()) { const p = pendingLoad; pendingLoad = null; viewer.load(p.snap); hudFrom(p.snap); viewer.apply(p.rest); }
      return;
    }
    if (held) {
      // Cut 7 §4: the clock runs on (8× through dead air) to the ending, then the exit batch plays and the exit flow waits for it
      if (now < endingFrom) return;
      const hb = held; held = null; feed(hb.evs, hb.snap);
      exitTier = hb.tier; exitAt = performance.now() + EXIT_GRACE_MS;
      return;
    }
    if (exitTier) {
      if (!(viewerIdle() || performance.now() > exitAt)) return;
      release(Infinity);
      // Cut 2 §1: `rest 12m` for a beat, then the exit flow continues
      if (restS !== undefined && !restUntil) { restUntil = performance.now() + REST_BEAT_MS; showBanner(/* copy:callout */ `rest ${spanOf(restS)}`, REST_BEAT_MS); return; }
      if (performance.now() < restUntil) return;
      void finish(exitTier); return;
    }
    if (speed <= 0 || inflight || engineTick - now >= (speed >= AUTO_FAST ? LEAD_FAST : LEAD)) return;
    inflight = true;
    app.engine.step(speed >= AUTO_FAST ? BATCH_FAST : BATCH).then((r) => { inflight = false; if (!disposed && !done) handle(r); })
      .catch((e) => { inflight = false; console.warn("step failed", e); exitTier = "return"; exitAt = 0; });
  }
  /** The rate the clock should run at right now: the mode's, or for auto 8× / 1× by what is near (flat 8× while bailing).
   *  Cut 7 §4: 1× through a scene and through the run's last ENDING_TICKS (bailing too: the walk-out is still the end). */
  function rate(): number {
    if (paused) return 0;
    if (vaultClose) return 1;                 // Cut 5 §4: the vault sheet holds the clock at 1× while the engine's grace runs
    if (frame === "fight") return 1;          // Cut 8A: a fight is watched at 1×, whatever the mode
    if (mode !== "auto") return RATE[mode];
    const v = viewerTick();
    if (v >= endingFrom) return 1;
    return overridden || (v >= slowUntil && v >= sceneUntil) ? AUTO_FAST : 1;
  }
  function applySpeed(): void {
    const n = rate();
    if (n === speed) return;
    if (!viewer?.tick) viewerTick();          // placeholder clock: bank the ticks run at the old rate first
    speed = n; viewer?.setSpeed(n);
    el.dataset.speed = String(n);
    for (const m of Object.keys(modeBtn) as Mode[]) modeBtn[m].classList.toggle("slowed", m === mode && m !== "slow" && n === 1);
  }
  function setMode(m: Mode): void {
    mode = m; paused = false;
    for (const k of Object.keys(modeBtn) as Mode[]) modeBtn[k].classList.toggle("on", k === m);
    paintPause(); applySpeed();
  }
  function togglePause(): void { paused = !paused; paintPause(); applySpeed(); }
  function paintPause(): void { pause.classList.toggle("on", paused); replace(pause, paused ? "▶" : "⏸"); }
  let skipQueued = false;
  async function skipToEvent(): Promise<void> {
    if (done || !viewer || exitTier) return;
    // a floor change waits for the viewer to drain; a skip drains it now instead of at 1×
    if (pendingLoad) {
      // drain the old floor, apply the new one, and drop its queued events straight into place
      const p = pendingLoad; pendingLoad = null;
      const fv = viewer as Viewer & { seek?: (t: number) => void };
      viewer.load(p.snap); hudFrom(p.snap); viewer.apply(p.rest);
      if (fv.seek) fv.seek(engineTick);
      fbTick = engineTick; release(viewerTick());
      return;
    }
    // a press while a step is in flight is not lost: one skip is queued behind it
    if (inflight) { skipQueued = true; return; }
    if (held) { endingFrom = 0; return; } // skip overrides the ending hold: the pump releases the exit batch now
    inflight = true;
    let inFight = false;
    try {
      // In a fight, every blow is "interesting"; a skip pressed there means "past this fight":
      // step until the fight closes (or the run ends), then to the next interesting event.
      // judged on the engine's own snapshot (the viewer lags it): a fight is any awake hostile
      // within 2 tiles of the hero, or a scene room with ≥ 2 hostiles
      const fighting = (sn: Snapshot): boolean => {
        const hx = sn.hero.x, hy = sn.hero.y;
        if ((sn.room?.hostiles ?? 0) >= 2) return true;
        return sn.entities.some((e) => hostile(e) && !e.remembered && Math.max(Math.abs(e.x - hx), Math.abs(e.y - hy)) <= 2);
      };
      inFight = snap ? fighting(snap) : false;
      let hit = false;
      for (let i = 0; i < (inFight ? 120 : 30) && !hit && !disposed; i++) {
        const r = await app.engine.step(BATCH);
        const stillFighting = fighting(r.snapshot);
        hit = r.run_over || (!stillFighting && (!inFight || i > 0) && r.events.some((e) => INTERESTING.has(e.k)));
        if (inFight && !stillFighting) hit = true;   // the fight closed: stop here so the player sees the map again
        handle(r);
        if (pendingLoad || held) break;
      }
    } catch (e) { console.warn("skip failed", e); }
    inflight = false;
    if (!pendingLoad && !held) {
      // past a fight the viewer must land where the engine is, not at the next blow; otherwise
      // 120 engine batches leave the viewer replaying the whole fight at 1×
      // land the viewer where the engine stopped (just before the interesting event it found);
      // replaying the skipped span at 1× is what made ▶▶| feel dead in a fight
      const fv = viewer as Viewer & { seek?: (t: number) => void };
      if (fv.seek) fv.seek(Math.max(viewerTick(), engineTick - BATCH)); else viewer.skipToEvent();
      fbTick = Math.max(fbTick, engineTick - BATCH); release(viewerTick());
    }
    if (skipQueued) { skipQueued = false; void skipToEvent(); }
  }
  // Cut 5 §5: `return` fires on the next hero action as a chore (`engine.bail()`, the rules untouched); the run plays out to
  // the exit at 8× under `returning`, then the exit sheet. An engine without `bail` gets the row prepend, restored at the exit.
  function doBail(): void {
    if (done || overridden) return;
    overridden = true; bail.classList.add("on"); bail.disabled = true; if (snap) paintStake(snap);
    callout(/* copy:callout */ "returning", "", 1800);
    app.engine.bail().catch((e) => {
      console.warn("bail", e); prepended = true;
      void app.engine.setRules({ rows: [{ conds: [], verb: { v: "return" } }, ...app.rules.rows] }).catch((e2) => console.warn("bail", e2));
    });
    setMode("auto");
  }
  // Cut 5 §4: the vault choice sheet — opens once per cage, closes when the engine's snapshot no longer carries it
  function vaultFrom(s: Snapshot): void {
    const vc = s.vault_choice;
    if (!vc || !vc.items.length) { if (vaultClose) { vaultClose(); vaultClose = null; } return; }
    const key = vc.items.map((it) => it.id).join(",");
    if (vaultClose || key === vaultKey) return;
    vaultKey = key; vaultSheet(vc);
  }
  function vaultSheet(vc: VaultChoice): void {
    let sent = false;
    openSheet((close) => {
      vaultClose = () => { vaultClose = null; close(); applySpeed(); };
      const chips = h("div", { class: "chips" }, ...vc.items.map((it) => h("button", { class: "chip item", onclick: () => {
        if (sent) return; sent = true;
        app.engine.choose(it.id).catch((e) => console.warn("choose", e)).finally(() => vaultClose?.());   // the next step's snapshot carries the pickup
      } }, h("b", { class: "glyph" }, kindGlyph(it.kind)), " ", it.label)));
      return h("div", { class: "sheet-body vault-choice" }, h("div", { class: "label row-label" }, /* copy:label */ "vault"), chips);
    });
    applySpeed();
  }
  function xpGained(): number {
    const c = app.lineage.classes?.[cls] ?? { level: 1, xp: 0 }; let g = c.xp - before.xp;
    for (let l = before.level; l < c.level; l++) g += xpToNext(l);
    return Math.max(0, g);
  }
  /** An engine call that hangs must never strand the player on the black exit screen. */
  function bounded<T>(p: Promise<T>, ms: number, what: string): Promise<T | undefined> {
    return new Promise((res) => {
      const t = window.setTimeout(() => { console.warn(`${what}: no answer in ${ms} ms`); res(undefined); }, ms);
      p.then((v) => { clearTimeout(t); res(v); }, (e) => { clearTimeout(t); console.warn(what, e); res(undefined); });
    });
  }
  async function finish(tier: Tier): Promise<void> {
    if (done) return;
    done = true; clearInterval(pumpTimer);
    // whatever happens below, the player reaches a screen with buttons
    const guard = window.setTimeout(() => { if (!disposed && app.view.kind === "watch") { console.warn("exit flow stalled; falling back to camp"); app.go({ kind: "camp" }); } }, 20_000);
    try {
      if (prepended) await bounded(app.engine.setRules(app.rules), 8000, /* copy:none */ "setRules after bail");
      // The keep sheet is a decision only when there is a free vault slot; otherwise the engine keeps by preference.
      const freeSlots = Math.max(0, vaultSlots(app.lineage.unlocks) - app.lineage.vault.length);
      if (pendingExit && pendingExit.items.length && freeSlots > 0) { const p = pendingExit; pendingExit = undefined; clearTimeout(guard); exitSheet(p, () => { done = false; void finish(tier); }); return; }
      if (pendingExit) { pendingExit = undefined; await bounded(app.engine.keep([]), 8000, "keep by preference"); }
      pendingExit = undefined;
      // (`refresh` resolves void, so a sentinel tells a timeout from success)
      if (!(await bounded(app.refresh().then(() => true), 8000, "refresh at exit")) && !disposed) { clearTimeout(guard); app.go({ kind: "camp" }); return; }
    } finally { /* guard cleared on every normal path below */ }
    await finishAfterRefresh(tier, guard);
  }
  async function finishAfterRefresh(tier: Tier, guard: number): Promise<void> {
    clearTimeout(guard);
    app.runsSeen += 1;
    if (disposed) return;
    tamed.push(...tamedIds.map(compLabel));
    lost.push(...lostIds.map(compLabel));
    if (tier === "death") {
      for (const c of partyAtStart) if (!lost.some((l) => l === c || l.endsWith(c.slice(c.indexOf(" · "))))) lost.push(c);
      try {
        const death = await app.busy(/* copy:label */ "verdict", () => app.engine.death(runId));
        death.line ??= exitLine;   // Cut 6 §1: the verdict may lack the line; the exit event carried it
        if (!disposed) app.go({ kind: "death", death, lost });
      } catch (e) { console.warn("no death record", e); app.go({ kind: "camp" }); }
      return;
    }
    const L = app.lineage;
    const bests: string[] = []; for (let d = before.best + 1; d <= L.best_depth; d++) bests.push(`D${d}`);
    const report: ReturnReport = {
      elapsed_s: Math.round((engineTick - startTick) / 10), runs: 1, sampled: false, learned, bests, found, deaths: [], pending: [],
      reel: notes.slice(-5), marks_earned: L.marks - before.marks, live: snap!, tamed, hatched: [], lost,
      xp: { class: cls, gained: xpGained(), level_ups: (L.classes?.[cls]?.level ?? 1) - before.level },
      salvaged: [], renown: { gained: (L.renown ?? 0) - before.renown, rank: L.rank ?? 0, ranks_up: (L.rank ?? 0) - before.rank },
      banked: tier === "bank" ? 1 : 0, returned: tier === "return" ? 1 : 0, bones_found: bonesFound,   // rest is still ahead: the camp shows it
      exits: exitLine ? [exitLine] : undefined,                                                       // Cut 6 §1
    };
    app.go({ kind: "report", report });
  }

  // Addendum D: choose what to keep before the run settles
  function exitSheet(p: { items: InvItem[]; tier: string }, then: () => void): void {
    const free = Math.max(0, vaultSlots(app.lineage.unlocks) - app.lineage.vault.length);
    const keep = new Set<number>();
    let sent = false;
    openSheet((close) => {
      const chips = h("div", { class: "chips" });
      const count = h("span", { class: "num dim" });
      const paint = (): void => {
        replace(count, `${keep.size}/${free}`);
        replace(chips, ...p.items.map((it) => h("button", { class: `chip item${keep.has(it.id) ? " on" : ""}`, onclick: () => {
          if (keep.has(it.id)) keep.delete(it.id); else if (keep.size < free) keep.add(it.id);
          paint();
        } }, it.label, " ", keep.has(it.id) ? h("b", null, "⌂") : h("b", { class: "num gold" }, `$${salvageValue(it.kind, p.tier)}`))));
      };
      paint();
      const bones = p.tier === "death" && bonesLeft !== undefined ? h("div", { class: "bones-line dim num" }, /* copy:callout */ `bones left · ${items(bonesLeft)}`) : null;
      const ledger = exitLine?.text ? h("div", { class: "ledger-line num dim" }, exitLine.text) : null;   // Cut 6 §1: engine data, verbatim
      return h("div", { class: "sheet-body" },
        h("div", { class: "label row-label" }, /* copy:label */ "vault", " ", count),
        chips, bones, ledger,
        h("button", { class: "btn primary wide", onclick: () => {
          if (sent) return; sent = true;
          app.engine.keep([...keep]).then((L) => { app.lineage = L; }).catch((e) => console.warn("keep", e)).finally(() => { close(); then(); });
        } }, /* copy:button */ "keep"));
    });
  }

  async function init(): Promise<void> {
    let s: Snapshot;
    try { s = await app.engine.send(); } catch (e) { console.warn("send failed", e); if (!disposed) app.go({ kind: "camp" }); return; }
    if (disposed) return;
    snap = s; runId = s.run.id; engineTick = startTick = s.turn;
    for (const e of s.entities) note_(e);
    hudFrom(s);
    const { viewer: v } = await makeViewer(canvas);
    if (disposed) { v.dispose(); return; }
    viewer = v; v.resize?.(); v.load(s); v.setSpeed(speed); el.dataset.speed = String(speed); el.dataset.frame = frame; fbTick = s.turn; fbAt = performance.now();
    if ("__riddle" in window) (window as unknown as { __viewer: Viewer }).__viewer = v;   // dev inspection
    lastHp = s.hero.hp; lastAlert = s.alert; sceneFrom(s);
    pumpTimer = window.setInterval(pump, PUMP_MS);
  }
  void init();
  const onResize = (): void => viewer?.resize?.();
  window.addEventListener("resize", onResize);
  return { el, dispose: () => {
    disposed = true; window.removeEventListener("resize", onResize); clearInterval(pumpTimer); clearTimeout(tickerTimer); clearTimeout(bannerTimer); viewer?.dispose();
    if (vaultClose) { const c = vaultClose; vaultClose = null; c(); }
    if (prepended && !done) void app.engine.setRules(app.rules);
  } };
}
