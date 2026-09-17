// Cut 11 §2 — a chain link's replay: a small viewer in a sheet, the floor the `because` happened on loaded from the run
// log, seeked to `because.t − LEAD` in the fight frame, played 40 ticks at 1×, then paused. A tap on the canvas plays the
// same 40 ticks again. The renderer's `seek(t)` rebuilds from the floor's loaded snapshot and replays the log up to `t`,
// so only floors the watch loaded can be scrubbed (`runlog.ts`); the Canvas-2D placeholder has no clock and no seek — it
// shows the floor at the end of the window instead.
import type { Because } from "../engine/types";
import { h } from "./dom";
import { floorFor, floorSnapshot, type RunLog } from "./runlog";
import { openSheet } from "./sheet";
import { makeViewer, type Viewer } from "./viewer";

const LEAD = 20, WINDOW = 40;   // ticks: seek to t − LEAD, play WINDOW, pause
const POLL_MS = 50;
type SeekViewer = Viewer & { seek?(t: number): void; setFrame?(frame: "map" | "fight"): void };

export function openReplay(log: RunLog, b: Because): void {
  const floor = floorFor(log, b);
  if (!floor) return;
  const from = Math.max(floor.snap.turn, b.t - LEAD), to = from + WINDOW;
  openSheet(() => {
    const canvas = h("canvas", { class: "replay-view" });
    const at = h("span", { class: "num dim" }, /* copy:none */ `D${b.depth} · t${b.t}`);
    const body = h("div", { class: "sheet-body replay", "data-t": b.t },
      h("div", { class: "label row-label" }, h("span", { class: "because" }, "← ", b.text), " ", at),
      canvas);
    let viewer: SeekViewer | null = null, timer = 0, playing = false;
    const play = (): void => {
      if (!viewer) return;
      if (viewer.seek) { viewer.seek(from); viewer.setSpeed(1); playing = true; }
      else viewer.setSpeed(0);                                   // placeholder: no clock; the floor stands at the window's end
    };
    const stop = (): void => { clearInterval(timer); viewer?.dispose(); viewer = null; if ("__replay" in window) delete (window as { __replay?: unknown }).__replay; };
    // the sheet mounts after build returns: the canvas needs its layout size before the renderer measures it
    requestAnimationFrame(() => {
      if (!document.contains(canvas)) return;
      void makeViewer(canvas).then(({ viewer: v }) => {
        if (!document.contains(canvas)) { v.dispose(); return; }
        viewer = v; v.resize?.();
        v.load(floorSnapshot(floor)); v.apply(floor.evs);
        viewer.setFrame?.("fight");
        play();
        if ("__riddle" in window) (window as unknown as { __replay: Viewer }).__replay = v;   // dev: tests read `tick()`
        timer = window.setInterval(() => {
          if (!document.contains(canvas)) { stop(); return; }          // closed by backdrop / Escape: the sheet has no close hook
          if (playing && viewer?.tick && viewer.tick() >= to) { viewer.setSpeed(0); playing = false; }
        }, POLL_MS);
      });
    });
    canvas.onclick = () => { if (!playing) play(); };
    return body;
  });
}
