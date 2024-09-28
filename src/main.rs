//@+leo-ver=5-thin
//@+node:ekr.20240927151332.1: * @file src/main.rs
//@@language rust
// test_lexer.main.rs

// From https://docs.rs/rustpython-parser/0.3.1/rustpython_parser/lexer/index.html

// Must be first.
// #![allow(unused_imports)]
#![allow(unused_variables)]

extern crate rustpython_parser;
use rustpython_parser::{lexer::lex, Mode};
use std::fs;
use std::time::Instant; // Tok, StringKind

#[macro_use]
extern crate fstrings;

fn main() {
    println!("");
    let t1 = Instant::now();
    // let source = "x    =      'RustPython'";
    let file_path = "C:\\Repos\\leo-editor\\leo\\core\\leoApp.py";
    let short_file_name = "leoApp.py";
    let contents = fs::read_to_string(file_path).expect("Can not read file");

    // let tokens = lex(contents, Mode::Module)
    let tokens = lex(&contents, Mode::Module)
        .map(|tok| tok.expect("Failed to lex"))
        .collect::<Vec<_>>();

    // :? is debugging format.
    let mut n_tokens: usize = 0;
    for (token, range) in tokens {
        // Range is a TextRange.
        n_tokens += 1;
        // To do: Find gaps in the ranges.

        // These conversions are fast!
        let start_i = usize::from(range.start());
        let end_i = usize::from(range.end());
        
        if true {
            if n_tokens < 20 {
                println!("{start_i:>3}..{end_i:3} token: {token:?}");
            }
        }
    }

    // Print time.
    let duration = t1.elapsed();
    println_f!("{short_file_name}: {n_tokens} tokens in {duration:?}\n");
}
//@-leo
