import type { App } from '../app';
import type { ReturnReport } from '../engine/types';
import { h } from './dom';
import { icon } from './skin';
import { detailHost } from './tips';
import { openGoldSheet } from './gold';

/** Same reported income that funds the core's away budget; never use current gold. */
export function reportIncome(r: Pick<ReturnReport, 'gold'>): number | undefined {
  return r.gold ? r.gold.home + (r.gold.salvage ?? 0) + (r.gold.wake ?? 0) : undefined;
}
export function supplyLimit(app: App, r: Pick<ReturnReport, 'gold'>, prominent = false): HTMLElement {
  const income = reportIncome(r);
  const button = h('button', { class: `supply-limit capped warn ledger-link${prominent ? ' report-supply-limit chip' : ''}`, onclick: () => openGoldSheet(app) },
    prominent ? icon('v_drink', '▣') : null,
    h('span', null, /* copy:callout */ 'Supplies limited'),
    income !== undefined ? h('small', { class: 'num' }, ` · $${income}`, /* copy:label */ ' budget') : null);
  return detailHost(button, () => [h('div', { class: 'kw-tip-head' }, h('b', null, /* copy:label */ 'Supply budget')),
    h('div', { class: 'kw-tip-gloss' }, /* copy:tooltip */ 'Automatic supplies use gold earned while away')]);
}
