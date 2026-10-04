// Loopback-only Vite transport. Every client lane owns an ordered native process.
import { spawn } from "node:child_process";
import { createInterface } from "node:readline";
import { statSync, readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
class Session {
  constructor(binary, threads, balanceFile) { this.binary = binary; this.threads = threads; this.balanceFile = balanceFile; this.tail = Promise.resolve(); this.last = Date.now(); this.active = 0; this.start(); }
  profile() { return this.balanceFile ? readFileSync(this.balanceFile, 'utf8') : undefined; }
  start(profile = this.profile()) {
    this.balance = profile;
    this.stamp = `${statSync(this.binary).mtimeMs}:${statSync(this.binary).size}`;
    const env = { ...process.env, RIDDLE_NATIVE_THREADS: String(this.threads) };
    if (profile !== undefined) env.RIDDLE_BALANCE_JSON = profile;
    else delete env.RIDDLE_BALANCE_JSON;
    const child = spawn(this.binary, [], { cwd: ROOT, env, stdio: ["pipe", "pipe", "pipe"] });
    const pending = new Map(); this.child = child; this.pending = pending; this.next = 1;
    let error = ""; child.stderr.on("data", (b) => { error = (error + b).slice(-4000); });
    createInterface({ input: child.stdout }).on("line", (line) => {
      let response; try { response = JSON.parse(line); } catch { child.kill(); return; }
      const p = pending.get(response.id); if (!p) return;
      pending.delete(response.id);
      response.ok ? p.resolve(response.r) : p.reject(new Error(response.e));
    });
    const failed = (e) => { for (const p of pending.values()) p.reject(new Error(e)); pending.clear(); };
    child.on("error", (e) => failed(e.message));
    child.on("exit", (code, signal) => failed(`native engine exited (${signal ?? code}) ${error}`));
  }
  exchange(method, args) {
    if (this.child.exitCode !== null || this.child.killed) return Promise.reject(new Error("native engine stopped; reload the page"));
    const id = this.next++;
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      this.child.stdin.write(JSON.stringify({ id, method, args }) + "\n", (e) => { if (e) { this.pending.delete(id); reject(e); } });
    });
  }
  call(method, args) {
    this.active++; this.last = Date.now();
    const result = this.tail.then(async () => {
      const stat = statSync(this.binary), stamp = `${stat.mtimeMs}:${stat.size}`;
      const profile = this.profile();
      if (stamp !== this.stamp || profile !== this.balance) {
        if (profile !== undefined) await this.exchange('__validateBalance', [profile]);
        const save = await this.exchange("save", []);
        this.child.kill(); this.start(profile);
        await this.exchange("load", [save]);
      }
      return this.exchange(method, args);
    });
    this.tail = result.catch(() => {});
    return result.finally(() => { this.active--; this.last = Date.now(); });
  }
  close() { this.child.kill(); }
}
export class NativeHost {
  constructor({ binary = process.env.RIDDLE_NATIVE_BIN, threads = Number(process.env.RIDDLE_NATIVE_THREADS ?? 8), balanceFile = process.env.RIDDLE_BALANCE_FILE } = {}) {
    if (!binary || !Number.isInteger(threads) || threads < 1 || threads > 256) throw new Error("build native_dev first and select a valid native thread count");
    this.binary = resolve(binary); this.threads = threads; this.balanceFile = balanceFile && resolve(balanceFile); this.sessions = new Map(); this.expired = new Set();
    this.timer = setInterval(() => {
      for (const [id, s] of this.sessions) if (!s.active && Date.now() - s.last > 5 * 60_000) { s.close(); this.sessions.delete(id); this.expired.add(id); }
    }, 60_000); this.timer.unref();
  }
  call(id, method, args) {
    if (!/^[a-zA-Z0-9-]{1,80}$/.test(id) || typeof method !== "string" || !Array.isArray(args)) throw new Error("invalid native request");
    let session = this.sessions.get(id);
    if (!session) {
      if (this.expired.has(id) || !["version", "__bridge"].includes(method)) throw new Error("native session unavailable; reload the page");
      if (this.sessions.size >= 24) throw new Error("native session limit reached; close unused pages");
      session = new Session(this.binary, this.threads, this.balanceFile); this.sessions.set(id, session);
    }
    return session.call(method, args);
  }
  closeSession(id) {
    if (typeof id !== 'string' || !/^[a-zA-Z0-9-]{1,80}$/.test(id)) throw new Error('invalid native session');
    const session = this.sessions.get(id);
    if (session) { session.close(); this.sessions.delete(id); }
    this.expired.add(id);
  }
  health() { return { native: true, binary: this.binary, sha256: createHash("sha256").update(readFileSync(this.binary)).digest("hex"), balanceFile: this.balanceFile ?? null, balanceSha256: this.balanceFile ? createHash('sha256').update(readFileSync(this.balanceFile)).digest('hex') : null, threads: this.threads, sessions: this.sessions.size }; }
  close() { clearInterval(this.timer); for (const s of this.sessions.values()) s.close(); this.sessions.clear(); }
}
export function nativePlugin() {
  return { name: "riddle-native-dev", apply: "serve", configureServer(server) {
    if (process.env.RIDDLE_NATIVE_DEV !== "1") return;
    const host = new NativeHost(); server.httpServer?.once("close", () => host.close());
    server.middlewares.use("/__native", async (req, res) => {
      res.setHeader("Content-Type", "application/json"); res.setHeader("Cache-Control", "no-store");
      const reply = (status, value) => { res.statusCode = status; res.end(JSON.stringify(value)); };
      if (!["127.0.0.1", "::1", "::ffff:127.0.0.1"].includes(req.socket.remoteAddress)) return reply(403, { error: "native development requires a loopback connection" });
      if (!/^(localhost|127\.0\.0\.1|\[::1\])(?::\d+)?$/.test(req.headers.host ?? "")) return reply(403, { error: "native development is loopback only" });
      if (req.method === "GET" && req.url === "/health") return reply(200, host.health());
      if (req.method !== "POST" || !['/rpc', '/close'].includes(req.url) || req.headers.origin !== `http://${req.headers.host}`) return reply(403, { error: "same-origin native POST required" });
      try {
        const chunks = []; let size = 0;
        for await (const chunk of req) { size += chunk.length; if (size > 32 * 1024 * 1024) throw new Error("native request too large"); chunks.push(chunk); }
        const { session, method, args } = JSON.parse(Buffer.concat(chunks).toString());
        if (req.url === '/close') { host.closeSession(session); return reply(200, { ok: true }); }
        return reply(200, { ok: true, r: await host.call(session, method, args) });
      } catch (e) { return reply(400, { ok: false, e: e.message }); }
    });
  } };
}
