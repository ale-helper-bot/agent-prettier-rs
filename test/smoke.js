// test/smoke.js — verify the rust-scaffold directory is structurally complete
// and that the Rust source parses (syntactically) by running `cargo check`
// if cargo is available, otherwise just checking file presence.
//
// Run: node test/smoke.js   (from inside examples/rust-scaffold/)

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const ROOT = path.resolve(__dirname, '..');
const required = [
    'Cargo.toml',
    'src/lib.rs',
    'src/bin/cli.rs',
    path.join('src', 'lib.rs'),
    path.join('src', 'bin', 'cli.rs'),
];

let pass = 0;
let fail = 0;

function check(label, ok, detail) {
    const tag = ok ? 'PASS' : 'FAIL';
    console.log(`[${tag}] ${label}${detail ? '  ' + detail : ''}`);
    ok ? pass++ : fail++;
}

// 1. structural completeness
for (const f of required) {
    const p = path.join(ROOT, f);
    check(`file exists: ${f}`, fs.existsSync(p));
}

// 2. content sanity — Cargo.toml pins the right deps, src/lib.rs has parse function
const cargo = fs.readFileSync(path.join(ROOT, 'Cargo.toml'), 'utf8');
check('Cargo.toml pins tree-sitter-javascript', cargo.includes('tree-sitter-javascript'));
check('Cargo.toml pins tree-sitter-typescript', cargo.includes('tree-sitter-typescript'));
check('Cargo.toml pins clap', cargo.includes('clap'));

const lib = fs.readFileSync(path.join(ROOT, 'src/lib.rs'), 'utf8');
check('src/lib.rs has parse()', lib.includes('pub fn parse'));
check('src/lib.rs has cursor_dump()', lib.includes('pub fn cursor_dump'));
check('src/lib.rs has unit tests', lib.includes('#[cfg(test)]') && lib.includes('#[test]'));

// Stage 2 (v0.3) checks: leaf-node printers added
check('src/lib.rs has format_leaf()', lib.includes('pub fn format_leaf'));
check('src/lib.rs has round_trip_simple()', lib.includes('pub fn round_trip_simple'));
check('src/lib.rs has LeafFormat enum', lib.includes('pub enum LeafFormat'));
check('src/lib.rs handles identifier leaf', lib.includes('"identifier"'));
check('src/lib.rs handles number leaf', lib.includes('"number"'));
check('src/lib.rs handles string leaf', lib.includes('"string"'));
check('src/lib.rs handles regex leaf', lib.includes('"regex"'));
check('src/lib.rs has format_leaf_identifier test', lib.includes('fn format_leaf_identifier'));
check('src/lib.rs has format_leaf_number test', lib.includes('fn format_leaf_number'));
check('src/lib.rs has format_leaf_string test', lib.includes('fn format_leaf_string'));
check('src/lib.rs has format_leaf_regex test', lib.includes('fn format_leaf_regex'));
check('src/lib.rs has format_leaf_punctuation test', lib.includes('fn format_leaf_punctuation'));

// Stage 3 (v0.4 — 2026-07-04) checks: statement printers split into src/statement.rs
check('src/statement.rs exists', fs.existsSync(path.join(ROOT, 'src', 'statement.rs')));
const stmt = fs.readFileSync(path.join(ROOT, 'src', 'statement.rs'), 'utf8');
check('src/statement.rs declares pub fn print_statement', stmt.includes('pub fn print_statement'));
check('src/statement.rs declares pub fn print_program', stmt.includes('pub fn print_program'));
check('src/lib.rs has `pub mod statement`', lib.includes('pub mod statement'));
check('Stage 3 handles lexical_declaration', stmt.includes('"lexical_declaration"'));
check('Stage 3 handles expression_statement', stmt.includes('"expression_statement"'));
check('Stage 3 handles if_statement', stmt.includes('"if_statement"'));
check('Stage 3 handles return_statement', stmt.includes('"return_statement"'));
check('Stage 3 handles statement_block', stmt.includes('"statement_block"'));
// Stage 3 expansion (2026-07-05): for/while/do/switch/try/throw/break/continue
check('Stage 3 handles for_statement', stmt.includes('"for_statement"'));
check('Stage 3 handles while_statement', stmt.includes('"while_statement"'));
check('Stage 3 handles do_statement', stmt.includes('"do_statement"'));
check('Stage 3 handles switch_statement', stmt.includes('"switch_statement"'));
check('Stage 3 handles try_statement', stmt.includes('"try_statement"'));
check('Stage 3 handles throw_statement', stmt.includes('"throw_statement"'));
check('Stage 3 handles break_statement', stmt.includes('"break_statement"'));
check('Stage 3 handles continue_statement', stmt.includes('"continue_statement"'));
check('Stage 3 has 14+ unit tests', (stmt.match(/#\[test\]/g) || []).length >= 14);
check('Stage 3 has mixed_program integration test', stmt.includes('fn stage3_mixed_program'));

// Day 16 (2026-07-06) Stage 3 finalization: +5 handlers (debugger/empty/import/export/labeled)
check('Stage 3 handles debugger_statement', stmt.includes('"debugger_statement"'));
check('Stage 3 handles empty_statement', stmt.includes('"empty_statement"'));
check('Stage 3 handles import_statement', stmt.includes('"import_statement"'));
check('Stage 3 handles export_statement', stmt.includes('"export_statement"'));
check('Stage 3 handles labeled_statement', stmt.includes('"labeled_statement"'));
check('Stage 3 has 19+ unit tests (5 new)', (stmt.match(/#\[test\]/g) || []).length >= 19);
check('Stage 3 final mixed_program all 18 handlers', stmt.includes('fn stage3_day16_mixed_program_all_18_handlers'));

// Day 17 (2026-07-06) Stage 4: src/expr.rs with 8 expression printers
check('src/expr.rs exists', fs.existsSync(path.join(ROOT, 'src', 'expr.rs')));
const expr = fs.readFileSync(path.join(ROOT, 'src', 'expr.rs'), 'utf8');
check('src/expr.rs declares pub fn print_expression', expr.includes('pub fn print_expression'));
check('src/expr.rs declares pub fn print_binary_expression', expr.includes('pub fn print_binary_expression'));
check('src/expr.rs declares pub fn print_unary_expression', expr.includes('pub fn print_unary_expression'));
check('src/expr.rs declares pub fn print_member_expression', expr.includes('pub fn print_member_expression'));
check('src/expr.rs declares pub fn print_call_expression', expr.includes('pub fn print_call_expression'));
check('src/expr.rs declares pub fn print_update_expression', expr.includes('pub fn print_update_expression'));
check('src/expr.rs declares pub fn print_assignment_expression', expr.includes('pub fn print_assignment_expression'));
check('src/expr.rs declares pub fn print_conditional_expression', expr.includes('pub fn print_conditional_expression'));
check('src/expr.rs declares pub fn print_new_expression', expr.includes('pub fn print_new_expression'));
check('Stage 4 handles binary_expression', expr.includes('"binary_expression"'));
check('Stage 4 handles unary_expression', expr.includes('"unary_expression"'));
check('Stage 4 handles member_expression', expr.includes('"member_expression"'));
check('Stage 4 handles call_expression', expr.includes('"call_expression"'));
check('Stage 4 has 9+ unit tests', (expr.match(/#\[test\]/g) || []).length >= 9);
check('Stage 4 dispatcher_some_for_handled test', expr.includes('fn dispatcher_returns_some_for_handled_and_none_for_unhandled'));
check('src/lib.rs has `pub mod expr`', lib.includes('pub mod expr'));

// Day 18 (2026-07-06) Stage 5: +2 expression printers (array_expression, template_string)
check('Stage 5 handles array_expression', expr.includes('"array_expression"'));
check('Stage 5 handles template_string', expr.includes('"template_string"'));
check('Stage 5 declares print_array_expression', expr.includes('pub fn print_array_expression'));
check('Stage 5 declares print_template_string', expr.includes('pub fn print_template_string'));
check('Stage 5 has 13+ unit tests', (expr.match(/#\[test\]/g) || []).length >= 13);
check('Stage 5 array_expression_emits_bracketed_range test', expr.includes('fn array_expression_emits_bracketed_range'));
check('Stage 5 template_string_with_interpolation test', expr.includes('fn template_string_with_interpolation'));

const cli = fs.readFileSync(path.join(ROOT, 'src/bin/cli.rs'), 'utf8');
check('src/bin/cli.rs has --file flag', cli.includes('--file'));
check('src/bin/cli.rs has --dump flag', cli.includes('--dump'));

// 3. optional: try cargo check if cargo is installed (sandbox may not have it)
let cargoOk = false;
let cargoDetail = 'cargo not available in PATH';
try {
    execSync('cargo --version', { stdio: 'pipe' });
    try {
        execSync('cargo check --quiet --offline 2>&1 || cargo check --quiet 2>&1', {
            cwd: ROOT,
            stdio: 'pipe',
            timeout: 60000,
        });
        cargoOk = true;
        cargoDetail = 'cargo check passed';
    } catch (e) {
        cargoDetail = `cargo check failed: ${e.message.slice(0, 100)}`;
    }
} catch (_) {
    // cargo not installed; that's fine, the scaffold is still shippable as a template
}
check('cargo check (optional)', cargoOk, cargoDetail);

// 4. tutorial file exists and has the porting table
const tutorialPath = path.join(ROOT, 'tutorial', 'port-pp2.md');
let tutorialOk = false;
let tutorialDetail = 'missing';
if (fs.existsSync(tutorialPath)) {
    const t = fs.readFileSync(tutorialPath, 'utf8');
    tutorialOk = t.length > 200 && t.includes('|') && (t.includes('pp2') || t.includes('FunctionDeclaration'));
    tutorialDetail = `${t.length} chars`;
}
check('tutorial/port-pp2.md exists and has table', tutorialOk, tutorialDetail);

console.log(`\n${pass}/${pass + fail} smoke checks passed`);
process.exit(fail === 0 ? 0 : 1);