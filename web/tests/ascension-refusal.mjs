import { execFileSync } from 'node:child_process';
import { launchBrowser } from '../../tools/browser.mjs';
const url = execFileSync('bash', ['tools/dev.sh'], { cwd: new URL('../../', import.meta.url), encoding: 'utf8' }).trim();
const browser = await launchBrowser();
try {
  for (const [width, height] of [[320, 568], [400, 900], [1440, 900], [800, 400]]) {
    const p = await browser.newPage({ viewport: { width, height } });
    await p.goto(`${url}?engine=fake&fresh=1&runs=0`);
    await p.waitForFunction(() => window.__riddle?.booted);
    const n = await p.evaluate(async () => {
      const a = window.__riddle; a.runnerOn = false;
      const ascend = a.engine.ascend.bind(a.engine), vocabulary = a.engine.vocabulary.bind(a.engine);
      let resets = 0, calls = 0, checks = 0;
      a.again = async () => { resets++; };
      const check = (ok, name) => { if (!ok) throw Error(name); checks++; };
      const settle = () => new Promise(r => setTimeout(r, 20));
      const save = await a.engine.save(), lineage = a.lineage;
      a.loadout = [1, 2]; a.runsSeen = 23;
      a.engine.ascend = async () => { calls++; throw Error('actual rejection'); };
      a.go({ kind: 'ending' });
      check(document.querySelectorAll('.ascension-description').length === 4, 'all four challenges explained');
      check(document.querySelector('.ending-body').getBoundingClientRect().top >= 0 && document.documentElement.scrollWidth <= innerWidth, 'ending stays in viewport');
      check(document.querySelector('.ending-body').innerText.split(/\s+/).filter(w => /[a-z]/i.test(w)).length <= 60, 'ending surface stays within 60 words');
      for (const choice of document.querySelectorAll('.variants button')) {
        choice.click();
        check(calls === 0 && await a.engine.save() === save, 'review is read-only');
        const review = document.querySelector('.ascension-review');
        check(review.textContent.includes('Shared gold:') && review.textContent.includes('Legacy') && review.textContent.includes('Savings'), 'review explains shared cost and persistence');
        check(document.activeElement === review.querySelector('button'), 'focus starts on cancellation');
        const bounds = review.closest('.sheet').getBoundingClientRect();
        check(bounds.left >= 0 && bounds.right <= innerWidth && bounds.top >= 0 && bounds.bottom <= innerHeight, 'review fits viewport');
        check(Math.abs((bounds.top + bounds.bottom) / 2 - innerHeight / 2) <= 2, 'review is centered even outside framed camp');
        review.querySelector('button').click();
        check(document.activeElement === choice, 'cancel returns focus to choice');
      }
      const start = () => { document.querySelector('.variants button').click(); document.querySelector('.ascension-confirm').click(); };
      start(); await settle();
      check(resets === 0 && calls === 1, 'refusal never creates a new town');
      check(a.lineage === lineage && a.runsSeen === 23 && a.loadout.join() === '1,2', 'refusal keeps client progression and loadout');
      check(await a.engine.save() === save, 'refusal keeps complete engine save');
      check(document.querySelector('.ascension-status').textContent === 'Try again', 'failure feedback visible');
      check([...document.querySelectorAll('.variants button')].every(b => !b.disabled), 'refusal re-enables choices');
      a.engine.ascend = undefined;
      document.querySelector('.ascension-review button').click();
      start(); await settle();
      check(resets === 0 && a.lineage === lineage && await a.engine.save() === save, 'missing bridge never resets town');
      let release;
      a.engine.ascend = () => { calls++; return new Promise(r => release = r); };
      const button = document.querySelector('.ascension-confirm'); button.click(); button.click();
      check(calls === 2 && [...document.querySelectorAll('.variants button')].every(b => b.disabled), 'one in-flight UI request');
      check(await a.ascend('hunted') === false && calls === 2, 'app also rejects concurrent request');
      release(ascend('no_rest')); await settle();
      check(a.screen === 'camp' && a.lineage.ascension.variant === 'no_rest', 'successful core result reaches camp');
      check(a.runsSeen === 0 && a.loadout.length === 0 && resets === 0, 'success performs ascension cleanup exactly');
      check(document.documentElement.scrollWidth <= innerWidth, 'success has no overflow');
      // A committed core transition must never be retried because metadata fails.
      a.go({ kind: 'ending' }); a.engine.ascend = async v => ascend(v);
      a.engine.vocabulary = async () => { throw Error('metadata unavailable'); };
      check(await a.ascend('hunted') === true && a.screen === 'camp' && a.lineage.ascension.variant === 'hunted', 'committed transition survives metadata failure');
      a.engine.vocabulary = vocabulary;
      return checks;
    });
    console.log(`${width}x${height}`, n, 'ascension review/cancel/refusal/single-flight/success PASS'); await p.close();
  }
} finally { await browser.close(); }
