import { execFileSync } from 'node:child_process';
import { launchBrowser } from '../../tools/browser.mjs';
const url = execFileSync('bash', ['tools/dev.sh'], { cwd: new URL('../../', import.meta.url), encoding: 'utf8' }).trim();
const browser = await launchBrowser();
try {
  const page = await browser.newPage();
  await page.goto(`${url}?engine=fake&fresh=1&seed=3002&runs=0`);
  await page.waitForFunction(() => window.__riddle?.booted);
  const checks = await page.evaluate(async () => {
    const { exitDepth, traceLabel } = await import('/src/ui/report.ts');
    const a = window.__riddle;
    let n = 0;
    const check = (ok, label) => { if (!ok) throw Error(label); n++; };
    const x = { bloodline_id: 2, run_id: 1, kept: 447, keep_pct: 100, carried: 447, spent: 0, spent_on: [], text: 'banked $447', reached: 6 };
    a.lineage.gold_ledger = [{ bloodline_id: 1, t: 10, delta: 447, why: 'banked D3' }, { bloodline_id: 2, t: 20, delta: 447, why: 'banked D6' }, { bloodline_id: 1, t: 30, delta: 447, why: 'banked D9' }];
    check(exitDepth(a, x) === 6, 'actual wire depth wins over interleaved ledger');
    check(traceLabel(a, x) === 'D6 · collected · log', 'rendered caption agrees with run D6');
    const legacy = { ...x, reached: undefined };
    check(exitDepth(a, legacy) === 6, 'legacy depth belongs to the exiting bloodline');
    check(exitDepth(a, legacy, [{ ...x, bloodline_id: 1 }]) === 6, 'another hero duplicate cannot skip this exit');
    check(exitDepth(a, { ...legacy, reached: 0 }) === 6, 'zero-depth old wire uses honest ledger fallback');
    a.lineage.gold_ledger = [];
    check(exitDepth(a, { ...legacy, text: 'died $0 · bones on D7' }) === 7, 'old text depth fallback retained');
    const report = { elapsed_s: 0, runs: 1, sampled: false, learned: [], bests: [], deepest: 6, found: [], deaths: [], pending: [], marks_earned: 0, tamed: [], hatched: [], lost: [], reel: [], banked: 1, exits: [{ ...x, trace: { turns: [{ t: 1, row: -1, verb: { v: 'explore' }, hp: 31, foes: 0, telegraphs: [] }] } }] };
    a.go({ kind: 'report', report });
    check(document.querySelector('.report .exit-row button.chip')?.textContent === 'D6 · collected · log', 'actual DOM log caption agrees with report depth');
    const saved = await a.engine.save(); traceLabel(a, x);
    check(await a.engine.save() === saved, 'reading caption leaves exact saved engine unchanged');
    return n;
  });
  console.log(checks, 'report exit-depth checks PASS');
} finally { await browser.close(); }
