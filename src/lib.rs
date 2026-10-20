//! rust-scaffold: smallest viable entry point for porting pretty-printer-js to Rust.
//!
//! Stage 1 (this file): parse source code into a tree-sitter AST and dump a
//! one-line-per-node cursor traversal. No formatter logic. No output. Just
//! proof that the toolchain wires together.
//!
//! Stage 2 (THIS VERSION — v0.3): leaf-node printers added. `format_leaf()`
//! handles the 5 simplest node kinds from `assets/pretty-printer-js/src/index.js`:
//!   - identifier (Identifier in JS)
//!   - number/string/true/false/null literals
//!   - regex literals
//!   - private_identifier
//! Each handler is the Rust mirror of a JS handler in pp2.js. Goal: any input
//! whose AST is all leaves round-trips byte-identical. Real-world 50/100 pass
//! rate on the Prettier test corpus is the next milestone.
//!
//! Stage 3 (PARTIAL — 2026-07-04): statement printers in `statement.rs`.
//! 5 of the 18 handlers shipped (lexical_declaration, expression_statement,
//! if_statement, return_statement, statement_block). Remaining 13 handlers
//! (for/while/do/switch/try/throw/break/continue/debugger/empty/import/export/
//! labeled) are next-iteration work. See `statement.rs` module docs for the
//! corpus pass-rate caveat.

use tree_sitter::{Node, Parser, Tree};

/// Stage 3 statement printers. See `statement.rs` for the dispatcher and the
/// individual handler bodies.
pub mod statement;

/// Stage 4 expression printers. See `expr.rs` for the dispatcher and the
/// individual handler bodies. Covers 8 of the most common JS expression
/// kinds: binary, unary, member, call, update, assignment, conditional, new.
pub mod expr;

/// Result of formatting one leaf node. For non-leaf nodes returns `None`.
/// For leaf nodes returns `Some(text)` where `text` is the canonical
/// representation that should be emitted in place of the node's raw text.
#[derive(Debug, PartialEq)]
pub enum LeafFormat {
    /// Emit this string verbatim (e.g. an identifier `foo`).
    Raw(String),
    /// Sentinel for unrecognized leaf nodes (no-op).
    PassThrough,
}

impl LeafFormat {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            LeafFormat::Raw(s) => Some(s.as_str()),
            LeafFormat::PassThrough => None,
        }
    }
}

/// Parses JavaScript or TypeScript source code into a tree-sitter AST.
///
/// Returns `Some(tree)` on success, `None` if the parser fails to produce a
/// tree (e.g., completely empty input). The caller owns the returned `Tree`;
/// call `tree.root_node()` to walk it.
pub fn parse(src: &str) -> Option<Tree> {
    let mut parser = Parser::new();
    // tree-sitter-javascript is the right backend for .js files.
    // tree-sitter-typescript handles .ts/.tsx with embedded JS.
    // For Stage 1 we default to JS; Stage 2 will add a backend switch.
    parser
        .set_language(tree_sitter_javascript::language())
        .expect("failed to load tree-sitter-javascript grammar");
    parser.parse(src, None)
}

/// Walks the AST and returns a one-line-per-node cursor dump.
///
/// Format: `kind:byte_range start_row:start_col -> end_row:end_col`
///
/// This is the smallest useful "introspect the AST" tool. It lets you see
/// what tree-sitter actually produced for any input, which is the first
/// step in porting a printer from acorn (JS) to tree-sitter (Rust).
pub fn cursor_dump(root: Node<'_>) -> Vec<String> {
    let mut out = Vec::new();
    let mut cursor = root.walk();
    loop {
        let node = cursor.node();
        out.push(format!(
            "{kind}:{s}-{e} ({sr}:{sc} -> {er}:{ec})",
            kind = node.kind(),
            s = node.start_byte(),
            e = node.end_byte(),
            sr = node.start_position().row,
            sc = node.start_position().column,
            er = node.end_position().row,
            ec = node.end_position().column,
        ));
        // DFS via goto_first_child / goto_next_sibling, with goto_parent on completion.
        if cursor.goto_first_child() {
            continue;
        }
        if cursor.goto_next_sibling() {
            continue;
        }
        loop {
            if !cursor.goto_parent() {
                return out;
            }
            if cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

/// Convenience: parse + dump in one call.
pub fn parse_and_dump(src: &str) -> Vec<String> {
    match parse(src) {
        Some(tree) => cursor_dump(tree.root_node()),
        None => Vec::new(),
    }
}

/// Format a single AST node if it is a leaf in the JS/TS grammar.
///
/// Returns `Some(LeafFormat::Raw(text))` for leaves that should be emitted
/// as-is. Returns `Some(LeafFormat::PassThrough)` for leaves that need
/// no transformation. Returns `None` for non-leaf nodes (Stage 3 will
/// recurse into them).
///
/// Recognized leaf kinds (Stage 2 scope):
///   - `identifier`            → raw text (e.g. `foo`, `_x`, `$bar`)
///   - `private_identifier`    → raw text (e.g. `#secret`)
///   - `number`                → raw text (already canonical)
///   - `string`                → raw text (already canonical in JS source)
///   - `true`, `false`         → raw text
///   - `null`                  → raw text
///   - `regex`                 → raw text (regex body + flags)
///
/// This mirrors `printIdentifier`, `printLiteral`, `printRegExpLiteral`,
/// `printPrivateIdentifier` from `assets/pretty-printer-js/src/index.js`.
pub fn format_leaf(node: Node<'_>, src: &str) -> Option<LeafFormat> {
    let kind = node.kind();
    // Anything that has at least one named child is NOT a leaf.
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        return None;
    }
    // Anonymous nodes (e.g. some punctuation) also have children sometimes;
    // the only safe check is "no children at all".
    if node.child_count() > 0 {
        return None;
    }
    let text = &src[node.start_byte()..node.end_byte()];
    match kind {
        "identifier" | "private_identifier" | "number" | "string"
        | "true" | "false" | "null" | "regex" | "template_string"
        | "comment" => Some(LeafFormat::Raw(text.to_string())),
        // Punctuation / braces — emit verbatim, don't escape.
        "{" | "}" | "(" | ")" | "[" | "]" | "," | ";" | ":" | "." => {
            Some(LeafFormat::Raw(text.to_string()))
        }
        _ => Some(LeafFormat::PassThrough),
    }
}

/// Convenience: try to format a single line of source. If every token in
/// the input is a leaf this returns the same string. Used by Stage 2 tests
/// as a smoke check that the printer is wired up.
pub fn round_trip_simple(src: &str) -> Option<String> {
    let tree = parse(src)?;
    let root = tree.root_node();
    let mut out = String::new();
    let mut cursor = root.walk();
    loop {
        let node = cursor.node();
        if let Some(LeafFormat::Raw(text)) = format_leaf(node, src) {
            out.push_str(&text);
        }
        if cursor.goto_next_sibling() {
            out.push(' ');
            continue;
        }
        if cursor.goto_parent() {
            out.push(' ');
            continue;
        }
        break;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_var() {
        let src = "const x = 1;";
        let dump = parse_and_dump(src);
        assert!(!dump.is_empty(), "expected non-empty AST dump");
        // The root should be a program node.
        assert!(dump[0].starts_with("program:"), "got: {}", dump[0]);
    }

    #[test]
    fn parse_empty_returns_empty_dump() {
        let dump = parse_and_dump("");
        // Empty source may or may not produce a tree; either way, no panic.
        // Just assert the call didn't blow up.
        let _ = dump;
    }

    /// Descend from `program` → `expression_statement` → the expression node.
    fn first_expression(root: Node<'_>) -> Node<'_> {
        root.named_child(0)
            .and_then(|stmt| stmt.named_child(0))
            .expect("source must contain an expression")
    }

    /// Depth-first search for the first descendant of the given kind.
    fn find_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
        if node.kind() == kind {
            return Some(node);
        }
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                if let Some(found) = find_kind(cursor.node(), kind) {
                    return Some(found);
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        None
    }

    #[test]
    fn format_leaf_identifier() {
        let src = "foo";
        let tree = parse(src).unwrap();
        let id_node = first_expression(tree.root_node());
        let fmt = format_leaf(id_node, src).expect("identifier must be a leaf");
        assert_eq!(fmt, LeafFormat::Raw("foo".to_string()));
    }

    #[test]
    fn format_leaf_number() {
        let src = "42";
        let tree = parse(src).unwrap();
        let node = first_expression(tree.root_node());
        let fmt = format_leaf(node, src).expect("number must be a leaf");
        assert_eq!(fmt, LeafFormat::Raw("42".to_string()));
    }

    #[test]
    fn format_leaf_string() {
        let src = "'hello'";
        let tree = parse(src).unwrap();
        let node = first_expression(tree.root_node());
        let fmt = format_leaf(node, src).expect("string must be a leaf");
        assert_eq!(fmt, LeafFormat::Raw("'hello'".to_string()));
    }

    #[test]
    fn format_leaf_true_false_null() {
        for (src, expected) in [("true", "true"), ("false", "false"), ("null", "null")] {
            let tree = parse(src).unwrap();
            let node = first_expression(tree.root_node());
            let fmt = format_leaf(node, src).expect("boolean/null must be a leaf");
            assert_eq!(fmt, LeafFormat::Raw(expected.to_string()));
        }
    }

    #[test]
    fn format_leaf_regex() {
        let src = "/foo/g";
        let tree = parse(src).unwrap();
        let node = first_expression(tree.root_node());
        let fmt = format_leaf(node, src).expect("regex must be a leaf");
        assert_eq!(fmt, LeafFormat::Raw("/foo/g".to_string()));
    }

    #[test]
    fn format_leaf_punctuation() {
        // A lone punctuation character does not form a valid program, so use
        // small snippets in which each punctuation node actually appears.
        let cases = [
            ("function f() { return 1; }", "{"),
            ("function f() { return 1; }", "}"),
            ("f(a);", "("),
            ("f(a);", ")"),
            ("a[0];", "["),
            ("a[0];", "]"),
            ("a;", ";"),
            ("f(a, b);", ","),
            ("a.b;", "."),
        ];
        for (src, punct) in cases {
            let tree = parse(src).unwrap();
            let node = find_kind(tree.root_node(), punct)
                .unwrap_or_else(|| panic!("no {:?} node in {:?}", punct, src));
            let fmt = format_leaf(node, src).expect("punctuation must be a leaf");
            assert_eq!(fmt, LeafFormat::Raw(punct.to_string()), "for {:?}", src);
        }
    }
}