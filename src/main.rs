//@+leo-ver=5-thin
//@+node:ekr.20240927151332.1: * @file src/main.rs
//@@language rust
// test_lexer.main.rs

// From https://docs.rs/rustpython-parser/0.3.1/rustpython_parser/lexer/index.html

// Must be first.
#![allow(unused_imports)]

extern crate rustpython_parser;
use rustpython_parser::{lexer::lex, Mode};  // Tok, StringKind

#[macro_use]
extern crate fstrings;

fn main() {
    println!("");
    
    // 'x' @ 0..1
    // '=' @ 1..2

    let source = "x    =      'RustPython'";
    let tokens = lex(source, Mode::Module)
        .map(|tok| tok.expect("Failed to lex"))
        .collect::<Vec<_>>();

    // :? is debugging format.
    for (token, range) in tokens {
        println!(
            "{token:>20} @ {range:?}"
        );
    }
}
//@-leo
