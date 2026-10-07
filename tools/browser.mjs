#!/usr/bin/env node
// Browser harness: headless Chromium for DOM/text checks, isolated headed GPU Chromium in WSL for
// anything that renders or is timed.
//
//   import { launchBrowser, launchGpu } from "./tools/browser.mjs";
//   const b = await launchBrowser();              headless (SwiftShader WebGL): tests, walks, raters — fast, parallel-safe, no windows
//   const b = await launchBrowser({ gpu: true }); headed on the real GPU: frame times, render QA, feel
//   const b = await launchGpu();                  the same as { gpu: true }, on a private X display (no desktop focus)
//   RIDDLE_BROWSER=headed|headless                 overrides the default for callers that pass nothing
//   node tools/browser.mjs --probe [--headless]   prints the WebGL renderer (D3D12 (NVIDIA …) headed; SwiftShader headless)
//   node tools/browser.mjs <url> [out.png] [w] [h] [dpr] [--headed]   screenshot after 3 s
//
// GPU: headless Playwright Chromium in WSL2 falls back to SwiftShader (software Vulkan) whatever the
// flags (probed: --headless=new with the Mesa env below still reports SwiftShader/llvmpipe); headed
// Chromium under WSLg reaches Mesa, and Mesa reaches the Windows GPU only through its D3D12 gallium
// driver with the WSL runtime libraries on the loader path. Verified: "ANGLE (Microsoft, D3D12
// (NVIDIA GeForce RTX 3080), OpenGL 4.6)" vs llvmpipe/SwiftShader otherwise.
//
// DPI: WSLg exports GDK_SCALE=2 and Xft.dpi=144 for a 150 % Windows display, and Chromium multiplies
// them into a native scale of 3 (screen 1280×720 CSS px; a 400×800 phone window is taller than the
// monitor). The headed launch pins GDK_SCALE=1 and forces the Windows scale (Xft.dpi / 96, default
// 1.5) so windows are life-size; per-page deviceScaleFactor is CDP emulation and is unaffected.
// Headed Chromium also draws 15 px classic scrollbars inside the viewport (full-page shots came out
// 385 CSS px wide); --hide-scrollbars restores the overlay-scrollbar phone geometry.
import { chromium } from "playwright";
import { execFileSync, spawn } from "node:child_process";

function windowsScale() {
  if (process.env.WSL_SCALE) return +process.env.WSL_SCALE;
  try {
    const m = /Xft\.dpi:\s*(\d+)/.exec(execFileSync("xrdb", ["-query"], { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] }));
    if (m) return +m[1] / 96;
  } catch {}
  return 1.5;
}
export const WINDOWS_SCALE = windowsScale();

export const GPU_ENV = {
  GALLIUM_DRIVER: "d3d12",
  MESA_LOADER_DRIVER_OVERRIDE: "d3d12",
  LD_LIBRARY_PATH: "/usr/lib/wsl/lib:" + (process.env.LD_LIBRARY_PATH ?? ""),
  GDK_SCALE: "1",
  GDK_DPI_SCALE: "1",
};
// Audio: headed Chromium under WSLg has a real sink, so a walk or a rater session would play the
// game's cues through the desk's speakers; every launch is muted at the browser (WebAudio still
// schedules, so `window.__audio` and web/tests/audio.mjs see the cues).
export const GPU_ARGS = [
  "--ignore-gpu-blocklist", "--use-gl=angle", "--use-angle=gl", "--enable-gpu-rasterization",
  `--force-device-scale-factor=${WINDOWS_SCALE}`, "--hide-scrollbars", "--mute-audio",
];

export const HEADLESS_ARGS = ["--hide-scrollbars", "--mute-audio"];
// Muted anyway, so no sound server either: a wedged WSLg PulseAudio socket made Chromium's first
// mouse-down wait ~5 s on it (Playwright's click timeout; 5 of 10 client gates failed on it).
const NO_PULSE = { PULSE_SERVER: "unix:/nonexistent" };

/// Headed on the GPU when `gpu` is set or RIDDLE_BROWSER=headed; headless otherwise.
// A private X server keeps every headed automation window off the WSLg desktop.
// Mesa D3D12 still reaches the Windows GPU (probe it); never fall back to DISPLAY=:0.
async function privateDisplay() {
  // xvfb-run handles WSL's shared /tmp/.X11-unix and Xauthority, allocating a
  // free display from :99. Its child stays alive only until our stdin closes.
  const keeper = "process.stdout.write(JSON.stringify({name:process.env.DISPLAY,authority:process.env.XAUTHORITY})+'\\n');process.stdin.resume();";
  const server = spawn("xvfb-run", ["-a", "-s", "-screen 0 1920x1080x24 -nolisten tcp", process.execPath, "-e", keeper],
    { stdio: ["pipe", "pipe", "pipe"] });
  let diagnostics = "", output = "";
  server.stderr.on("data", chunk => { diagnostics = (diagnostics + chunk).slice(-2000); });
  let stopped = false;
  const stop = () => { if (!stopped) { stopped = true; server.stdin.end(); } };
  try {
    const display = await new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error(`Private browser display timed out: ${diagnostics}`)), 5000);
      const fail = error => { clearTimeout(timer); reject(error); };
      server.once("error", fail);
      server.once("exit", code => fail(new Error(`Private browser display exited (${code}): ${diagnostics}`)));
      server.stdout.on("data", chunk => {
        output += chunk;
        if (output.includes("\n")) {
          clearTimeout(timer);
          try { resolve(JSON.parse(output.trim())); } catch (error) { reject(error); }
        }
      });
    });
    return { ...display, stop };
  } catch (error) { stop(); throw error; }
}

export async function launchBrowser({ gpu = false, headed, ...extra } = {}) {
  const env = process.env.RIDDLE_BROWSER;
  const useHeaded = extra.headless === false || (headed ?? (env === "headed" ? true : env === "headless" ? false : gpu));
  if (useHeaded) {
    const display = await privateDisplay();
    const cleanup = () => { display.stop(); process.removeListener("exit", cleanup); };
    process.once("exit", cleanup);
    try {
      const browser = await chromium.launch({ ...extra, headless: false,
        args: [...(extra.args ?? GPU_ARGS), "--ozone-platform=x11", `--display=${display.name}`],
        env: { ...process.env, ...GPU_ENV, ...NO_PULSE, ...extra.env, DISPLAY: display.name, XAUTHORITY: display.authority, WAYLAND_DISPLAY: "" } });
      browser.once("disconnected", cleanup);
      return browser;
    } catch (error) { cleanup(); throw error; }
  }
  return chromium.launch({ ...extra, headless: true, args: extra.args ?? HEADLESS_ARGS, env: { ...process.env, ...NO_PULSE, ...extra.env } });
}

/// The GPU path: headed Chromium on D3D12. For frame times, render QA and anything a person would feel.
export function launchGpu(extra = {}) {
  return launchBrowser({ gpu: true, headed: true, ...extra });
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const flags = process.argv.slice(2).filter((x) => x.startsWith("--") && x !== "--probe");
  const [a, out = "shot.png", w = "400", h = "800", dpr = "3"] = process.argv.slice(2).filter((x) => !flags.includes(x));
  const headed = flags.includes("--headed") ? true : flags.includes("--headless") ? false : a !== "--probe" && !!a ? false : true;
  const b = await launchBrowser({ headed });
  const p = await b.newPage({ viewport: { width: +w, height: +h }, deviceScaleFactor: +dpr });
  if (a === "--probe" || !a) {
    await p.goto("about:blank");
    console.log(await p.evaluate(() => { const gl = document.createElement("canvas").getContext("webgl2"); const d = gl.getExtension("WEBGL_debug_renderer_info"); return gl.getParameter(d.UNMASKED_RENDERER_WEBGL); }));
    const n = await b.newPage({ viewport: null }); await n.goto("about:blank");
    console.log(headed ? "headed" : "headless", "· native scale", await n.evaluate(() => devicePixelRatio), "· screen", await n.evaluate(() => `${screen.width}×${screen.height}`), headed ? `(forced ${WINDOWS_SCALE})` : "");
  } else {
    await p.goto(a); await p.waitForTimeout(3000); await p.screenshot({ path: out }); console.log("wrote", out);
  }
  await b.close();
}
