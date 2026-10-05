import type { App } from "./app";
import type { SaveBlob } from "./store";

type Note = { at: string; kind: string; text: string };
const notes: Note[] = [];
/** Bounded diagnostics; never record arbitrary storage or URL parameters. */
export function debugNote(kind: string, text: string): void {
  const clean = text.slice(0, 2000).replace(/https?:\/\/[^\s]+/g, (value) => {
    try { const u = new URL(value); return `${u.origin}${u.pathname}`; } catch { return "[url]"; }
  });
  notes.push({ at: new Date().toISOString(), kind, text: clean });
  if (notes.length > 64) notes.shift();
}
let installed = false;
export function installDiagnostics(): void {
  if (installed) return;
  installed = true;
  window.addEventListener("error", (e) => debugNote("error", e.message));
  window.addEventListener("unhandledrejection", (e) => debugNote("rejection", e.reason instanceof Error ? e.reason.message : String(e.reason)));
}

export type DebugBundle = {
  format: "riddle-debug"; schema: 1; captured_at: string;
  build: { commit: string; date: string; engine: string; version: string };
  capture: { state: "fresh"; context: "at-request"; requested_at: string; unavailable: string[] };
  save: SaveBlob; context: unknown; diagnostics: Note[];
};

export async function captureDebug(app: App): Promise<DebugBundle> {
  // Save is posted immediately to the ordered engine lane. UI editing metadata
  // is frozen before posting; reject/retry if it changes while waiting.
  const identity = (): string => JSON.stringify([app.lineage.selected_bloodline, app.active, app.sets, app.loadout, app.view.kind, app.watchMode]);
  for (let attempt = 0; attempt < 3; attempt++) {
    const key = identity(), engine = app.engine, requested_at = new Date().toISOString();
    const blob = JSON.parse(app.exportSave()) as SaveBlob;
    const context = structuredClone({ view: app.view, bloodline: app.lineage.selected_bloodline ?? 1,
      active_set: app.active, editing_sets: app.sets, loadout: app.loadout, over_budget: app.overBudget,
      live_at_request: app.lineage.live ?? null, watch_mode: app.watchMode, slowdowns: app.slowdowns,
      replay_at_request: { ...document.querySelector<HTMLElement>(".watch")?.dataset } });
    let timer: ReturnType<typeof setTimeout> | undefined;
    const state = await Promise.race([engine.save(), new Promise<never>((_resolve, reject) => {
      timer = setTimeout(() => reject(new Error("Engine capture timed out")), 30_000);
    })]).finally(() => clearTimeout(timer)); // Never fall back to lastSave.
    if (engine !== app.engine || key !== identity()) continue;
    if (!state || typeof state !== "string") throw new Error("Empty engine save");
    JSON.parse(state); // Do not download a corrupt/partial save as a success.
    const unavailable: string[] = [];
    let environment: unknown;
    try {
      const prefs: Record<string, string | null> = {};
      for (const name of ["riddle.mute", "riddle.slowdowns", "riddle.autoContinue", "riddle.editing", "riddle.meters"]) prefs[name] = localStorage.getItem(name);
      environment = { browser: navigator.userAgent, viewport: { width: innerWidth, height: innerHeight, scale: devicePixelRatio },
        visibility: document.visibilityState, preferences: prefs,
        canvases: [...document.querySelectorAll("canvas")].map((c) => ({ width: c.width, height: c.height })) };
    } catch { unavailable.push("environment"); }
    const captured_at = new Date().toISOString();
    debugNote("export", app.view.kind);
    return { format: "riddle-debug", schema: 1, captured_at,
      build: { commit: import.meta.env.VITE_BUILD_COMMIT ?? "unknown", date: import.meta.env.VITE_BUILD_DATE ?? "unknown", engine: app.kind, version: app.version },
      capture: { state: "fresh", context: "at-request", requested_at, unavailable },
      save: { ...blob, engine: state, last_seen: Date.parse(captured_at) }, context: { ...context, environment }, diagnostics: structuredClone(notes) };
  }
  throw new Error("State changed during export");
}

export function debugFilename(bundle: DebugBundle): string {
  return `riddle-debug-${bundle.build.commit.replace(/[^\w-]/g, "").slice(0, 20)}-${bundle.captured_at.replace(/[:.]/g, "-")}.json`;
}

export function downloadDebug(text: string, filename: string): void {
  const url = URL.createObjectURL(new Blob([text], { type: "application/json" }));
  const a = document.createElement("a"); a.href = url; a.download = filename;
  document.body.append(a);
  try { a.click(); } finally { a.remove(); window.setTimeout(() => URL.revokeObjectURL(url), 30_000); }
}
