/** Decorative air currents only; never touches simulation state. */
export interface Mote { x: number; y: number; vx: number; vy: number; phase: number; lift: number; }
export interface MotePointer { x: number; y: number; vx: number; vy: number; strength: number; }

/** Smooth changing currents, drag, and a local pointer wake; all speeds in CSS px/s. */
export function advanceMote(m: Mote, dt: number, time: number, pointer: MotePointer): void {
  const wind = Math.sin(time * .55 + m.phase) * 13 + Math.cos(time * .29 + m.y * .009) * 9;
  let ax = (wind - m.vx) * 1.4;
  let ay = (-m.lift + Math.sin(time * .8 + m.phase) * 6 - m.vy) * 1.4;
  const dx = m.x - pointer.x, dy = m.y - pointer.y, distance = Math.hypot(dx, dy);
  if (pointer.strength > 0 && distance < 100) {
    const falloff = (1 - distance / 100) ** 2 * pointer.strength;
    const divisor = Math.max(8, distance);
    // Displace nearby dust and entrain it in the hand's wake; no collision/hit testing.
    ax += (dx / divisor * 150 - dy / divisor * 45 + pointer.vx * .45) * falloff;
    ay += (dy / divisor * 150 + dx / divisor * 45 + pointer.vy * .45) * falloff;
  }
  m.vx = Math.max(-75, Math.min(75, m.vx + ax * dt));
  m.vy = Math.max(-75, Math.min(75, m.vy + ay * dt));
  m.x += m.vx * dt; m.y += m.vy * dt;
}

/** One bounded layer on the current UI screen; no layout reads in the animation loop. */
export function ambientMotes(app: HTMLElement): void {
  const preference = matchMedia('(prefers-reduced-motion: reduce)');
  let main: HTMLElement | null = null, layer: HTMLElement | null = null;
  let motes: { body: Mote; el: HTMLElement }[] = [];
  let width = 0, height = 0, left = 0, top = 0, frame = 0, last = 0, time = 0;
  const pointer: MotePointer = { x: -1000, y: -1000, vx: 0, vy: 0, strength: 0 };
  let pointerAt = -Infinity;
  const measure = (): void => {
    if (!main) return;
    const rect = main.getBoundingClientRect();
    width = rect.width; height = rect.height; left = rect.left; top = rect.top;
  };
  const resize = new ResizeObserver(measure);
  const stop = (): void => {
    cancelAnimationFrame(frame); frame = 0; last = 0;
    resize.disconnect(); layer?.remove(); layer = null; motes = [];
    pointer.strength = 0; pointerAt = -Infinity;
  };
  const tick = (now: number): void => {
    frame = requestAnimationFrame(tick);
    if (last && now - last < 1000 / 30 - .5) return; // decorative, capped at 30 updates/s
    const dt = last ? Math.min(.065, (now - last) / 1000) : 1 / 30;
    last = now; time += dt;
    pointer.strength = Math.max(0, 1 - (now - pointerAt) / 1200);
    pointer.vx *= Math.exp(-dt * 5); pointer.vy *= Math.exp(-dt * 5);
    for (const { body: m, el } of motes) {
      advanceMote(m, dt, time, pointer);
      if (m.y < -12) m.y = height + 12;
      if (m.y > height + 16) m.y = -8;
      if (m.x < -12) m.x = width + 12;
      if (m.x > width + 16) m.x = -8;
      el.style.transform = `translate3d(${m.x.toFixed(2)}px,${m.y.toFixed(2)}px,0)`;
    }
  };
  const sync = (): void => {
    const next = app.querySelector<HTMLElement>(':scope > main.frame:not(.watch)');
    if (next === main && layer && !preference.matches && !document.hidden) return;
    stop(); main = next;
    if (!main || preference.matches || document.hidden || document.documentElement.dataset.juice !== 'on') return;
    measure(); resize.observe(main);
    layer = document.createElement('div'); layer.className = 'ambient-motes'; layer.setAttribute('aria-hidden', 'true');
    // Area-scaled density, with a hard ceiling even on an ultrawide display.
    const count = Math.min(36, Math.max(16, Math.round(width * height / 32000)));
    for (let i = 0; i < count; i++) {
      const el = document.createElement('i'); el.className = 'ambient-mote';
      const size = (width >= 1024 ? 5 : 3) + Math.random() * 3;
      el.style.width = el.style.height = `${size}px`; el.style.opacity = `${.35 + Math.random() * .45}`;
      const body = { x: Math.random() * width, y: Math.random() * height, vx: 0, vy: -8, phase: Math.random() * Math.PI * 2, lift: 7 + Math.random() * 14 };
      motes.push({ body, el }); layer.appendChild(el);
      el.style.transform = `translate3d(${body.x}px,${body.y}px,0)`;
    }
    main.appendChild(layer); frame = requestAnimationFrame(tick);
  };
  document.addEventListener('pointermove', (e) => {
    if (!layer || e.pointerType !== 'mouse') return;
    const now = performance.now(), dt = Math.max(.016, (now - pointerAt) / 1000);
    const x = e.clientX - left, y = e.clientY - top;
    pointer.vx = Math.max(-250, Math.min(250, (x - pointer.x) / dt));
    pointer.vy = Math.max(-250, Math.min(250, (y - pointer.y) / dt));
    pointer.x = x; pointer.y = y; pointerAt = now;
  }, { passive: true });
  document.addEventListener('pointerout', (e) => { if (!e.relatedTarget) { pointerAt = -Infinity; pointer.strength = 0; } }, { passive: true });
  document.addEventListener('visibilitychange', sync);
  preference.addEventListener('change', sync);
  addEventListener('resize', measure, { passive: true });
  new MutationObserver(sync).observe(app, { childList: true });
  sync();
}
