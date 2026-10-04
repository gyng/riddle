// Dev-only rebuild queue. Edits are debounced; one build runs at a time.
import { spawn } from 'node:child_process';
import { relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
const ROOT = fileURLToPath(new URL('../', import.meta.url));

export function nativeInput(file, root = ROOT) {
  const path = relative(root, resolve(file)).replaceAll('\\', '/');
  return ['Cargo.toml', 'Cargo.lock', 'crates/riddle-core/Cargo.toml', 'crates/riddle-wasm/Cargo.toml',
    'crates/riddle-core/examples/native_dev.rs', 'tools/native-build.mjs', 'tools/native-codegen.mjs'].includes(path)
    || /^(crates\/riddle-(core|wasm)\/src\/|crates\/riddle-core\/presets\/)/.test(path);
}

export class NativeRebuilder {
  constructor({ build, publish = () => {}, debounceMs = 250 }) {
    this.build = build; this.publish = publish; this.debounceMs = debounceMs;
    this.state = { phase: 'ready', revision: 0, durationMs: null, error: null };
    this.closed = false; this.running = false; this.dirty = false;
  }
  request() {
    if (this.closed) return;
    this.dirty = true; this.state.revision++; this.editedAt = Date.now();
    if (!this.running && this.state.phase !== 'queued') {
      this.state = { ...this.state, phase: 'queued', error: null };
      this.publish({ ...this.state });
    }
    clearTimeout(this.timer);
    this.timer = setTimeout(() => this.drain(), this.debounceMs);
  }
  async drain() {
    if (this.closed || this.running || !this.dirty) return;
    this.running = true; this.dirty = false;
    const editedAt = this.editedAt;
    this.state = { ...this.state, phase: 'building', error: null };
    this.publish({ ...this.state });
    try {
      await this.build();
      this.state = { ...this.state, phase: this.dirty ? 'queued' : 'ready', durationMs: Date.now() - editedAt, error: null };
    } catch (e) {
      this.state = { ...this.state, phase: 'error', durationMs: Date.now() - editedAt, error: String(e.message ?? e).slice(-4000) };
    } finally {
      this.running = false;
      if (!this.closed) {
        this.publish({ ...this.state });
        if (this.dirty) {
          clearTimeout(this.timer);
          this.timer = setTimeout(() => this.drain(), this.debounceMs);
        }
      }
    }
  }
  close() { this.closed = true; clearTimeout(this.timer); }
}

export function watchNative(server) {
  let child;
  const watcher = new NativeRebuilder({
    publish: data => server.ws.send({ type: 'custom', event: 'riddle:native-build', data }),
    build: () => new Promise((resolveBuild, reject) => {
      let errors = '';
      child = spawn(process.execPath, ['tools/native-build.mjs'], { cwd: ROOT, env: process.env, detached: process.platform !== 'win32', stdio: ['ignore', 'ignore', 'pipe'] });
      child.stderr.on('data', b => { errors = (errors + b).slice(-4000); });
      child.on('error', reject);
      child.on('exit', (code, signal) => {
        child = undefined;
        code === 0 ? resolveBuild() : reject(new Error(errors || `native build exited (${signal ?? code})`));
      });
    }),
  });
  server.watcher.add(['Cargo.toml', 'Cargo.lock', 'crates/riddle-core', 'crates/riddle-wasm', 'tools/native-build.mjs', 'tools/native-codegen.mjs'].map(p => resolve(ROOT, p)));
  const changed = file => { if (nativeInput(file)) watcher.request(); };
  for (const event of ['add', 'change', 'unlink']) server.watcher.on(event, changed);
  return {
    get state() { return watcher.state; },
    close() {
      watcher.close();
      for (const event of ['add', 'change', 'unlink']) server.watcher.off(event, changed);
      if (child) {
        try { process.platform === 'win32' ? child.kill() : process.kill(-child.pid, 'SIGTERM'); } catch { /* already exited */ }
      }
    },
  };
}
