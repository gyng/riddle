//! A gate job's result kept across runs (docs/ITERATION_SPEED.md, round 4). A job is a pure function of
//! the core's code, the harness code it runs and its parameters; `tools/gates.mjs` sets `RIDDLE_SRC_KEY`
//! to the hash of the core's sources, the manifests, the lockfile and the toolchain, and a job kept under
//! that key, the source texts it names and its parameters is read back instead of played — so an edit to a
//! table row, or to a harness file a job does not run, reprints from the jobs kept. Without the variable
//! (a tool run by hand) nothing is kept or read.
#![allow(dead_code)]
use serde::de::DeserializeOwned;
use serde::Serialize;

fn fnv(bytes: &[u8], mut h: u64) -> u64 {
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// The environment the core reads that moves a result (none of the harness's debug switches do).
const ENV: [&str; 1] = ["RIDDLE_SKIP_TWIST"];

/// The file a job's result is kept in, when keeping is on: `target/gates/<dir>/<key>.json`.
pub fn path(dir: &str, srcs: &[&str], params: &str) -> Option<std::path::PathBuf> {
    let k = std::env::var("RIDDLE_SRC_KEY").ok()?;
    let mut h = fnv(k.as_bytes(), 0xcbf2_9ce4_8422_2325);
    for s in srcs {
        h = fnv(s.as_bytes(), fnv(b"\0src", h));
    }
    for e in ENV {
        h = fnv(std::env::var(e).unwrap_or_default().as_bytes(), fnv(e.as_bytes(), h));
    }
    h = fnv(params.as_bytes(), fnv(b"\0params", h));
    let d = std::path::PathBuf::from("target/gates").join(dir);
    let _ = std::fs::create_dir_all(&d);
    Some(d.join(format!("{h:016x}.json")))
}

/// `f()`, or its result kept from an earlier run under the same key.
pub fn cached<T: Serialize + DeserializeOwned>(dir: &str, srcs: &[&str], params: &str, f: impl FnOnce() -> T) -> T {
    let p = path(dir, srcs, params);
    // (`RIDDLE_CACHE_FRESH`: `gates.mjs --fresh` — play every job, keep the results)
    let read = std::env::var_os("RIDDLE_CACHE_FRESH").is_none();
    if let Some(v) = p.as_ref().filter(|_| read).and_then(|p| std::fs::read_to_string(p).ok()).and_then(|t| serde_json::from_str::<T>(&t).ok()) {
        return v;
    }
    let v = f();
    if let Some(p) = p {
        // (written whole, then renamed: a run stopped mid-write leaves no half a result)
        let tmp = p.with_extension(format!("tmp{}", std::process::id()));
        if std::fs::write(&tmp, serde_json::to_string(&v).unwrap_or_default()).is_ok() {
            let _ = std::fs::rename(&tmp, &p);
        }
    }
    v
}
