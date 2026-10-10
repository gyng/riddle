// Cut 122 client (docs/CUT122_SMOOTH_AND_HONEST.md §8–11, §6): the vs line names what moved it (`edit`, `forge +1`, else `this change`,
// never `this edit` after a forge buy); Legacy shows the next rank (`388 / 600 → Health III`) with a thin bar; the dock keeps every tile
// where it stood (a new one appends, lit; a gone one leaves a gap); a tactic compare inside the noise says which walls differ or that none
// does; the enemy guide never `adds` a counter whose card is not owned (it offers `unlock`).
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
  if (errors.length) throw new Error(errors.join("\n"));
  console.log(`cut122: ${total} checks PASS`);
} finally { await browser.close(); }
