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
import type { App, Mounted } from "../app";
import type { Ev, Highlight, InvItem, ReturnReport, Row, Snapshot, StepResult, VaultChoice } from "../engine/types";
import { h, items, replace, spanOf } from "./dom";
import { makeViewer, type Viewer } from "./viewer";
import { verbsAt, xpToNext } from "../engine/classes";
import { openSheet } from "./sheet";
import { salvageValue } from "./salvage";
import { vaultSlots } from "./unlocks";
import { kindGlyph, verbLabel } from "./tokens";

type Tier = "bank" | "return" | "death";
const INTERESTING = new Set(["hurt", "die", "telegraph", "pickup", "use", "fact", "steal", "ally", "descend", "exit", "spawn", "tame"]);
const LEAD = 12, BATCH = 10;        // ticks: pump when the engine is < LEAD ahead; step BATCH at a time
const BATCH_FAST = 12;              // at 8× the viewer eats 4 ticks per pump; a bigger batch keeps the queue fed through a slow step
const PUMP_MS = 50;
type Mode = "slow" | "fast" | "auto";
const RATE: Record<Mode, number> = { slow: 1, fast: 4, auto: 8 };
const AUTO_FAST = 8, AUTO_TAIL = 20; // auto: 8× when nothing is near; 1× until AUTO_TAIL ticks after the last sighting / hp change
const ITEM_NEAR = 3;                // tiles: an item this close to the hero keeps auto at 1×
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
  let speed = AUTO_FAST, done = false, disposed = false, overridden = false, tickerTimer = 0, bannerTimer = 0, pumpTimer = 0;
  // Cut 2: rest after the exit, bones left (death) / found, bosses already announced
  let restS: number | undefined, restUntil = 0, bonesLeft: number | undefined;
  const bonesFound: string[] = []; const bossSeen = new Set<number>();
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
      const known = app.lineage.facts.includes(`boss:${e.kind}:counter`) || learned.includes(`boss:${e.kind}:counter`);
      at(s.turn, () => showBanner(known ? /* copy:callout */ "boss · counter: known" : /* copy:callout */ "boss · counter: unknown", BOSS_BANNER_MS, "boss"));
    }
  }
  function callout(text: string, cls = "", ms = 1800 / Math.max(1, speed)): void {
    lastShown = text;
    replace(ticker, text); ticker.className = `ticker show ${cls}`;
    clearTimeout(tickerTimer); tickerTimer = window.setTimeout(() => ticker.classList.remove("show"), ms);
  }
  function ruleCallout(ev: Extract<Ev, { k: "rule" }>): string | null {
    if (ev.row >= 0) return `R${ev.row + 1} · ${verbLabel(ev.verb)}`;
    if (ev.row === -1) return ev.text;                       // trait deviation, e.g. "cowardly → retreat"
    return CHORE_CALLOUT[ev.verb.v] ?? null;                  // chores are silent (pillar 2)
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
        case "descend": at(ev.t, () => { hud.depth = ev.depth; paintHud(); }); break;
        case "fact": learned.push(ev.fact); break;
        case "pickup": if (ev.id === heroId) found.push({ id: ev.id, kind: ev.item, known: true, label: ev.item }); break;
        case "note": notes.push({ pattern: "note", score: 0, t: ev.t, run_id: runId, text: ev.text }); break;
        case "exit": exit = ev.tier; break;
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
  // Cut 5 §5: what holds auto at 1× — a hostile in view, an item within ITEM_NEAR tiles, the hero's hp moving
  const hostile = (e: { ally?: boolean; kind: string; tags: string[] }): boolean => !e.ally && e.kind !== "bones" && e.kind !== "captive" && !e.tags.includes("captive") && !e.tags.includes("ally");
  function nearNow(s: Snapshot): boolean {
    if (s.entities.some((e) => hostile(e) && s.visible[e.y * s.w + e.x])) return true;
    const hx = s.hero.x, hy = s.hero.y;
    return s.items.some((it) => Math.max(Math.abs(it.x - hx), Math.abs(it.y - hy)) <= ITEM_NEAR);
  }
  function near(t: number): void { slowUntil = Math.max(slowUntil, t + AUTO_TAIL); }
  function handle(r: StepResult): void {
    const s = r.snapshot;
    engineTick = s.turn;
    for (const e of s.entities) note_(e);
    if (nearNow(s) || s.hero.hp < lastHp) near(s.turn);   // hp lost by any means; a rest's +1 per turn is a dead stretch, a drink is a `use` event
    lastHp = s.hero.hp;
    const exit = absorb(r.events, s);
    const di = r.events.findIndex((e) => e.k === "descend");
    if (viewer && di >= 0) { viewer.apply(r.events.slice(0, di + 1)); pendingLoad = { snap: s, rest: r.events.slice(di + 1) }; }
    else { viewer?.apply(r.events); if (viewer?.sync) { const v = viewer; at(s.turn, () => v.sync!(s)); } } // Cut 4 §3: remembered foes
    snap = s;
    hud.maxHp = s.hero.max_hp; paintHud(); paintStake(s); bossSighted(s);
    vaultFrom(s);
    if (r.exit_pending) pendingExit = r.exit_pending;
    if (exit) { exitTier = exit; exitAt = performance.now() + EXIT_GRACE_MS; }
    if (performance.now() - lastPersist > PERSIST_MS) { lastPersist = performance.now(); app.persist(); }
  }
  function pump(): void {
    if (done || disposed || !viewer || !snap) return;
    if (vaultClose && !document.querySelector(".vault-choice")) vaultClose = null;   // dismissed by backdrop / Escape: the engine's grace decides
    applySpeed();
    const now = viewerTick();
    el.dataset.tick = String(now);            // dev: tools sample the cadence off the DOM
    release(now);
    if (pendingLoad) {
      if (viewerIdle()) { const p = pendingLoad; pendingLoad = null; viewer.load(p.snap); hudFrom(p.snap); viewer.apply(p.rest); }
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
    if (speed <= 0 || inflight || engineTick - now >= LEAD) return;
    inflight = true;
    app.engine.step(speed >= AUTO_FAST ? BATCH_FAST : BATCH).then((r) => { inflight = false; if (!disposed && !done) handle(r); })
      .catch((e) => { inflight = false; console.warn("step failed", e); exitTier = "return"; exitAt = 0; });
  }
  /** The rate the clock should run at right now: the mode's, or for auto 8× / 1× by what is near (flat 8× while bailing). */
  function rate(): number {
    if (paused) return 0;
    if (vaultClose) return 1;                 // Cut 5 §4: the vault sheet holds the clock at 1× while the engine's grace runs
    if (mode !== "auto") return RATE[mode];
    return overridden || viewerTick() >= slowUntil ? AUTO_FAST : 1;
  }
  function applySpeed(): void {
    const n = rate();
    if (n === speed) return;
    if (!viewer?.tick) viewerTick();          // placeholder clock: bank the ticks run at the old rate first
    speed = n; viewer?.setSpeed(n);
    el.dataset.speed = String(n);
    modeBtn.auto.classList.toggle("slowed", mode === "auto" && n === 1);
  }
  function setMode(m: Mode): void {
    mode = m; paused = false;
    for (const k of Object.keys(modeBtn) as Mode[]) modeBtn[k].classList.toggle("on", k === m);
    paintPause(); applySpeed();
  }
  function togglePause(): void { paused = !paused; paintPause(); applySpeed(); }
  function paintPause(): void { pause.classList.toggle("on", paused); replace(pause, paused ? "▶" : "⏸"); }
  async function skipToEvent(): Promise<void> {
    if (done || inflight || !viewer || exitTier || pendingLoad) return;
    inflight = true;
    try {
      let hit = false;
      for (let i = 0; i < 30 && !hit && !disposed; i++) {
        const r = await app.engine.step(BATCH);
        hit = r.run_over || r.events.some((e) => INTERESTING.has(e.k));
        handle(r);
        if (pendingLoad) break;
      }
    } catch (e) { console.warn("skip failed", e); }
    inflight = false;
    if (!pendingLoad) { viewer.skipToEvent(); fbTick = engineTick; release(viewerTick()); }
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
      return h("div", { class: "sheet-body" },
        h("div", { class: "label row-label" }, /* copy:label */ "vault", " ", count),
        chips, bones,
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
    viewer = v; v.resize?.(); v.load(s); v.setSpeed(speed); el.dataset.speed = String(speed); fbTick = s.turn; fbAt = performance.now();
    if ("__riddle" in window) (window as unknown as { __viewer: Viewer }).__viewer = v;   // dev inspection
    lastHp = s.hero.hp; if (nearNow(s)) near(s.turn);
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
