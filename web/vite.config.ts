import { defineConfig, type Plugin } from "vite";
import { createHash } from "node:crypto";
import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

// Injects the emitted bundle's file list into public/sw.js as a versioned precache manifest.
function swPrecache(): Plugin {
  let outDir = "dist";
  const files: string[] = [];
  return {
    name: "riddle-sw-precache",
    apply: "build",
    configResolved(c) { outDir = c.build.outDir; },
    generateBundle(_o, bundle) { for (const f of Object.keys(bundle)) if (!/\.map$/.test(f)) files.push(`/${f}`); },
    closeBundle() {
      const p = join(outDir, "sw.js");
      let src: string;
      try { src = readFileSync(p, "utf8"); } catch { return; }
      // the self-hosted faces (public/fonts, docs/ART_DIRECTION.md §Typography) are precached with the bundle: the chrome never waits on the network
      let fonts: string[] = [];
      try { fonts = readdirSync(join(outDir, "fonts")).filter((f) => f.endsWith(".woff2")).map((f) => `/fonts/${f}`); } catch { /* no fonts dir */ }
      const list = ["/", "/index.html", "/manifest.webmanifest", "/favicon.svg", "/icons.svg", ...fonts, ...files.filter((f) => f !== "/index.html")];
      const version = createHash("sha1").update(list.join("\n") + readFileSync(join(outDir, "index.html"), "utf8")).digest("hex").slice(0, 12);
      writeFileSync(p, src.replace('"__VERSION__"', JSON.stringify(version)).replace("__PRECACHE__", JSON.stringify(list)));
    },
  };
}

export default defineConfig({
  plugins: [swPrecache()],
  build: { target: "es2022", sourcemap: false, chunkSizeWarningLimit: 900 },
});
