#!/usr/bin/env node
// The ajv arm of .github/scripts/ledger-envelope-engines.py.
//
// Reads {"schema": <the committed ledger envelope schema>, "cases": [{"name",
// "instance"}, ...]} as JSON on stdin and prints [[name, formatsOn,
// formatsOff], ...] on stdout. JSON.parse reads the input, so an integer past
// 2^53 rounds to the nearest double exactly as it does for a JS writer.
//
// The formats-on instance is the JS WRITER'S RECIPE in rust/src/ledger.rs's
// module docs, verbatim: Ajv2020 (the schema declares draft 2020-12) plus
// ajv-formats in its default mode, with schemars' `uint32`, `uint64` and
// `double` registered as annotation-only formats (`true`), and
// `strictTypes: false` because `SafeJson` is deliberately type-less. Without
// the three formats ajv's default constructor refuses to compile the schema
// at all. Keep the two in step: the check proves what a writer is told to run.
//
// Exits non-zero (and prints nothing on stdout) when the schema does not
// compile or the input is malformed; the orchestrator reads that as UNKNOWN.
"use strict";

// Load ajv from NODE_PATH (the pinned `npm ci` tree) and REFUSE any other
// copy. A bare require() searches every node_modules above this file first,
// and a require anchored at NODE_PATH still falls back to every node_modules
// above NODE_PATH. So: resolve, check the resolved file sits under NODE_PATH,
// and check each package's version is the one the committed package-lock.json
// pins. "Under NODE_PATH" alone would pass a NODE_PATH naming the parent of
// some other node_modules.
const fs = require("fs");
const path = require("path");
const LOCK = path.join(__dirname, "ledger-envelope-engines", "package-lock.json");

function loadPinned() {
  const np = process.env.NODE_PATH || "";
  if (!np || np.includes(path.delimiter) || !fs.existsSync(np) || !fs.statSync(np).isDirectory()) {
    throw new Error(`NODE_PATH must name ONE existing directory, the pinned node_modules (got ${JSON.stringify(np)})`);
  }
  const root = fs.realpathSync(np) + path.sep;
  const locked = JSON.parse(fs.readFileSync(LOCK, "utf8")).packages || {};
  const pinned = require("module").createRequire(path.join(root, "noop.js"));
  const load = (m, pkg) => {
    const file = fs.realpathSync(pinned.resolve(m));
    if (!file.startsWith(root)) {
      throw new Error(`${m} resolved to ${file}, outside the pinned NODE_PATH ${root}`);
    }
    const want = (locked[`node_modules/${pkg}`] || {}).version;
    const got = JSON.parse(fs.readFileSync(pinned.resolve(`${pkg}/package.json`), "utf8")).version;
    if (!want) {
      throw new Error(`${LOCK} has no "node_modules/${pkg}" entry to check ${pkg} against`);
    }
    if (got !== want) {
      throw new Error(`${pkg} under NODE_PATH is ${got}, but ${LOCK} pins ${want}`);
    }
    return pinned(file);
  };
  return { Ajv2020: load("ajv/dist/2020", "ajv"), addFormats: load("ajv-formats", "ajv-formats") };
}

const ANNOTATION_ONLY = { uint32: true, uint64: true, double: true };

function main() {
  const { Ajv2020, addFormats } = loadPinned();
  const input = JSON.parse(require("fs").readFileSync(0, "utf8"));
  if (!input || typeof input.schema !== "object" || !Array.isArray(input.cases)) {
    throw new Error("expected {schema, cases[]} on stdin");
  }

  const on = new Ajv2020({ formats: ANNOTATION_ONLY, strictTypes: false });
  addFormats(on);
  const validateOn = on.compile(input.schema);

  // Formats off: a writer who sets `validateFormats: false` (JSON Schema's
  // annotation-only default). A writer who merely omits ajv-formats gets no
  // verdict at all: ajv refuses to compile on the unknown `uuid`. This arm
  // must miss exactly the corpus's format-only cases.
  const off = new Ajv2020({
    formats: ANNOTATION_ONLY,
    strictTypes: false,
    validateFormats: false,
  });
  const validateOff = off.compile(input.schema);

  const rows = input.cases.map((c) => [
    c.name,
    validateOn(c.instance) === true,
    validateOff(c.instance) === true,
  ]);
  process.stdout.write(JSON.stringify(rows));
}

try {
  main();
} catch (e) {
  process.stderr.write(`ledger-envelope-engines.cjs: ${e && e.stack ? e.stack : e}\n`);
  process.exit(1);
}
