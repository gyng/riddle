import {test} from 'node:test';
import assert from 'node:assert/strict';
import {failureOutput} from '../web/tests/lib/failure-output.mjs';
test('browser failure summary keeps the target and cause, dropping successful checks', () => {
  const raw = `ok a successful check\nwalk aborted: locator.click: Timeout 5000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('.editor .chip.cond').first()\u001b[22m\n      - element is not visible\nFAIL ui\n`;
  assert.deepEqual(failureOutput(raw), [
    'walk aborted: locator.click: Timeout 5000ms exceeded.',
    "  - waiting for locator('.editor .chip.cond').first()",
    '      - element is not visible', 'FAIL ui']);
});
test('assertion failures, runtime errors and stack traces remain bounded', () => {
  assert.deepEqual(failureOutput('ok first\nFAIL budget\nError: broken\n    at run (test:4)\nnot ok gate\n', 3),
    ['Error: broken', 'FAIL budget', '    at run (test:4)']);
  assert.deepEqual(failureOutput('ok all checks\nok no runtime errors\n'), []);
});

test('a long failure list cannot hide the blocking locator', () => {
  const raw = Array.from({length:30}, (_,i) => `FAIL assertion ${i}`).join('\n')
    + "\nwalk aborted: locator.click: Timeout 5000ms exceeded.\n  - waiting for getByRole('button', { name: 'Speed' })\n";
  const lines = failureOutput(raw);
  assert.equal(lines.length, 20);
  assert.equal(lines[0], 'walk aborted: locator.click: Timeout 5000ms exceeded.');
  assert.ok(lines[1].includes("getByRole('button'"));
});
