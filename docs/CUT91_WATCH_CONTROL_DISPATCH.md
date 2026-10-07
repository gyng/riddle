# Cut91 — dispatch visible watch controls without protocol races

The full client and isolated clarity checks expose the same helper race:
resumed ending replay tick377/420 still accepts Skip, but several browser round
trips opening Speed let it reach endingFrom400 and disable the option before
dispatch. Ended runs must continue refusing Skip; do not change game controls.

Open Speed and press the visible enabled option in one page operation. Use only
DOM controls, preserving caller modal refusal and exact parent menu closure
when Skip opens a modeless loot choice. Next gem uses the same single-operation
snapshot of the visible enabled node. No engine shortcuts or locator retries.

Gates: unchanged clarity frontier/resume/ending checks pass; Cut13 ended-run
mode/Skip refusal preserved; RunsUI and screen walk pass. One round trip per
watch interaction instead of multiple discovery/handle/disposal operations.

Validation: Cut13 77, clarity43, screens58 and RunsUI all PASS195.5s.
