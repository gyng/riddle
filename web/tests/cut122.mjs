// Cut 122 client (docs/CUT122_SMOOTH_AND_HONEST.md §8–11, §6): the vs line names what moved it (`edit`, `forge +1`, else `this change`,
// never `this edit` after a forge buy); Legacy shows the next rank (`388 / 600 → Health III`) with a thin bar; the dock keeps every tile
// where it stood (a new one appends, lit; a gone one leaves a gap); a tactic compare inside the noise says which walls differ or that none
// does; the enemy guide never `adds` a counter whose card is not owned (it offers `unlock`).
// Wire (§2–§7 core): trade-offs priced at the wall (numbers, sorted below the fixes, never the gem; the TRY pending until its deltas);
// unowned counters (`wear` / `unlock`, applyPatch refuses, setRules' refusal toasted); `written` → own rule, `picked` → picked; `died to
// the King`, and `King · slain` only in the descent that felled him; the ration beside the hunger; a long walk home one beat. Real wasm
// (deep.json): buyRation, the TRY's price after deathDeltas, a setRules refusal shown.
import { execFileSync } from "node:child_process";
import { launchBrowser } from "../../tools/browser.mjs";
const url = execFileSync("bash", ["tools/dev.sh"], { cwd: new URL("../../", import.meta.url), encoding: "utf8" }).trim();
const browser = await launchBrowser(); let total = 0; const errors = [];
try {
  for (const width of [400, 1440]) {
    const page = await browser.newPage({ viewport: { width, height: 900 }, deviceScaleFactor: 1 });
    page.on("pageerror", (e) => errors.push(e.message));
    await page.goto(`${url}?engine=fake&fresh=1&seed=3122&runs=0`); await page.waitForFunction(() => window.__riddle?.booted);
    const checks = await page.evaluate(async () => {
      const app = window.__riddle; let checks = 0; const fails = [];
      const check = (v, label) => { if (!v) fails.push(label); checks++; };
      const tick = () => new Promise((r) => setTimeout(r, 30));

      // §8: the vs line's label
      app.resetVs();
      check(app.vsLabel() === "this change", `no cause yet reads this change (${app.vsLabel()})`);
      check(app.causeOf("Forged", false) === "forge +1" && app.causeOf("buy", false) === "buy" && app.causeOf(undefined, false) === "?", "a move's word names its cause");
      // B's case: an edit, then a forge buy before the sent set is painted again — both named, never `edit` (or `this edit`) alone
      app.rules.rows.push({ conds: [{ k: "hp<", n: 30 }], verb: { v: "drink", a: "heal" } }); app.rulesChanged();
      check(app.vsLabel() === "edit", `an edit reads edit (${app.vsLabel()})`);
      await app.mutate(async () => app.lineage, "Forged");
      check(app.vsLabel() === "edit & forge +1", `an edit and a forge buy read both (${app.vsLabel()})`);
      app.resetVs();
      // a purchase with no word is unknown: `this change`
      app.rules.rows.pop(); app.rulesChanged();
      await app.mutate(async () => app.lineage);
      check(app.vsLabel() === "this change", `an unnamed purchase after an edit reads this change (${app.vsLabel()})`);
      app.resetVs();

      // §9: Legacy toward the next rank
      const { legacyNext, legacyNextText } = await import("/src/ui/legacy.ts");
      const up = (o) => ({ effect: "", affordable: false, ...o });
      const L0 = { heir: 1, bloodline: { points: 388 }, hero_legacy: [{ heir: 1, points: 388, runs: 3, best_depth: 9, class: "warrior" }],
        legacy_upgrades: [up({ id: "health", name: "Health", rank: 2, cap: 5, price: 600 }), up({ id: "damage", name: "Damage", rank: 5, cap: 5, price: 0 }),
          up({ id: "armour", name: "Armour", rank: 1, cap: 5, price: 900 }), up({ id: "venom", name: "Venom", rank: 0, cap: 1, price: 200, blocked: "Reach D18" })] };
      const n = legacyNext(L0);
      check(n && legacyNextText(n) === "388 / 600 → Health III", `the next rank: ${n && legacyNextText(n)}`);
      check(n && Math.abs(n.fill - 388 / 600) < 1e-9, "its share");
      check(legacyNext({ ...L0, legacy_upgrades: [up({ id: "damage", rank: 5, cap: 5, price: 0 })] }) === null, "a finished tree: no bar");
      const real = app.lineage;
      app.lineage = { ...real, town: { ...(real.town ?? {}), home: true }, ...L0 };
      const { renderBar } = await import("/src/ui/frame.ts");
      const bar = renderBar(app); document.body.appendChild(bar.el ?? bar);
      const mark = (bar.el ?? bar).querySelector(".stat.legacy .legacy-bar");
      check(!!mark && mark.title === "388 / 600 → Health III" && mark.querySelector(".fill")?.style.width === "65%", `the header's Legacy carries its bar (${mark?.title} · ${mark?.querySelector(".fill")?.style.width})`);
      (bar.el ?? bar).remove();
      app.lineage = real;

      // §10: the dock never reflows under a tap
      const { renderConsole, tile } = await import("/src/ui/frame.ts");
      const t = (id) => tile({ id, label: id, icon: "edit", onclick: () => {} });
      const cons = renderConsole({ portrait: document.createElement("div"), tiles: [t("forge"), t("edit"), t("loadout")], gem: document.createElement("button"), stable: true });
      const ids = () => [...cons.el.querySelectorAll(".cmd > *")].map((x) => x.dataset.tile ?? (x.classList.contains("tile-gap") ? "_" : "")).filter(Boolean);
      check(ids().join() === "forge,edit,loadout", `first paint keeps the given order (${ids()})`);
      cons.setTiles([t("forge"), t("quest"), t("edit"), t("loadout")]);
      check(ids().join() === "forge,edit,loadout,quest", `a new tile appends at the end (${ids()})`);
      check(cons.el.querySelector('[data-tile="quest"]').classList.contains("tile-new"), "and is lit");
      check(!cons.el.querySelector('[data-tile="edit"]').classList.contains("tile-new"), "the old ones are not");
      cons.setTiles([t("forge"), t("quest"), t("loadout")]);
      check(ids().join() === "forge,_,loadout,quest", `a gone tile leaves its place (${ids()})`);
      const free = renderConsole({ portrait: document.createElement("div"), tiles: [t("a"), t("b")], gem: document.createElement("button") });
      free.setTiles([t("c"), t("a")]);
      check([...free.el.querySelectorAll(".cmd > [data-tile]")].map((x) => x.dataset.tile).join() === "c,a", "an unstable bar is as given (the report's)");

      // §11: a compare inside the noise says what differs
      const { priceOf, closeText } = await import("/src/ui/packages.ts");
      const base = { action: "equip", past: .3, bank: .4, death: .2, d_past: 0, d_bank: 0, d_death: 0 };
      const w = (depth, better, worse) => ({ depth, boss: "B", n: 8, better, worse });
      check(priceOf({ ...base, n: 8, better: 3, worse: 2, walls: [w(23, 3, 0), w(28, 0, 2), w(33, 0, 0)] }).text === "better at D23 · worse at D28", "the walls that lean, by depth");
      check(priceOf({ ...base, n: 48, better: 3, worse: 3, walls: [w(23, 0, 0)] }).text === "no wall differs · 48 sends", "no wall differs, with its sends");
      check(priceOf({ ...base, n: 8, better: 3, worse: 3 }).text === "no wall differs · 8 sends", "an older core's compare (no walls)");
      check(closeText({ n: 8, walls: [w(28, 2, 2)] }) === "turns on D28", "a wall split both ways evenly: what it turns on");
      check(!/^close$/.test(priceOf({ ...base, n: 8, better: 0, worse: 0, even: false }).text), "the core's not-even is never a bare close");

      // §6: a counter whose card is not owned offers its unlock, never `add`
      const { openLedger } = await import("/src/ui/party.ts");
      const { closeAllSheets } = await import("/src/ui/sheet.ts");
      const counter = { row: { conds: [{ k: "foe_tag", t: "boss" }], verb: { v: "tactic", a: "cadence" } }, text: "cadence" };
      const ledger = [{ kind: "goblin", seen: true, known: true, studied: false, tamed: false, bred: false, counter }];
      app.lineage = { ...real, ledger, unlocks: (real.unlocks ?? []).filter((u) => u !== "cadence") };
      closeAllSheets(); openLedger(app); await tick();
      const locked = document.querySelector('.ledger .counter-unlock');
      check(!!locked && locked.textContent === "unlock" && !!locked.closest('[data-card="cadence"]'), `an unowned card's counter offers unlock (${locked?.textContent})`);
      const before = app.rules.rows.length; locked?.closest("button")?.click(); await tick();
      check(app.rules.rows.length === before, "and never adds the row");
      closeAllSheets();
      app.lineage = { ...real, ledger, unlocks: [...(real.unlocks ?? []), "cadence"] };
      openLedger(app); await tick();
      check(!document.querySelector(".ledger .counter-unlock") && !!document.querySelector(".ledger .chip.counter"), "an owned card's counter adds as before");
      closeAllSheets(); app.lineage = real;
      return { checks, fails };
    });
    if (checks.fails.length) throw new Error(`cut122 ${width}: ${checks.fails.join("\n")}`);
    total += checks.checks; console.log(`cut122 ${width}: ${checks.checks} checks PASS`); await page.close();
  }
  // Cut 122 wire: trade-offs, owned counters, credit, the King, hunger, the walk home (the fake engine, the client's own reads)
  {
    const page = await browser.newPage({ viewport: { width: 400, height: 900 }, deviceScaleFactor: 1 });
    page.on("pageerror", (e) => errors.push(e.message));
    await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=3123&runs=0&fake_drain=1`); await page.waitForFunction(() => window.__riddle?.booted && window.__riddle.screen === "camp", null, { timeout: 60_000 });
    const r = await page.evaluate(async () => {
      const app = window.__riddle; let checks = 0; const fails = [];
      const check = (v, label) => { if (!v) fails.push(label); checks++; };
      const tick = (ms = 30) => new Promise((z) => setTimeout(z, ms));
      const { priceText, patchRows, isTradeOff } = await import("/src/ui/patches.ts");
      const { deathAction } = await import("/src/ui/death.ts");
      const { creditLine } = await import("/src/ui/report.ts");
      const { deathWhy } = await import("/src/ui/gold-words.ts");
      const { kingSlain, kingLine } = await import("/src/ui/king-eta.ts");
      const { tryOwn } = await import("/src/ui/forecast.ts");
      const { rationChip } = await import("/src/ui/feats.ts");
      const { hungerTip } = await import("/src/ui/watch.ts");
      const { closeAllSheets } = await import("/src/ui/sheet.ts");
      const price = { depth: 33, past_from: 0.5, past_to: 0, past_pm: 0.1, death_from: 0.2, death_to: 0.35, death_pm: 0.08, sims: 48, trade_off: true };

      // §1 trade-offs: the numbers, sorted below the fixes, never the gem
      check(priceText(price) === "trade-off · past D33 50→0% · death 20→35%", `a trade-off reads its numbers (${priceText(price)})`);
      check(priceText({ ...price, trade_off: false }, false) === "past D33 50→0% · death 20→35%", "a fix's price without the word");
      const fix = { row: { conds: [{ k: "hp<", n: 30 }], verb: { v: "drink", a: "heal" } }, insert_at: 0, survive: 0.8, forecast_delta: 0.1, whole: { reach: 0.1, reach_pm: 0.05, death: -0.1, death_pm: 0.05, price: { ...price, past_to: 0.7, death_to: 0.1, trade_off: false } } };
      const trade = { row: { conds: [{ k: "hp<", n: 70 }], verb: { v: "return" } }, insert_at: 0, survive: 1, forecast_delta: 0.2, whole: { reach: -0.3, reach_pm: 0.05, death: -0.15, death_pm: 0.05, price, trade_off: true } };
      const ps = [trade, fix];
      const box = patchRows(app, ps, 0.5, undefined, { moment: 33, replays: 12, select: () => {} });
      const tabs = [...box.querySelectorAll("button.patch")];
      check(ps[0] === fix && ps[1] === trade, "the trade-off sorts below the fix (the list and the tablets one order)");
      check(tabs[1]?.classList.contains("trade-off") && tabs[1].classList.contains("harms") && !tabs[0].classList.contains("trade-off"), "the trade-off tablet is marked (never the gem's)");
      check(/trade-off · past D33 50→0% · death 20→35%/.test(tabs[1]?.textContent ?? ""), `the tablet shows the numbers (${tabs[1]?.textContent})`);
      check(tabs[1]?.dataset.eff === "trade-off", "its short effect says trade-off");
      check(isTradeOff({ price }) && isTradeOff({ trade_off: true }) && !isTradeOff({ price: { ...price, trade_off: false } }), "trade_off on the item or its price");

      // §1 the TRY tablet: pending until deathDeltas lands, then death(id) again; a trade-off sinks and shows its numbers
      const real = app.lineage, engine = app.engine, saved = { death: engine.death, deathDeltas: engine.deathDeltas, takeFix: engine.takeFix };
      const pk = app.lineage.packages; const penWas = pk?.pen_open; if (pk) pk.pen_open = true;
      const pick = { kind: "tactic", text: "gas step", id: "gas_step", variant: 0, pending: true };
      const d = { run_id: 77, depth: 13, cause: "bloat", margin: "", verdict: "gap", baseline: 0.4, replays: 12, morgue: "", trace: { turns: [] },
        patches: [{ ...fix, whole: undefined }], pick, credit: "written", package: "pen · drink heal" };
      let calls = [];
      engine.takeFix ??= async () => app.lineage;
      engine.deathDeltas = async (id) => { calls.push(`deltas ${id}`); return []; };
      engine.death = async (id) => { calls.push(`death ${id}`); return { ...d, pick: { ...pick, pending: false, price, trade_off: true } }; };
      app.go({ kind: "death", death: d, kept: false });
      const pend = document.querySelector(".death .death-pick .lever-price.pending");
      check(pend?.textContent === "measuring", `the pick reads measuring while pending (${pend?.textContent})`);
      for (let i = 0; i < 40 && !document.querySelector(".death .death-pick.trade-off"); i++) await tick(25);
      const pk2 = document.querySelector(".death .death-pick");
      check(calls.join() === "deltas 77,death 77", `deathDeltas, then death(id) again (${calls.join()})`);
      check(!!pk2?.classList.contains("trade-off") && /trade-off · past D33 50→0% · death 20→35%/.test(pk2.textContent), `the priced pick shows the trade-off (${pk2?.textContent})`);
      const kids = [...document.querySelectorAll(".death .death-now .patches > button")];
      check(kids[kids.length - 1] === pk2, "and sits below the fixes");
      check(!document.querySelector(".death .patch-gem")?.textContent.includes("gas step"), "never the gem");
      Object.assign(engine, saved); if (pk) pk.pen_open = penWas; app.go({ kind: "camp" }); await tick();

      // §3 credit: written → own rule, picked → picked (the death line and the report's bar)
      const dd = { package: "Steady · R1: hp<30% → drink heal", cause_row: 0, rules: { rows: [{ conds: [{ k: "hp<", n: 30 }], verb: { v: "drink", a: "heal" } }] }, trace: { turns: [] } };
      check(/· own rule$/.test(deathAction({ ...dd, credit: "written" })), `written reads own rule (${deathAction({ ...dd, credit: "written" })})`);
      check(/· picked$/.test(deathAction({ ...dd, credit: "picked" })) && !/own rule/.test(deathAction({ ...dd, credit: "picked" })), "picked never reads own rule");
      const cl = creditLine({ credit: [{ credit: "written", fires: 4, share: 0.4 }, { credit: "picked", fires: 2, share: 0.2 }, { credit: "chores", fires: 4, share: 0.4 }] });
      check(cl === "own rules 40% · picked 20% · chores 40%", `the report's credit bar (${cl})`);

      // §4 the King: `slain · King` reads as his kill; `King · slain` only in the descent that felled him
      check(deathWhy("slain · King") === "died to the King" && deathWhy("slain · Mirror King") === "died to the King", "the King's blow reads died to the King");
      check(deathWhy("slain · jackal") === "died to jackal" && deathWhy("slain · Warlord") === "died to the Warlord" && deathWhy("hurt · banked") === "hurt · banked", "other reasons");
      check(!kingSlain({ king_eta_h: 30, trophies: ["slain:mirror_king"], walls: [{ boss: "mirror_king", slain: false }] }), "a new descent: the trophy of an old one is no kill here");
      check(!kingSlain({ walls: [{ boss: "mirror_king", slain: false }], trophies: ["mirror_king"] }), "the core's walls read this descent");
      check(kingSlain({ king_eta_h: 0 }) && kingSlain({ walls: [{ boss: "mirror_king", slain: true }] }) && kingSlain({ trophies: ["mirror_king"] }), "slain here (or an older save's trophy)");
      check(!/slain/.test(kingLine({ age_h: 100, best_depth: 20, king_eta_h: 30, king_pct: 60, trophies: ["mirror_king"], walls: [] })?.textContent ?? ""), "no `King · slain` line in a descent that has not felled him");

      // §2 unowned counters: never written, refused out loud; the guide's `wear`, the forecast's try
      const unowned = { conds: [], verb: { v: "tactic", a: "cadence" } };
      app.lineage = { ...real, unlocks: (real.unlocks ?? []).filter((u) => u !== "cadence") };
      const n0 = app.rules.rows.length;
      check(app.applyPatch({ row: unowned, insert_at: 0, survive: 0, forecast_delta: 0 }) === undefined && app.rules.rows.length === n0, "applyPatch never writes a card not owned");
      check(document.body.dataset.toast === "card not owned" && document.querySelector(".toast.refused")?.title === "card not owned: cadence", `the refusal is shown, its reason in the tip (${document.body.dataset.toast} · ${document.querySelector(".toast")?.title})`);
      const pbox = patchRows(app, [{ row: unowned, insert_at: 0, survive: 0.9, forecast_delta: 0.1 }], 0.5);
      const ub = pbox.querySelector("button.patch");
      check(ub?.classList.contains("unowned") && /^ · (unlock|wear .+)$/.test(ub.querySelector(".own-tag")?.textContent ?? ""), `a death patch on a card not owned offers its unlock or wear (${ub?.querySelector(".own-tag")?.textContent})`);
      ub?.click(); await tick();
      check(app.rules.rows.length === n0, "its tap never adds the row");
      closeAllSheets();
      const pkgId = app.lineage.packages?.all.find((p) => p.kind === "tactic")?.id;
      const own = tryOwn(app, { row: unowned, text: "cadence", owned: false, refusal: "card not owned: cadence", wear: pkgId });
      check(!!own && own.card === "cadence" && (!pkgId || own.wear?.id === pkgId), `a forecast try not owned: wear its package (${JSON.stringify(own?.wear)})`);
      check(tryOwn(app, { row: { conds: [], verb: { v: "attack", a: "tag:boss" } }, text: "attack boss" }) === null, "a plain row is owned");
      // setRules' rejection: a toast and its reason, not a console line only
      const sr = engine.setRules; engine.setRules = async () => { throw new Error("card not owned: cadence"); };
      delete document.body.dataset.refused;
      app.rules.rows.push({ conds: [], verb: { v: "attack", a: "nearest" } }); app.rulesChanged(); await tick(60);
      check(document.body.dataset.refused === "card not owned: cadence" && document.body.dataset.toast === "card not owned", `setRules' rejection is shown (${document.body.dataset.refused})`);
      engine.setRules = sr; app.rules.rows.pop(); app.rulesChanged(); app.lineage = real; await tick();

      // §5 hunger: the ration beside the loss, its tip, the call wired
      check(/ration/.test(hungerTip("starving", 12)) && /^max hp −12 · starving/.test(hungerTip("starving", 12)), `the hunger's tip names the ration (${hungerTip("starving", 12)})`);
      const H = { lost: 237, unfed: 237, fed: 65, kept: 172, packed: false, price: 40 };
      const chip = rationChip(app, H); document.body.appendChild(chip);
      check(chip.textContent === "ration keeps +172 hp · $40" && chip.tagName === "BUTTON", `the ration chip (${chip.textContent})`);
      check(rationChip(app, { ...H, packed: true }).textContent === "ration packed · +172 hp", "packed");
      const br = engine.buyRation; let bought = 0; engine.buyRation = async () => { bought++; return app.lineage; };
      if (!chip.disabled) { chip.click(); chip.click(); await tick(60); check(bought === 1 && document.body.dataset.toast === "ration packed", `two taps buy one ration (${bought} · ${document.body.dataset.toast})`); }
      engine.buyRation = async () => { throw new Error("ration packed"); };
      const chip2 = rationChip(app, H); document.body.appendChild(chip2);
      if (!chip2.disabled) { chip2.click(); chip2.click(); await tick(60); check(document.body.dataset.toast === "ration refused" && document.body.dataset.refused === "ration packed", `a refused ration says why (${document.body.dataset.toast} · ${document.body.dataset.refused})`); }
      engine.buyRation = br; chip.remove(); chip2.remove();
      check(typeof engine.buyRation === "function", "the engine proxies buyRation");
      // the forecast panel: `starving −237 · ration keeps +172 hp · $40` (the fake's hunger under fake_drain)
      const { renderForecast } = await import("/src/ui/forecast.ts");
      const fc = renderForecast(app); document.body.appendChild(fc.el); await app.emitForecast();
      for (let i = 0; i < 40 && fc.el.querySelector(".fc-hunger")?.hidden !== false; i++) await tick(25);
      const fh = fc.el.querySelector(".fc-hunger");
      check(fh && !fh.hidden && /^starving −237 · ration keeps \+172 hp · \$40$/.test(fh.textContent), `the forecast's hunger and its answer (${fh?.textContent})`);
      fc.dispose(); fc.el.remove();
      return { checks, fails };
    });
    if (r.fails.length) throw new Error(`cut122 wire: ${r.fails.join("\n")}`);
    total += r.checks; console.log(`cut122 wire: ${r.checks} checks PASS`);

    // §6 the walk home: a long one is one beat (`heading home · banked $X`) and the watch goes on to the next fight or the exit
    const w = await page.evaluate(async () => {
      const app = window.__riddle, step = app.engine.step.bind(app.engine); let put = false;
      app.engine.step = async (n) => { const r = await step(n); if (!put && !r.run_over && r.snapshot.turn > 30) { put = true; r.events.push({ t: r.snapshot.turn, k: "homeward", ticks: 50000, bank: true }); } return r; };
      app.watchMode = "fast"; app.go({ kind: "watch" });
      const t0 = performance.now();
      while (performance.now() - t0 < 30000 && !(window.__homeLog?.length && (window.__riddle.screen !== "watch" || Number(document.querySelector(".watch")?.dataset.tick ?? 0) > window.__homeLog[0].from + 20))) await new Promise((z) => setTimeout(z, 50));
      const log = window.__homeLog ?? [], tick = Number(document.querySelector(".watch")?.dataset.tick ?? 0);
      app.engine.step = step;
      return { log, tick, screen: window.__riddle.screen, fold: document.querySelector(".watch")?.dataset.homeFold ?? "" };
    });
    const hl = w.log[0];
    if (!hl || !/^heading home · banked \$\d+$/.test(hl.text)) throw new Error(`cut122 home: no beat (${JSON.stringify(w)})`);
    if (w.screen === "watch" && !(w.tick > hl.from + 20)) throw new Error(`cut122 home: the picture stood on the walk (${JSON.stringify(w)})`);
    total += 2; console.log(`cut122 home: ${hl.text} · went on (${w.screen} · tick ${w.tick})`);
    await page.close();
  }

  // Real wasm (deep.json): the ration call, the TRY's price after its deltas, a setRules refusal shown
  {
    const { readFileSync } = await import("node:fs");
    const source = readFileSync(new URL("./fixtures/deep.json", import.meta.url), "utf8");
    const page = await browser.newPage({ viewport: { width: 400, height: 860 }, deviceScaleFactor: 1, reducedMotion: "reduce" });
    page.on("pageerror", (e) => errors.push(`real pageerror: ${e.message}`));
    await page.goto(`${url}?dev=1&fresh=1&runs=0&seed=52`, { waitUntil: "domcontentloaded" });
    await page.waitForFunction(() => window.__riddle?.booted, null, { timeout: 90_000 });
    const r = await page.evaluate(async ({ source }) => {
      const a = window.__riddle; a.runnerOn = false; let checks = 0; const fails = [], notes = [];
      const check = (v, label) => { if (!v) fails.push(label); checks++; };
      if (a.kind !== "wasm") return { checks, fails: ["not wasm"], notes };
      const ok = await a.importSave(JSON.stringify({ v: 2, engine: source, loadout: JSON.parse(source).loadout ?? [], last_seen: Date.now(), runs: 0 }));
      check(ok, "deep.json imported");
      // §5: buyRation — a ration once a send; a second is refused in the core's words
      let first = "", second = "";
      try { await a.engine.buyRation(); first = "ok"; } catch (e) { first = String(e.message ?? e); }
      try { await a.engine.buyRation(); second = "ok"; } catch (e) { second = String(e.message ?? e); }
      check(first === "ok" ? /ration packed/.test(second) : /not enough gold/.test(first), `buyRation (${first} · ${second})`);
      const f = await a.engine.forecast();
      check(!f.hunger || (["lost", "unfed", "fed", "kept", "price"].every((k) => typeof f.hunger[k] === "number") && f.hunger.packed === (first === "ok")), `Forecast.hunger's shape (${JSON.stringify(f.hunger)})`);
      notes.push(`hunger ${JSON.stringify(f.hunger ?? null)}`);
      // §1: the TRY tablet's price — pending until deathDeltas, then death(id) carries it
      const ids = (a.lineage.graveyard ?? []).map((g) => g.death_id).filter((x) => x !== undefined).reverse();
      let priced = 0;
      for (const id of ids.slice(0, 8)) {
        const d = await a.engine.death(id);
        // (the TRY tablet with the pen open; before it the tactic lever is the pick)
        const which = d.pick?.id ? "pick" : d.lever?.kind === "tactic" ? "lever" : null;
        if (!which) continue;
        const l0 = d[which];
        if (l0.pending) { await a.engine.deathDeltas(id); const l2 = (await a.engine.death(id))[which]; check(!l2?.pending, `death ${id}: the ${which} is priced after its deltas`); if (l2?.price) { priced++; const p = l2.price; check(typeof p.depth === "number" && p.past_from >= 0 && p.past_to <= 1 && p.death_from >= 0 && (l2.trade_off ?? false) === (p.trade_off ?? l2.trade_off ?? false), `death ${id}: WallPrice (${JSON.stringify(p)})`); } }
        notes.push(`death ${id} ${which} ${l0.text} pending ${!!l0.pending}`);
        break;
      }
      notes.push(`priced ${priced}`);
      // §2: setRules' refusal of a card not owned is shown (toast + its reason)
      const card = [...(a.vocab.verbs ?? []).filter((v) => v.v === "tactic").map((v) => v.a), ...(a.unlockCat ?? []).map((u) => u.id), "cadence"].find((c) => c && (a.unlockCat.some((u) => u.id === c && u.rows?.some((r) => r.verb.v === "tactic")) || c === "cadence") && !a.cardOwned(c));
      if (card) {
        delete document.body.dataset.refused;
        a.rules.rows.push({ conds: [], verb: { v: "tactic", a: card } }); a.rulesChanged();
        for (let i = 0; i < 60 && !document.body.dataset.refused; i++) await new Promise((z) => setTimeout(z, 50));
        check(/card not owned/.test(document.body.dataset.refused ?? ""), `the core's refusal is shown (${card}: ${document.body.dataset.refused})`);
        a.rules.rows.pop(); a.rulesChanged();
        check(a.applyPatch({ row: { conds: [], verb: { v: "tactic", a: card } }, insert_at: 0, survive: 0, forecast_delta: 0 }) === undefined, "applyPatch refuses it before the core does");
      } else notes.push("no unowned card in the vocabulary");
      return { checks, fails, notes };
    }, { source });
    if (r.fails.length) throw new Error(`cut122 real: ${r.fails.join("\n")}`);
    total += r.checks; console.log(`cut122 real: ${r.checks} checks PASS · ${r.notes.join(" · ")}`);
    await page.close();
  }
  if (errors.length) throw new Error(errors.join("\n"));
  console.log(`cut122: ${total} checks PASS`);
} finally { await browser.close(); }
