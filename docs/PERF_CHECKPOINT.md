# Checkpoint follow-through — 2026-10-05

Measure checkpoint encoding, bytes and recovery independently of simulation.
Shared configurations currently serialize the identical Play separately for
each destination at a check-in, but routine gates disable sharing; prioritize
the common codec instead. Traverse the existing JSON tree in place and serialize
the envelope by reference, retaining identical atomic files, integrity checks
and complete internal state. Preserve float bits, seeds,
gate bounds, runtime cache identity and result ordering. Do not change shipping
game behavior.

Use real bot states and alternating encode/write measurements; compare complete
file bytes and restored final outputs/saves. Require lower serialization cost
on real states without regression for one destination. Report its fraction
of a real routine job, avoiding a whole-suite speedup claim from a microbench.
Run checkpoint/harness tests, tooling checks and canonical routine verification
for a retained harness change. Evidence: scratchpad/checkpoint-perf-20261005.

## Measurements

Fresh seed15 fourteen-day IDLE plus its automatically included IDLE30 reference,
one thread: original phase diagnostic36s (checkpoint8CPU s, offline28) and29s
(checkpoint6, offline23). CPU phase totals are rounded to whole seconds; this is
a diagnostic two-case run, not routine acceptance. A captured real late state
encodes to8,812,092bytes; this is one file size, not total writes or peak memory.

Seven quiet alternating real-state encoding pairs: original0.422908s,
final0.291354s median (31.11% lower). All14 complete encoded files are identical
to the original reference byte for byte, including checksums and float markers.
In-place traversal alone measured3.73%; most gain comes from avoiding the second
whole-tree clone in json!'s envelope conversion. Shipping simulation is untouched.

Initial whole-run script compared CLI text including elapsed time and correctly
stopped on that difference; it is excluded from performance evidence. Corrected
comparisons use complete serialized bot-result files, preserving all fields.

Three quiet alternating fresh fourteen-day pairs (IDLE and IDLE30, seed15,
one thread, real checkpoint writes): original66.5639s→final63.9735s median
(3.89% lower). Every complete serialized bot result is byte-identical. Individual
pairs vary from0.81% to7.14%; this small sample is not a universal suite speedup.
The consistent31% encoder reduction justifies the simple codec change; ticks
remain dominant. Last-pair phase totals: final checkpoints7+5CPU s versus
original8+6; whole-second CPU phase rounding limits precision.

Diagnostic command (freshly built example, no gate acceptance):

```
cargo run -q --profile fast -p riddle-core --example dayplayer -- --checkpoint-bench CHECKPOINT.json --repeats 7 --output ENCODED.json
```

It validates the input envelope/checksum, decodes the complete internal state
without player-save migrations, measures read/decode separately, then repeats
encoding without writes. --output writes the final bytes outside timed encoding.

Frozen diagnostic:8,812,092bytes, exact original bytes; read/decode0.2207s,
seven encoding repetitions median0.2917s. This read time is one warm local
sample, not a recovery speedup comparison.

Acceptance PASS:14 harness tests (100.29s), including complete restored bot
outputs and raw saves, corruption/wrong-key rejection and nested float-bit/
escaped-marker checks. Canonical tools/verify.sh --full PASS447s:524 engine
tests/1ignored,10 tooling tests, tsc/copy/clippy, shipping/web; genuinely new
metrics84.2s and18-case dayplayer234.8s, QA genuine existing cache hit. Selected
regression bars pass; exhaustive historical/system-removal audit was not run.
Compilation contributes to total447s; do not present it as fresh gate-table CPU.

Fresh public GPU screenshot/check PASS: transparent background,0px border,
no header,54px log height. This harness-only change does not alter public UI or
shipping simulation. Screenshot: private public-checkpoint.png.

Follow-through core rebuild measurement: quiet equivalent vision helper body
edit28.125s edit-to-ready; exact original source restoration28.859s. Original
stable native executable SHA restored exactly; no production source change.
This one pair does not replace the earlier2.24s adapter-only measurement.

Cargo compiler breakdown afterward: core27.04s (frontend2.84s, codegen24.20s),
adapter0.50s; original source/executable restored again. The watcher requests a
second build on the edit, so use this for phase attribution, not another quiet
watcher timing. Next investigate isolated codegen/debug settings with measured
runtime tradeoffs, preserving the default shared core build.
