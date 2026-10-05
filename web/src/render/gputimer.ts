// GPU frame timing via EXT_disjoint_timer_query_webgl2 (Chrome desktop/Android, ANGLE). A ring of
// queries is issued one per frame around the whole pipeline; results are polled a few frames later
// so nothing stalls. `ms` is the newest resolved sample; `p50`/`p95` come from the last 120 samples.
// If the extension is missing, `available` is false and every reading is NaN: fall back to the CPU
// wall-time numbers, which on a phone are a lower bound only (the GPU runs asynchronously).
type Ext = {
  TIME_ELAPSED_EXT: number; GPU_DISJOINT_EXT: number;
};

const RING = 16;
const HIST = 120;

const STALL = 120; // frame attempts with pending queries but no progress → give up

export class GpuTimer {
  available: boolean; // false if the extension is missing or (after STALL frames) never resolves
  private gl: WebGL2RenderingContext;
  private ext: Ext | null;
  private ring: (WebGLQuery | null)[] = [];
  private head = 0;      // next query slot to issue
  private tail = 0;      // oldest unresolved slot
  private active = false;
  private waiting = 0;
  private samples: number[] = [];
  ms = NaN;
  resolved = 0;   // queries that produced a sample
  disjoint = 0;   // queries dropped because the GPU clock was disjoint

  constructor(gl: WebGL2RenderingContext) {
    this.gl = gl;
    this.ext = gl.getExtension("EXT_disjoint_timer_query_webgl2") as Ext | null;
    this.available = this.ext !== null;
  }

  begin(): void {
    if (!this.ext || this.active) return;
    if (this.gl.isContextLost()) { this.dispose(); return; }
    this.poll();
    if (this.head > this.tail && ++this.waiting >= STALL) { this.dispose(); return; }
    // don't overrun the ring: leave the oldest unresolved query in flight rather than reuse it
    if (this.head - this.tail >= RING) return;
    const slot = this.head % RING;
    let q = this.ring[slot] ?? null;
    if (!q) { q = this.gl.createQuery(); if (!q) { this.dispose(); return; } this.ring[slot] = q; }
    this.gl.beginQuery(this.ext.TIME_ELAPSED_EXT, q);
    this.active = true;
  }

  end(): void {
    if (!this.ext || !this.active) return;
    if (this.gl.isContextLost()) { this.dispose(); return; }
    this.gl.endQuery(this.ext.TIME_ELAPSED_EXT);
    this.active = false;
    this.head++;
    this.poll();
  }

  private poll(): void {
    const gl = this.gl, ext = this.ext!;
    const disjoint = gl.getParameter(ext.GPU_DISJOINT_EXT) as boolean;
    if (disjoint) {
      // A clock discontinuity invalidates every pending query, even if it is
      // not ready yet. Do not accept it later when the flag has cleared.
      this.disjoint += this.head - this.tail;
      this.clearQueries(); this.samples = []; this.ms = NaN;
      return;
    }
    while (this.tail < this.head) {
      const q = this.ring[this.tail % RING]!;
      const avail = gl.getQueryParameter(q, gl.QUERY_RESULT_AVAILABLE);
      if (!avail) break;
      const ns = gl.getQueryParameter(q, gl.QUERY_RESULT) as number;
      this.ms = ns / 1e6;
      this.samples.push(this.ms);
      if (this.samples.length > HIST) this.samples.shift();
      this.resolved++;
      this.tail++;
      this.waiting = 0;
    }
  }

  /** percentile of recent samples, NaN until any resolved */
  pct(p: number): number {
    if (this.samples.length === 0) return NaN;
    const s = this.samples.slice().sort((a, b) => a - b);
    return s[Math.min(s.length - 1, Math.floor(s.length * p))]!;
  }

  private clearQueries(): void {
    for (const q of this.ring) if (q) this.gl.deleteQuery(q);
    this.ring = [];
    this.head = this.tail = this.waiting = 0;
  }

  dispose(): void {
    if (this.active && this.ext && !this.gl.isContextLost()) this.gl.endQuery(this.ext.TIME_ELAPSED_EXT);
    this.active = false; this.clearQueries(); this.samples = [];
    this.ext = null;
    this.available = false;
    this.ms = NaN;
  }
}

/** Rolling percentile histogram of wall-clock durations (ms). */
export class Hist {
  private samples: number[] = [];
  private cap: number;
  last = NaN;
  constructor(cap = HIST) { this.cap = cap; }
  push(ms: number): void { this.last = ms; this.samples.push(ms); if (this.samples.length > this.cap) this.samples.shift(); }
  pct(p: number): number {
    if (this.samples.length === 0) return NaN;
    const s = this.samples.slice().sort((a, b) => a - b);
    return s[Math.min(s.length - 1, Math.floor(s.length * p))]!;
  }
  mean(): number { return this.samples.length ? this.samples.reduce((a, b) => a + b, 0) / this.samples.length : NaN; }
}
