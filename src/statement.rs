//! Stage 3: statement printers for rust-scaffold.
//!
//! Mirrors the handlers in `assets/pretty-printer-js/src/index.js` and the
//! Stage 3 row of `tutorial/port-pp2.md`. Goal: produce text from a tree-sitter
//! `program` node by dispatching on each top-level statement kind.
//!
//! Current scope (Stage 3 COMPLETE — 18 of the 18 handlers in the tutorial):
//!   - `lexical_declaration`   → `print_lexical_declaration`
//!   - `expression_statement`  → `print_expression_statement`
//!   - `if_statement`          → `print_if_statement`
//!   - `return_statement`      → `print_return_statement`
//!   - `statement_block`       → `print_block_statement`
//!   - `for_statement`         → `print_for_statement`
//!   - `while_statement`       → `print_while_statement`
//!   - `do_statement`          → `print_do_statement`
//!   - `switch_statement`      → `print_switch_statement`
//!   - `try_statement`         → `print_try_statement`
//!   - `throw_statement`       → `print_throw_statement`
//!   - `break_statement`       → `print_break_statement`
//!   - `continue_statement`    → `print_continue_statement`
//!   - `debugger_statement`    → `print_debugger_statement`   (Day 16)
//!   - `empty_statement`       → `print_empty_statement`      (Day 16)
//!   - `import_statement`      → `print_import_statement`     (Day 16)
//!   - `export_statement`      → `print_export_statement`     (Day 16)
//!   - `labeled_statement`     → `print_labeled_statement`    (Day 16)
//! Each is a 5-15 line addition once the dispatcher pattern is in place.
//!
//! IMPORTANT (Stage 3 caveat): This MVP emits whitespace as it walks the tree
//! in source order. It does NOT yet implement the doc-printer (Prettier's
//! layout IR), so line wrapping and indentation are still source-derived.
//! The corpus regression will report 50-65/100 pass after Stage 3 ships; the
//! remaining 30+ points come from Stages 4-7 (expressions, declarations,
//! doc-printer, comments, op-precedence). See `tutorial/port-pp2.md` for the
//! full plan.

use tree_sitter::Node;

/// Build a textual representation of a single statement node.
///
/// `src` is the original source text; `node` is a tree-sitter node whose kind
/// must be one of the 13 handled in Stage 3 MVP. Returns `Some(text)` if the
/// kind was handled, `None` otherwise (caller should skip or fallback to
/// emitting the raw byte range).
pub fn print_statement(node: Node<'_>, src: &str) -> Option<String> {
    match node.kind() {
        "lexical_declaration" => Some(print_lexical_declaration(node, src)),
        "expression_statement" => Some(print_expression_statement(node, src)),
        "if_statement" => Some(print_if_statement(node, src)),
        "return_statement" => Some(print_return_statement(node, src)),
        "statement_block" => Some(print_block_statement(node, src)),
        "for_statement" => Some(print_for_statement(node, src)),
        "while_statement" => Some(print_while_statement(node, src)),
        "do_statement" => Some(print_do_statement(node, src)),
        "switch_statement" => Some(print_switch_statement(node, src)),
        "try_statement" => Some(print_try_statement(node, src)),
        "throw_statement" => Some(print_throw_statement(node, src)),
        "break_statement" => Some(print_break_statement(node, src)),
        "continue_statement" => Some(print_continue_statement(node, src)),
        "debugger_statement" => Some(print_debugger_statement(node, src)),
        "empty_statement" => Some(print_empty_statement(node, src)),
        "import_statement" => Some(print_import_statement(node, src)),
        "export_statement" => Some(print_export_statement(node, src)),
        "labeled_statement" => Some(print_labeled_statement(node, src)),
        _ => None,
    }
}

/// Format a complete `program` node (a sequence of top-level statements) by
/// dispatching each statement through `print_statement`. Statements joined by
/// `\n`; unknown statement kinds fall back to their raw byte range so no input
/// is silently dropped (debug aid for Stage 3 iteration).
pub fn print_program(node: Node<'_>, src: &str) -> String {
    let mut out = String::new();
    let mut cursor = node.walk();
    let mut first = true;
    if cursor.goto_first_child() {
        loop {
            if !first {
                out.push('\n');
            }
            first = false;
            let stmt = cursor.node();
            match print_statement(stmt, src) {
                Some(text) => out.push_str(&text),
                None => {
                    let s = stmt.start_byte();
                    let e = stmt.end_byte();
                    out.push_str(&src[s..e]);
                }
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
    out
}

// ---------- handler internals ----------

fn slice(node: Node<'_>, src: &str) -> &str {
    &src[node.start_byte()..node.end_byte()]
}

fn print_lexical_declaration(node: Node<'_>, src: &str) -> String {
    // e.g. `const x = 1;` — emit verbatim in Stage 3 MVP.
    // Stage 4 will replace `=` spacing with operator-precedence rules.
    slice(node, src).to_string()
}

fn print_expression_statement(node: Node<'_>, src: &str) -> String {
    // e.g. `foo();` — emit verbatim.
    slice(node, src).to_string()
}

fn print_if_statement(node: Node<'_>, src: &str) -> String {
    // e.g. `if (x) { y; } else { z; }` — emit verbatim.
    // Stage 4 will normalize whitespace around the condition and the body.
    slice(node, src).to_string()
}

fn print_return_statement(node: Node<'_>, src: &str) -> String {
    // e.g. `return x;` or `return;` — emit verbatim.
    slice(node, src).to_string()
}

fn print_block_statement(node: Node<'_>, src: &str) -> String {
    // e.g. `{ x; y; }` — recurse into children, joining with `\n`.
    // This is the first handler that walks the tree, not just slicing.
    let mut out = String::new();
    let mut cursor = node.walk();
    // Emit the opening brace as-is from source.
    let open = slice(node, src);
    if let Some(open_end) = open.find('{') {
        out.push_str(&open[..open_end + 1]);
        out.push('\n');
    }
    let mut first = true;
    if cursor.goto_first_child() {
        loop {
            // Skip the `{` and `}` anonymous nodes themselves.
            let child = cursor.node();
            if child.kind() != "}" && child.kind() != "{" {
                if !first {
                    out.push('\n');
                }
                first = false;
                match print_statement(child, src) {
                    Some(text) => out.push_str(&text),
                    None => out.push_str(slice(child, src)),
                }
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
    out.push('}');
    out
}

fn print_for_statement(node: Node<'_>, src: &str) -> String {
    // e.g. `for (let i = 0; i < n; i++) { ... }` — emit verbatim.
    // Stage 4 will normalize whitespace and split init/test/update cleanly.
    slice(node, src).to_string()
}

fn print_while_statement(node: Node<'_>, src: &str) -> String {
    // e.g. `while (cond) { ... }` — emit verbatim.
    slice(node, src).to_string()
}

fn print_do_statement(node: Node<'_>, src: &str) -> String {
    // e.g. `do { ... } while (cond);` — emit verbatim.
    // The trailing `;` is part of the source range.
    slice(node, src).to_string()
}

fn print_switch_statement(node: Node<'_>, src: &str) -> String {
    // e.g. `switch (x) { case 1: a; break; default: b; }` — emit verbatim.
    // Stage 5 will handle the switch_body / switch_case / switch_default
    // normalization (Prettier indents cases at the same level as the body).
    slice(node, src).to_string()
}

fn print_try_statement(node: Node<'_>, src: &str) -> String {
    // e.g. `try { ... } catch (e) { ... } finally { ... }` — emit verbatim.
    // Stage 5 will normalize catch/finally clause spacing.
    slice(node, src).to_string()
}

fn print_throw_statement(node: Node<'_>, src: &str) -> String {
    // e.g. `throw new Error('x');` — emit verbatim.
    slice(node, src).to_string()
}

fn print_break_statement(node: Node<'_>, src: &str) -> String {
    // e.g. `break;` or `break label;` — emit verbatim.
    slice(node, src).to_string()
}

fn print_continue_statement(node: Node<'_>, src: &str) -> String {
    // e.g. `continue;` or `continue label;` — emit verbatim.
    slice(node, src).to_string()
}

fn print_debugger_statement(node: Node<'_>, src: &str) -> String {
    // `debugger;` — emit verbatim.
    slice(node, src).to_string()
}

fn print_empty_statement(_node: Node<'_>, _src: &str) -> String {
    // bare `;` — emit a single semicolon so the source remains parseable.
    ";".to_string()
}

fn print_import_statement(node: Node<'_>, src: &str) -> String {
    // `import foo from 'bar';` / `import { a, b } from 'mod';` — emit verbatim.
    slice(node, src).to_string()
}

fn print_export_statement(node: Node<'_>, src: &str) -> String {
    // `export default ...;` / `export const x = 1;` / `export { a, b };` — emit verbatim.
    slice(node, src).to_string()
}

fn print_labeled_statement(node: Node<'_>, src: &str) -> String {
    // `label: while (cond) { ... }` — emit verbatim (label + statement).
    slice(node, src).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse, format_leaf, LeafFormat};

    fn format_program(src: &str) -> String {
        let root = parse(src).expect("parse must succeed");
        print_program(root, src)
    }

    #[test]
    fn stage3_dispatch_returns_none_for_unknown() {
        // A `program` node is NOT a statement itself.
        let src = "const x = 1;";
        let root = parse(src).unwrap();
        assert_eq!(print_statement(root, src), None);
    }

    #[test]
    fn stage3_lexical_declaration() {
        let src = "const x = 1;";
        let root = parse(src).unwrap();
        let mut c = root.walk();
        c.goto_first_child(); // → lexical_declaration
        let stmt = c.node();
        let out = print_statement(stmt, src).expect("lexical_declaration must be handled");
        assert_eq!(out, "const x = 1;");
    }

    #[test]
    fn stage3_expression_statement() {
        let src = "console.log('hi');";
        let root = parse(src).unwrap();
        let mut c = root.walk();
        c.goto_first_child();
        let stmt = c.node();
        let out = print_statement(stmt, src).expect("expression_statement must be handled");
        assert_eq!(out, "console.log('hi');");
    }

    #[test]
    fn stage3_if_statement() {
        let src = "if (a) { b; }";
        let root = parse(src).unwrap();
        let mut c = root.walk();
        c.goto_first_child();
        let out = print_statement(c.node(), src).expect("if_statement must be handled");
        // Stage 3 MVP emits the source verbatim.
        assert_eq!(out, "if (a) { b; }");
    }

    #[test]
    fn stage3_return_with_value() {
        let src = "return 42;";
        let root = parse(src).unwrap();
        let mut c = root.walk();
        c.goto_first_child();
        let out = print_statement(c.node(), src).expect("return_statement must be handled");
        assert_eq!(out, "return 42;");
    }

    #[test]
    fn stage3_block_recurses() {
        let src = "{ a; b; }";
        let root = parse(src).unwrap();
        let mut c = root.walk();
        c.goto_first_child();
        let out = print_statement(c.node(), src).expect("statement_block must be handled");
        // Stage 3 MVP emits `{` then newline-joined children then `}`.
        assert!(out.starts_with('{'));
        assert!(out.ends_with('}'));
        assert!(out.contains("a;"));
        assert!(out.contains("b;"));
    }

    #[test]
    fn stage3_full_program_two_statements() {
        let src = "const x = 1;\nconst y = 2;";
        let out = format_program(src);
        assert!(out.contains("const x = 1;"));
        assert!(out.contains("const y = 2;"));
        assert!(out.contains('\n')); // joined by newline
    }

    // Regression: ensure Stage 2 leaf machinery still works unchanged.
    #[test]
    fn stage2_leaf_still_works_alongside_stage3() {
        let src = "foo";
        let root = parse(src).unwrap();
        let mut c = root.walk();
        c.goto_first_child();
        let fmt = format_leaf(c.node(), src).expect("identifier must still be a leaf");
        assert_eq!(fmt, LeafFormat::Raw("foo".to_string()));
    }

    // ---- Stage 3 expansion: for / while / do / switch / try / throw / break / continue ----

    #[test]
    fn stage3_for_statement() {
        let src = "for (let i = 0; i < n; i++) { f(i); }";
        let root = parse(src).unwrap();
        let mut c = root.walk();
        c.goto_first_child();
        let out = print_statement(c.node(), src).expect("for_statement must be handled");
        assert!(out.contains("for"));
        assert!(out.contains("i < n"));
        assert!(out.contains("f(i)"));
    }

    #[test]
    fn stage3_while_statement() {
        let src = "while (cond) { step(); }";
        let root = parse(src).unwrap();
        let mut c = root.walk();
        c.goto_first_child();
        let out = print_statement(c.node(), src).expect("while_statement must be handled");
        assert!(out.contains("while"));
        assert!(out.contains("step()"));
    }

    #[test]
    fn stage3_do_statement() {
        let src = "do { step(); } while (cond);";
        let root = parse(src).unwrap();
        let mut c = root.walk();
        c.goto_first_child();
        let out = print_statement(c.node(), src).expect("do_statement must be handled");
        assert!(out.starts_with("do"));
        assert!(out.contains("while"));
    }

    #[test]
    fn stage3_switch_statement() {
        let src = "switch (x) { case 1: a; break; default: b; }";
        let root = parse(src).unwrap();
        let mut c = root.walk();
        c.goto_first_child();
        let out = print_statement(c.node(), src).expect("switch_statement must be handled");
        assert!(out.contains("switch"));
        assert!(out.contains("case 1"));
        assert!(out.contains("default"));
    }

    #[test]
    fn stage3_try_statement() {
        let src = "try { f(); } catch (e) { g(e); } finally { h(); }";
        let root = parse(src).unwrap();
        let mut c = root.walk();
        c.goto_first_child();
        let out = print_statement(c.node(), src).expect("try_statement must be handled");
        assert!(out.contains("try"));
        assert!(out.contains("catch"));
        assert!(out.contains("finally"));
    }

    #[test]
    fn stage3_throw_statement() {
        let src = "throw new Error('x');";
        let root = parse(src).unwrap();
        let mut c = root.walk();
        c.goto_first_child();
        let out = print_statement(c.node(), src).expect("throw_statement must be handled");
        assert!(out.contains("throw"));
        assert!(out.contains("Error"));
    }

    #[test]
    fn stage3_break_statement() {
        let src = "while (true) { if (done) break; }";
        let root = parse(src).unwrap();
        let mut c = root.walk();
        // First child is the while_statement; recurse into its block → break_statement.
        c.goto_first_child();
        c.goto_first_child(); // → statement_block
        c.goto_first_child(); // → if_statement
        c.goto_first_child(); // → statement_block (if body)
        c.goto_first_child(); // → break_statement
        let out = print_statement(c.node(), src).expect("break_statement must be handled");
        assert!(out.contains("break"));
    }

    #[test]
    fn stage3_continue_statement() {
        let src = "for (let i = 0; i < n; i++) { if (skip) continue; }";
        let root = parse(src).unwrap();
        let mut c = root.walk();
        c.goto_first_child(); // → for_statement
        c.goto_first_child(); // → statement_block
        c.goto_first_child(); // → if_statement
        c.goto_first_child(); // → statement_block
        c.goto_first_child(); // → continue_statement
        let out = print_statement(c.node(), src).expect("continue_statement must be handled");
        assert!(out.contains("continue"));
    }

    #[test]
    fn stage3_mixed_program_handles_all_13_kinds() {
        // Build a small program that exercises every dispatched kind.
        let src = "\
const x = 1;
foo();
if (a) { b; } else { c; }
return x;
for (let i = 0; i < n; i++) { f(i); }
while (cond) { step(); }
do { once(); } while (again);
switch (x) { case 1: a; break; default: b; }
try { work(); } catch (e) { rescue(e); } finally { cleanup(); }
throw new Error('boom');
for (let j = 0; j < m; j++) { if (skip) continue; }
for (let k = 0; k < p; k++) { if (stop) break; }
";
        let out = format_program(src);
        // Every line of source has at least one recognizable token preserved.
        for keyword in &[
            "const x = 1",
            "foo()",
            "if (a)",
            "return x",
            "for (let i",
            "while (cond)",
            "do {",
            "switch (x)",
            "try {",
            "throw new Error",
            "continue",
            "break",
        ] {
            assert!(out.contains(keyword), "missing {:?} in: {}", keyword, out);
        }
    }

    #[test]
    fn stage3_day16_debugger_statement() {
        let src = "debugger;";
        let root = parse(src).unwrap();
        let mut c = root.walk();
        c.goto_first_child();
        let stmt = c.node();
        let out = print_statement(stmt, src).expect("debugger must be handled");
        assert_eq!(out, "debugger;");
    }

    #[test]
    fn stage3_day16_empty_statement() {
        let src = ";";
        let root = parse(src).unwrap();
        let mut c = root.walk();
        c.goto_first_child();
        let stmt = c.node();
        let out = print_statement(stmt, src).expect("empty must be handled");
        assert_eq!(out, ";");
    }

    #[test]
    fn stage3_day16_import_statement() {
        let src = "import foo from 'bar';";
        let root = parse(src).unwrap();
        let mut c = root.walk();
        c.goto_first_child();
        let stmt = c.node();
        let out = print_statement(stmt, src).expect("import must be handled");
        assert_eq!(out, "import foo from 'bar';");
    }

    #[test]
    fn stage3_day16_export_statement() {
        let src = "export const x = 1;";
        let root = parse(src).unwrap();
        let mut c = root.walk();
        c.goto_first_child();
        let stmt = c.node();
        let out = print_statement(stmt, src).expect("export must be handled");
        assert_eq!(out, "export const x = 1;");
    }

    #[test]
    fn stage3_day16_labeled_statement() {
        let src = "outer: while (cond) { break outer; }";
        let root = parse(src).unwrap();
        let mut c = root.walk();
        c.goto_first_child();
        let stmt = c.node();
        let out = print_statement(stmt, src).expect("labeled must be handled");
        assert!(out.starts_with("outer:"), "expected label prefix, got: {}", out);
        assert!(out.contains("while (cond)"), "expected inner while, got: {}", out);
    }

    #[test]
    fn stage3_day16_mixed_program_all_18_handlers() {
        // A program exercising all 18 statement kinds end-to-end.
        let src = "\
const x = 1;
foo();
if (a) { b; } else { c; }
return x;
for (let i = 0; i < n; i++) { f(i); }
while (cond) { step(); }
do { once(); } while (again);
switch (x) { case 1: a; break; default: b; }
try { work(); } catch (e) { rescue(e); } finally { cleanup(); }
throw new Error('boom');
for (let j = 0; j < m; j++) { if (skip) continue; }
for (let k = 0; k < p; k++) { if (stop) break; }
debugger;
;
import foo from 'bar';
export const y = 2;
outer: for (;;) { break outer; }
";
        let out = format_program(src);
        for keyword in &[
            "import foo",
            "export const y",
            "outer:",
            "debugger;",
        ] {
            assert!(out.contains(keyword), "missing {:?} in: {}", keyword, out);
        }
    }
}
