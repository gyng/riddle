#!/usr/bin/env node
// Cut 119 — the client shows the companions' core (docs/CUT119_COMPANIONS.md); headless, tools/browser.mjs.
//   real   — real wasm (the earned fixture, 16 h away): the kennel cards lead with the role (glyph + word), the core's name (a bred pup's
//            generation `Ashak II`), the level and an XP bar, `lame N`, the old hound, the grudge; the signatures in the tip; the report's
//            one pet line on the card and the `fetched` ledger gloss; the kennel keeper's order chip (setOrders) and the tame drill
//            (revokeTame) on the real engine; a watched death's `other heirs` fold where `heir_traits.offer` fills (one card lit, a swap).
//   fake   — fixtures through the ui modules: the death's `Rook brought $47`, a fallen pet `lamed`, the report's condensed pet lines
//            (one fetch line a pet, one old-hound line), the crier's pet cries, the build's pet synergy with its effect on hover.
//   node web/tests/cut119.mjs [--part=real,fake]
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { launchBrowser } from "../../tools/browser.mjs";

const root = new URL("../../", import.meta.url);
const url = execFileSync("bash", ["tools/dev.sh"], { cwd: root, encoding: "utf8" }).trim();
const parts = (process.argv.find((a) => a.startsWith("--part="))?.slice(7) ?? "real,fake").split(",");
const engine = readFileSync(new URL("./fixtures/earned-gunner-home.json", import.meta.url), "utf8");
const out = [], errors = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };
const browser = await launchBrowser();
try {
  if (parts.includes("real")) {
    const page = await browser.newPage({ viewport: { width: 400, height: 900 }, reducedMotion: "reduce" });
    page.on("pageerror", (e) => errors.push(`real pageerror: ${e.message}`));
    await page.goto(`${url}?fresh=1&seed=52`);
    await page.waitForFunction(() => window.__riddle?.booted, null, { timeout: 90_000 });
    const r = await page.evaluate(async ({ engine }) => {
      const a = window.__riddle, sleep = (ms) => new Promise((res) => setTimeout(res, ms));
      if (!await a.importSave(JSON.stringify({ v: 2, engine, loadout: [], last_seen: Date.now(), runs: 0 }))) throw Error("import");
      const rep1 = await a.engine.runOfflineQuick(8 * 3600);
      window.__rep1 = rep1;
      await a.refresh();
      const L = a.lineage, all = [...L.party, ...L.kennel];
      // the kennel cards, in a sheet as the camp's party strip
      const { renderParty } = await import("/src/ui/party.ts"), { openWindow, closeAllSheets } = await import("/src/ui/sheet.ts"), { h } = await import("/src/ui/dom.ts");
      const party = renderParty(a);
      openWindow(() => h("div", { class: "sheet-body" }, party.el));
      await sleep(200);
      const cards = [...party.el.querySelectorAll(".card.comp")].map((el) => {
        const c = all.find((x) => String(x.id) === el.dataset.pet);
        const name = el.querySelector(".name");
        return { c, first: name?.firstElementChild?.className ?? "", role: name?.querySelector(".pet-role")?.dataset.role, glyph: !!name?.querySelector(".pet-role svg"),
          pname: el.querySelector(".pet-name")?.textContent, lv: name?.querySelector("b.num")?.textContent, xp: !!el.querySelector(".pet-xp .pet-xp-fill"),
          lame: el.querySelector(".pet-lame")?.textContent ?? "", tenure: el.querySelector(".pet-tenure")?.textContent ?? "", grudge: el.querySelector(".pet-grudge")?.textContent ?? "",
          tip: el.querySelector(".comp-main")?.dataset.detailTip !== undefined };
      });
      // the tip: hover the first card's main button
      const main = party.el.querySelector(".comp-main");
      main.dispatchEvent(new PointerEvent("pointerover", { bubbles: true, pointerType: "mouse" })); main.dispatchEvent(new PointerEvent("pointerenter", { bubbles: true, pointerType: "mouse" }));
      main.dispatchEvent(new MouseEvent("mouseover", { bubbles: true })); main.focus();
      await sleep(900);
      const tip = document.getElementById("kw-tip");
      const tipText = tip && !tip.hidden ? tip.textContent : "";
      closeAllSheets();
      return { cards, tipText, pets: L.pets };
    }, { engine });
    const withRole = r.cards.filter((x) => x.c?.life?.role);
    check(withRole.length >= 2 && withRole.every((x) => x.first.includes("pet-role") && x.role === x.c.life.role && x.glyph), `role first on every card, with its glyph (${withRole.map((x) => `${x.pname}:${x.role}`).join(", ")})`);
    check(withRole.every((x) => x.pname === x.c.name && x.lv === `L${x.c.level}` && x.xp), "the core's name, the level and an XP bar");
    const bred = withRole.find((x) => x.c.life.bred);
    check(!bred || / [IVX]+$/.test(bred.pname), `a bred pup carries its generation (${bred?.pname ?? "none bred"})`);
    const lamed = withRole.filter((x) => x.c.life.lame);
    check(lamed.every((x) => x.lame === `lame ${x.c.life.lame}`) && withRole.filter((x) => !x.c.life.lame).every((x) => !x.lame), `lame N on a lamed pet only (${lamed.map((x) => x.lame).join(", ") || "none lamed"})`);
    const hounds = withRole.filter((x) => (x.c.life.heirs?.length ?? 0) >= 3);
    check(hounds.every((x) => x.tenure === "old hound") && withRole.filter((x) => (x.c.life.heirs?.length ?? 0) > 0 && (x.c.life.heirs?.length ?? 0) < 3).every((x) => /^\d+ heirs?$/.test(x.tenure)), `tenure: heirs served, the old hound's mark (${withRole.map((x) => x.tenure || "—").join(", ")})`);
    const grudged = withRole.filter((x) => x.c.life.grudge);
    check(grudged.every((x) => /^grudge \S/.test(x.grudge)), `the grudge (${grudged.map((x) => x.grudge).join(", ") || "none"})`);
    check(/L3 \S+.*L5 \S+/.test(r.tipText), `the tip names the L3 / L5 signatures (${r.tipText.slice(0, 80)})`);
    // the kennel keeper's order and the tame drill, on the real engine
    await page.evaluate(() => window.__riddle.go({ kind: "camp" }));
    await page.locator(".orders-tab:visible").click(); await page.waitForTimeout(400);
    const o = await page.evaluate(async () => {
      const a = window.__riddle, sleep = (ms) => new Promise((res) => setTimeout(res, ms));
      const tab = document.querySelector(".orders-tab");
      const shown = !!tab && !tab.hidden;
      const chips = [...document.querySelectorAll(".orders-sheet [data-kennel]")].map((b) => `${b.dataset.kennel}${b.classList.contains("on") ? "*" : ""}`);
      document.querySelector('.orders-sheet [data-kennel="best"]')?.click(); await sleep(500);
      const after = (await a.engine.lineage()).orders?.kennel, sum = document.querySelector(".orders-sum")?.textContent ?? "";
      document.querySelector('.orders-sheet [data-kennel="breed"]')?.click(); await sleep(500);
      const back = (await a.engine.lineage()).orders?.kennel;
      const { closeAllSheets } = await import("/src/ui/sheet.ts"); closeAllSheets();
      const { openPackages } = await import("/src/ui/packages.ts");
      openPackages(a); await sleep(500);
      const det = document.querySelector(".pkg-advanced"); if (det) det.open = true;
      const line = document.querySelector('.pkg-drill.tame-drill');
      const text = line?.textContent ?? "";
      line?.querySelector("button.drill")?.click(); await sleep(600);
      const revoked = (await a.engine.lineage()).pets?.tame_revoked;
      const lit = document.querySelector('.pkg-drill.tame-drill button.drill')?.getAttribute("aria-pressed");
      document.querySelector('.pkg-drill.tame-drill button.drill')?.click(); await sleep(600);
      const restored = (await a.engine.lineage()).pets?.tame_revoked;
      closeAllSheets();
      return { shown, chips, after, sum, back, text, revoked, lit, restored };
    });
    check(o.shown && o.chips.join(",") === "breed*,best,off", `the kennel order chip in the run setup (${o.chips.join(",")})`);
    check(o.after === "best" && /kennel best/.test(o.sum) && o.back === "breed", `setOrders sets the keeper's order; off-default reads in the summary (${o.after} · ${o.sum})`);
    check(/tames strays/.test(o.text) && o.revoked === true && o.lit === "false" && o.restored === false, `the tame drill shown and revocable (revokeTame: ${o.revoked} → ${o.restored})`);

    // the report (a second 8 h: the King's siege, the packs fetched home): one pet line on the card, the rest under details
    Object.assign(r, await page.evaluate(async () => {
      const a = window.__riddle, sleep = (ms) => new Promise((res) => setTimeout(res, ms));
      const rep1 = window.__rep1, rep2 = await a.engine.runOfflineQuick(8 * 3600);
      await a.refresh();
      const rep = [rep1, rep2].find((x) => x.feats?.some((f) => f.k === "fetched")) ?? rep2;
      a.go({ kind: "report", report: rep, absence: true });
      await sleep(600);
      const cardFeats = [...document.querySelectorAll(".report-sheet > .report-feats .feat-line")].map((li) => ({ k: li.dataset.k, pet: li.classList.contains("pet-line"), text: li.textContent }));
      const restPet = [...document.querySelectorAll(".report-feats.more .pet-line")].map((li) => li.dataset.k);
      return { cardFeats, restPet, hasFetchedTerm: !!rep.gold?.ledger?.terms?.some((t) => t.label === "fetched"), feats: (rep.feats ?? []).map((f) => f.k) };
    }));
    await page.locator(".report-gold").hover(); await page.waitForTimeout(900);
    r.ledgerTip = await page.evaluate(() => { const t = document.getElementById("kw-tip"); return t && !t.hidden ? t.textContent : ""; });
    check(r.cardFeats.filter((x) => x.pet).length === 1 && r.cardFeats.filter((x) => !x.pet).length <= 3, `the report's card: ≤ 3 feat lines and one pet line (${r.cardFeats.map((x) => x.k).join(", ")})`);
    check(!r.feats.includes("fetched") || r.cardFeats.some((x) => x.k === "fetched" && /brought \$\d+/.test(x.text)), `a fetch leads the pet line (${r.cardFeats.find((x) => x.pet)?.text})`);
    check(r.restPet.filter((k) => k === "fetched").length === 0 && r.restPet.filter((k) => k === "old_hound").length <= 1, `the pet lines condense (rest: ${r.restPet.join(", ")})`);
    check(!r.hasFetchedTerm || /fetched[^]*packs pets carried/.test(r.ledgerTip), `the ledger tip glosses fetched (${r.hasFetchedTerm})`);

    // a watched death: `other heirs` where heir_traits.offer fills
    const d = await page.evaluate(async () => {
      const a = window.__riddle, sleep = (ms) => new Promise((res) => setTimeout(res, ms));
      let L = await a.engine.lineage();
      for (let s = 0; s < 6; s++) {
        if (!L.live) await a.engine.send();
        let res, n = 0;
        do { res = await a.engine.step(2000); n++; } while (!res.run_over && n < 200);
        L = await a.engine.lineage();
        if ((L.heir_traits?.offer?.length ?? 0) >= 2) break;
      }
      await a.refresh();
      const offer = a.lineage.heir_traits?.offer ?? [], dead = a.lineage.graveyard.at(-1);
      if (offer.length < 2 || dead?.death_id === undefined) return { offer: offer.length };
      const death = await a.engine.death(dead.death_id);
      a.go({ kind: "death", death, lost: [] }); await sleep(800);
      const fold = document.querySelector(".death .heirs-fold");
      const lit = [...(fold?.querySelectorAll(".heir-card.on") ?? [])].map((b) => b.dataset.chip);
      const other = [...(fold?.querySelectorAll("button.heir-card:not(.on)") ?? [])][0];
      other?.click(); await sleep(800);
      const born = (await a.engine.lineage()).heir_traits?.born?.chip;
      const litAfter = [...(document.querySelectorAll(".death .heirs-fold .heir-card.on"))].map((b) => b.dataset.chip);
      return { offer: offer.length, fold: !!fold, closed: fold && !fold.open, cards: fold?.querySelectorAll("button.heir-card").length ?? 0, lit, picked: other?.dataset.chip, born, litAfter, wornShown: !!fold?.querySelector(".heir-card.worn") };
    });
    check(d.offer >= 2 && d.fold && d.closed && d.cards === d.offer, `the death's other heirs fold on a real offer (${d.cards}/${d.offer} cards, closed)`);
    check(d.lit?.length === 1, `one heir lit, never none (${d.lit?.join(",")}${d.wornShown ? " · worn temperament" : ""})`);
    check(!!d.picked && d.born === d.picked && d.litAfter?.length === 1 && d.litAfter[0] === d.picked, `a swap sets the heir (setTrait → ${d.born})`);
    await page.close();
  }

  if (parts.includes("fake")) {
    const page = await browser.newPage({ viewport: { width: 400, height: 860 } });
    page.on("pageerror", (e) => errors.push(`fake pageerror: ${e.message}`));
    await page.goto(`${url}?dev=1&engine=fake&systems=all&fresh=1&seed=5&runs=0`, { waitUntil: "domcontentloaded" });
    await page.waitForFunction(() => window.__riddle?.booted && window.__riddle.screen === "camp", null, { timeout: 60_000 });
    const f = await page.evaluate(async () => {
      const a = window.__riddle, sleep = (ms) => new Promise((res) => setTimeout(res, ms));
      const feats = [
        { k: "fetched", text: "Rook brought Ada's pack · $47", day: 3 }, { k: "fetched", text: "Rook brought Bram's pack · $30", day: 3 },
        { k: "old_hound", text: "Rook · old hound · served Ada, Bram, Cole", day: 3 }, { k: "old_hound", text: "Pip · old hound · served Ada, Bram, Cole", day: 3 },
        { k: "pet_synergy", text: "Falconer · scout sees further", day: 3 }, { k: "bred", text: "Rook egg · I", day: 3 }, { k: "released", text: "Gnaw released", day: 3 },
        { k: "pet", text: "Rook · guard L4", day: 3 }, { k: "siege", text: "King · try 1 · +2%", day: 3 }];
      const report = { elapsed_s: 28800, runs: 12, sampled: false, learned: [], bests: ["Rook L3 · taunt"], found: [], deaths: [], pending: [], marks_earned: 0, tamed: [], hatched: [], lost: [], reel: [], deepest: 14,
        gold: { home: 900, salvage: 40, wake: 0, spent: 0 }, feats, fallen: [{ name: "Pip", kind: "jackal", level: 3, depth: 12, why: "fell D12 to lurker", heir: 2, lamed: true }] };
      const { petLines, petCries, fetchedFor } = await import("/src/ui/pets.ts"), { cries } = await import("/src/ui/crier.ts");
      const lines = petLines(feats);
      const cs = cries(report, a.lineage).map((c) => c.k);
      const pc = petCries(report);
      // the death: `Rook brought $47` under the epitaph
      a.lastAbsence = { report, played: true };
      const d = { run_id: 7, depth: 13, cause: "mirror_king", margin: "", verdict: "gap", baseline: 0, replays: 12, patches: [], morgue: "", trace: { turns: [] }, hero: { name: "Ada", bloodline_id: 1, heir: 1, class: "warrior" },
        memorial: { lead: "+2 Legacy · King try 1 · +2%", epitaph: "fell to the King, D33", name: "Ada", grave_gold: 188, legacy: 2 }, lever: { kind: "wait", text: "drill next" } };
      a.go({ kind: "death", death: d, lost: [] }); await sleep(400);
      const fetched = document.querySelector(".death .pet-fetched")?.textContent ?? "";
      const order = [...document.querySelector(".death-well").children].map((c) => c.className.split(" ")[0]);
      a.go({ kind: "death", death: { ...d, hero: { ...d.hero, name: "Cole" }, memorial: { ...d.memorial, name: "Cole" } }, lost: [] }); await sleep(300);
      const none = !document.querySelector(".death .pet-fetched");
      // the report: the fallen line reads lamed
      a.go({ kind: "report", report, absence: true }); await sleep(500);
      const fallen = document.querySelector(".fallen-line")?.textContent ?? "";
      const petCard = [...document.querySelectorAll(".report-sheet > .report-feats .pet-line")].map((li) => li.textContent);
      // the build's pet synergy, its effect on hover
      const { openPackages } = await import("/src/ui/packages.ts");
      const P = a.lineage.packages;
      let syn = null;
      if (P) {
        a.lineage = { ...a.lineage, packages: { ...P, build: { name: "Guarded skirmisher", picks: ["Steady"], pet_synergy: "Falconer", pet_effect: "the scout sees 3 further" } } };
        a.engine.lineage = async () => a.lineage;
        a.go({ kind: "camp" }); await sleep(300);
        openPackages(a); await sleep(400);
        const el = document.querySelector(".pkg-build .build-pet-synergy");
        syn = el ? { text: el.textContent, title: el.title, after: el.previousElementSibling?.className ?? el.parentElement.firstElementChild.className } : null;
      }
      // a crafted card: a bred, lamed old hound with a grudge (the real fixture's pets need not carry every mark at once)
      const { renderParty } = await import("/src/ui/party.ts"), { closeAllSheets } = await import("/src/ui/sheet.ts");
      closeAllSheets();
      const pet = { id: 7101, kind: "ghoul", name: "Rook III", level: 3, gen: 2, tags: [], rules: { rows: [] }, max_rows: 3, hp: 20, max_hp: 20,
        life: { role: "guard", xp: 1800, lame: 2, falls: 1, heirs: [1, 2, 3, 4], grudge: "mirror_king", runs: 9, bred: true } };
      const old = { id: 7102, kind: "rat", name: "Nib", level: 1, gen: 0, tags: [], rules: { rows: [] }, max_rows: 2, hp: 8, max_hp: 8 };
      a.lineage = { ...a.lineage, party: [pet], kennel: [old], eggs: [{ id: 9, kind: "ghoul", tags: [], gen: 3, hatch_in: 2, from_loss: false, sire: "Rook" }], party_slots: 1,
        pets: { tame: "tames strays", tame_revoked: false, kennel: "breed", fetched: 0, bred: 1, released: 0, old_hounds: ["Rook III · old hound · served Ada, Bram, Cole"] },
        feats: { ...(a.lineage.feats ?? {}), siege: [{ boss: "mirror_king", title: "King", depth: 33, tries: 1, best_pct: 10, edge_pct: 2, heirs: [] }] } };
      const party = renderParty(a); document.body.append(party.el);
      const card = party.el.querySelector('.card.comp[data-pet="7101"]'), plain = party.el.querySelector('.card.comp[data-pet="7102"]');
      const crafted = { lamed: card.classList.contains("lamed"), first: card.querySelector(".name").firstElementChild.className, name: card.querySelector(".pet-name").textContent,
        meta: card.querySelector(".pet-meta")?.textContent, hound: card.querySelector(".pet-tenure")?.title, fill: card.querySelector(".pet-xp-fill")?.style.width,
        plain: plain.querySelector(".name").textContent, plainRole: !!plain.querySelector(".pet-role"), egg: party.el.querySelector(".chip.egg")?.textContent };
      party.el.remove();
      return { lines, cs, pc, fetched, order, none, fallen, petCard, syn, crafted };
    });
    const k = f.lines.map((x) => x.k);
    check(k[0] === "fetched" && k.filter((x) => x === "fetched").length === 1 && f.lines[0].text === "Rook brought $77", `one fetch line a pet, summed (${f.lines[0]?.text})`);
    check(k.filter((x) => x === "old_hound").length === 1 && /Rook, Pip · old hounds/.test(f.lines.find((x) => x.k === "old_hound")?.text ?? ""), "the old hounds on one line");
    check(k.includes("pet_synergy") && k.includes("bred") && k.includes("pet") && k.includes("released") && !k.includes("siege"), `every pet kind condensed, none other (${k.join(",")})`);
    check(f.pc.every((c) => c.short.split(/\s+/).filter((w) => /\p{L}/u.test(w)).length <= 3) && f.pc.some((c) => c.k === "fetched") && f.pc.some((c) => c.k === "pet_sig" && c.short === "Rook L3"), `the crier's pet cries ≤ 3 words (${f.pc.map((c) => c.short).join(" | ")})`);
    check(f.cs.includes("fetched") && f.cs.length <= 6, `the crier cries the fetch among ≤ 6 (${f.cs.join(",")})`);
    check(/Rook brought \$47/.test(f.fetched) && f.order.indexOf("pet-fetched") === f.order.indexOf("memorial-epitaph") + 1, `the death names the pet that fetched, under the epitaph (${f.fetched.trim()})`);
    check(f.none, "no fetch line for a heir no pet fetched for");
    check(/Pip · jackal L3 · lamed · fell D12 to lurker/.test(f.fallen), `a fallen pet reads lamed (${f.fallen})`);
    check(f.petCard.length === 1 && /Rook brought \$77/.test(f.petCard[0]), `the report card's one pet line (${f.petCard.join(" | ")})`);
    check(!f.syn || (f.syn.text.includes("Falconer") && f.syn.title === "the scout sees 3 further"), `the build's pet synergy, effect on hover (${JSON.stringify(f.syn)})`);
    const c = f.crafted;
    check(c.first.includes("pet-role") && c.name === "Rook III" && c.lamed, `a crafted card: role first, the generation in the name, the face greyed when lamed (${c.name})`);
    check(c.meta === "lame 2 · old hound · grudge King" && /served Ada, Bram, Cole/.test(c.hound), `lame N · old hound · grudge (${c.meta})`);
    check(c.fill === "50%", `the XP bar fills within the level (1800 of 1200…2400: ${c.fill})`);
    check(!c.plainRole && /rat Nib L1 g0/.test(c.plain), `a pre-Cut 119 pet keeps the old face (${c.plain})`);
    check(/ghoul · Rook/.test(c.egg ?? ""), `a bred egg names its sire (${c.egg})`);
    await page.close();
  }
} finally {
  await browser.close();
}
for (const e of errors) out.push(`FAIL ${e}`);
console.log(out.join("\n"));
const n = out.length, bad = failed + errors.length;
console.log(bad ? `cut119: ${bad} of ${n} FAIL` : `cut119: ${n} checks PASS`);
process.exit(bad ? 1 : 0);
