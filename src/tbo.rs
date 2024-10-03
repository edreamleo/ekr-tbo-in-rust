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
//@+node:ekr.20241003093554.1: **  pub fn entry
pub fn entry() {
    if true {
        test();
    } else {
        main();
    }
}
//@+node:ekr.20241003094145.1: **  struct TestTok
#[derive(Clone)]
#[derive(Debug)]
// Weird
#[allow(dead_code)]
pub struct TestTok {
    value: i32,
}
//@+node:ekr.20241003093722.1: ** fn main
pub fn main() {
    println!("main: not ready yet");
}
//@+node:ekr.20241001093308.1: ** fn test
fn test() {
    test_vec();
    test_struct();
}
//@+node:ekr.20241003094218.2: ** fn test_struct
fn test_struct() {
    // Test code for Vec.
    let mut v: Vec<TestTok> = Vec::new();
    push_struct(&mut v, 1);
    push_struct(&mut v, 2);
    for i in [3, 4, 6] {
        push_struct(&mut v, i);
    }
    let mut i = 7;
    while i < 10 {
        push_struct(&mut v, i);
        i += 1
    }
    // This fails
    // let mut tok = v[0];
    // tok.value = 666;
    // println!("{tok:?}");
    println!("");
    for z in v {
        println!("{z:?}");
    }
}

fn push_struct(v: &mut Vec<TestTok>, val: i32) {
    let mut tok = TestTok{value: 0};
    tok.value = val;  // To test mutability.
    v.push(tok);
}
//@+node:ekr.20241003094218.1: ** fn test_vec & push_vec
fn test_vec() {
    // Test code for Vec.
    let mut v: Vec<i32> = Vec::new();
    push_vec(&mut v, 1);
    push_vec(&mut v, 2);
    for i in [3, 4, 6] {
        push_vec(&mut v, i);
    }
    let mut i = 7;
    while i < 10 {
        push_vec(&mut v, i);
        i += 1
    }
    println!("");
    println!("v: {v:?}");
}

fn push_vec(v: &mut Vec<i32>, val: i32) {
    v.push(val);
}
//@-others

//@-leo
