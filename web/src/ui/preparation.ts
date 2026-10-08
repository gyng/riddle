// Links to actual, optional preparation. The core owns readiness and prices.
import type { App } from '../app';
import type { Lineage } from '../engine/types';
import { h, replace } from './dom';
import { icon } from './skin';
import { paintFace } from './frame';
import { openWindow } from './sheet';
import { openHero } from './town';
import { goldSink, kitAffordable, openForge } from './forge';
import { openPackages, packagesShown } from './packages';
import { unitLabel } from './unit-icon';
import { enemyHost } from './enemy-tips';

const forgeBuilt = (L: Lineage): boolean => !!L.town?.buildings.some(b => b.id === 'blacksmith');
const forgeReady = (L: Lineage): boolean => L.town?.next === 'blacksmith' && L.town.next_ready === true && !forgeBuilt(L);

/** A recommendation cannot treat an unlocked plot as a standing forge. */
export function openPreparationForge(app: App, anchor?: HTMLElement | null): void {
  if (!app.lineage.town || forgeBuilt(app.lineage)) { openForge(app, anchor); return; }
  let off = (): void => {}, busy = false;
  const engine = app.engine;
  openWindow(close => {
    const body = h('div', { class: 'sheet-body preparation-build' });
    let shown = '';
    const paint = (): void => {
      const L = app.lineage;
      const current = app.engine === engine;
      const built = forgeBuilt(L);
      const ready = current && forgeReady(L) && !!engine.buildTown;
      const key = JSON.stringify([current, built, ready, busy, L.town?.next, L.town?.next_trigger]);
      if (shown === key) return;
      shown = key;
      replace(body, h('h3', null, built ? /* copy:label */ 'Forge gear' : /* copy:label */ 'Build forge'),
        h('div', { class: 'preparation-build-icon' }, icon('forge', '⚒')),
        built ? null : h('p', { class: 'num' }, ready ? /* copy:label */ 'Free' : L.town?.next === 'blacksmith' ? L.town.next_trigger ?? /* copy:label */ 'Not ready' : /* copy:label */ 'Not ready'),
        h('button', { class: 'chip preparation-build-button', disabled: busy || !(current && built || ready), onclick: () => {
          if (!busy && app.engine === engine && forgeBuilt(app.lineage)) { close(); openForge(app); return; }
          if (busy || app.engine !== engine || !forgeReady(app.lineage) || !engine.buildTown) return;
          busy = true; paint();
          void app.mutate(async () => {
            const lineage = await engine.buildTown!('blacksmith');
            if (app.engine !== engine) throw Error(/* copy:none */ 'Preparation session changed');
            return lineage;
          }, /* copy:callout */ 'Built', false, { saveBeforePaint: true }).then(ok => {
            if (ok && body.isConnected && forgeBuilt(app.lineage)) { close(); openForge(app); }
          }).finally(() => { busy = false; if (body.isConnected) paint(); });
        } }, built ? /* copy:button */ 'Open forge' : /* copy:button */ 'Build forge'));
    };
    paint(); off = app.onChange(() => { if (body.isConnected) paint(); });
    return body;
  }, { anchor, onClose: () => off() });
}

export function preparationActions(app: App, options: { collapsed?: boolean; obstacle?: string; cause?: string; showObstacle?: boolean; report?: boolean } = {}) {
  const host = h('div', { class: options.report ? 'report-upgrade-host preparation-host' : 'preparation-host' });
  const actions = h('div', { class: 'preparation-actions' });
  const contents = options.collapsed
    ? h('details', { class: 'preparation-fold' }, h('summary', null, /* copy:button */ 'Next run'), actions)
    : h('section', { class: 'preparation-section' }, h('h3', null, /* copy:label */ 'Next run'), options.obstacle && options.showObstacle !== false ? h('div', { class: 'preparation-obstacle num' },
      options.cause ? enemyHost(unitLabel(options.cause, options.obstacle, { px: 30 }), options.cause, app.lineage) : options.obstacle) : null, actions);
  host.append(contents);
  let key = '';
  const paint = (): void => {
    const L = app.lineage, points = L.bloodline?.points ?? L.hero_legacy?.find(x => x.heir === L.heir)?.points;
    const home = !L.live && L.town?.home !== false && !L.ended;
    const hero = home && points !== undefined && !!app.engine.upgradeHero && !!L.legacy_upgrades?.some(u => u.affordable && u.rank < u.cap);
    const build = home && forgeReady(L) && !!app.engine.buildTown;
    const kit = kitAffordable(L);
    const forge = home && (!L.town || forgeBuilt(L)) && kit > 0 && !!app.engine.buyKit;
    // blind 77030eb (B): a complete kit with gold to spare names the next town work (the forge holds its button)
    const sink = home && !forge && !build && !!app.engine.commission && (!L.town || forgeBuilt(L)) ? goldSink(L) : null;
    const tactics = home && !!options.obstacle && packagesShown(L) && !!app.engine.equipPackage && !!L.packages?.all.some(p => p.owned && p.slot === undefined);
    const next = JSON.stringify([hero, build, forge, tactics, kit, points, L.selected_bloodline, L.class, L.look, sink]);
    if (next === key) return;
    key = next; host.hidden = !(hero || build || forge || tactics || sink);
    const row = (className: string, art: Node, label: string, detail: string, open: (anchor: HTMLElement) => void): HTMLElement =>
      h('button', { class: `chip preparation-action ${className}`, onclick: (e: Event) => open(e.currentTarget as HTMLElement) }, art,
        h('span', { class: 'report-upgrade-copy' }, h('b', null, label), h('small', { class: 'num' }, detail)));
    const face = h('span', { class: 'icon-socket', 'aria-hidden': 'true' });
    if (hero) paintFace(face, L.class, 44, L.look);
    replace(actions,
      ...(hero ? [row('report-upgrade', face, /* copy:button */ 'Upgrade hero', /* copy:label */ `Bloodline ${L.selected_bloodline ?? 1} · ${points} Legacy`, anchor => openHero(app, anchor))] : []),
      ...(build || forge ? [row('preparation-forge', icon('forge', '⚒'), build ? /* copy:button */ 'Build forge' : /* copy:button */ 'Forge gear', build ? /* copy:label */ 'Free' : /* copy:label */ `${kitAffordable(L)} upgrades ready`, anchor => openPreparationForge(app, anchor))] : []),
      ...(sink ? [row('preparation-sink', icon('camp', '⌂'), /* copy:button */ 'Town upgrade', /* copy:label */ `${sink.label} · $${sink.price}`, anchor => openForge(app, anchor))] : []),
      ...(tactics ? [row('preparation-tactics', icon('unlocks', '✦'), /* copy:button */ 'Tactics', /* copy:label */ 'Owned choices', anchor => openPackages(app, anchor))] : []));
  };
  paint();
  const off = app.onChange(paint), offLive = app.onLive(paint);
  return { el: host, dispose: () => { off(); offLive(); } };
}
