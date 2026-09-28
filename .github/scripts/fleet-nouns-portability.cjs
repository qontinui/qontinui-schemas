// JS side of fleet-nouns-portability.py: reads {class, texts} on stdin, prints
// [[text, [class ids hit]], ...]. Patterns are built with `new RegExp(p)` as the
// fleet-nouns.toml header requires; the `g` flag is added ONLY to iterate every
// match (it does not change what a match is).
const input = JSON.parse(require("fs").readFileSync(0, "utf8"));

function spans(source, text) {
  const re = new RegExp(source, "g");
  const out = [];
  let m;
  while ((m = re.exec(text)) !== null) {
    out.push([m.index, m.index + m[0].length]);
    if (m[0].length === 0) re.lastIndex++;
  }
  return out;
}

function hits(c, text) {
  new RegExp(c.pattern); // must compile exactly as consumers build it
  const excl = c.exclude ? spans(c.exclude, text) : [];
  return spans(c.pattern, text).some(
    ([ms, me]) => !excl.some(([s, e]) => ms < e && s < me),
  );
}

const rows = input.texts.map((t) => [
  t,
  input.class.filter((c) => hits(c, t)).map((c) => c.id),
]);
process.stdout.write(JSON.stringify(rows));
