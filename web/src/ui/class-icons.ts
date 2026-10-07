import '../class-icons.css';
import { h } from './dom';
import { icon } from './skin';

// Class symbols reuse the game's painted silhouettes and existing path insignias.
const SYMBOLS: Record<string, [string, string]> = {
  fighter: ['v_shield', '⚔'], rogue: ['v_shadow', '†'], ranger: ['it_bow', '➶'],
  caster: ['v_magic', '✦'], gunner: ['v_gun_aim', '⌁'],
  sentinel: ['style_sentinel', '⬟'], hexbinder: ['style_hexbinder', '✧'],
};
export function classIcon(cls: string, path?: string | null): HTMLElement {
  const key = path && SYMBOLS[path] ? path : cls;
  const [id, fallback] = SYMBOLS[key] ?? ['', '◇'];
  return h('span', { class: 'class-icon', 'data-class': key, 'aria-hidden': 'true' }, icon(id, fallback));
}
