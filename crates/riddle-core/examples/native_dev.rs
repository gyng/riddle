//! Ordered JSON-line transport for the real engine during local development.
#[path = "native_dev/generated.rs"]
mod generated;
use std::io::{BufRead, Write};
use std::sync::atomic::AtomicUsize;
static WIDTH: AtomicUsize = AtomicUsize::new(8);
fn main() {
    riddle_core::chronicle::use_shipping_words();
    riddle_core::balance::configure_from_env().expect("valid immutable balance profile");
    let width: usize = std::env::var("RIDDLE_NATIVE_THREADS").unwrap_or_else(|_| "8".into()).parse().expect("native thread count");
    assert!(width > 0 && width <= 256);
    WIDTH.store(width, std::sync::atomic::Ordering::Relaxed);
    let mut game = generated::new_game();
    let mut out = std::io::stdout().lock();
    for line in std::io::stdin().lock().lines() {
        let line = line.expect("native request read");
        let v: serde_json::Value = serde_json::from_str(&line).expect("native request JSON");
        let id = v["id"].as_u64().expect("request id");
        let method = v["method"].as_str().expect("request method");
        let args = v["args"].as_array().expect("request arguments");
        let started = std::time::Instant::now();
        let result = riddle_core::forecast::with_sim_width(&WIDTH, || generated::dispatch(&mut game, method, args));
        let response = match result {
            Ok(r) => serde_json::json!({ "id": id, "ok": true, "r": r, "ms": started.elapsed().as_secs_f64() * 1000.0 }),
            Err(e) => serde_json::json!({ "id": id, "ok": false, "e": e }),
        };
        writeln!(out, "{response}").expect("native reply write");
        out.flush().expect("native reply flush");
    }
}
