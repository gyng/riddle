import type { App } from '../app';
import type { ReturnReport } from '../engine/types';
import { h } from './dom';
import { icon } from './skin';
import { detailHost } from './tips';
import { openGoldSheet } from './gold';

/** Same reported income that funds the core's away budget; never use current gold. Cut 117 §4: the core's `supply_budget.income`
 *  when it says. */
export function reportIncome(r: Pick<ReturnReport, 'gold' | 'supply_budget'>): number | undefined {
  if (r.supply_budget) return r.supply_budget.income;
  return r.gold ? r.gold.home + (r.gold.salvage ?? 0) + (r.gold.wake ?? 0) : undefined;
}
/** Cut 117 §4 (blind 8cf9050 B: `Supplies limited · $0 budget` after the apprentice spent, no word why): the core's reason code
 *  (`SupplyBudget.reason`, `WorkerAct.reason`) as ≤ 3 words and its tooltip; null for an unknown code. */
export function supplyReason(code: string | undefined): { text: string; tip: string } | null {
  switch (code) {
    case 'no_income': return { text: /* copy:callout */ 'no income', tip: /* copy:tooltip */ 'Nothing came home · supplies never spend the purse' };
    case 'income_spent': return { text: /* copy:callout */ 'income spent', tip: /* copy:tooltip */ 'Supplies used all gold earned while away' };
    case 'purse_short': return { text: /* copy:callout */ 'purse short', tip: /* copy:tooltip */ 'Purse could not pay · forge or purchases took it' };
    default: return null;
  }
}
export function supplyLimit(app: App, r: Pick<ReturnReport, 'gold' | 'supply_budget'>, prominent = false): HTMLElement {
  const income = reportIncome(r);
  const sb = r.supply_budget, why = supplyReason(sb?.reason);
  const button = h('button', { class: `supply-limit capped warn ledger-link${prominent ? ' report-supply-limit chip' : ''}`, onclick: () => openGoldSheet(app), 'data-reason': sb?.reason ?? '' },
    prominent ? icon('v_drink', '▣') : null,
    h('span', null, /* copy:callout */ 'Supplies limited'),
    income !== undefined ? h('small', { class: 'num' }, ` · $${income}`, /* copy:label */ ' budget') : null,
    why ? h('small', { class: 'supply-reason' }, ` · ${why.text}`) : null);
  return detailHost(button, () => [h('div', { class: 'kw-tip-head' }, h('b', null, /* copy:label */ 'Supply budget')),
    ...(sb ? [h('div', { class: 'num' }, /* copy:label */ 'Income', ` · $${sb.income}`),
      h('div', { class: 'num' }, /* copy:label */ 'Spent', ` · −$${sb.spent}`),
      h('div', { class: 'num' }, /* copy:label */ 'Left', ` · $${sb.left}`)] : []),
    h('div', { class: 'kw-tip-gloss' }, why ? why.tip : /* copy:tooltip */ 'Automatic supplies use gold earned while away')]);
}
