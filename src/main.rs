//@+leo-ver=5-thin
//@+node:ekr.20240927151332.1: * @file src/main.rs
//@@language rust
// test_lexer.main.rs

// From https://docs.rs/rustpython-parser/0.3.1/rustpython_parser/lexer/index.html

// Must be first.
#![allow(unused_imports)]

extern crate rustpython_parser;
use rustpython_parser::{lexer::lex, Mode}; // Tok, StringKind
use std::fmt;

#[macro_use]
extern crate fstrings;

fn main() {
    println!("");
    let source = "x    =      'RustPython'";
    let tokens = lex(source, Mode::Module)
        .map(|tok| tok.expect("Failed to lex"))
        .collect::<Vec<_>>();

    // :? is debugging format.
    for (token, range) in tokens {  // Range is a TextRange.
        // let (start, end) = (range.start(), range.end());  // TextSize's.
        // let (start_s, end_s) = (f!("{start:?}"), f!("{end:?}"));  // String's.
        let start = range.start();  // TextSize's...
        let end = range.end();
        let start_s = f!("{start:?}");  // String's...
        let end_s = f!("{end:?}");
        let start_i: usize = start_s.parse().unwrap();
        let end_i: usize = end_s.parse().unwrap();
        // println!("{start_s:>2}..{end_s:2} token: {token}");
        println!("{start_i:>2}..{end_i:2} token: {token}");
    }
}
//@-leo
