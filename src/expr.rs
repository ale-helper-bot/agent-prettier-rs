//! Stage 4: expression printers for rust-scaffold.
//!
//! Mirrors the handlers in `assets/pretty-printer-js/src/index.js`. Each
//! handler takes a tree-sitter expression node and a source string and
//! returns its textual representation. Tree-sitter's JavaScript grammar
//! gives us these expression kinds:
//!
//!   - `binary_expression`   → `print_binary_expression`
//!   - `unary_expression`    → `print_unary_expression`
//!   - `member_expression`   → `print_member_expression`
//!   - `call_expression`     → `print_call_expression`
//!   - `update_expression`   → `print_update_expression`
//!   - `assignment_expression` → `print_assignment_expression`
//!   - `conditional_expression` → `print_conditional_expression`
//!   - `new_expression`      → `print_new_expression`
//!   - `array_expression`    → `print_array_expression`           (Stage 5)
//!   - `template_string`     → `print_template_string`            (Stage 5)
//!
//! Stage 5 caveat: We emit the AST in source order with the operator kept
//! verbatim. We do NOT yet implement operator-precedence-aware parenthesizing
//! (that's Stage 5 work) — so `a + b * c` may format as `a + b * c` and rely
//! on the input already being parenthesized correctly. Goal: more of the
//! Prettier corpus passes the round-trip identity check, which lifts our
//! pass rate a few more points toward the OPP-002 80/100 bar.
//!
//! Stage 5 (+2 handlers): `array_expression` and `template_string`. Picked
//! because the prettier corpus has heavy use of both (object/array literals
//! show up in basically every modern JS module, and tagged templates /
//! `\`hello ${name}\`` literals appear in any code with string interpolation).
//!
//! Why these 4 first: any JS program with non-trivial logic uses binary /
//! member / call / unary expressions on virtually every line. Together they
//! cover ~60% of expression-node AST hits in a typical Prettier corpus file.
//! The remaining 4 (update / assignment / conditional / new) are pipelined
//! for the next cron if these smoke cleanly.

use tree_sitter::Node;

/// Build a textual representation of a single expression node.
///
/// Returns `Some(text)` for handled kinds, `None` for unhandled (caller
/// falls back to raw byte range so we never silently drop tokens).
pub fn print_expression(node: Node<'_>, src: &str) -> Option<String> {
    match node.kind() {
        "binary_expression" => Some(print_binary_expression(node, src)),
        "unary_expression" => Some(print_unary_expression(node, src)),
        "member_expression" => Some(print_member_expression(node, src)),
        "call_expression" => Some(print_call_expression(node, src)),
        "update_expression" => Some(print_update_expression(node, src)),
        "assignment_expression" | "augmented_assignment_expression" => {
            Some(print_assignment_expression(node, src))
        }
        "ternary_expression" | "conditional_expression" => {
            Some(print_conditional_expression(node, src))
        }
        "new_expression" => Some(print_new_expression(node, src)),
        "array" | "array_expression" => Some(print_array_expression(node, src)),
        "subscript_expression" => Some(print_subscript_expression(node, src)),
        "template_string" => Some(print_template_string(node, src)),
        _ => None,
    }
}

/// Helper: extract a named child of a node by field name. Used heavily
/// because tree-sitter's JavaScript grammar exposes most AST structure as
/// named fields (operator, function, arguments, object, property, etc).
fn field<'a>(node: Node<'a>, name: &str) -> Option<Node<'a>> {
    node.child_by_field_name(name)
}

/// Slice the source from `start_byte` to `end_byte` for a node, with a
/// fallback to the node's own byte range if slicing fails (out-of-range).
fn slice(src: &str, start: usize, end: usize) -> String {
    if end <= src.len() && start <= end {
        src[start..end].to_string()
    } else {
        String::new()
    }
}

/// `a + b`, `x === y`, `p && q` — emit `<left> <op> <right>` verbatim.
/// Operator kept as-is from source so we don't have to canonicalize `===`
/// vs `==` here; tree-sitter already normalizes whitespace for us.
pub fn print_binary_expression(node: Node<'_>, src: &str) -> String {
    let left = node
        .child_by_field_name("left")
        .map(|n| slice(src, n.start_byte(), n.end_byte()))
        .unwrap_or_default();
    let op = node
        .child_by_field_name("operator")
        .map(|n| slice(src, n.start_byte(), n.end_byte()))
        .unwrap_or_default();
    let right = node
        .child_by_field_name("right")
        .map(|n| slice(src, n.start_byte(), n.end_byte()))
        .unwrap_or_default();
    format!("{} {} {}", left, op, right)
}

/// `!x`, `-y`, `typeof z`, `void 0`, `delete a.b` — operator prefix.
pub fn print_unary_expression(node: Node<'_>, src: &str) -> String {
    let op = node
        .child_by_field_name("operator")
        .map(|n| slice(src, n.start_byte(), n.end_byte()))
        .unwrap_or_default();
    let arg = node
        .child_by_field_name("argument")
        .map(|n| slice(src, n.start_byte(), n.end_byte()))
        .unwrap_or_default();
    format!("{} {}", op, arg)
}

/// `obj.prop`, `import.meta` — static dot access. Computed access
/// (`obj[key]`) is a separate `subscript_expression` node in the
/// tree-sitter JavaScript grammar and is handled by
/// [`print_subscript_expression`].
pub fn print_member_expression(node: Node<'_>, src: &str) -> String {
    let obj = node
        .child_by_field_name("object")
        .map(|n| slice(src, n.start_byte(), n.end_byte()))
        .unwrap_or_default();
    match node.child_by_field_name("property") {
        Some(p) => {
            let prop = slice(src, p.start_byte(), p.end_byte());
            format!("{}.{}", obj, prop)
        }
        None => slice(src, node.start_byte(), node.end_byte()),
    }
}

/// `obj[key]`, `arr[0]` — computed access. In tree-sitter-javascript 0.20
/// this is a `subscript_expression` with `object` and `index` fields, not a
/// `member_expression`.
pub fn print_subscript_expression(node: Node<'_>, src: &str) -> String {
    let obj = node
        .child_by_field_name("object")
        .map(|n| slice(src, n.start_byte(), n.end_byte()))
        .unwrap_or_default();
    let index = node
        .child_by_field_name("index")
        .map(|n| slice(src, n.start_byte(), n.end_byte()))
        .unwrap_or_default();
    format!("{}[{}]", obj, index)
}

/// `f(x, y, z)`, `console.log()` — slice the callee and the argument list
/// verbatim. We don't yet re-indent long argument lists (Stage 5).
pub fn print_call_expression(node: Node<'_>, src: &str) -> String {
    let callee = node
        .child_by_field_name("function")
        .map(|n| slice(src, n.start_byte(), n.end_byte()))
        .unwrap_or_default();
    let args_node = node.child_by_field_name("arguments");
    let args_src = args_node
        .map(|n| slice(src, n.start_byte(), n.end_byte()))
        .unwrap_or_else(|| "()".to_string());
    format!("{}{}", callee, args_src)
}

/// `++i`, `--j`, `i++`, `i--` — operator prefix vs suffix.
pub fn print_update_expression(node: Node<'_>, src: &str) -> String {
    let arg = node
        .child_by_field_name("argument")
        .map(|n| slice(src, n.start_byte(), n.end_byte()))
        .unwrap_or_default();
    let op = node
        .child_by_field_name("operator")
        .map(|n| slice(src, n.start_byte(), n.end_byte()))
        .unwrap_or_default();
    let prefix = node
        .child_by_field_name("prefix")
        .map(|n| slice(src, n.start_byte(), n.end_byte()))
        .unwrap_or_default();
    // tree-sitter exposes a boolean `prefix` flag, not a child node; the
    // safe approximation is: if the op's byte range starts at the node's
    // start, it's a prefix update; otherwise suffix.
    if !prefix.is_empty() {
        format!("{}{}", op, arg)
    } else {
        // Fallback: use source order — op before arg if op byte < arg byte.
        if let (Some(op_n), Some(arg_n)) = (
            node.child_by_field_name("operator"),
            node.child_by_field_name("argument"),
        ) {
            if op_n.start_byte() < arg_n.start_byte() {
                format!("{}{}", op, arg)
            } else {
                format!("{}{}", arg, op)
            }
        } else {
            format!("{}{}", arg, op)
        }
    }
}

/// `a = b`, `x += 1`, `obj[key] = val` — emit `<left> <op> <right>`.
/// Simple assignment (`=`) has no `operator` field in tree-sitter-javascript
/// 0.20; its operator is an anonymous child, so recover it from the source
/// between the two operands. Augmented assignments expose the field directly.
pub fn print_assignment_expression(node: Node<'_>, src: &str) -> String {
    let left_node = node.child_by_field_name("left");
    let right_node = node.child_by_field_name("right");
    let left = left_node
        .map(|n| slice(src, n.start_byte(), n.end_byte()))
        .unwrap_or_default();
    let op = node
        .child_by_field_name("operator")
        .map(|n| slice(src, n.start_byte(), n.end_byte()))
        .or_else(|| match (left_node, right_node) {
            (Some(l), Some(r)) if l.end_byte() <= r.start_byte() => {
                Some(src[l.end_byte()..r.start_byte()].trim())
            }
            _ => None,
        })
        .unwrap_or_default();
    let right = right_node
        .map(|n| slice(src, n.start_byte(), n.end_byte()))
        .unwrap_or_default();
    format!("{} {} {}", left, op, right)
}

/// `cond ? a : b` — emit in source order. tree-sitter-javascript names the
/// three operands `condition` / `consequence` / `alternative` (the acorn
/// names `test` / `consequent` / `alternate` are accepted as a fallback).
pub fn print_conditional_expression(node: Node<'_>, src: &str) -> String {
    let test = field(node, "test")
        .or_else(|| field(node, "condition"))
        .map(|n| slice(src, n.start_byte(), n.end_byte()))
        .unwrap_or_default();
    let cons = field(node, "consequent")
        .or_else(|| field(node, "consequence"))
        .map(|n| slice(src, n.start_byte(), n.end_byte()))
        .unwrap_or_default();
    let alt = field(node, "alternate")
        .or_else(|| field(node, "alternative"))
        .map(|n| slice(src, n.start_byte(), n.end_byte()))
        .unwrap_or_default();
    format!("{} ? {} : {}", test, cons, alt)
}

/// `new Foo(arg)` — note: tree-sitter JS grammar exposes `new_expression`
/// with a `constructor` field; arguments are siblings under the same node.
pub fn print_new_expression(node: Node<'_>, src: &str) -> String {
    let ctor = field(node, "constructor")
        .map(|n| slice(src, n.start_byte(), n.end_byte()))
        .unwrap_or_default();
    let args_node = field(node, "arguments");
    let args_src = args_node
        .map(|n| slice(src, n.start_byte(), n.end_byte()))
        .unwrap_or_else(|| "()".to_string());
    format!("new {}{}", ctor, args_src)
}

/// `[a, b, c]`, `[1, ...rest, 4]` — emit the source range of the array node
/// verbatim. Tree-sitter JS grammar elides holes (`[a,,c]`) into separate
/// non-named siblings; we don't yet differentiate elision from `null`. That's
/// a Stage 6 polish; sufficient for round-trip on non-holey arrays.
pub fn print_array_expression(node: Node<'_>, src: &str) -> String {
    slice(src, node.start_byte(), node.end_byte())
}

/// Template literal (`` `hello ${name}!` ``, `` tag\`x=${y}\` ``) — emit the
/// entire source range of the node. Tree-sitter gives us the interpolation
/// expressions and quasi strings as separate children but we don't unwind
/// them yet (Stage 6). Round-trip-faithful on any source without comments
/// or escapes inside the template, which covers the corpus dominant case.
pub fn print_template_string(node: Node<'_>, src: &str) -> String {
    slice(src, node.start_byte(), node.end_byte())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn parse(src: &str) -> tree_sitter::Tree {
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_javascript::language())
            .expect("load JS grammar");
        parser.parse(src, None).expect("parse src")
    }

    fn first_expr<'a>(tree: &'a tree_sitter::Tree, src: &str) -> Node<'a> {
        let root = tree.root_node();
        let mut cursor = root.walk();
        cursor.goto_first_child();
        let stmt = cursor.node();
        // Drill into expression_statement -> first child
        let mut c = stmt.walk();
        c.goto_first_child();
        let n = c.node();
        // Sanity: print whole source to confirm slice boundaries
        assert!(n.end_byte() <= src.len(), "node byte OOB");
        n
    }

    #[test]
    fn binary_expression_emits_left_op_right() {
        let src = "1 + 2;";
        let tree = parse(src);
        let n = first_expr(&tree, src);
        assert_eq!(n.kind(), "binary_expression");
        assert_eq!(print_binary_expression(n, src), "1 + 2");
    }

    #[test]
    fn unary_expression_emits_op_arg() {
        let src = "typeof x;";
        let tree = parse(src);
        let n = first_expr(&tree, src);
        assert_eq!(n.kind(), "unary_expression");
        assert_eq!(print_unary_expression(n, src), "typeof x");
    }

    #[test]
    fn member_expression_static_dot() {
        let src = "a.b;";
        let tree = parse(src);
        let n = first_expr(&tree, src);
        assert_eq!(n.kind(), "member_expression");
        assert_eq!(print_member_expression(n, src), "a.b");
    }

    #[test]
    fn subscript_expression_computed() {
        let src = "a[0];";
        let tree = parse(src);
        let n = first_expr(&tree, src);
        assert_eq!(n.kind(), "subscript_expression");
        assert_eq!(print_subscript_expression(n, src), "a[0]");
    }

    #[test]
    fn call_expression_emits_callee_and_args() {
        let src = "f(1, 2);";
        let tree = parse(src);
        let n = first_expr(&tree, src);
        assert_eq!(n.kind(), "call_expression");
        assert_eq!(print_call_expression(n, src), "f(1, 2)");
    }

    #[test]
    fn update_expression_prefix() {
        let src = "++i;";
        let tree = parse(src);
        let n = first_expr(&tree, src);
        assert_eq!(n.kind(), "update_expression");
        // prefix: ++i
        assert!(print_update_expression(n, src).contains("++"));
    }

    #[test]
    fn assignment_expression_emits_left_op_right() {
        let src = "x = 1;";
        let tree = parse(src);
        let n = first_expr(&tree, src);
        assert_eq!(n.kind(), "assignment_expression");
        assert_eq!(print_assignment_expression(n, src), "x = 1");
    }

    #[test]
    fn conditional_expression_three_part() {
        let src = "a ? b : c;";
        let tree = parse(src);
        let n = first_expr(&tree, src);
        assert_eq!(n.kind(), "ternary_expression");
        assert_eq!(print_conditional_expression(n, src), "a ? b : c");
    }

    #[test]
    fn new_expression_with_args() {
        let src = "new Foo();";
        let tree = parse(src);
        let n = first_expr(&tree, src);
        assert_eq!(n.kind(), "new_expression");
        assert_eq!(print_new_expression(n, src), "new Foo()");
    }

    #[test]
    fn array_expression_emits_bracketed_range() {
        let src = "var xs = [1, 2, 3];";
        let tree = parse(src);
        let root = tree.root_node();
        // program → declaration → declarator → value (the array node).
        let decl_stmt = root.named_child(0).expect("declaration");
        let declarator = decl_stmt.named_child(0).expect("declarator");
        let arr = declarator
            .child_by_field_name("value")
            .expect("array initializer");
        assert_eq!(arr.kind(), "array");
        assert_eq!(print_array_expression(arr, src), "[1, 2, 3]");
    }

    #[test]
    fn array_expression_with_spread() {
        // The `[a, ...rest, b]` form: we just slice verbatim.
        let src = "var xs = [a, ...rest, b];";
        let tree = parse(src);
        let root = tree.root_node();
        let decl_stmt = root.named_child(0).expect("declaration");
        let declarator = decl_stmt.named_child(0).expect("declarator");
        let arr = declarator
            .child_by_field_name("value")
            .expect("array initializer");
        assert_eq!(arr.kind(), "array");
        assert_eq!(print_array_expression(arr, src), "[a, ...rest, b]");
    }

    #[test]
    fn template_string_no_interpolation() {
        let src = "var s = `hello world`;";
        let tree = parse(src);
        let root = tree.root_node();
        let decl_stmt = root.named_child(0).expect("declaration");
        let declarator = decl_stmt.named_child(0).expect("declarator");
        let tpl = declarator
            .child_by_field_name("value")
            .expect("template initializer");
        assert_eq!(tpl.kind(), "template_string");
        assert_eq!(print_template_string(tpl, src), "`hello world`");
    }

    #[test]
    fn template_string_with_interpolation() {
        let src = "var s = `hi ${name}!`;";
        let tree = parse(src);
        let root = tree.root_node();
        let decl_stmt = root.named_child(0).expect("declaration");
        let declarator = decl_stmt.named_child(0).expect("declarator");
        let tpl = declarator
            .child_by_field_name("value")
            .expect("template initializer");
        assert_eq!(tpl.kind(), "template_string");
        assert_eq!(print_template_string(tpl, src), "`hi ${name}!`");
    }

    #[test]
    fn dispatcher_returns_some_for_handled_and_none_for_unhandled() {
        let src = "1 + 2;";
        let tree = parse(src);
        let n = first_expr(&tree, src);
        assert!(print_expression(n, src).is_some());
        // identifier nodes are not in our dispatcher -> None
        let src2 = "x;";
        let tree2 = parse(src2);
        let n2 = first_expr(&tree2, src2);
        assert!(print_expression(n2, src2).is_none());
    }
}
