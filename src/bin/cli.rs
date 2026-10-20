//! rust-scaffold CLI: parse a source file and dump its tree-sitter AST.
//!
//! Usage:
//!   cargo run --quiet -- --file path/to/source.js
//!   cargo run --quiet -- --dump ast --file path/to/source.js
//!
//! Exits 0 if the parser produced a tree, 1 if not.

use clap::Parser as ClapParser;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use agent_prettier_rs::{cursor_dump, parse};

#[derive(ClapParser, Debug)]
#[command(name = "rust-scaffold", version, about = "Tree-sitter AST dumper (Stage 1 scaffold for OPP-002 Prettier re-entry)")]
struct Args {
    /// Path to a JavaScript or TypeScript source file.
    #[arg(long)]
    file: PathBuf,

    /// What to emit. Currently only "ast" is supported.
    #[arg(long, default_value = "ast")]
    dump: String,
}

fn main() -> ExitCode {
    let args = Args::parse();

    let src = match fs::read_to_string(&args.file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: failed to read {}: {}", args.file.display(), e);
            return ExitCode::from(2);
        }
    };

    match args.dump.as_str() {
        "ast" => match parse(&src) {
            Some(tree) => {
                for line in cursor_dump(tree.root_node()) {
                    println!("{}", line);
                }
                ExitCode::SUCCESS
            }
            None => {
                eprintln!("error: parser returned no tree");
                ExitCode::from(1)
            }
        },
        other => {
            eprintln!("error: unknown --dump value: {}", other);
            ExitCode::from(2)
        }
    }
}