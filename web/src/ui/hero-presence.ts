import type { HeroSlot, LiveRun } from '../engine/types';
import { spanOf } from './dom';

export type ObservedPresence = { slot: number; live: LiveRun; ended: boolean };
/** Presentation data may replace an older summary only for the exact live hero/run. */
export function heroPresence(s: HeroSlot, observed?: ObservedPresence) {
  const useObserved = s.state === 'live' && observed?.slot === s.id && observed.live.heir === s.heir
    && (!s.live || observed.live.run_id === s.live.run_id && observed.live.turn >= s.live.turn);
  const live = useObserved ? observed!.live : s.live;
  const activity = s.state !== 'live' ? s.state : useObserved && observed!.ended ? 'ended' : live?.activity ?? 'unknown';
  const names: Record<string, string> = { combat: /* copy:label */ 'In combat', returning: /* copy:label */ 'Heading home', exploring: /* copy:label */ 'Exploring', ended: /* copy:label */ 'Run ended', unknown: /* copy:label */ 'Delving' };
  const text = s.state === 'live' ? live ? `D${live.depth} · ${names[activity] ?? names.unknown}` : /* copy:label */ 'Starting run'
    : s.state === 'rests' && s.rest_s > 0 ? /* copy:label */ `Resting ${spanOf(s.rest_s)}` : /* copy:label */ 'Ready';
  return { text, activity, detail: live ? /* copy:label */ `${live.hp}/${live.max_hp} hp · run ${live.run_id}` : text };
}
