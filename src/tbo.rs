//@+leo-ver=5-thin
//@+node:ekr.20240928161210.1: * @file src/tbo.rs
// tbo.rs

// From https://docs.rs/rustpython-parser/0.3.1/rustpython_parser/lexer/index.html

// Must be first.
// #![allow(unused_imports)]
// #![allow(dead_code)]
#![allow(unused_variables)]

extern crate rustpython_parser;
use rustpython_parser::{lexer::lex, Mode};
use std::fs;
use std::time::Instant;  // {Duration, Instant}, 

fn fmt_ms(t: u128) -> String {
    //! Convert a time in microsecond to fractional millisecons.
    let ms = t / 1000;
    let micro = (t % 1000) / 10;
    return f!("{ms}.{micro}");
}

pub fn entry() {
    // Sign on.
    println!("");
    let file_path = "C:\\Repos\\leo-editor\\leo\\core\\leoApp.py";
    let short_file_name = "leoApp.py";
    println_f!("     tbo: {short_file_name}\n");

    // Read leoApp.py.
    let t1 = Instant::now();
    let contents = fs::read_to_string(file_path).expect("Can not read file");
    let read_time = t1.elapsed().as_micros();
    let read_ms = fmt_ms(read_time);
    println_f!("    read: {read_ms:>6} ms");

    // Tokenize.
    let t2 = Instant::now();
    let tokens = lex(&contents, Mode::Module)
        .map(|tok| tok.expect("Failed to lex"))
        .collect::<Vec<_>>();
    let tokenize_time = t1.elapsed().as_micros();
    let tokenize_ms = fmt_ms(tokenize_time);
    println_f!("tokenize: {tokenize_ms:>6} ms");

    // :? is debugging format.
    let t3 = Instant::now();
    let mut n_tokens: usize = 0;
    for (token, range) in tokens {
        // Range is a TextRange.
        n_tokens += 1;
        // To do: Find gaps in the ranges.

        // These conversions are fast!
        let start_i = usize::from(range.start());
        let end_i = usize::from(range.end());
        
        if false {
            if n_tokens < 20 {
                println_f!("{start_i:>3}..{end_i:3} token: {token:?}");
            }
        }
    }

    // Print stats.
    let loop_time = t3.elapsed().as_micros();
    let total_time = t1.elapsed().as_micros();
    let loop_ms = fmt_ms(loop_time);
    let total_ms = fmt_ms(total_time);
    println_f!("    loop: {loop_ms:>6} ms");
    println_f!("   total: {total_ms:>6} ms");
    println!("");
    println_f!("    {n_tokens} tokens");
}
//@-leo
