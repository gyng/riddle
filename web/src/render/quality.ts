// Juice (docs/JUICE.md): the effects tier. `low` is the pre-juice look, pixel for pixel (no light field, no bloom, no particles,
// no squash, no hit-stop); `med` adds the shadowed light field, bloom on emissives, particles, damage numbers and hit feedback;
// `high` adds the sprite rim light, the stone's relief under the light, heat shimmer over fire and dust motes.
//
// Auto: a software GL (SwiftShader / llvmpipe — the headless client gates) is `low`; any real GPU starts `high` and steps down
// one tier when two 3 s windows in a row of rAF intervals show more than 30 % of frames over 20 ms (never back up in a
// session). `?fx=low|med|high` (or localStorage `riddle.fx`) pins it.
// `prefers-reduced-motion: reduce` keeps the light and the bloom but drops every motion effect: shake, hit-stop, slow-mo,
// squash, the popping numbers' rise, the vignette's pulse, ambient particles.
export type Fx = "low" | "med" | "high";
const ORDER: Fx[] = ["low", "med", "high"];

export function softwareGl(gl: WebGLRenderingContext | WebGL2RenderingContext | null): boolean {
  if (!gl) return true;
  try {
    const d = gl.getExtension("WEBGL_debug_renderer_info");
    const r = String(d ? gl.getParameter(d.UNMASKED_RENDERER_WEBGL) : gl.getParameter(gl.RENDERER));
    return /swiftshader|llvmpipe|softpipe|software|basic render/i.test(r);
  } catch { return false; }
}

function pinned(): Fx | null {
  try {
    const q = new URLSearchParams(location.search).get("fx");
    if (q && (ORDER as string[]).includes(q)) return q as Fx;
    const s = localStorage.getItem("riddle.fx");
    if (s && (ORDER as string[]).includes(s)) return s as Fx;
  } catch { /* storage blocked: auto */ }
  return null;
}

export function reducedMotion(): boolean {
  try { return matchMedia("(prefers-reduced-motion: reduce)").matches; } catch { return false; }
}

export class Quality {
  fx: Fx;
  readonly pinned: boolean;
  readonly motion: boolean;
  private win: number[] = [];
  private cpu: number[] = [];
  private t0 = 0;
  private warm = 0;
  private bad = 0;
  onChange: ((fx: Fx) => void) | null = null;

  constructor(gl: WebGLRenderingContext | WebGL2RenderingContext | null) {
    const p = pinned();
    this.pinned = p !== null;
    this.fx = p ?? (softwareGl(gl) ? "low" : "high");
    this.motion = !reducedMotion();
  }

  at(min: Fx): boolean { return ORDER.indexOf(this.fx) >= ORDER.indexOf(min); }

  /** one rAF interval (ms) and this frame's pipeline cost; steps the tier down on a sustained miss (auto only) */
  sample(intervalMs: number, cpuMs: number, now: number): void {
    if (this.pinned || this.fx === "low") return;
    if (this.warm < 90) { this.warm++; this.t0 = now; return; }   // the first ~1.5 s (shader compiles, atlas upload) never count
    if (intervalMs > 250) { this.win.length = 0; this.cpu.length = 0; this.t0 = now; return; }   // a hidden tab, not a slow frame
    this.win.push(intervalMs); this.cpu.push(cpuMs);
    if (now - this.t0 < 3000) return;
    const slow = this.win.filter((x) => x > 20).length / this.win.length;
    this.win.length = 0; this.cpu.length = 0; this.t0 = now;
    // two bad windows in a row (a busy host's one-off stall is not the pipeline)
    this.bad = slow > 0.3 ? this.bad + 1 : 0;
    if (this.bad >= 2) { this.bad = 0; this.fx = ORDER[ORDER.indexOf(this.fx) - 1]!; this.warm = 0; this.onChange?.(this.fx); }
  }
}
