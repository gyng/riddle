import { defineConfig, type Plugin } from "vite";
import { createHash } from "node:crypto";
import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

// Injects the emitted bundle's file list into public/sw.js as a versioned precache manifest.
function swPrecache(): Plugin {
  let outDir = "dist", base = "/";
  const files: string[] = [];
  return {
    name: "riddle-sw-precache",
    apply: "build",
    configResolved(c) { outDir = c.build.outDir; base = c.base; },
    generateBundle(_o, bundle) { for (const f of Object.keys(bundle)) if (!/\.map$/.test(f)) files.push(`${base}${f}`); },
    closeBundle() {
      const p = join(outDir, "sw.js");
      let src: string;
      try { src = readFileSync(p, "utf8"); } catch { return; }
      // the self-hosted faces (public/fonts, docs/ART_DIRECTION.md §Typography) are precached with the bundle: the chrome never waits on the network
      let fonts: string[] = [];
      try { fonts = readdirSync(join(outDir, "fonts")).filter((f) => f.endsWith(".woff2")).map((f) => `${base}fonts/${f}`); } catch { /* no fonts dir */ }
      const list = [base, `${base}index.html`, `${base}manifest.webmanifest`, `${base}favicon.svg`, `${base}icons.svg`, ...fonts, ...files.filter((f) => f !== `${base}index.html`)];
      const version = createHash("sha1").update(list.join("\n") + readFileSync(join(outDir, "index.html"), "utf8")).digest("hex").slice(0, 12);
      writeFileSync(p, src.replace('"__VERSION__"', JSON.stringify(version)).replace("__PRECACHE__", JSON.stringify(list)));
    },
  };
}

export default defineConfig({
  base: process.env.RIDDLE_BASE ?? "/",
  define: { "import.meta.env.VITE_BUILD_DATE": JSON.stringify(new Date().toISOString().slice(0, 10)) },
  plugins: [swPrecache()],
  build: { target: "es2022", sourcemap: false, chunkSizeWarningLimit: 900 },
});
