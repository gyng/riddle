import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import { runtimeKey } from "./runtime-key.mjs";

test("real compiler separates unit tests from runtime and included inputs", () => {
  const root = mkdtempSync(join(tmpdir(), "riddle-runtime-key-"));
  const source = join(root, "crates/riddle-core/src");
  mkdirSync(source, { recursive: true });
  mkdirSync(join(root, "crates/riddle-core/examples"));
  const write = (file, text) => writeFileSync(join(root, file), text);
  const cargo = (args, env = {}) => spawnSync("cargo", args, { cwd: root, env: { ...process.env, ...env }, encoding: "utf8", maxBuffer: 4 * 1024 * 1024 });
  const rustc = spawnSync("rustc", ["-vV"], { encoding: "utf8" });
  assert.equal(rustc.status, 0);
  const build = (env) => {
    const result = cargo(["build", "-q", "--profile", "fast", "--example", "gate", "--message-format=json"], env);
    assert.equal(result.status, 0, result.stderr);
    const artifact = result.stdout.split("\n").filter(Boolean).map((s) => JSON.parse(s))
      .find((m) => m.reason === "compiler-artifact" && m.target.name === "riddle_core");
    return { artifact, key: runtimeKey(artifact, root, rustc.stdout) };
  };
  try {
    write("Cargo.toml", '[workspace]\nmembers=["crates/riddle-core"]\nresolver="2"\n[profile.fast]\ninherits="release"\ndebug=1\n');
    write("crates/riddle-core/Cargo.toml", '[package]\nname="riddle-core"\nversion="0.1.0"\nedition="2021"\n');
    const lib = 'pub fn value() -> u32 { 3 }\npub fn marker() -> &\'static str { include_str!("runtime.txt") }\n#[cfg(test)] mod tests;\n';
    write("crates/riddle-core/src/lib.rs", lib);
    write("crates/riddle-core/src/runtime.txt", "first runtime input");
    write("crates/riddle-core/src/tests.rs", '#[test] fn independent() { assert_eq!(super::value(), 3); }\n');
    write("crates/riddle-core/examples/gate.rs", 'fn main() { println!("{}", riddle_core::value()); }\n');
    const initial = build();
    write("crates/riddle-core/src/tests.rs", '#[test] fn independent() { panic!("fresh tests run"); }\n');
    assert.equal(build().key, initial.key, "unit-test edits must preserve simulation identity");
    const tests = cargo(["test", "-q", "--profile", "fast", "--lib"]);
    assert.notEqual(tests.status, 0, "tests still run and fail independently");
    assert.match(tests.stdout + tests.stderr, /fresh tests run/);
    write("crates/riddle-core/examples/gate.rs", 'fn main() { println!("harness changed: {}", riddle_core::value()); }\n');
    assert.equal(build().key, initial.key, "harness keys belong to each job, not every core simulation");
    write("crates/riddle-core/src/lib.rs", lib.replace("{ 3 }", "{ 4 }"));
    assert.notEqual(build().key, initial.key, "runtime edits must invalidate simulations");
    write("crates/riddle-core/src/lib.rs", lib);
    assert.equal(build().key, initial.key);
    write("crates/riddle-core/src/runtime.txt", "second runtime input");
    const included = build();
    assert.notEqual(included.key, initial.key, "transitive include inputs must invalidate simulations");
    const flagged = build({ RUSTFLAGS: "-C opt-level=2" });
    assert.notEqual(flagged.key, included.key, "compiler flags must invalidate simulations");
    assert.notEqual(runtimeKey(flagged.artifact, root, rustc.stdout + "different compiler"), flagged.key);
    write("crates/riddle-core/Cargo.toml", '[package]\nname="riddle-core"\nversion="0.1.1"\nedition="2021"\n');
    assert.notEqual(build().key, included.key, "manifest/lock inputs must invalidate simulations");
    assert.throws(() => runtimeKey(undefined, root, rustc.stdout), /production core/);
    assert.throws(() => runtimeKey({ ...initial.artifact, profile: { test: true } }, root, rustc.stdout), /production core/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});
