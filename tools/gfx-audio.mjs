// node tools/gfx-audio.mjs [port] [out.json] (docs/JUICE.md §7, §10) — renders every cue (6 seeds; per family for the combat cues) and every biome bed in an
// OfflineAudioContext through the game's own master chain, and measures peak / RMS / short-term loudness (K-weighted, LUFS-ish) /
// clipping / spectral centroid; checks that consecutive renders of a cue differ (jitter) and that live cues log distinct signatures.
import { writeFileSync } from "node:fs";
import { launchBrowser, HEADLESS_ARGS } from "./browser.mjs";
const port = process.argv[2] ?? "5523";
const b = await launchBrowser({ args: [...HEADLESS_ARGS, "--autoplay-policy=no-user-gesture-required"] });
const p = await b.newPage();
await p.goto(`http://localhost:${port}/render-demo.html?busy=1&fx=low`);
const res = await p.evaluate(async () => {
  const A = await import("/src/audio.ts");
  const SR = 48000;
  const db = (x) => (x > 0 ? 20 * Math.log10(x) : -Infinity);
  async function kweight(x) {
    const ctx = new OfflineAudioContext(1, x.length, SR), buf = ctx.createBuffer(1, x.length, SR); buf.copyToChannel(x, 0);
    const s = ctx.createBufferSource(); s.buffer = buf;
    const hs = ctx.createBiquadFilter(); hs.type = "highshelf"; hs.frequency.value = 1500; hs.gain.value = 4;
    const hp = ctx.createBiquadFilter(); hp.type = "highpass"; hp.frequency.value = 38; hp.Q.value = 0.5;
    s.connect(hs); hs.connect(hp); hp.connect(ctx.destination); s.start();
    return (await ctx.startRendering()).getChannelData(0);
  }
  function fftMag(x, n = 8192) {
    const re = new Float64Array(n), im = new Float64Array(n);
    for (let i = 0; i < Math.min(n, x.length); i++) re[i] = x[i] * (0.5 - 0.5 * Math.cos((2 * Math.PI * i) / (Math.min(n, x.length) - 1)));
    for (let i = 1, j = 0; i < n; i++) { let bit = n >> 1; for (; j & bit; bit >>= 1) j ^= bit; j ^= bit; if (i < j) { [re[i], re[j]] = [re[j], re[i]]; [im[i], im[j]] = [im[j], im[i]]; } }
    for (let len = 2; len <= n; len <<= 1) { const a = (-2 * Math.PI) / len; for (let i = 0; i < n; i += len) for (let k = 0; k < len / 2; k++) {
      const wr = Math.cos(a * k), wi = Math.sin(a * k), ur = re[i + k], ui = im[i + k], vr = re[i + k + len / 2] * wr - im[i + k + len / 2] * wi, vi = re[i + k + len / 2] * wi + im[i + k + len / 2] * wr;
      re[i + k] = ur + vr; im[i + k] = ui + vi; re[i + k + len / 2] = ur - vr; im[i + k + len / 2] = ui - vi; } }
    const m = new Float64Array(n / 2); for (let i = 0; i < n / 2; i++) m[i] = Math.hypot(re[i], im[i]); return m;
  }
  const centroid = (x, pw = 1) => { const m = fftMag(x); let a = 0, w = 0; for (let i = 1; i < m.length; i++) { const f = (i * SR) / 8192, v = m[i] ** pw; a += f * v; w += v; } return w ? a / w : 0; };
  async function measure(x, activeS) {
    let pk = 0, ss = 0, clip = 0; const n = Math.min(x.length, Math.floor(SR * activeS));
    for (let i = 0; i < x.length; i++) { const v = Math.abs(x[i]); if (v > pk) pk = v; if (v >= 0.999) clip++; }
    for (let i = 0; i < n; i++) ss += x[i] * x[i];
    const k = await kweight(x); let ks = 0; for (let i = 0; i < n; i++) ks += k[i] * k[i];
    return { peak: +db(pk).toFixed(1), rms: +db(Math.sqrt(ss / n)).toFixed(1), lufs: +(-0.691 + 10 * Math.log10(ks / n)).toFixed(1), clip, centroid: Math.round(centroid(x)), centroidP: Math.round(centroid(x, 2)) };
  }
  const FAMS = { flesh: "goblin", bone: "skeleton", ooze: "bloat", metal: "iron_golem", spirit: "wraith" };
  const cases = [];
  for (const name of A.CUE_NAMES) {
    if (["hit", "strike", "slay"].includes(name)) for (const [fam, kind] of Object.entries(FAMS)) cases.push({ id: `${name}:${fam}`, name, opts: { dmg: 6, kind } });
    else if (name === "scene_end") { cases.push({ id: "scene_end:up", name, opts: { up: true } }); cases.push({ id: "scene_end:down", name, opts: { up: false } }); }
    else cases.push({ id: name, name, opts: {} });
  }
  const cues = [];
  for (const c of cases) {
    const renders = [];
    for (let seed = 1; seed <= 6; seed++) renders.push(await A.renderCue(c.name, c.opts, seed, SR));
    const active = c.name === "exit_death" ? 1.0 : 0.2;
    const ms = []; for (const r of renders) ms.push(await measure(r, active + 0.01));
    const pre = await A.renderCue(c.name, c.opts, 1, SR, true); let prePk = 0; for (const v of pre) prePk = Math.max(prePk, Math.abs(v));
    // consecutive renders differ: the smallest max-abs difference between render i and i+1
    let minDiff = Infinity; for (let i = 0; i + 1 < renders.length; i++) { let d = 0; for (let j = 0; j < renders[i].length; j++) d = Math.max(d, Math.abs(renders[i][j] - renders[i + 1][j])); minDiff = Math.min(minDiff, d); }
    const avg = (k) => +(ms.reduce((a, m) => a + m[k], 0) / ms.length).toFixed(1);
    const cs = ms.map((m) => m.centroid);
    cues.push({ id: c.id, peak: Math.max(...ms.map((m) => m.peak)), rms: avg("rms"), lufs: avg("lufs"), clip: ms.reduce((a, m) => a + m.clip, 0), prePeak: +db(prePk).toFixed(1),
      centroid: Math.round(avg("centroid")), centroidP: Math.round(avg("centroidP")), centroidSpread: Math.max(...cs) - Math.min(...cs), minConsecDiff: +minDiff.toFixed(4) });
  }
  const beds = [];
  for (const bi of A.BED_BIOMES) { const x = await A.renderBed(bi, 12, 1, SR); beds.push({ biome: bi, ...(await measure(x, 12)) }); }
  // a worst case: 3 heavy cues (the burst cap) at one instant, pre and post limiter
  const stack = async (pre) => { const xs = await Promise.all([A.renderCue("boss_down", {}, 1, SR, pre), A.renderCue("hit", { dmg: 24, kind: "ogre" }, 2, SR, pre), A.renderCue("boss_break", {}, 3, SR, pre)]); const y = new Float32Array(xs[0].length); for (const x of xs) for (let i = 0; i < y.length; i++) y[i] += x[i]; let pk = 0; for (const v of y) pk = Math.max(pk, Math.abs(v)); return +db(pk).toFixed(1); };
  // live: 12 hits in a row through the real object (Math.random jitter) — the logged signatures
  A.audio.unlock(); await new Promise((r) => setTimeout(r, 200));
  const sigs = []; for (let i = 0; i < 12; i++) { await new Promise((r) => setTimeout(r, 5)); A.audio.cue("strike", { dmg: 5, kind: "goblin" }); sigs.push(A.audio.log.at(-1)?.v); }
  let same = 0; for (let i = 1; i < sigs.length; i++) if (sigs[i] === sigs[i - 1]) same++;
  return { cues, beds, stackPre: await stack(true), stackNaive: "sum of the three pre-limiter renders", liveStrikes: sigs.length, liveConsecutiveIdentical: same, sampleSig: sigs.slice(0, 3) };
});
await b.close();
writeFileSync(process.argv[3] ?? "scratchpad/gfx-eval/audio.json", JSON.stringify(res, null, 1));
const f = (r) => `${r.id.padEnd(16)} peak ${String(r.peak).padStart(6)}  rms ${String(r.rms).padStart(6)}  lufs ${String(r.lufs).padStart(6)}  clip ${r.clip}  pre ${String(r.prePeak).padStart(6)}  centroid ${String(r.centroid).padStart(5)} ±${r.centroidSpread} (pow ${r.centroidP})  Δmin ${r.minConsecDiff}`;
for (const r of res.cues) console.log(f(r));
for (const r of res.beds) console.log(`bed ${r.biome.padEnd(8)} peak ${r.peak}  rms ${r.rms}  lufs ${r.lufs}  clip ${r.clip}  centroid ${r.centroid} (pow ${r.centroidP})`);
console.log(`stack of 3 heavy cues, pre-limiter peak ${res.stackPre} dBFS; live strikes ${res.liveStrikes}, consecutive identical ${res.liveConsecutiveIdentical}`);
