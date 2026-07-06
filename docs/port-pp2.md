# Porting `pretty-printer-js/` to Rust — a 4-6 week tutorial

This tutorial maps each handler in `assets/pretty-printer-js/src/index.js` (the JS
pretty-printer built for OPP-024's Gumroad kit) to its Rust equivalent in this
scaffold. It is designed so a developer familiar with JS but new to Rust can
port the printer in 4-6 weeks of focused work.

## Stage 0 — What you already have

- `assets/pretty-printer-js/src/index.js` — a 350-line acorn-based AST
  pretty-printer in JavaScript. Pass-rate: 44/100 → 63/100 after the Day-8
  bug fixes (excluding `/tests/_errors_/`). 95% is the OPP-002 bounty bar.
- This scaffold (`examples/rust-scaffold/`) — a `cargo new` project with
  tree-sitter-typescript pinned and a `parse(src) -> AST` and `cursor_dump(tree) -> Vec<String>`
  function. Build with `cargo build`, run with `cargo run -- --file foo.js`.

## Stage 1 — Day 1 (already done in this scaffold)

Wire up tree-sitter-typescript. Dump the AST for any input file. No formatter
logic yet. This proves the toolchain.

Verify: `cargo run --quiet -- --file assets/pretty-printer-js/src/index.js`
prints 200+ AST lines.

## Stage 2 — Week 1: leaf node printers

Port the simplest handlers from pp2.js. Each handler is a function that takes
an AST node and returns a string. In JS (acorn), nodes are accessed via
`node.type` and `node.start`/`node.end`. In Rust (tree-sitter), nodes are
accessed via `node.kind()` (string), `node.child(i)` (field accessors), and
`&src[node.start_byte()..node.end_byte()]` for raw text.

| pp2.js handler (JS)             | tree-sitter node kind       | Rust equivalent (stub) |
|---|---|---|
| `printIdentifier(node)`         | `identifier`                | `Identifier` struct, `Display` impl |
| `printLiteral(node)`            | `number`, `string`, `true`, `false`, `null` | match on `node.kind()` |
| `printRegExpLiteral(node)`      | `regex`                     | raw text + re-emit |
| `printPrivateIdentifier(node)`  | `private_identifier`        | raw text |
| `printDirective(node)`          | `expression_statement` whose expr is a string literal starting with `"use "` | handle in ExpressionStatement arm |

Target after Week 1: every leaf node in your input prints as itself. 50/100
test cases pass.

## Stage 3 — Week 2: statement printers

| pp2.js handler           | tree-sitter kind            | Notes |
|---|---|---|
| `printStatement(node)`    | dispatch on first child     | match on `node.child(0).unwrap().kind()` |
| `printBlockStatement`     | `statement_block`           | needs `{`, indented children, `}` |
| `printEmptyStatement`     | `;` (empty statement)       | trivial |
| `printDebuggerStatement`  | `debugger_statement`        | trivial |
| `printExpressionStatement`| `expression_statement`      | call `printExpression` on first child |
| `printIfStatement`        | `if_statement`              | if/elif/else + braces |
| `printSwitchStatement`    | `switch_statement`          | switch_body with case nodes |
| `printTryStatement`       | `try_statement`             | try/catch/finally + clauses |
| `printForStatement`       | `for_statement`             | init/test/update + body |
| `printForInStatement`     | `for_in_statement`          | lhs in rhs body |
| `printForOfStatement`     | `for_of_statement`          | lhs of rhs body |
| `printWhileStatement`     | `while_statement`           | condition + body |
| `printDoStatement`        | `do_statement`              | body + `while (cond)` |
| `printReturnStatement`    | `return_statement`          | optional argument |
| `printThrowStatement`     | `throw_statement`           | argument required |
| `printBreakStatement`     | `break_statement`           | optional label |
| `printContinueStatement`  | `continue_statement`        | optional label |
| `printLabeledStatement`   | `labeled_statement`         | label + statement |
| `printImportDeclaration`  | `import_statement`          | specifiers + source |
| `printExportDeclaration`  | `export_statement`          | declaration vs default-clause |

Target after Week 2: 70/100 pass.

## Stage 4 — Week 3: expression printers (binary/unary/assignment)

| pp2.js handler               | tree-sitter kind       | Notes |
|---|---|---|
| `printExpression(node)`      | dispatch on kind       | ~30 kinds, mostly mechanical |
| `printBinaryExpression`      | `binary_expression`    | operator from `node.child_by_field_name("operator")` |
| `printLogicalExpression`     | `logical_expression`   | `&&` `\|\|` `??` |
| `printUnaryExpression`       | `unary_expression`     | `!` `-` `+` `~` `typeof` `void` `delete` |
| `printUpdateExpression`      | `update_expression`    | `++` `--` prefix/postfix |
| `printAssignmentExpression`  | `assignment_expression`| operator + right |
| `printConditionalExpression` | `ternary_expression`   | test ? cons : alt |
| `printSequenceExpression`    | `sequence_expression`  | comma-separated |
| `printYieldExpression`       | `yield_expression`     | optional delegate + argument |
| `printAwaitExpression`       | `await_expression`     | argument |
| `printChainExpression`       | `chain_expression`     | wraps optional chains in tree-sitter |

Target after Week 3: 82/100 pass.

## Stage 5 — Week 4: complex expressions + declarations

| pp2.js handler                | tree-sitter kind        | Notes |
|---|---|---|
| `printFunction`               | `function` / `function_declaration` / `arrow_function` | name, params, body |
| `printClass`                  | `class_declaration` / `class` | name, heritage, body |
| `printClassBody`              | `class_body`            | field + method definitions |
| `printMethodDefinition`       | `method_definition`     | the Day-8 bug-fix path applies here directly |
| `printNewExpression`          | `new_expression`        | constructor + arguments |
| `printCallExpression`         | `call_expression`       | callee + arguments |
| `printMemberExpression`       | `member_expression`     | object + property + computed |
| `printArrayExpression`        | `array`                 | `[elts]` |
| `printObjectExpression`       | `object`                | `{props}` — Day-8 "constructor" duplicate-key bug applies here |
| `printProperty`               | `pair` / `shorthand_property_identifier` | key + value + shorthand |
| `printTemplateLiteral`        | `template_string`       | quasi + expressions |
| `printTaggedTemplateExpression` | `tagged_template_expression` | tag + quasi |
| `printSpreadElement`          | `spread_element`        | `...x` |
| `printArrowFunctionExpression`| `arrow_function`        | params + `=>` + body |
| `printAssignmentPattern`      | `assignment_pattern`    | default parameter values |

Target after Week 4: 88/100 pass.

## Stage 6 — Week 5: JSX, TypeScript, comments, doc-printer

These are the 4 features that took Prettier 2 years to nail. You will not nail
them in a week. Target: 92/100 pass (one new pass per 1-2 days of focused work).

- **JSX**: `jsx_element`, `jsx_self_closing_element`, `jsx_attribute`, `jsx_expression`.
  Use `tree-sitter-typescript` with `tsx` parser. The hard part is preserving
  original attribute string values (no re-quote).
- **TypeScript**: `type_alias_declaration`, `interface_declaration`, `type_annotation`,
  `as_expression`. Same grammar; new node kinds only.
- **Comments**: walk the tree-sitter `extra` nodes (tree-sitter hides comments
  in the syntax tree by default; enable them via `parser.set_included_ranges` or
  post-process). Attach to nearest statement. Re-emit in original position.
- **Doc-printer**: Prettier's `doc` IR (concat, indent, group, line, softline,
  hardline). Tree-sitter gives you an AST; the doc-printer gives you a layout
  engine. Without this, you cannot do line wrapping. With this, you can wrap
  long function calls.

## Stage 7 — Week 6: operator precedence, quote selection, polish

- **Operator precedence**: rewrite `printBinaryExpression` to use Prettier's
  precedence table. Each binary op has a parenthesization rule based on the
  parent's precedence. This single change moves you from 92/100 to 95+/100.
- **Quote selection**: choose `"` vs `'` based on the source's existing choice.
  If the input has both, pick whichever has fewer escapes needed.
- **Blank-line preservation**: tree-sitter doesn't carry these. Diff the source
  to detect blank lines; emit them at the same positions.
- **Trailing newline**: ensure the output ends with exactly one `\n`.

Target after Week 6: 96+/100 pass → submit to OPP-002 Algora Prettier bounty.

## What this tutorial does NOT cover

- **Performance**. Tree-sitter is fast enough for files up to ~50K LOC. Larger
  files need a parallel parser. Not required for OPP-002.
- **Edge cases in modern JS** (private class fields, top-level await, dynamic
  import). Tree-sitter handles them; you just need to add node kinds.
- **Formatting of generated code** (auto-fix, prettier --write semantics). Not
  required for OPP-002 (read-only printer).

## What to ship

A `cargo run -- --file test.js` that, for ~95%+ of the test corpus, produces
output identical to `prettier --check`. Submit a PR to Algora's Prettier
challenge repo with the binary, the test report, and a 1-page memo.

## Kill criteria

If pass-rate is below 80/100 after Week 4, the port is not viable inside 4-6
weeks and you should pivot back to the JS pretty-printer as a Gumroad-only
asset. The JS version is still 63/100 valid-pass on Day 8 and can be sold
without the bounty entry.

---

*Tutorial version: 2026-07-03 18:05 UTC. Scaffold version: 0.1.0.*