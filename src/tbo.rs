//@+leo-ver=5-thin
//@+node:ekr.20240928161210.1: * @file src/tbo.rs
// tbo.rs

// From https://docs.rs/rustpython-parser/0.3.1/rustpython_parser/lexer/index.html

// Must be first.
// #![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

extern crate rustpython_parser;
use rustpython_parser::{lexer::lex, Mode, Tok, text_size::TextRange};
use std::fmt;  // For InputTok.
use std::fs;
use std::time::Instant;

//@+others
//@+node:ekr.20240929024648.120: ** class InputTok
// Only Clone is valid for String.
#[derive(Clone)]
struct InputTok {
    kind: String,
    value: String,
}

impl fmt::Debug for InputTok {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind_s = format!("{:?}", self.kind);
        let mut value = self.value.to_string();
        if true {
            return write!(f, "{value} ");
        }
        else {  // Debug format.
            value.truncate(60);
            // repr format is not useful.
            // let value_s = format!("{:?}", value);
            let value_s = format!("{}", value);
            return write!(f, "InputTok: {kind_s:>10}: {value_s}");
        }
    }
}
//@+node:ekr.20240929033044.1: ** function: add_input_token (not used yet)
fn add_input_token (mut input_list: Vec<InputTok>, kind: &str, value: &str) {
    //! Add one token to the output list.
    // println!("{:?}", kind);

    let new_tok = InputTok {
        kind: kind.to_string(), value: value.to_string()
    };
    input_list.push(new_tok);
}
//@+node:ekr.20240929032636.1: ** function: entry
pub fn entry() {
    // Set file name. leoFrame.py is a typical size
    let file_path = "C:\\Repos\\leo-editor\\leo\\core\\leoFrame.py";
    let short_file_name = "leoFrame.py";
    // Read.
    let t1 = Instant::now();
    let contents = fs::read_to_string(file_path).expect("Can not read file");
    let read_time = fmt_ms(t1.elapsed().as_micros());
    // Lex.
    let t2 = Instant::now();
    let tokens = lex(&contents, Mode::Module)
        .map(|tok| tok.expect("Failed to lex"))
        .collect::<Vec<_>>();
    let lex_time = fmt_ms(t2.elapsed().as_micros());
    // Loop on tokens.
    let t3 = Instant::now();
    let input_list: Vec<InputTok> = Vec::new();
    // n_tokens = scan_input_list(contents, tokens);
    let n_tokens = make_input_list(contents, input_list, tokens);
    let loop_time = fmt_ms(t3.elapsed().as_micros());
    let total_time = fmt_ms(t1.elapsed().as_micros());

    // Sign on.
    println!("");
    println_f!("     tbo: {short_file_name}: {n_tokens} tokens\n");
    // Print stats.
    println_f!("    read: {read_time:>5} ms");
    println_f!("     lex: {lex_time:>5} ms");
    println_f!("    loop: {loop_time:>5} ms");
    println_f!("   total: {total_time:>5} ms");
}
//@+node:ekr.20240929032710.1: ** function: fmt_ms
fn fmt_ms(t: u128) -> String {
    //! Convert microseconds to fractional milliseconds.
    let ms = t / 1000;
    let micro = (t % 1000) / 10;
    return f!("{ms}.{micro:02}");  // Two-digits for fraction.
}

//@+node:ekr.20240929024648.113: ** function: make_input_list
fn make_input_list(
    contents: String,
    mut input_list: Vec<InputTok>,
    tokens: Vec<(Tok, TextRange)>
) -> usize {

    let mut count: usize = 0;
    for (token, range) in tokens { 
        use Tok::*;
        count += 1;
        let tok_value = &contents[range];

        // Variants names are necessary, but otherwise not used.
        #[allow(unused_variables)]
        
        let class_name = match token {
            // Tokens with values...
            // Use tok_value for *all* values.
            Comment(value) => "Comment",  // No idea why parens are needed here.
            Complex { real, imag } => "Complex",
            Float { value } => "Float",
            Int { value } => "Int",
            Name { name } => "Name",
            Tok::String { value, kind, triple_quoted } => "String",
            
            // Common tokens...
            Class => "Class",
            Dedent => "Dedent",
            Def => "Def",
            Indent => "Indent",
            Newline => "Newline",
            NonLogicalNewline => "NonLogicalNewline",

            // All other tokens...
            Amper => "Amper",
            AmperEqual => "AmperEqual",
            And => "And",
            As => "As",
            Assert => "Assert",
            Async => "Async",
            At => "At",
            AtEqual => "AtEqual",
            Await => "Await",
            Break => "Break",
            Case => "Case",
            CircumFlex => "CircumFlex",
            CircumflexEqual => "CircumflexEqual",
            Colon => "Colon",
            ColonEqual => "ColonEqual",
            Comma => "Comma",
            Continue => "Continue",
            Del => "Del",
            Dot => "Dot",
            DoubleSlash => "DoubleSlash",
            DoubleSlashEqual => "DoubleSlashEqual",
            DoubleStar => "DoubleStar",
            DoubleStarEqual => "DoubleStarEqual",
            Elif => "Elif",
            Ellipsis => "Ellipsis",
            Else => "Else",
            EndOfFile => "EndOfFile",
            EqEqual => "EqEqual",
            Equal => "Equal",
            Except => "Except",
            False => "False",
            Finally => "Finally",
            For => "For",
            From => "From",
            Global => "Global",
            Greater => "Greater",
            GreaterEqual => "GreaterEqual",
            If => "If",
            Import => "Import",
            In => "In",
            Is => "Is",
            Lambda => "Lambda",
            Lbrace => "Lbrace",
            LeftShift => "LeftShift",
            LeftShiftEqual => "LeftShiftEqual",
            Less => "Less",
            LessEqual => "LessEqual",
            Lpar => "Lpar",
            Lsqb => "Lsqb",
            Match => "Match",
            Minus => "Minus",
            MinusEqual => "MinusEqual",
            None => "None",
            Nonlocal => "Nonlocal",
            Not => "Not",
            NotEqual => "NotEqual",
            Or => "Or",
            Pass => "Pass",
            Percent => "Percent",
            PercentEqual => "PercentEqual",
            Plus => "Plus",
            PlusEqual => "PlusEqual",
            Raise => "Raise",
            Rarrow => "Rarrow",
            Rbrace => "Rbrace",
            Return => "Return",
            RightShift => "RightShift",
            RightShiftEqual => "RightShiftEqual",
            Rpar => "Rpar",
            Rsqb => "Rsqb",
            Semi => "Semi",
            Slash => "Slash",
            SlashEqual => "SlashEqual",
            Star => "Star",
            StarEqual => "StarEqual",
            StartExpression => "StartExpression",
            StartInteractive => "StartInteractive",
            StartModule => "StartModule",
            Tilde => "Tilde",
            True => "True",
            Try => "Try",
            Type => "Type",
            Vbar => "Vbar",
            VbarEqual => "VbarEqual",
            While => "While",
            With => "With",
            Yield => "Yield",
        };
        // add_input_token(input_list, class_name, tok_value);
        let new_tok = InputTok{
            kind: class_name.to_string(),value: tok_value.to_string()
        };
        input_list.push(new_tok);
    }
    return count;
}
//@+node:ekr.20240929031635.1: ** function: scan_input_list
fn scan_input_list(contents: String, tokens: Vec<(Tok, TextRange)>) -> usize {

    let mut count: usize = 0;
    for (token, range) in tokens {
        // Range is a TextRange.
        count += 1;
        // To do: Find gaps in the ranges.
        let start_i = usize::from(range.start());
        let end_i = usize::from(range.end());
        if false {
            if count < 20 {
                println_f!("{start_i:>3}..{end_i:3} token: {token:?}");
            }
        }
    }
    return count;
}
//@-others

//@-leo
