// Native simulation jobs depend on the compiled core, independently of its unit tests.
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

export function runtimeKey(artifact, root, rustcVersion) {
  if (!artifact || artifact.target.name !== "riddle_core" || !artifact.target.kind.includes("lib") || artifact.profile.test
      || resolve(artifact.manifest_path) !== resolve(root, "crates/riddle-core/Cargo.toml")) {
    throw new Error("runtime key requires the compiled production core library");
  }
  const libraries = artifact.filenames.filter((f) => f.endsWith(".rlib"));
  if (libraries.length !== 1 || !rustcVersion.trim()) throw new Error("missing core library or compiler identity");
  const hash = createHash("sha1");
  hash.update("native-core-rlib-v1\0").update(readFileSync(libraries[0])).update("\0");
  hash.update(JSON.stringify({ profile: artifact.profile, features: artifact.features })).update("\0");
  for (const file of ["crates/riddle-core/Cargo.toml", "Cargo.toml", "Cargo.lock"]) {
    hash.update(file).update("\0").update(readFileSync(resolve(root, file))).update("\0");
  }
  hash.update(rustcVersion);
  return hash.digest("hex").slice(0, 16);
}
