#!/usr/bin/env node
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const version = readFileSync(join(root, "VERSION"), "utf8").trim();
const check = process.argv.includes("--check");
const write = process.argv.includes("--write");

if (check === write) {
  console.error("choose exactly one of --check or --write");
  process.exit(2);
}

function syncJson(path, label) {
  const full = join(root, path);
  const payload = JSON.parse(readFileSync(full, "utf8"));
  if (payload.version === version) return true;
  if (!write) {
    console.error(`${label}: expected ${version}, found ${payload.version}`);
    return false;
  }
  payload.version = version;
  writeFileSync(full, `${JSON.stringify(payload, null, 2)}\n`, "utf8");
  return true;
}

function syncCargo() {
  const path = join(root, "src-tauri/Cargo.toml");
  const text = readFileSync(path, "utf8");
  const next = text.replace(/^version = ".*"$/m, `version = "${version}"`);
  if (text === next) return true;
  if (!write) {
    console.error(`Cargo.toml: expected version ${version}`);
    return false;
  }
  writeFileSync(path, next, "utf8");
  return true;
}

const ok =
  syncJson("package.json", "package.json") &
  syncJson("packaging/npm/package.json", "packaging/npm/package.json") &
  syncCargo();

process.exit(ok ? 0 : 1);
