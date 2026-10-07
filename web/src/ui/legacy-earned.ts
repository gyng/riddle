import type { ReturnReport } from '../engine/types';
import { h } from './dom';

/** Historical earned points, never a difference inferred from today's balance. */
export function legacyEarnedBlock(r: Pick<ReturnReport, 'legacy_earned' | 'bloodlines'>): HTMLElement | null {
  const gains = r.bloodlines?.some(s => s.legacy_earned !== undefined)
    ? [...r.bloodlines].sort((a,b) => a.id-b.id).map(s => ({ earned:s.legacy_earned, owner:s.name, id:s.id }))
    : [{ earned:r.legacy_earned, owner:undefined, id:undefined }];
  const positive = gains.filter(s => (s.earned ?? 0) > 0);
  if (!positive.length) return null;
  return h('section', { class:'report-legacy-earned report-class-xp' },
    h('b', { class:'row-label' }, /* copy:label */ 'Legacy earned'),
    ...positive.map(s => h('div', { class:'class-xp-line num', ...(s.id===undefined ? {} : {'data-bloodline':s.id}) },
      s.owner ? h('small', { class:'class-xp-owner' },s.owner) : '',
      h('b', { class:'legacy-earned-gain' }, `+${s.earned} Legacy`))));
}
