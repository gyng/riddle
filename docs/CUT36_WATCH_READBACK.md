# Cut 36 — watch startup without skipped beats

Local cold-watch GPU profile (400×800, DPR2, fake seed7): getImageData
samples include658.5ms in deriveNormals,232.7ms in tile aliases,138.4ms in
sprite downsampling and37.8ms in wall-top generation. Atlas canvases explicitly
request GPU storage while repeatedly reading pixels; deriveNormals cannot
change that choice after context creation. This is a renderer cost, separate
from native simulation and build timings.

- A cage found under the floor card applies its fight frame immediately, before
  restoring playback speed. Existing beat/override/world-wait checks stay intact.
- Test canvas storage suited to pixel reads. Preserve exact full atlas and normal
  map RGBA hashes, including aliases, wall variants and hero/foe fallbacks.
- Paired cold GPU profiles: target >=50% reduction in sampled getImageData time;
  record full startup costs and remaining driver/shader work separately. No
  general frame-rate or sim-performance claim from startup samples.
- Keep card duration <=1200ms, the existing live-watch interaction bars and zero
  text-collision gate. Do not change gameplay, art, cadence or numeric limits to
  hide startup stalls. Show real-WASM watch screenshots400/1440.
- Temporary internal watch probes are investigation only; remove before commit.

The95fun goal remains unverified. This contract covers a concrete playback and
startup defect; it does not certify the full client suite or core balance.

## Profile evidence (acceptance checks pending)

Evidence: scratchpad/watch-timeline-20261007. Three exploratory unchanged
profiles report getImageData1083.009/1107.232/1112.019ms. The two matched repeats
have median1109.6255ms; two candidate profiles147.607/154.254ms,
median150.9305ms,86.40% lower. Each is a fresh headed GPU browser,400×800 DPR2,
fake seed7,10s after Send. No core compilation or other browser run overlapped.
Sampled non-idle work in matched baseline3554.197/3665.922ms versus candidate
2650.870/2608.204ms. These are sampled call costs, not elapsed simulation speed.

One timing pair's first fight2155.8→2059.8ms; insufficient for a general startup
latency claim. Remaining context creation is408–436ms and shader info calls
374–556ms. Candidate retains139ms of procedural fallback canvas readback; this
is identified separately, not claimed fixed.

Full1024×1024 environment atlas,512×512 sprite atlas and512×512 normal output
RGBA SHA-256 hashes match exactly. Both states allocate549environment and125
sprite slots. Baseline-full/candidate-full-pixels.json contain complete hashes.
The initial baseline-pixels.json mistakenly hashed a300×150 output; superseded
by the full-size captures and not used for acceptance.

Internal immutable frame trace frozen.json shows an unshown cage atturn30,
picture advancing30→70→110 at16× across a614ms rendering stall; the beat is then
marked skipped. Applying its frame during vaultFrom avoids waiting for the next
pump. No card cap/timer or gameplay changes are part of this implementation.
Temporary probe removed. The existing1200ms card gate must pass on this source.

## Corner encounter requirement

Overlap avoidance must use the hero's grounded stack position, exactly as the
sprite draw does. A diagonal move around a wall corner can reflect the fan;
using its pre-reflection position leaves a full-size adjacent boss over the
hero. Share one grounding helper for both calculations. Preserve HERO_COVER0.3,
BOSS_COVER0.1, sprite sizes, world state and existing floor visibility rules.

Paired identical corner fixture on preserved5384/current5385: boss coverage
84.375%→0%; other sprites<=30%. New hero-grounding gate uses the same mid-move
encounter at400/1440, waits for drawn frames within5s and retains original10%/
30% bars plus1% rounding. It fails the baseline400 at84.375%, passes both widths
on corrected source. Evidence wall-overlap.json and
/tmp/riddle-hero-grounding-{baseline,final}.log. Ordinary run probes are retained
separately; their paused positions differ and are not a controlled comparison.

## Playback acceptance and inspection race

Atlas/cage candidate fresh focused run: clarity:card14checks PASS65.3s,
cut28w6checks PASS73.9s and watch-console5widths PASS24.8s. Fights then exposed
the corner overlap, now independently reproduced and fixed above.

Corrected renderer run: cut28w6checks PASS73.8s and original combat coverage
checks pass; remaining fights failure was ending clock164→164. Trace skip.json
compares actual viewer.tick and watch data-tick. Engine ending flag can already
be1 while the picture is still219/220; Skip advances it to ending400+ normally.
The old wait returned on that early engine flag before the asynchronous seek.
Wait for ending AND picture clock advancement within the original20s, then keep
both original end/jump assertions. No production timer/skip changes or weaker
clock assertion. Final fights/clarity repeat is recorded separately on completion.

Real earned Gunner WASM corrected-renderer400/1440 Speed Fast→Town→reload PASS,
no overflow/HUD overlap/pageerrors. Final screenshots shown from
scratchpad/watch-grounded-real-20261007; /tmp/riddle-watch-grounded-real.log.
TS/copy1730zero/web build/diff PASS. Built WASM remains5482237bytes. Core inputs
unchanged; native balance and broader exhaustive audits are not rerun or claimed.

## Final current-source result — partial acceptance

/tmp/riddle-watch-cut36-final.log terminal1: fights48checks PASS200.1s; clarity
card12/14, remaining spans1218/1237ms exceed1200ms. Cage and other assertions
pass. Earlier14check candidate pass is not repeatable card timing acceptance.
Retain the readback/cage/grounding improvements, leave the timing requirement
open and profile residual driver/shader stalls. No cap change, no full suite
certification, no95score claim. Final hero-grounding2widths PASS8.8s; paired old
renderer fails84.375%. Real UI screenshots and TS/copy/build pass above.
