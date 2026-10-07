import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";

const PKG_PATH = "package.json";
const PKG_VERSION_PATTERN = /"version"\s*:\s*"([^"]+)"/;

function isValidSemver(version) {
  const match = version.match(/^(\d+)\.(\d+)\.(\d+)(?:-(.+))?$/);
  if (!match) {
    return false;
  }
  const [, major, minor, patch, prerelease] = match;
  if ([major, minor, patch].some((part) => part !== String(Number(part)))) {
    return false;
  }
  if (prerelease === undefined) {
    return true;
  }
  return prerelease
    .split(".")
    .every(
      (identifier) =>
        identifier !== "" &&
        (/^\d+$/.test(identifier)
          ? identifier === String(Number(identifier))
          : /^[0-9A-Za-z-]+$/.test(identifier)),
    );
}

const MANIFESTS = [
  { path: "src-tauri/tauri.conf.json", pattern: /"version"\s*:\s*"([^"]+)"/ },
  { path: "src-tauri/Cargo.toml", pattern: /^version\s*=\s*"([^"]+)"/m },
  { path: "crates/swift-bill-core/Cargo.toml", pattern: /^version\s*=\s*"([^"]+)"/m },
  { path: "crates/swift-bill-db/Cargo.toml", pattern: /^version\s*=\s*"([^"]+)"/m },
  { path: "crates/swift-bill-excel/Cargo.toml", pattern: /^version\s*=\s*"([^"]+)"/m },
  { path: "crates/swift-bill-pdf/Cargo.toml", pattern: /^version\s*=\s*"([^"]+)"/m },
];

function readVersion(path, pattern) {
  const match = readFileSync(path, "utf8").match(pattern);
  if (!match) {
    throw new Error(`Could not find a version in ${path}`);
  }
  return match[1];
}

function replaceVersion(path, pattern, version) {
  const text = readFileSync(path, "utf8");
  const updated = text.replace(pattern, (match) =>
    match.replace(/"[^"]+"$/, `"${version}"`),
  );
  if (updated !== text) {
    writeFileSync(path, updated);
    return true;
  }
  return false;
}

function check() {
  const expected = readVersion(PKG_PATH, PKG_VERSION_PATTERN);
  let failed = false;
  console.log(`${PKG_PATH} (single source): ${expected}`);
  for (const manifest of MANIFESTS) {
    const actual = readVersion(manifest.path, manifest.pattern);
    const ok = actual === expected;
    failed ||= !ok;
    console.log(`${ok ? "ok  " : "FAIL"} ${manifest.path}: ${actual}`);
  }
  if (failed) {
    console.error("\nVersion drift detected. Run: bun run version:sync");
    process.exit(1);
  }
  console.log("\nAll manifests match.");
}

function sync() {
  const version = readVersion(PKG_PATH, PKG_VERSION_PATTERN);
  for (const manifest of MANIFESTS) {
    if (replaceVersion(manifest.path, manifest.pattern, version)) {
      console.log(`updated   ${manifest.path} -> ${version}`);
    } else {
      console.log(`unchanged ${manifest.path}`);
    }
  }
}

function bump(version) {
  if (!version || !isValidSemver(version)) {
    console.error("Usage: bun run version:bump <x.y.z>");
    process.exit(1);
  }
  const current = readVersion(PKG_PATH, PKG_VERSION_PATTERN);
  if (current === version) {
    console.log(`${PKG_PATH} is already at ${version}.`);
  } else if (replaceVersion(PKG_PATH, PKG_VERSION_PATTERN, version)) {
    console.log(`updated   ${PKG_PATH} -> ${version}`);
  }
  sync();
  try {
    execFileSync("cargo", ["update", "--workspace"], { stdio: "inherit" });
  } catch {
    console.error(
      "cargo update --workspace failed. The manifests were updated, but Cargo.lock may be stale. Install Rust or refresh the lock manually, then commit.",
    );
    process.exit(1);
  }
  console.log("Cargo.lock refreshed.");
  console.log(
    [
      "",
      "Next steps:",
      "  git add package.json src-tauri crates Cargo.lock",
      `  git commit -m "chore: bump version to ${version}"`,
      `  git tag v${version}`,
      "  git push && git push --tags",
    ].join("\n"),
  );
}

const [command = "check", argument] = process.argv.slice(2);

switch (command) {
  case "check":
    check();
    break;
  case "sync":
    sync();
    break;
  case "bump":
    bump(argument);
    break;
  default:
    console.error(`Unknown command: ${command}. Use check, sync, or bump <x.y.z>.`);
    process.exit(1);
}
