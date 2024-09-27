// test_lexer.main.rs

// From https://docs.rs/rustpython-parser/0.3.1/rustpython_parser/lexer/index.html

extern crate rustpython_parser;

use rustpython_parser::{lexer::lex, Mode};  // Tok, StringKind

fn main() {
    println!("Hello from test_lexer/main.rs");

    let source = "x = 'RustPython'";
    let tokens = lex(source, Mode::Module)
        .map(|tok| tok.expect("Failed to lex"))
        .collect::<Vec<_>>();

    for (token, range) in tokens {
        println!(
            "{token:?}@{range:?}",
        );
    }
}