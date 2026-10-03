//! Saved-camp benchmark for local performance work. Loading is outside each timed sample.
//! cargo run -q --profile fast -p riddle-core --example perf_local -- SAVE [samples] [hours]
use riddle_core::Game;
use std::{alloc::{GlobalAlloc, Layout, System}, hint::black_box, sync::atomic::{AtomicBool, AtomicU64, Ordering}, time::Instant};

struct CountingAllocator;
static TRACK: AtomicBool = AtomicBool::new(false);
static ALLOCS: AtomicU64 = AtomicU64::new(0);
static BYTES: AtomicU64 = AtomicU64::new(0);

fn count(size: usize) {
    if TRACK.load(Ordering::Relaxed) {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        BYTES.fetch_add(size as u64, Ordering::Relaxed);
    }
}

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        count(layout.size());
        unsafe { System.alloc(layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        count(layout.size());
        unsafe { System.alloc_zeroed(layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        count(size);
        unsafe { System.realloc(ptr, layout, size) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325_u64, |h, b| (h ^ u64::from(*b)).wrapping_mul(0x100000001b3))
}

fn clones<T: Clone>(label: &str, value: &T) {
    let n = 20_000;
    let t = Instant::now();
    for _ in 0..n {
        black_box(black_box(value).clone());
    }
    println!("clone {label}: {:.3} us", t.elapsed().as_secs_f64() * 1e6 / f64::from(n));
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let path = args.get(1).expect("usage: perf_local SAVE [samples] [hours]");
    let samples: usize = args.get(2).map(|v| v.parse().expect("samples")).unwrap_or(7);
    let hours: u64 = args.get(3).map(|v| v.parse().expect("hours")).unwrap_or(8);
    let allocations = args.iter().any(|arg| arg == "--alloc");
    assert!(samples > 0);
    let input = std::fs::read_to_string(path).expect("read save");
    println!("input {:016x}; samples {samples}; hours {hours}; allocation counters {allocations}", hash(input.as_bytes()));
    riddle_core::forecast::set_parallel_sims(false);
    let mut times = Vec::new();
    let mut reference = None;
    for sample in 0..samples {
        let mut g = Game::load(&input).expect("load camp");
        ALLOCS.store(0, Ordering::Relaxed);
        BYTES.store(0, Ordering::Relaxed);
        TRACK.store(allocations, Ordering::Relaxed);
        let t = Instant::now();
        let report = riddle_core::offline::run_offline_quick(&mut g, hours * 3600);
        let elapsed = t.elapsed().as_secs_f64();
        TRACK.store(false, Ordering::Relaxed);
        if allocations {
            println!("allocations {}; requested bytes {}", ALLOCS.load(Ordering::Relaxed), BYTES.load(Ordering::Relaxed));
        }
        let output = hash(format!("{}{}", serde_json::to_string(&report).unwrap(), g.save()).as_bytes());
        assert_eq!(*reference.get_or_insert(output), output, "sample outputs differ");
        println!("sample {sample}: {elapsed:.6} s; output {output:016x}; ticks {}", g.batch.turns);
        times.push(elapsed);
        if sample == 0 {
            // The deepest retained run exercises the fields copied into the history ring.
            let run = g.history.iter().map(|(run, _)| run).chain(g.run.iter())
                .chain(g.deaths.values().filter_map(|d| d.t10.as_ref()))
                .max_by_key(|r| r.depth).expect("a retained run");
            println!("snapshot depth {}; monsters {}", run.depth, run.monsters.len());
            clones("run", run);
            clones("monsters", &run.monsters);
            clones("map", &run.floor.map);
            clones("meters", &run.meters);
        }
    }
    times.sort_by(f64::total_cmp);
    println!("median {:.6} s; min {:.6} s; max {:.6} s", times[samples / 2], times[0], times[samples - 1]);
}
