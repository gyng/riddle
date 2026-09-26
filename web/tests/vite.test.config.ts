// The client suite's own dev server (tests/run.mjs starts one per run on a free port): the project's config with the file
// watcher and HMR off, so an edit elsewhere in the tree (another agent's render or audio work) never reloads the suite's
// pages mid-check — "Execution context was destroyed" failed 12 of 31 gates in one run on the shared :5219 server. Each
// module is served as it stood when first asked for; the server lives as long as the run.
import { mergeConfig } from "vite";
import base from "../vite.config";

export default mergeConfig(base, { server: { hmr: false, watch: null } });
