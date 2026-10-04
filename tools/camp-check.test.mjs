import { test } from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { parseArgs, checkReference, checkOutput } from "./camp-check.mjs";

test("saved-camp checks reject incomplete or ambiguous requests", () => {
  assert.deepEqual(parseArgs(["save.json"]).modes, ["offline"]);
  assert.deepEqual(parseArgs(["save.json", "--mode", "wall,offline,wall"]).modes, ["wall", "offline"]);
  for (const args of [[], ["save", "--unknown"], ["save", "--record"], ["save", "--mode", "bogus"],
    ["save", "--repeat", "0"], ["save", "--repeat", "1.5"], ["save", "--record", "a", "--compare", "b"]]) assert.throws(() => parseArgs(args));
});

test("references bind exact inputs and workloads and require uncorrupted full bytes", () => {
  const inputs = [{ sha256: "camp-hash" }], modes = ["offline"];
  const bytes = Buffer.from('{"every":"field matters"}'), hash = createHash("sha256").update(bytes).digest("hex");
  const reference = { version: 1, binary: "baseline-binary", inputs, modes, cases: [{ file: "0-offline.json", sha256: hash }] };
  checkReference(reference, inputs, modes);
  checkOutput(bytes, bytes, hash);
  assert.throws(() => checkOutput(Buffer.from('{"every":"different"}'), bytes, hash), /output changed/);
  assert.throws(() => checkOutput(bytes, bytes, "corrupt"), /corrupt/);
  assert.throws(() => checkReference(reference, [{ sha256: "changed-camp" }], modes), /inputs differ/);
  assert.throws(() => checkReference(reference, inputs, ["wall"]), /workloads differ/);
  assert.throws(() => checkReference({ ...reference, cases: [] }, inputs, modes), /incomplete/);
  assert.throws(() => checkReference({ ...reference, cases: [{ file: "../outside", sha256: hash }] }, inputs, modes), /invalid reference case/);
  assert.throws(() => checkReference({ version: 1 }, inputs, modes), /invalid reference/);
});
