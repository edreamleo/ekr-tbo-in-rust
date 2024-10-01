//@+leo-ver=5-thin
//@+node:ekr.20240928161210.1: * @file src/tbo.rs
// tbo.rs

// From https://docs.rs/rustpython-parser/0.3.1/rustpython_parser/lexer/index.html

// Must be first.
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

extern crate rustpython_parser;
use rustpython_parser::{lexer::lex, Mode, Tok, text_size::TextRange};
use std::env;  // For Beautifier.
use std::fmt;  // For InputTok.
use std::fs;
use std::path;
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
//@+node:ekr.20240929074037.1: ** class LeoBeautifier (new)
#[derive(Debug)]
pub struct Beautifier {
    args: Vec<String>,
    files_list: Vec<String>,
    input_list: Vec<InputTok>,
    output_list: Vec<String>,
    stats: Stats,
}

///// Temporary.
#[allow(dead_code)]
#[allow(non_snake_case)]
impl Beautifier {
    //@+others
    //@+node:ekr.20240929074037.114: *3*  LB::new
    pub fn new() -> Beautifier {
        let mut x = Beautifier {
            args: Vec::new(),
            files_list: Vec::new(),
            input_list: Vec::new(),
            output_list: Vec::new(),
            stats: Stats::new(),
        };
        x.get_args();
        return x;
    }
    //@+node:ekr.20240929074037.3: *3* LB::add_input_token
    // #[allow(dead_code)]
    fn add_input_token (&mut self, kind: &str, value: &str) {
        //! Add one token to the output list.
        self.input_list.push(InputTok {
            kind: kind.to_string(),
            value: value.to_string(),
        });
    }
    //@+node:ekr.20240929074037.2: *3* LB::add_output_string
    #[allow(unused_variables)]
    fn add_output_string (&mut self, kind: &str, value: &str) {
        //! Add one string to the output list.
        self.output_list.push(value.to_string())
    }
    //@+node:ekr.20240929074037.4: *3* LB::beautify_all_files
    pub fn beautify_all_files(&mut self) {
        // for file_name in self.files_list.clone() {
        for file_name in self.files_list.clone() {
            self.beautify_one_file(&file_name);
        }
    }

    //@+node:ekr.20240929074037.5: *3* LB::beautify_one_file
    fn beautify_one_file(&mut self, file_name: &str) {
        // Compute short_file_name from file_name.
        let file_path = path::Path::new(file_name);
        let os_str = file_path.file_name().unwrap(); // &OsStr
        let short_file_name = os_str.to_str().unwrap();
        // Read the file into contents (a String).
        self.output_list = Vec::new();
        let t1 = std::time::Instant::now();
        let contents = fs::read_to_string(file_name)
            .expect("Error reading{file_name}");
        // print_type(&contents, "contents");
        let read_time = t1.elapsed().as_micros();
        // Tokenize.
        let t3 = std::time::Instant::now();
        let n_tokens = self.tokenize_contents(contents);
        let make_tokens_time = t3.elapsed().as_micros();
        let write_time = 0;
        self.stats.update(n_tokens, make_tokens_time, read_time, write_time);
        // Report
        if true { // self.enabled("--report") {
            self.stats.report();
            // println!(" file name: {short_file_name}");
            // println!("      read: {:.2?}", t2);
            // println!("  tokenize: {:.2?}", t4);
            // println!("    tokens: {n_tokens}");
        }
        // Show tokens.
        //@+<< show output_list >>
        //@+node:ekr.20240929074037.6: *4* << show output_list >>
        if false {  // --show-output
            self.show_output_list()
        }
        //@-<< show output_list >>
    }
    //@+node:ekr.20240929074037.7: *3* LB::do_*
    //@+node:ekr.20240929074037.8: *4* LB:Handlers with values
    //@+node:ekr.20240929074037.9: *5* LB::do_Comment
    fn do_Comment(&mut self, tok_value: &str) {
        // print!("{tok_value}");  // Correct.
        // print!("{value} ");  // Wrong!
        self.add_output_string("Comment", tok_value);
    }
    //@+node:ekr.20240929074037.10: *5* LB::do_Complex
    fn do_Complex(&mut self, tok_value: &str) {
        self.add_output_string("Complex", tok_value);
    }
    //@+node:ekr.20240929074037.11: *5* LB::do_Float
    fn do_Float(&mut self, tok_value: &str) {
        self.add_output_string("Float", tok_value);
    }
    //@+node:ekr.20240929074037.12: *5* LB::do_Int
    fn do_Int(&mut self, tok_value: &str) {
        self.add_output_string("Int", tok_value);
    }
    //@+node:ekr.20240929074037.13: *5* LB::do_Name
    fn do_Name(&mut self, tok_value: &str) {
        self.add_output_string("Name", tok_value);
    }
    //@+node:ekr.20240929074037.14: *5* LB::do_String
    fn do_String(&mut self, tok_value: &str) {
        // correct.
        // print!("{tok_value}");
        
        // incorrect.
            // let quote = if *triple_quoted {"'''"} else {"'"};
            // print!("{:?}:{quote}{value}{quote}", kind);

        self.add_output_string("String", tok_value);
    }
    //@+node:ekr.20240929074037.15: *4* LB:Handlers using lws
    //@+node:ekr.20240929074037.16: *5* LB::do_Dedent
    fn do_Dedent(&mut self, tok_value: &str) {
        self.add_output_string("Dedent", tok_value);
    }
    //@+node:ekr.20240929074037.17: *5* LB::do_Indent
    fn do_Indent(&mut self, tok_value: &str) {
        self.add_output_string("Indent", tok_value);
    }
    //@+node:ekr.20240929074037.18: *5* LB::do_Newline
    fn do_Newline(&mut self) {
        self.add_output_string("Indent", "\n");
    }
    //@+node:ekr.20240929074037.19: *5* LB::do_NonLogicalNewline
    fn do_NonLogicalNewline(&mut self) {
        self.add_output_string("Indent", "\n");
    }
    //@+node:ekr.20240929074037.20: *4* LB:Handlers w/o values
    //@+node:ekr.20240929074037.21: *5* LB::do_Amper
    fn do_Amper(&mut self) {
        self.add_output_string("Amper", "&");
    }
    //@+node:ekr.20240929074037.22: *5* LB::do_AmperEqual
    fn do_AmperEqual(&mut self) {
        self.add_output_string("AmperEqual", "&=");
    }
    //@+node:ekr.20240929074037.23: *5* LB::do_And
    fn do_And(&mut self) {
        self.add_output_string("And", "and");
    }
    //@+node:ekr.20240929074037.24: *5* LB::do_As
    fn do_As(&mut self) {
        self.add_output_string("As", "as");
    }
    //@+node:ekr.20240929074037.25: *5* LB::do_Assert
    fn do_Assert(&mut self) {
        self.add_output_string("Assert", "assert");
    }
    //@+node:ekr.20240929074037.26: *5* LB::do_Async
    fn do_Async(&mut self) {
        self.add_output_string("Async", "async");
    }
    //@+node:ekr.20240929074037.27: *5* LB::do_At
    fn do_At(&mut self) {
        self.add_output_string("At", "@");
    }
    //@+node:ekr.20240929074037.28: *5* LB::do_AtEqual
    fn do_AtEqual(&mut self) {
        self.add_output_string("AtEqual", "@=");
    }
    //@+node:ekr.20240929074037.29: *5* LB::do_Await
    fn do_Await(&mut self) {
        self.add_output_string("Await", "await");
    }
    //@+node:ekr.20240929074037.30: *5* LB::do_Break
    fn do_Break(&mut self) {
        self.add_output_string("Break", "break");
    }
    //@+node:ekr.20240929074037.31: *5* LB::do_Case
    fn do_Case(&mut self) {
        self.add_output_string("Case", "case");
    }
    //@+node:ekr.20240929074037.32: *5* LB::do_CircumFlex
    fn do_CircumFlex(&mut self) {
        self.add_output_string("CircumFlex", "^");
    }
    //@+node:ekr.20240929074037.33: *5* LB::do_CircumflexEqual
    fn do_CircumflexEqual(&mut self) {
        self.add_output_string("CircumflexEqual", "^=");
    }
    //@+node:ekr.20240929074037.34: *5* LB::do_Class
    fn do_Class(&mut self) {
        self.add_output_string("Class", "class");
    }
    //@+node:ekr.20240929074037.35: *5* LB::do_Colon
    fn do_Colon(&mut self) {
        self.add_output_string("Colon", ":");
    }
    //@+node:ekr.20240929074037.36: *5* LB::do_ColonEqual
    fn do_ColonEqual(&mut self) {
        self.add_output_string("ColonEqual", ":=");
    }
    //@+node:ekr.20240929074037.37: *5* LB::do_Comma
    fn do_Comma(&mut self) {
        self.add_output_string("Comma", ",");
    }
    //@+node:ekr.20240929074037.38: *5* LB::do_Continue
    fn do_Continue(&mut self) {
        self.add_output_string("Continue", "continue");
    }
    //@+node:ekr.20240929074037.39: *5* LB::do_Def
    fn do_Def(&mut self) {
        self.add_output_string("Def", "def");
    }
    //@+node:ekr.20240929074037.40: *5* LB::do_Del
    fn do_Del(&mut self) {
        self.add_output_string("Del", "del");
    }
    //@+node:ekr.20240929074037.41: *5* LB::do_Dot
    fn do_Dot(&mut self) {
        self.add_output_string("Dot", ".");
    }
    //@+node:ekr.20240929074037.42: *5* LB::do_DoubleSlash
    fn do_DoubleSlash(&mut self) {
        self.add_output_string("DoubleSlash", "//");
    }
    //@+node:ekr.20240929074037.43: *5* LB::do_DoubleSlashEqual
    fn do_DoubleSlashEqual(&mut self) {
        self.add_output_string("DoubleSlashEqual", "//=");
    }
    //@+node:ekr.20240929074037.44: *5* LB::do_DoubleStar
    fn do_DoubleStar(&mut self) {
        self.add_output_string("DoubleStar", "**");
    }
    //@+node:ekr.20240929074037.45: *5* LB::do_DoubleStarEqual
    fn do_DoubleStarEqual(&mut self) {
        self.add_output_string("DoubleStarEqual", "**=");
    }
    //@+node:ekr.20240929074037.46: *5* LB::do_Elif
    fn do_Elif(&mut self) {
        self.add_output_string("Elif", "elif");
    }
    //@+node:ekr.20240929074037.47: *5* LB::do_Ellipsis
    fn do_Ellipsis(&mut self) {
        self.add_output_string("Ellipsis", "...");
    }
    //@+node:ekr.20240929074037.48: *5* LB::do_Else
    fn do_Else(&mut self) {
        self.add_output_string("Else", "else");
    }
    //@+node:ekr.20240929074037.49: *5* LB::do_EndOfFile
    fn do_EndOfFile(&mut self) {
        self.add_output_string("EndOfFile", "EOF");
    }
    //@+node:ekr.20240929074037.50: *5* LB::do_EqEqual
    fn do_EqEqual(&mut self) {
        self.add_output_string("EqEqual", "==");
    }
    //@+node:ekr.20240929074037.51: *5* LB::do_Equal
    fn do_Equal(&mut self) {
        self.add_output_string("Equal", "=");
    }
    //@+node:ekr.20240929074037.52: *5* LB::do_Except
    fn do_Except(&mut self) {
        self.add_output_string("Except", "except");
    }
    //@+node:ekr.20240929074037.53: *5* LB::do_False
    fn do_False(&mut self) {
        self.add_output_string("False", "False");
    }
    //@+node:ekr.20240929074037.54: *5* LB::do_Finally
    fn do_Finally(&mut self) {
        self.add_output_string("Finally", "finally");
    }
    //@+node:ekr.20240929074037.55: *5* LB::do_For
    fn do_For(&mut self) {
        self.add_output_string("For", "for");
    }
    //@+node:ekr.20240929074037.56: *5* LB::do_From
    fn do_From(&mut self) {
        self.add_output_string("From", "from");
    }
    //@+node:ekr.20240929074037.57: *5* LB::do_Global
    fn do_Global(&mut self) {
        self.add_output_string("Global", "global");
    }
    //@+node:ekr.20240929074037.58: *5* LB::do_Greater
    fn do_Greater(&mut self) {
        self.add_output_string("Greater", ">");
    }
    //@+node:ekr.20240929074037.59: *5* LB::do_GreaterEqual
    fn do_GreaterEqual(&mut self) {
        self.add_output_string("GreaterEqual", ">-");
    }
    //@+node:ekr.20240929074037.60: *5* LB::do_If
    fn do_If(&mut self) {
        self.add_output_string("If", "if");
    }
    //@+node:ekr.20240929074037.61: *5* LB::do_Import
    fn do_Import(&mut self) {
        self.add_output_string("Import", "import");
    }
    //@+node:ekr.20240929074037.62: *5* LB::do_In
    fn do_In(&mut self) {
        self.add_output_string("In", "in");
    }
    //@+node:ekr.20240929074037.63: *5* LB::do_Is
    fn do_Is(&mut self) {
        self.add_output_string("Is", "is");
    }
    //@+node:ekr.20240929074037.64: *5* LB::do_Lambda
    fn do_Lambda(&mut self) {
        self.add_output_string("Lambda", "lambda");
    }
    //@+node:ekr.20240929074037.65: *5* LB::do_Lbrace
    fn do_Lbrace(&mut self) {
        self.add_output_string("Lbrace", "[");
    }
    //@+node:ekr.20240929074037.66: *5* LB::do_LeftShift
    fn do_LeftShift(&mut self) {
        self.add_output_string("LeftShift", "<<");
    }
    //@+node:ekr.20240929074037.67: *5* LB::do_LeftShiftEqual
    fn do_LeftShiftEqual(&mut self) {
        self.add_output_string("LeftShiftEqual", "<<=");
    }
    //@+node:ekr.20240929074037.68: *5* LB::do_Less
    fn do_Less(&mut self) {
        self.add_output_string("Less", "<");
    }
    //@+node:ekr.20240929074037.69: *5* LB::do_LessEqual
    fn do_LessEqual(&mut self) {
        self.add_output_string("LessEqual", "<=");
    }
    //@+node:ekr.20240929074037.70: *5* LB::do_Lpar
    fn do_Lpar(&mut self) {
        self.add_output_string("Lpar", "(");
    }
    //@+node:ekr.20240929074037.71: *5* LB::do_Lsqb
    fn do_Lsqb(&mut self) {
        self.add_output_string("Lsqb", "[");
    }
    //@+node:ekr.20240929074037.72: *5* LB::do_Match
    fn do_Match(&mut self) {
        self.add_output_string("Match", "match");
    }
    //@+node:ekr.20240929074037.73: *5* LB::do_Minus
    fn do_Minus(&mut self) {
        self.add_output_string("Minus", "-");
    }
    //@+node:ekr.20240929074037.74: *5* LB::do_MinusEqual
    fn do_MinusEqual(&mut self) {
        self.add_output_string("MinusEqual", "-=");
    }
    //@+node:ekr.20240929074037.75: *5* LB::do_None
    fn do_None(&mut self) {
        self.add_output_string("None", "None");
    }
    //@+node:ekr.20240929074037.76: *5* LB::do_Nonlocal
    fn do_Nonlocal(&mut self) {
        self.add_output_string("Nonlocal", "nonlocal");
    }
    //@+node:ekr.20240929074037.77: *5* LB::do_Not
    fn do_Not(&mut self) {
        self.add_output_string("Not", "not");
    }
    //@+node:ekr.20240929074037.78: *5* LB::do_NotEqual
    fn do_NotEqual(&mut self) {
        self.add_output_string("NotEqual", "!=");
    }
    //@+node:ekr.20240929074037.79: *5* LB::do_Or
    fn do_Or(&mut self) {
        self.add_output_string("Or", "or");
    }
    //@+node:ekr.20240929074037.80: *5* LB::do_Pass
    fn do_Pass(&mut self) {
        self.add_output_string("Pass", "pass");
    }
    //@+node:ekr.20240929074037.81: *5* LB::do_Percent
    fn do_Percent(&mut self) {
        self.add_output_string("Percent", "%");
    }
    //@+node:ekr.20240929074037.82: *5* LB::do_PercentEqual
    fn do_PercentEqual(&mut self) {
        self.add_output_string("PercentEqual", "%=");
    }
    //@+node:ekr.20240929074037.83: *5* LB::do_Plus
    fn do_Plus(&mut self) {
        self.add_output_string("Plus", "+");
    }
    //@+node:ekr.20240929074037.84: *5* LB::do_PlusEqual
    fn do_PlusEqual(&mut self) {
        self.add_output_string("PlusEqual", "+=");
    }
    //@+node:ekr.20240929074037.85: *5* LB::do_Raise
    fn do_Raise(&mut self) {
        self.add_output_string("Raise", "raise");
    }
    //@+node:ekr.20240929074037.86: *5* LB::do_Rarrow
    fn do_Rarrow(&mut self) {
        self.add_output_string("Rarrow", "->");
    }
    //@+node:ekr.20240929074037.87: *5* LB::do_Rbrace
    fn do_Rbrace(&mut self) {
        self.add_output_string("Rbrace", "]");
    }
    //@+node:ekr.20240929074037.88: *5* LB::do_Return
    fn do_Return(&mut self) {
        self.add_output_string("Return", "return");
    }
    //@+node:ekr.20240929074037.89: *5* LB::do_RightShift
    fn do_RightShift(&mut self) {
        self.add_output_string("RightShift", ">>");
    }
    //@+node:ekr.20240929074037.90: *5* LB::do_RightShiftEqual
    fn do_RightShiftEqual(&mut self) {
        self.add_output_string("RightShiftEqual", ">>=");
    }
    //@+node:ekr.20240929074037.91: *5* LB::do_Rpar
    fn do_Rpar(&mut self) {
        self.add_output_string("Rpar", ")");
    }
    //@+node:ekr.20240929074037.92: *5* LB::do_Rsqb
    fn do_Rsqb(&mut self) {
        self.add_output_string("Rsqb", "]");
    }
    //@+node:ekr.20240929074037.93: *5* LB::do_Semi
    fn do_Semi(&mut self) {
        self.add_output_string("Semi", ";");
    }
    //@+node:ekr.20240929074037.94: *5* LB::do_Slash
    fn do_Slash(&mut self) {
        self.add_output_string("Slash", "/");
    }
    //@+node:ekr.20240929074037.95: *5* LB::do_SlashEqual
    fn do_SlashEqual(&mut self) {
        self.add_output_string("SlashEqual", "/=");
    }
    //@+node:ekr.20240929074037.96: *5* LB::do_Star
    fn do_Star(&mut self) {
        self.add_output_string("Star", "*");
    }
    //@+node:ekr.20240929074037.97: *5* LB::do_StarEqual
    fn do_StarEqual(&mut self) {
        self.add_output_string("StarEqual", "*=");
    }
    //@+node:ekr.20240929074037.98: *5* LB::do_StartExpression
    fn do_StartExpression(&mut self) {
        // self.add_output_string("StartExpression", "");
    }
    //@+node:ekr.20240929074037.99: *5* LB::do_StartInteractive
    fn do_StartInteractive(&mut self) {
        // self.add_output_string("StartModule", "");
    }
    //@+node:ekr.20240929074037.100: *5* LB::do_StarModule
    fn do_StartModule(&mut self) {
        // self.add_output_string("StartModule", "");
        println!("do_StartModule");
    }
    //@+node:ekr.20240929074037.101: *5* LB::do_Tilde
    fn do_Tilde(&mut self) {
        self.add_output_string("Tilde", "~");
    }
    //@+node:ekr.20240929074037.102: *5* LB::do_True
    fn do_True(&mut self) {
        self.add_output_string("True", "True");
    }
    //@+node:ekr.20240929074037.103: *5* LB::do_Try
    fn do_Try(&mut self) {
        self.add_output_string("Try", "try");
    }
    //@+node:ekr.20240929074037.104: *5* LB::do_Type
    fn do_Type(&mut self) {
        self.add_output_string("Type", "type");
    }
    //@+node:ekr.20240929074037.105: *5* LB::do_Vbar
    fn do_Vbar(&mut self) {
        self.add_output_string("Vbar", "|");
    }
    //@+node:ekr.20240929074037.106: *5* LB::do_VbarEqual
    fn do_VbarEqual(&mut self) {
        self.add_output_string("VbarEqual", "|=");
    }
    //@+node:ekr.20240929074037.107: *5* LB::do_While
    fn do_While(&mut self) {
        self.add_output_string("While", "while");
    }
    //@+node:ekr.20240929074037.108: *5* LB::do_With
    fn do_With(&mut self) {
        self.add_output_string("With", "with");
    }
    //@+node:ekr.20240929074037.109: *5* LB::do_Yield
    fn do_Yield(&mut self) {
        self.add_output_string("Yield", "yield");
    }
    //@+node:ekr.20240929074037.110: *3* LB::enabled
    fn enabled(&self, arg: &str) -> bool {
        //! Beautifier::enabled: return true if the given command-line argument is enabled.
        //! Example:  x.enabled("--report");
        return self.args.contains(&arg.to_string());

    }
    //@+node:ekr.20240929074037.111: *3* LB::get_args
    fn get_args(&mut self) {
        //! Beautifier::get_args: Set the args and files_list ivars.
        let args: Vec<String> = env::args().collect();
        let valid_args = vec![
            "--all", 
            "--beautified",
            "--diff",
            "-h", "--help",
            "--report",
            "--write",
        ];
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                if valid_args.contains(&arg.as_str()) {
                    self.args.push(arg.to_string())
                }
                else if 
                    arg.as_str().starts_with("--") ||
                    arg.as_str().starts_with("--")
                {
                    println!("Ignoring invalid arg: {arg}");
                }
                else {
                    println!("File: {arg}");
                    self.files_list.push(arg.to_string());
                }
            }
        }
    }
    //@+node:ekr.20240929074037.112: *3* LB::make_input_list
    fn make_input_list(&mut self, contents: &str) -> u128 {

        let mut count: u128 = 0;
        let results = lex(&contents, Mode::Module);  // An iterator yielding Option(Tok).
        for result in results {
            use Tok::*;
            count += 1;
            let token = result.ok().unwrap();
            let (ref tok_class, tok_range) = token;
            let tok_value = &contents[tok_range];

            // Variants names are necessary, but otherwise not used.
            #[allow(unused_variables)]
            let class_name = match tok_class {
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
            self.add_input_token(class_name, tok_value);
        }
        return count;
    }
    //@+node:ekr.20240929074037.113: *3* LB::make_output_list
    fn make_output_list(&mut self) {

        //// Prototype only.
        for input_token in &self.input_list.clone() {
            // println!("{:?}", input_token);
            self.add_output_string(input_token.kind.as_str(), input_token.value.as_str());
        }
    }
    //@+node:ekr.20240929074037.115: *3* LB::show_args
    fn show_args (&self) {
        println!("Command-line arguments...");
        for (i, arg) in self.args.iter().enumerate() {
            if i > 0 {
                println!("  {arg}");
            }
        }
        for file_arg in self.files_list.iter() {
            println!("  {file_arg}");
        }
    }
    //@+node:ekr.20240929074037.116: *3* LB::show_help
    fn show_help (&self) {
        //! Beautifier::show_help: print the help messages.
        println!("{}", textwrap::dedent("
            Beautify or diff files.

            -h --help:      Print this help message and exit.
            --all:          Beautify all files, even unchanged files.
            --beautified:   Report beautified files individually, even if not written.
            --diff:         Show diffs instead of changing files.
            --report:       Print summary report.
            --write:        Write beautifed files (dry-run mode otherwise).
        "));
    }
    //@+node:ekr.20240929074037.117: *3* LB::show_output_list
    fn show_output_list (&self) {
        println!("\nOutput list...");
        for (i, arg) in self.output_list.iter().enumerate() {
            if i > 0 {
                print!("{:?}", arg);
            }
        }
    }
    //@+node:ekr.20240929074037.118: *3* LB::tokenize_contents (prototype)
    fn tokenize_contents(&mut self, contents: String ) -> u128 {

        let count = self.make_input_list(&contents);
        
        // Simulate iterating the input list twice.
        for _z in &self.input_list.clone() {
        }
        for _z in &self.input_list.clone() {
        }

        // Simulate writing strings to the output list.
        // self.make_output_list();
        for _z in &self.input_list.clone() {
        }
        return count;
    }
    //@-others
}
//@+node:ekr.20240929074547.1: ** class Stats
#[derive(Debug)]
pub struct Stats {
    // Cumulative statistics for all files.
    n_files: u128,  // Number of files.
    n_tokens: u128, // Number of tokens.
    // Timing stat, in microseconds...
    make_tokens_time: u128,
    read_time: u128,
    write_time: u128,
}

// #[allow(dead_code)]
// #[allow(non_snake_case)]
impl Stats {
    //@+others
    //@+node:ekr.20241001100954.1: *3*  Stats::new
    pub fn new() ->Stats {
        let x = Stats {
            // Cumulative statistics for all files.
            n_files: 0,  // Number of files.
            n_tokens: 0, // Number of tokens.
            // Timing stat, in microseconds...
            make_tokens_time: 0,
            read_time: 0,
            write_time: 0,
        };
        return x;
    }
    //@+node:ekr.20240929080242.1: *3* Stats::fmt_ms
    fn fmt_ms(&mut self, t: u128) -> String {
        //! Convert microseconds to fractional milliseconds.
        let ms = t / 1000;
        let micro = (t % 1000) / 10;
        return f!("{ms}.{micro:02}");  // Two-digits for fraction.
    }

    //@+node:ekr.20240929075236.1: *3* Stats::report
    fn report (&mut self) {
        // Cumulative counts.
        let n_files = self.n_files;
        let n_tokens = self.n_tokens;
        // Print cumulative timing stats, in ms, using fmt_ms.
        let total_time = self.fmt_ms(self.make_tokens_time + self.read_time + self.write_time);
        let make_tokens_time = self.fmt_ms(self.make_tokens_time);
        let read_time = self.fmt_ms(self.read_time);
        let write_time = self.fmt_ms(self.write_time);
        println!("");
        println!("     tokens: {n_files}");
        println!("     tokens: {n_tokens}");
        println!("");
        println!("make_tokens: {make_tokens_time:>7} ms");
        println!("       read: {read_time:>7} ms");
        println!("      write: {write_time:>7} ms");
        println!("      total: {total_time:>7} ms");
    }
    //@+node:ekr.20240929074941.1: *3* Stats::update
    fn update (&mut self,
        n_tokens: u128,
        make_tokens: u128,
        read_time: u128,
        write_time: u128
    ) {
        // Update cumulative stats.
        self.n_files += 1;
        self.n_tokens += n_tokens;
        self.make_tokens_time += make_tokens;
        self.read_time += read_time;
        self.write_time += write_time;
    }
    //@-others
}
//@+node:ekr.20241001093308.1: ** pub fn entry & helpers
pub fn entry() {

    // Main line of beautifier.
    let mut x = Beautifier::new();
    if true {
        let file_path = "C:\\Repos\\leo-editor\\leo\\core\\leoFrame.py";
        x.beautify_one_file(&file_path);
    }
    else {
        if x.enabled("--help") || x.enabled("-h") {
            x.show_help();
            return;
        }
        x.show_args();
        x.beautify_all_files();
    }
}
//@+node:ekr.20241001093308.2: *3* fn tokenize
fn tokenize() {
    //@+<< tokenize: define contents >>
    //@+node:ekr.20241001093308.3: *4* << tokenize: define contents >>
    let contents = r#"
    def test():
    # Comment 1.
    print('abc')
    # Comment 2.
    "#;

    // print("xyz")
    // print(rf'pdb')
    // print(fr'pdb2')
    // return bool(i & 1)
    //@-<< tokenize: define contents >>
    println!("fn tokenize");
    println!("\nSource:\n{contents}");

    for debug in [true, false].iter() {

        println!("{}", if *debug {"Tokens..."} else {"\nBeautified:"});

        let results = lex(contents, Mode::Module);  // An iterator yielding Option(Tok).
        let mut count = 0;
        let mut lws = String::new();
        for (i, result) in results.enumerate() {
            use Tok::*;
            let token = result.ok().unwrap();
            let (ref tok_class, tok_range) = token;
            let tok_value = &contents[tok_range];

            if *debug {
                let s = format!("{tok_class}");
                print!("\nToken: {s:20} {:?}", tok_value);
            }
            else {
                // Comment(value), Name(name)
                #[allow(unused_variables)]
                match tok_class {
                    Comment(value) => {
                        // print!("{value} ");  // Wrong!
                        print!("{tok_value}");
                    },
                    Dedent => {
                        lws.pop();
                        lws.pop();
                        print!("{lws}");
                    },
                    Def => {
                        print!("{tok_value} ");
                    },
                    Indent => {
                        lws.push_str("    ");
                        print!("{lws}");
                    },
                    Name {name} => {
                        print!("{tok_value} ");
                    },
                    Newline => {
                        print!("{tok_value}");
                        print!("{lws}");
                        if false {  // old
                            println!("");
                            print!("{lws}");
                        }
                    },
                    NonLogicalNewline => {
                        println!("");
                        print!("{lws}");
                    },
                    Return => {
                        print!("{tok_value} ");
                    },
                    Tok::String {value, kind, triple_quoted} => {
                        // correct.
                        print!("{tok_value}");
                        if false {  // incorrect.
                            let quote = if *triple_quoted {"'''"} else {"'"};
                            print!("{:?}:{quote}{value}{quote}", kind);
                        }
                    },
                    _ => {
                        print!("{tok_value}");
                        if false {
                            // to_string quotes values!
                            let s = tok_class.to_string().replace("'", "");
                            print!("{s}");
                        }
                    },
                }
            }
            count = i
        }
        if *debug {
            println!("\n{count} tokens")
        }
    }
}
//@-others

//@-leo
