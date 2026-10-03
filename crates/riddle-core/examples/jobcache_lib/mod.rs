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
    let fresh = std::env::var_os("RIDDLE_CACHE_FRESH").is_some();
    if let Some(v) = p.as_ref().filter(|_| !fresh).and_then(|p| read(p)) {
        return v;
    }
    let v = f();
    if let Some(p) = p {
        keep(&p, &v);
    }
    v
}

/// A float kept as its bits (serde_json without `float_roundtrip` reads some decimal floats an ulp off).
const F64: &str = "\u{1}f64:";
const ESC: &str = "\u{1}s:";

fn floats(v: serde_json::Value, encode: bool) -> serde_json::Value {
    use serde_json::Value;
    match v {
        Value::Number(n) if encode && n.is_f64() => Value::String(format!("{F64}{:016x}", n.as_f64().unwrap_or(0.0).to_bits())),
        // (a string of the result's own that begins like a mark is kept escaped)
        Value::String(t) if encode && t.starts_with('\u{1}') => Value::String(format!("{ESC}{t}")),
        Value::String(t) if !encode && t.starts_with(ESC) => Value::String(t[ESC.len()..].to_string()),
        Value::String(t) if !encode && t.starts_with(F64) => u64::from_str_radix(&t[F64.len()..], 16).ok().and_then(|b| serde_json::Number::from_f64(f64::from_bits(b))).map_or(Value::String(t), Value::Number),
        Value::Array(a) => Value::Array(a.into_iter().map(|x| floats(x, encode)).collect()),
        Value::Object(o) => Value::Object(o.into_iter().map(|(k, x)| (k, floats(x, encode))).collect()),
        x => x,
    }
}

/// A result as it is kept: `{"f64bits": 1, "v": …}` with every float as its bits — read back exactly.
pub fn text<T: Serialize>(v: &T) -> Option<String> {
    let v = floats(serde_json::to_value(v).ok()?, true);
    serde_json::to_string(&serde_json::json!({ "f64bits": 1, "v": v })).ok()
}

/// A kept result. (A file in the plain form — written before the floats were kept as bits — is read only
/// when it prints back to its own text, i.e. when it was read exactly.)
pub fn read<T: Serialize + DeserializeOwned>(p: &std::path::Path) -> Option<T> {
    let t = std::fs::read_to_string(p).ok()?;
    let v: serde_json::Value = serde_json::from_str(&t).ok()?;
    match v {
        serde_json::Value::Object(mut o) if o.get("f64bits").is_some() => serde_json::from_value(floats(o.remove("v")?, false)).ok(),
        _ => {
            let r: T = serde_json::from_str(&t).ok()?;
            (serde_json::to_string(&r).ok()? == t).then_some(r)
        }
    }
}

/// Keep `v` at `p` (written whole, then renamed: a run stopped mid-write leaves no half a result).
pub fn keep<T: Serialize>(p: &std::path::Path, v: &T) {
    let Some(t) = text(v) else { return };
    let tmp = p.with_extension(format!("tmp{}", std::process::id()));
    if std::fs::write(&tmp, t).is_ok() {
        let _ = std::fs::rename(&tmp, p);
    }
}
