//@+leo-ver=5-thin
//@+node:ekr.20240927151332.1: * @file src/main.rs
//@@language rust
// test_lexer.main.rs

// From https://docs.rs/rustpython-parser/0.3.1/rustpython_parser/lexer/index.html

// Must be first.
// #![allow(unused_imports)]
#![allow(unused_variables)] 

extern crate rustpython_parser;
use std::fs;
use std::time::{Instant};
use rustpython_parser::{lexer::lex, Mode}; // Tok, StringKind

#[macro_use]
extern crate fstrings;

fn main() {
    println!("");
    let t1 = Instant::now();
    // let source = "x    =      'RustPython'";
    let file_path = "C:\\Repos\\leo-editor\\leo\\core\\leoApp.py";
    let short_file_name = "leoApp.py";
    let contents = fs::read_to_string(file_path)
        .expect("Can not read file");

    // let tokens = lex(contents, Mode::Module)
    let tokens = lex(&contents, Mode::Module)
        .map(|tok| tok.expect("Failed to lex"))
        .collect::<Vec<_>>();

    // :? is debugging format.
    let mut n_tokens: usize = 0;
    for (_token, range) in tokens {  // Range is a TextRange.
        n_tokens += 1;
        // To do: Find gaps in the ranges.
        let start = range.start();  // TextSize's...
        let end = range.end();
        let start_s = f!("{start:?}");  // String's...
        let end_s = f!("{end:?}");
        let start_i: usize = start_s.parse().unwrap();
        let end_i: usize = end_s.parse().unwrap();
        // println!("{start_i:>2}..{end_i:2} token: {token}");
    }
    
    // Print time.
    let duration = t1.elapsed();
    println_f!("{short_file_name}: {n_tokens} tokens in {duration:?}\n");
}
//@-leo
