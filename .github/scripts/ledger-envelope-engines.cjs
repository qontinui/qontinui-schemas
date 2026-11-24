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

// Resolve ajv ONLY from NODE_PATH (the pinned `npm ci` tree): a bare require()
// would search every node_modules above this file first, and on a box with an
// ancestor node_modules would check an unpinned ajv.
const path = require("path");
if (!process.env.NODE_PATH) {
  process.stderr.write("ledger-envelope-engines.cjs: NODE_PATH is unset; point it at the pinned node_modules\n");
  process.exit(1);
}
const pinned = require("module").createRequire(path.join(path.resolve(process.env.NODE_PATH), "noop.js"));
const Ajv2020 = pinned("ajv/dist/2020");
const addFormats = pinned("ajv-formats");

const ANNOTATION_ONLY = { uint32: true, uint64: true, double: true };

function main() {
  const input = JSON.parse(require("fs").readFileSync(0, "utf8"));
  if (!input || typeof input.schema !== "object" || !Array.isArray(input.cases)) {
    throw new Error("expected {schema, cases[]} on stdin");
  }

  const on = new Ajv2020({ formats: ANNOTATION_ONLY, strictTypes: false });
  addFormats(on);
  const validateOn = on.compile(input.schema);

  // Formats off: a writer who sets `validateFormats: false` (JSON Schema's
  // annotation-only default). A writer who merely omits ajv-formats gets no
  // verdict at all: ajv refuses to compile on the unknown `date-time`. This
  // arm must miss exactly the corpus's format-only cases.
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
