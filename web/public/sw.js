// Versioned precache service worker. The version and precache list are injected at build time
// (web/vite.config.ts) from the emitted bundle, so every build gets its own cache and old ones are dropped.
// Registered only in production builds (src/app.ts). Verify frontend changes on a fresh port.
const VERSION = "__VERSION__";
const PRECACHE = __PRECACHE__;
const CACHE_PREFIX = `riddle-${self.registration.scope}-`;
const CACHE = `${CACHE_PREFIX}${VERSION}`;
const INDEX = new URL("index.html", self.registration.scope).href;

self.addEventListener("install", (e) => {
  e.waitUntil(caches.open(CACHE).then((c) => c.addAll(PRECACHE)).then(() => self.skipWaiting()));
});
self.addEventListener("activate", (e) => {
  e.waitUntil(caches.keys().then((keys) => Promise.all(keys.filter((k) => k.startsWith(CACHE_PREFIX) && k !== CACHE).map((k) => caches.delete(k)))).then(() => self.clients.claim()));
});
self.addEventListener("fetch", (e) => {
  const req = e.request;
  if (req.method !== "GET" || !req.url.startsWith(self.registration.scope)) return;
  if (req.mode === "navigate") {
    // network first for the shell so a new deploy is picked up; cached index as offline fallback
    e.respondWith(fetch(req).then((r) => { const copy = r.clone(); caches.open(CACHE).then((c) => c.put(INDEX, copy)); return r; }).catch(() => caches.match(INDEX)));
    return;
  }
  // These are public same-origin assets. Precache and browser requests can differ in Origin headers.
  e.respondWith(caches.match(req, { ignoreVary: true }).then((hit) => hit ?? fetch(req).then((r) => { if (r.ok) { const copy = r.clone(); caches.open(CACHE).then((c) => c.put(req, copy)); } return r; })));
});
