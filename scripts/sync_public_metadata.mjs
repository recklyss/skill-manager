#!/usr/bin/env node
import { copyFileSync, existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const source = join(root, "LICENSE");
const target = join(root, "packaging/npm/LICENSE");
const check = process.argv.includes("--check");
const write = process.argv.includes("--write");

if (check === write) {
  console.error("choose exactly one of --check or --write");
  process.exit(2);
}

const expected = readFileSync(source, "utf8");
const current = existsSync(target) ? readFileSync(target, "utf8") : "";
if (expected === current) process.exit(0);
if (!write) {
  console.error(`${target}: expected contents synced from ${source}`);
  process.exit(1);
}
copyFileSync(source, target);
process.exit(0);
