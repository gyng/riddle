// Presentation-only: metadata belongs to the watched run, counts to released events.
import type { Ev, Packages, Row } from '../engine/types';
import { h } from './dom';
import { packageIcon } from './skin';
import { rowLabel } from './tokens';
import { openSheet } from './sheet';

export function tacticObserver(rows: Row[], packages?: Packages) {
  const frozen = structuredClone(rows), catalog = structuredClone(packages?.all ?? []);
  const el = h('div', { class: 'tactic-observer', 'aria-label': /* copy:label */ 'Watched tactics' });
  const entries = new Map<string, { button: HTMLButtonElement; count: number; row: number; timer: number; last: number; name: string }>();
  let ended = false, disabled = false;
  const ownership = frozen.map(row => {
    const [kind, id] = row.origin?.split(':') ?? [];
    if (kind === 'drill' && id) return { key: 'drill', id: 'boss_focus', name: /* copy:label */ 'Boss counters' };
    const p = catalog.find(p => p.id === id && (kind === 'temper' ? p.kind === 'temperament' : p.kind === kind));
    return p && ['stance', 'tactic', 'temper'].includes(kind) ? { key: `${kind}:${id}`, id: p.id, name: p.name } : undefined;
  });
  const routineAttack = (row: Row): boolean => row.verb.v === 'attack' && (
    !row.conds.length || !!row.origin?.startsWith('stance:') && row.conds.every(c => c.k === 'foes>=' && (c.n ?? Infinity) <= 1));
  const describe = (entry: { name: string; count: number; row: number }): string =>
    /* copy:label */ `${entry.name} · ${entry.count} activations${entry.row >= 0 ? ` · ${rowLabel(frozen[entry.row])}` : ''}`;
  ownership.forEach(owner => {
    if (!owner || entries.has(owner.key)) return;
    const button = h('button', { class: 'tactic-cue', 'data-origin': owner.key }, packageIcon(owner.id));
    const entry = { button, count: 0, row: -1, timer: 0, last: -Infinity, name: owner.name };
    button.title = describe(entry); button.setAttribute('aria-label', button.title);
    button.onclick = () => openSheet(() => h('div', { class: 'sheet-body tactic-detail' },
      h('h3', null, entry.name), h('p', { class: 'num' }, /* copy:label */ `${entry.count} activations`),
      ...frozen.flatMap((r, i) => ownership[i]?.key === owner.key ? [h('p', null, rowLabel(r))] : [])), { anchor: button, stay: true });
    entries.set(owner.key, entry); el.append(button);
  });
  const rank = (key: string): number => key.startsWith('stance:') ? 0 : key.startsWith('tactic:') ? 1 : key.startsWith('temper:') ? 2 : 3;
  el.replaceChildren(...[...entries.entries()].sort(([a], [b]) => rank(a) - rank(b)).map(([, e]) => e.button));
  el.hidden = !entries.size;
  return {
    el,
    observedRows: ownership.flatMap((o, i) => o ? [i] : []),
    meaningfulRows: ownership.flatMap((o, i) => o && !routineAttack(frozen[i]) ? [i] : []),
    owns: (row: number): boolean => !disabled && !!ownership[row],
    end(): void { ended = true; entries.forEach(entry => { clearTimeout(entry.timer); entry.button.classList.remove('acting'); }); },
    disable(): void { disabled = true; el.hidden = true; },
    fire(ev: Extract<Ev, { k: 'rule' }>, now = performance.now()): void {
      const owner = ownership[ev.row], row = frozen[ev.row];
      if (ended || disabled || !owner || !row) return;
      const entry = entries.get(owner.key)!;
      entry.count++;
      if (!routineAttack(row)) entry.row = ev.row; // Fallback swings never overwrite the last meaningful choice.
      entry.button.title = describe(entry); entry.button.setAttribute('aria-label', entry.button.title);
      entry.button.dataset.count = String(entry.count);
      // Ordinary fallback swings count in details but never flash. Repeats do not restart a glow.
      if (routineAttack(row) || now - entry.last < 1200) return;
      entry.last = now; entry.button.classList.add('acting');
      clearTimeout(entry.timer); entry.timer = window.setTimeout(() => entry.button.classList.remove('acting'), 850);
    },
    dispose(): void { entries.forEach(entry => clearTimeout(entry.timer)); },
  };
}
