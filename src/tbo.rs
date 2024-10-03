//@+leo-ver=5-thin
//@+node:ekr.20240928161210.1: * @file src/tbo.rs
// tbo.rs

// From https://docs.rs/rustpython-parser/0.3.1/rustpython_parser/lexer/index.html

// Must be first.

// #![allow(unused_imports)]
// #![allow(dead_code)]
// #![allow(unused_variables)]

// extern crate rustpython_parser;
// use rustpython_parser::{lexer::lex, Mode, Tok}; // text_size::TextRange
// use std::env;
// use std::fmt;
// use std::fs;
// use std::path;

//@+others
//@+node:ekr.20241001093308.1: ** pub fn entry (test)
pub fn entry() {
    // Test code for Vec.
    let mut v: Vec<i32> = Vec::new();
    push(&mut v, 1);
    push(&mut v, 2);
    let mut i = 3;
    while i < 10 {
        push(&mut v, i);
        i += 1
    }
    println!("");
    println!("v: {v:?}");
}

fn push(v: &mut Vec<i32>, val: i32) {
    v.push(val);
}
//@-others

//@-leo
