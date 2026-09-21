#!/usr/bin/env node
// GPU-accelerated Chromium under WSLg for playtests and screenshots.
// Headless Playwright Chromium in WSL2 falls back to SwiftShader (software Vulkan); headed Chromium
// under WSLg reaches Mesa, and Mesa reaches the Windows GPU only through its D3D12 gallium driver
// with the WSL runtime libraries on the loader path. Verified: "ANGLE (Microsoft, D3D12 (NVIDIA
// GeForce RTX 3080), OpenGL 4.6)" vs llvmpipe/SwiftShader otherwise.
//
//   import { launchGpu } from "./tools/browser.mjs"; const browser = await launchGpu();
//   node tools/browser.mjs --probe            prints the WebGL renderer string
//   node tools/browser.mjs <url> [out.png] [w] [h] [dpr]   screenshot after 3 s
//
// DPI: WSLg exports GDK_SCALE=2 and Xft.dpi=144 for a 150 % Windows display, and Chromium multiplies
// them into a native scale of 3 (screen 1280×720 CSS px; a 400×800 phone window is taller than the
// monitor). The launch pins GDK_SCALE=1 and forces the Windows scale (Xft.dpi / 96, default 1.5) so
// headed windows are life-size; per-page deviceScaleFactor is CDP emulation and is unaffected.
// Headed Chromium also draws 15 px classic scrollbars inside the viewport (full-page shots came out
// 385 CSS px wide); --hide-scrollbars restores the overlay-scrollbar phone geometry.
import { chromium } from "playwright";
import { execFileSync } from "node:child_process";

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
export const GPU_ARGS = [
  "--ignore-gpu-blocklist", "--use-gl=angle", "--use-angle=gl", "--enable-gpu-rasterization",
  `--force-device-scale-factor=${WINDOWS_SCALE}`, "--hide-scrollbars",
];

export function launchGpu(extra = {}) {
  return chromium.launch({ headless: false, args: GPU_ARGS, env: { ...process.env, ...GPU_ENV }, ...extra });
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const [a, out = "shot.png", w = "400", h = "800", dpr = "3"] = process.argv.slice(2);
  const b = await launchGpu();
  const p = await b.newPage({ viewport: { width: +w, height: +h }, deviceScaleFactor: +dpr });
  if (a === "--probe" || !a) {
    await p.goto("about:blank");
    console.log(await p.evaluate(() => { const gl = document.createElement("canvas").getContext("webgl2"); const d = gl.getExtension("WEBGL_debug_renderer_info"); return gl.getParameter(d.UNMASKED_RENDERER_WEBGL); }));
    const n = await b.newPage({ viewport: null }); await n.goto("about:blank");
    console.log("native scale", await n.evaluate(() => devicePixelRatio), "screen", await n.evaluate(() => `${screen.width}×${screen.height}`), "(forced", WINDOWS_SCALE + ")");
  } else {
    await p.goto(a); await p.waitForTimeout(3000); await p.screenshot({ path: out }); console.log("wrote", out);
  }
  await b.close();
}
