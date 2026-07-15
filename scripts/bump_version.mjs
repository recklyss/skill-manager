#!/usr/bin/env node
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const versionPath = join(root, "VERSION");

function parseVersion(value) {
  const match = String(value).trim().match(/^(\d+)\.(\d+)\.(\d+)$/);
  if (!match) {
    throw new Error(`invalid semver: ${value}`);
  }
  return {
    major: Number(match[1]),
    minor: Number(match[2]),
    patch: Number(match[3]),
  };
}

function formatVersion({ major, minor, patch }) {
  return `${major}.${minor}.${patch}`;
}

function readVersion() {
  return readFileSync(versionPath, "utf8").trim();
}

function writeVersion(next) {
  writeFileSync(versionPath, `${next}\n`, "utf8");
}

function bumpPatch(version) {
  const parts = parseVersion(version);
  parts.patch += 1;
  return formatVersion(parts);
}

const fromArg = process.argv.find((arg) => arg.startsWith("--from="));
const baseVersion = fromArg ? fromArg.slice("--from=".length) : readVersion();
const firstRelease = process.argv.includes("--first-release");
const nextVersion = firstRelease ? readVersion() : bumpPatch(baseVersion);

writeVersion(nextVersion);

const sync = spawnSync("node", ["scripts/sync_version.mjs", "--write"], {
  cwd: root,
  stdio: "inherit",
});
if (sync.status !== 0) {
  process.exit(sync.status ?? 1);
}

const lockfile = spawnSync("npm", ["install", "--package-lock-only"], {
  cwd: root,
  stdio: "inherit",
});
if (lockfile.status !== 0) {
  process.exit(lockfile.status ?? 1);
}

console.log(`bumped version to ${nextVersion}`);
