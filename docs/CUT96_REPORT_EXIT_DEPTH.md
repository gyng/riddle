# Cut96 — report logs show the run's actual depth

Independent QA-a's earned second bloodline banked atD6 ($447 plus29salvage),
but its report log button saidD3. The ledger range can contain other heroes'
exits; the caption takes its first depth without checking the bloodline.

Prefer the run's Rust-owned positive `ExitLine.reached` field. For old saves
without it, match only the same bloodline's actual last exit in the ledger
range; count newer duplicate exits only within that bloodline. Preserve text
fallback for records without a matching ledger. Never use the selected hero's
depth to describe another hero's report.

Gates: conflicting D3/D6 two-bloodline fixture yields D6 from wire and from
legacy ledger; newer same-value exits belonging to another bloodline do not
skip the owned exit. No-ledger old text fallback retained. Rendered log caption
agrees with report D6, engine save unchanged. Existing report/gold checks pass.
