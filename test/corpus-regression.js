// Corpus regression: measure pass-rate on the first 100 valid JS files of the
// prettier 3.10 test corpus (excluding _errors_). Requires the corpus at
// /tmp/prettier/prettier-main/tests/format/js/. Day 5 baseline = 44/100.
// Day 8 (after bug fix) = 63/100.
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const os = require('os');
const { format } = require('../src/index');

const root = '/tmp/prettier/prettier-main/tests/format/js';
if (!fs.existsSync(root)) {
  console.error(`Missing prettier test corpus at ${root}.`);
  console.error('Run: git clone --depth=1 https://github.com/prettier/prettier /tmp/prettier');
  process.exit(2);
}

const files = [];
function walk(d) {
  let entries;
  try { entries = fs.readdirSync(d); } catch (e) { return; }
  for (const f of entries) {
    const full = path.join(d, f);
    let st;
    try { st = fs.statSync(full); } catch (e) { continue; }
    if (st.isDirectory()) {
      if (f === '_errors_') continue;
      walk(full);
    } else if (f.endsWith('.js')) files.push(full);
  }
}
walk(root);
const sample = files.slice(0, 100);

let pass = 0, fail = 0, parseFail = 0, errors = [];
for (const f of sample) {
  const src = fs.readFileSync(f, 'utf8');
  let out, tmp;
  try {
    out = format(src);
  } catch (e) {
    parseFail++; fail++;
    if (errors.length < 5) errors.push('[parse-fail] ' + path.relative(root, f));
    continue;
  }
  try {
    tmp = path.join(os.tmpdir(), `pp-${Date.now()}-${Math.random().toString(36).slice(2)}.js`);
    fs.writeFileSync(tmp, out);
    execSync(`node --check ${tmp}`, { stdio: 'pipe' });
    pass++;
  } catch (e) {
    fail++;
    if (errors.length < 5) errors.push('[round-trip-fail] ' + path.relative(root, f));
  } finally {
    if (tmp) try { fs.unlinkSync(tmp); } catch {}
  }
}
console.log(`pp2 corpus regression (first 100 valid): ${pass}/${sample.length} pass, ${parseFail} parse-fail.`);
console.log('first 5 fails:', errors);
process.exit(fail === 0 ? 0 : 1);
