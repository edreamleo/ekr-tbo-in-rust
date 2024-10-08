//@+leo-ver=5-thin
//@+node:ekr.20240928161210.1: * @file src/tbo.rs
// tbo.rs

// From https://docs.rs/rustpython-parser/0.3.1/rustpython_parser/lexer/index.html

// Must be first.
// #![allow(dead_code)]
// #![allow(unused_imports)]
// #![allow(unused_variables)]

extern crate rustpython_parser;
use rustpython_parser::{lexer::lex, Mode, Tok}; // text_size::TextRange
use std::env;
use std::fs;
use std::collections::HashMap;
use std::path;

//@+others
//@+node:ekr.20241004095931.1: ** class AnnotatedInputTok
// *** Strange.
#[allow(dead_code)]
#[derive(Debug)]
struct AnnotatedInputTok<'a> {
    // context: String,
    context: &'a str,
    kind: &'a str,
    value: &'a str,
}

impl <'a> AnnotatedInputTok<'_> {
    fn new(context: &'a str, kind: &'a str, value: &'a str) -> AnnotatedInputTok<'a> {
        AnnotatedInputTok {
            context: context,
            kind: kind,
            value: value,
        }
    }
}
//@+node:ekr.20241004110721.1: ** class Annotator
#[allow(dead_code)]
struct Annotator<'a> {
    // Classes of tokens
    insignificant_tokens: [&'a str; 7],
    op_kinds: [&'a str; 29],
    // The present input token...
    input_tokens: &'a Vec<InputTok<'a>>,
    index: u32,  // The index within the tokens array of the token being scanned.
    index_dict: HashMap<usize, String>,
    lws: String,  // Leading whitespace. Required!
    // For whitespace.
    curly_brackets_level: u32,  // Number of unmatched '{' tokens.
    paren_level: u32,  // Number of unmatched '(' tokens.
    square_brackets_stack: Vec<bool>,  // A stack of bools, for     gen_word().
    indent_level: u32,  // Set only by do_indent and do_dedent.
    // Parse state.
    decorator_seen: bool,  // Set by do_name for do_op.
    in_arg_list: u32,  // > 0 if in an arg list of a def.
    in_doc_part: bool,
    state_stack: Vec<ParseState>,  // Stack of ParseState objects.
    valid_contexts: [&'a str; 8],  // *** will be 7 w/o the "test" context.
    verbatim: bool,  // True: don't beautify.
}

impl Annotator<'_> {
    //@+others
    //@+node:ekr.20241004153742.1: *3*  Annotator.new
    fn new<'a>(input_tokens: &'a Vec<InputTok>) -> Annotator<'a> {
        Annotator {
            curly_brackets_level: 0,
            decorator_seen: false,
            in_arg_list: 0,  // > 0 if in an arg list of a def.
            in_doc_part: false,
            indent_level: 0,
            index: 0,
            index_dict: HashMap::new(),
            input_tokens: input_tokens,
            insignificant_tokens: [
                //@+<< define Annotator::insignificant_tokens >>
                //@+node:ekr.20241007085552.1: *4* << define Annotator::insignificant_tokens >>
                "dummy", // placeholder so the token stack is never empty.
                "ws",  // pseudo-token.
                "Comment", "Dedent", "Indent", "Newline", "Nl",  // Real tokens.
                //@-<< define Annotator::insignificant_tokens >>
            ],
            lws: String::new(),
            op_kinds: [
                //@+<< define Annotator::op_kinds >>
                //@+node:ekr.20241007085705.1: *4* << define Annotator::op_kinds >>
                "And",
                "Colon", "ColonEqual", "Comma", "Dot", "DoubleStar",
                "Equal", "EqEqual", "Greater", "GreaterEqual",
                "Is",
                "Less", "LessEqual", "Lbrace", "Lpar", "Lsqb",
                "Minus", "MinusEqual",
                "Not", "NotEqual",
                "Or",
                "Percent", "Plus", "PlusEqual",
                "Rarrow", "Rbrace", "Rpar", "Rsqb",
                "Star",
                //@-<< define Annotator::op_kinds >>
            ],
            paren_level: 0,
            state_stack: Vec::new(),
            square_brackets_stack: Vec::new(),
            valid_contexts: [
                "test",  // *** testing only.
                "annotation", "arg", "complex-slice", "dict",
                "import", "initializer", "simple-slice"],
            verbatim: false, 
        }
    }
    //@+node:ekr.20241004095735.1: *3* Annotator.annotate
    fn annotate(&mut self) -> Vec::<AnnotatedInputTok> {
        //! Do the prepass, returning tokens annotated with context.
        let mut result = Vec::new();

        // Create self.index_dict.
        self.pre_scan();

        // Create the annotated tokens using self.index_dict.
        {
            let input_tokens_len = self.input_tokens.len();
            let dict_len = &self.index_dict.len();
            println!("");
            println!("annotate: self.input_tokens.len(): {input_tokens_len}");
            println!("annotate: self.index_dict: {dict_len}");
            println!("");
        }
        for (i, token) in self.input_tokens.into_iter().enumerate() {
            // *** println!("annotate: token: {token:?}");
            let context = match self.index_dict.get(&i) {
                Some(x) => x,
                None => "",
            };
            // *** println!("annotate: context: {context:?}");
            let annotated_token = AnnotatedInputTok::new(&context, &token.kind, &token.value);
            // ***
            // let annotated_token = AnnotatedInputTok {
                // // context: context.to_string(), kind: &token.kind, value: &token.value
                // context: context, kind: &token.kind, value: &token.value
            // };
            result.push(annotated_token);
        }
        return result;
    }
    //@+node:ekr.20241004153802.1: *3* Annotator.pre_scan & helpers
    fn pre_scan(&mut self) {
        //! Scan the entire file in one iterative pass, adding context (in self.index_dict)
        //! to a few kinds of tokens as follows:
        //!
        //! Token   Possible Contexts (or None)
        //! =====   ===========================
        //! ":"     "annotation", "dict", "complex-slice", "simple-slice"
        //! "="     "annotation", "initializer"
        //! "*"     "arg"
        //! "**"    "arg"
        //! "."     "import"

        // Push a dummy token on the scan stack so it is never empty.
        let mut scan_stack: Vec<ScanState> = Vec::new();
        let dummy_token = InputTok::new(0, "dummy", "");
        let dummy_state = ScanState::new("dummy-scan-state", &dummy_token);
        scan_stack.push(dummy_state);
        // Init prev_token to a dummy value.
        let mut prev_token = &dummy_token;
        // The main loop...
        let mut in_import = false;
        for (i, token) in self.input_tokens.into_iter().enumerate() {
            let (kind, value) = (token.kind, token.value);
            if false {  // *** Testing only
                self.set_context(i, "test");
            }
            // println!("pre_scan: {kind:>20} {value:?}");
            if kind == "Newline" {
                //@+<< pre-scan newline tokens >>
                //@+node:ekr.20241004154345.2: *4* << pre-scan newline tokens >>
                // "import" and "from x import" statements may span lines.
                // "ws" tokens represent continued lines like this:   ws: " \\\n    "

                if in_import && scan_stack.len() == 0 {
                    in_import = false;
                }
                //@-<< pre-scan newline tokens >>
            }
            else if self.op_kinds.contains(&kind) {
                // *** println!("   OP: kind: {kind:>12} value: {value:?}");  // ***
                //@+<< pre-scan op tokens >>
                //@+node:ekr.20241004154345.3: *4* << pre-scan op tokens >>
                // top_state: Optional[ScanState] = scan_stack[-1] if scan_stack else None
                let mut top_state = scan_stack[scan_stack.len() - 1].clone();

                if false {  // ***
                    println!("   OP: kind: {kind:>12} value: {value:?}");
                }

                // Handle "[" and "]".
                if value == "[" {
                    scan_stack.push(ScanState::new("slice", &token));
                }
                else if  value == "]" {
                    assert!(top_state.kind == "slice");
                    self.finish_slice(i, &top_state);
                    scan_stack.pop();
                }

                // Handle "{" and "}".
                if value == "{" {
                    scan_stack.push(ScanState::new("dict", &token));
                }
                else if value == "}" {
                    assert!(top_state.kind == "dict");
                    self.finish_dict(i, &top_state);
                    scan_stack.pop();
                }

                // Handle "(" and ")"
                else if value == "(" {
                    if is_python_keyword(&prev_token) || prev_token.kind != "name" {
                        scan_stack.push(ScanState::new("(", &token));
                    }
                    else {
                        scan_stack.push(ScanState::new("arg", &token));
                    }
                }
                else if value == ")" {
                    assert!(["arg", "("].contains(&top_state.kind));
                    if top_state.kind == "arg" {
                        self.finish_arg(i, &top_state);
                    }
                    scan_stack.pop();
                }

                // Handle interior tokens in "arg" and "slice" states.
                if true { // *** top_state.kind != "dummy-scan-state" {
                    if value == ":" && ["dict", "slice"].contains(&top_state.kind) {
                        top_state.indices.push(i);
                    }
                    //  *** else if top_state.kind == "arg" && ["**", "*", "=", ":", ","].contains(&value) {
                    else if ["**", "*", "=", ":", ","].contains(&value) {
                        println!("FOUND: kind: {kind:>12} value: {value:?}");
                        top_state.indices.push(i);
                    }
                }

                // Handle "." and "(" tokens inside "import" and "from" statements.
                if in_import && ["(", "."].contains(&value) {
                    self.set_context(i, "import");
                }
                //@-<< pre-scan op tokens >>
            }
            else if kind == "Name" {
                // println!("Name: {value:?}");  // ***
                //@+<< pre-scan name tokens >>
                //@+node:ekr.20241004154345.4: *4* << pre-scan name tokens >>
                // *** Python
                    // *** WRONG: in Rust, "From" and "Import" are separate tokens.

                    // prev_is_yield = prev_token and prev_token.kind == 'name' and prev_token.value == 'yield'
                    // if value in ('from', 'import') and not prev_is_yield:
                        // # 'import' and 'from x import' statements should be at the outer level.
                        // assert not scan_stack, scan_stack
                        // in_import = True
                        

                let prev_is_yield = prev_token.kind == "name" && prev_token.value == "yield";
                if !prev_is_yield && (value == "from" || value == "import") {
                    // "import" and "from x import" statements should be at the outer level.
                    assert!(scan_stack.len() == 1 && scan_stack[0].kind == "dummy");
                    in_import = true;
                }
                //@-<< pre-scan name tokens >>
            }
            else if ["Class", "Def"].contains(&kind) {
                // println!("{kind}");
                self.set_context(i, "test");
            }
            else if kind == "ws" {  // ***
            }
            else {
                // println!("Other: {kind:?}");
            }
            // Remember the previous significant token.
            if !self.insignificant_tokens.contains(&kind) {
                prev_token = &token;
            }
        }
        // Sanity check.
        if scan_stack.len() > 1 || scan_stack[0].kind != "dummy-scan-state" {
            println!("");
            println!("pre_scan: non-empty scan_stack");
            for scan_state in scan_stack {
                println!("{scan_state:?}");
            }
        }
    }
    //@+node:ekr.20241004154345.5: *4* Annotator.finish_arg***
    // *** Python
    // def finish_arg(self, end: int, state: Optional[ScanState]) -> None:
        // """Set context for all ':' when scanning from '(' to ')'."""

        // # Sanity checks.
        // if not state:
            // return
        // assert state.kind == 'arg', repr(state)
        // token = state.token
        // assert token.value == '(', repr(token)
        // values = state.value
        // assert isinstance(values, list), repr(values)
        // i1 = token.index
        // assert i1 < end, (i1, end)
        // if not values:
            // return

        // # Compute the context for each *separate* '=' token.
        // equal_context = 'initializer'
        // for i in values:
            // token = self.input_tokens[i]
            // assert token.kind == 'op', repr(token)
            // if token.value == ',':
                // equal_context = 'initializer'
            // elif token.value == ':':
                // equal_context = 'annotation'
            // elif token.value == '=':
                // self.set_context(i, equal_context)
                // equal_context = 'initializer'

        // # Set the context of all outer-level ':', '*', and '**' tokens.
        // prev: Optional[InputToken] = None
        // for i in range(i1, end):
            // token = self.input_tokens[i]
            // if token.kind not in self.insignificant_kinds:
                // if token.kind == 'op':
                    // if token.value in ('*', '**'):
                        // if self.is_unary_op_with_prev(prev, token):
                            // self.set_context(i, 'arg')
                    // elif token.value == '=':
                        // # The code above has set the context.
                        // assert token.context in ('initializer', 'annotation'), (i, repr(token.context))
                    // elif token.value == ':':
                        // self.set_context(i, 'annotation')
                // prev = token

    fn finish_arg(&mut self, end: usize, state: &ScanState) {
        //! Set context for all ":" when scanning from "(" to ")".
        
        println!("finish_arg: {end} {state:?}");
        
        // Sanity checks.
        if state.kind == "dummy" {
            println!("finish_arg: dummy state!");
            return
        }
        assert!(state.kind == "arg");
        assert!(state.token.value == "(");
        if state.indices.len() == 0 {
            return;
        }
        let i1 = state.indices[0];
        assert!(i1 < end);

        // Compute the context for each *separate* "=" token.
        let mut equal_context = "initializer";
        for i in state.indices.clone() {
            let token: &InputTok = &self.input_tokens[i];
            println!("finish_arg: {i} {token:?}");
            assert!(token.kind == "op");
            if token.value == "," {
                equal_context = "initializer";
            }
            else if token.value == ":" {
                equal_context = "annotation";
            }
            else if token.value == "=" {
                self.set_context(i, equal_context);
                equal_context = "initializer";
            }
        }
        // Set the context of all outer-level ":", "*", and "**" tokens.
        let mut prev_token = &InputTok::new(0, "dummy", "");
        for i in i1..end {
            let token = &self.input_tokens[i];
            if !self.insignificant_tokens.contains(&token.kind) {
                if token.kind == "op" {
                    if ["*", "**"].contains(&token.value) {
                        if is_unary_op_with_prev(&prev_token, &token) {
                            self.set_context(i, "arg");
                        }
                    }
                    else if token.value == "=" {
                        // The code above has set the context.
                        // assert token.context in ("initializer", "annotation"), (i, repr(token.context))
                    }
                    else if token.value == ":" {
                        self.set_context(i, "annotation")
                    }
                }
                prev_token = token;
            }
        }
    }
    //@+node:ekr.20241004154345.6: *4* Annotator.finish_slice
    fn finish_slice(&mut self, end: usize, state: &ScanState) {
        //! Set context for all ":" when scanning from "[" to "]".

        // Sanity checks.
        assert!(state.kind == "slice");
        
        let token = state.token;
        assert!(token.value == "[");

        let indices = &state.indices;
        
        // *** let mut i1 = token.index;
        let i1 = 0;
        // assert i1 < end, (i1, end)

        // Do nothing if there are no ":" tokens in the slice.
        if indices.len() == 0 {
            return;
        }

        // Compute final context by scanning the tokens.
        let mut final_context = "simple-slice";
        let mut inter_colon_tokens = 0;
        let mut prev_token = token;
        for i in i1 + 1..end - 1 {
            let token = &self.input_tokens[i];
            let (kind, value) = (token.kind, token.value);
            if !self.insignificant_tokens.contains(&kind) {
                if kind == "op" {
                    if *value == *"." {
                        // Ignore "." tokens and any preceding "name" token.
                        if prev_token.kind == "name" {
                            inter_colon_tokens -= 1;
                        }
                    }
                    else if *value == *":" {
                        inter_colon_tokens = 0;
                    }
                    else if *value == *"-" || *value == *"+" {
                        // Ignore unary "-" or "+" tokens.
                        if !is_unary_op_with_prev(&prev_token, &token) {
                            inter_colon_tokens += 1;
                            if inter_colon_tokens > 1 {
                                final_context = "complex-slice";
                                break;
                            }
                        }
                    }
                    else if *value == *"~" {
                        // "~" is always a unary op.
                    }
                    else {
                        // All other ops contribute.
                        inter_colon_tokens += 1;
                        if inter_colon_tokens > 1 {
                            final_context = "complex-slice";
                            break;
                        }
                    }
                }
                else {
                    inter_colon_tokens += 1;
                    if inter_colon_tokens > 1 {
                        final_context = "complex-slice";
                        break;
                    }
                }
                prev_token = token;
            }
        }
        // Set the context of all outer-level ":" tokens.
        for i in indices {
            self.set_context(*i, final_context);
        }    
    }
    //@+node:ekr.20241004154345.7: *4* Annotator.finish_dict
    // ***
    #[allow(unused_variables)]
    fn finish_dict(&mut self, end: usize, state: &ScanState) {
        //! Set context for all ":" when scanning from "{" to "}"
        //! 
        //! Strictly speaking, setting this context is unnecessary because
        //! Annotator.gen_colon generates the same code regardless of this context.
        //! 
        //! In other words, this method can be a do-nothing!

        // Sanity checks.
        if state.kind == "Dummy" {
            return;
        }
        assert!(state.kind == "dict");

        let token = state.token;
        assert!(token.value == "{");

        // *** Rewrite
            // let i1 = token.index;
            // assert i1 < end, (i1, end)

        // Set the context for all ":" tokens.
        let indices = &state.indices;
        for i in indices {
            self.set_context(*i, "dict");
        }
    }
    //@+node:ekr.20241004163018.1: *4* Annotator.set_context
    fn set_context(&mut self, i: usize, context: &str) {
        //! Set self.index_dict[i], but only if it does not already exist!

        if !self.valid_contexts.contains(&context) {
            println!("Unexpected context! {context:?}");
        }
        if false {  // Debugging.
            let token = &self.input_tokens[i];
            let token_kind = token.kind;
            let token_value = token.value;
            println!("set_context: {token_kind:20}: {context:20} {token_value}");
        }
        if !self.index_dict.contains_key(&i) {
            // println!("set_context: {i} {context:?}");
            self.index_dict.insert(i, context.to_string());
        }
    }
    //@-others
}
//@+node:ekr.20240929024648.120: ** class InputTok
#[allow(dead_code)]
#[derive(Debug)]
struct InputTok<'a> {
    index: u32,
    kind: &'a str,
    value: &'a str,
}

impl <'a> InputTok<'_> {
    fn new(index: u32, kind: &'a str, value: &'a str) -> InputTok<'a> {
        InputTok {
            index: index,
            kind: kind,
            value: value,
        }
    }
}
//@+node:ekr.20240929074037.1: ** class LeoBeautifier
#[derive(Debug)]
pub struct Beautifier {
    // Set in LB:beautify_one_file...
    args: Vec<String>,
    files_list: Vec<String>,
    stats: Stats,
    output_list: Vec<String>,
}

///// Temporary.
#[allow(dead_code)]
#[allow(non_snake_case)]
impl Beautifier {
    //@+others
    //@+node:ekr.20240929074037.114: *3*  LB.new
    pub fn new() -> Beautifier {
        let mut x = Beautifier {
            // Set in beautify_one_file
            args: Vec::new(),
            files_list: Vec::new(),
            output_list: Vec::new(),
            stats: Stats::new(),
        };
        x.get_args();
        return x;
    }
    //@+node:ekr.20240929074037.2: *3* LB.add_output_string
    #[allow(unused_variables)]
    fn add_output_string(&mut self, kind: &str, value: &str) {
        //! Add value to the output list.
        //! kind is for debugging.
        if !value.is_empty() {
            self.output_list.push(value.to_string())
        }
    }
    //@+node:ekr.20240929074037.113: *3* LB.beautify
    fn beautify(&mut self, annotated_tokens: &Vec<AnnotatedInputTok>) -> String {
        //! Beautify the annotated, creating the output String.

        // Init ivars first.
        // self.input_token = None;  // ???
        // self.pending_lws = "";  // ???
        // self.pending_ws = "";  // ???
        // self.prev_output_kind = None;    // ???
        // self.prev_output_value = None;  // ???

        // // Init state.
        // self.gen_token("file-start", "");
        // self.push_state("file-start");

        // // The main loop:
        // prev_line_number: i32 = 0;
        // for (self.index, self.input_token) in enumerate(input_tokens):
            // // Set global for visitors.
            // if prev_line_number != self.input_token.line_number {
                // prev_line_number = self.input_token.line_number;
            // }
            // // Call the proper visitor.
            // if self.verbatim {
                // self.do_verbatim();
            // } else {
                // // Use match ???
                // func = getattr(self, f"do_{self.input_token.kind}", self.no_visitor)
                // func()
            // }
            
        for annotated_token in annotated_tokens {
            // println!("beautify: {annotated_token:?}");
            //@+<< LB: beautify: dispatch on annotated_token.kind >>
            //@+node:ekr.20241002062655.1: *4* << LB: beautify: dispatch on annotated_token.kind >>
            let kind = annotated_token.kind;
            let value = annotated_token.kind;
            match kind {
                // Some of these could be replaced by inline code.
                "And" => self.do_And(),
                "As" => self.do_As(),
                "Assert" => self.do_Assert(),
                "At" => self.do_At(),
                "Break" => self.do_Break(),
                "Class" => self.do_Class(),
                "Colon" => self.do_Colon(),
                "ColonEqual" => self.do_ColonEqual(),
                "Comma" => self.do_Comma(),
                "Comment" => self.do_Comment(value),
                "Complex" => self.do_Complex(value),
                "Continue" => self.do_Continue(),
                "Dedent" => self.do_Dedent(value),
                "Def" => self.do_Def(),
                "Del" => self.do_Del(),
                "Dot" => self.do_Dot(),
                "DoubleStar" => self.do_DoubleStar(),
                "Elif" => self.do_Elif(),
                "Else" => self.do_Else(),
                "Equal" => self.do_Equal(),
                "EqEqual" => self.do_EqEqual(),
                "Except" => self.do_Except(),
                "Greater" => self.do_Greater(),
                "GreaterEqual" => self.do_GreaterEqual(),
                "False" => self.do_False(),
                "Finally" => self.do_Finally(),
                "Float" => self.do_Float(value),
                "For" => self.do_For(),
                "From" => self.do_From(),
                "If" => self.do_If(),
                "In" => self.do_In(),
                "Import" => self.do_Import(),
                "Indent" => self.do_Indent(value),
                "Int" => self.do_Int(value),
                "Is" => self.do_Is(),
                "Less" => self.do_Less(),
                "LessEqual" => self.do_LessEqual(),
                "Lbrace" => self.do_Lbrace(),
                "Lpar" => self.do_Lpar(),
                "Lsqb" => self.do_Lsqb(),
                "Minus" => self.do_Minus(),
                "MinusEqual" => self.do_MinusEqual(),
                "Name" => self.do_Name(value),
                "Newline" => self.do_Newline(),
                "None" => self.do_None(),
                "NonLogicalNewline" => self.do_NonLogicalNewline(),
                "Not" => self.do_Not(),
                "NotEqual" => self.do_NotEqual(),
                "Or" => self.do_Or(),
                "Pass" => self.do_Pass(),
                "Percent" => self.do_Percent(),
                "Plus" => self.do_Plus(),
                "PlusEqual" => self.do_PlusEqual(),
                "Raise" => self.do_Raise(),
                "Rarrow" => self.do_Rarrow(),
                "Rbrace" => self.do_Rbrace(),
                "Return" => self.do_Return(),
                "Rpar" => self.do_Rpar(),
                "Rsqb" => self.do_Rsqb(),
                "Star" => self.do_Star(),
                "String" => self.do_String(value),
                "True" => self.do_True(),
                "Try" => self.do_Try(),
                "While" => self.do_While(),
                "With" => self.do_With(),
                "ws" => self.do_ws(kind, value),
                _ => println!("No visitor for: {kind}"),
            }
            //@-<< LB: beautify: dispatch on annotated_token.kind >>
        }

        let mut result = String::new();
        for output_token in &self.output_list {
            result.push_str(output_token);
        }
        if false {
            let n = result.len();
            println!("result: {n} characters");
        }
        return result;
    }
    //@+node:ekr.20240929074037.4: *3* LB.beautify_all_files
    pub fn beautify_all_files(&mut self) {
        // for file_name in self.files_list.clone() {
        for file_name in self.files_list.clone() {
            self.beautify_one_file(&file_name);
        }
    }

    //@+node:ekr.20240929074037.5: *3* LB.beautify_one_file
    fn beautify_one_file(&mut self, file_name: &str) {
        self.stats.n_files += 1;
        if true {
            // Testing only: print the short file name.
            let file_path = path::Path::new(file_name);
            let os_str = file_path.file_name().unwrap(); // &OsStr
            let short_file_name = os_str.to_str().unwrap();
            println!("{short_file_name}");
        }
        // Read the file into contents (a String).
        let t1 = std::time::Instant::now();
        let contents = fs::read_to_string(file_name).expect("Error reading{file_name}");
        self.stats.read_time += t1.elapsed().as_nanos();
        // Create (an immutable!) list of input tokens.
        let t2 = std::time::Instant::now();
        let input_tokens = self.make_input_list(&contents);
        self.stats.make_tokens_time += t2.elapsed().as_nanos();
        // Annotate tokens (the prepass).
        let t3 = std::time::Instant::now();
        let mut annotator = Annotator::new(&input_tokens);
        let annotated_tokens = annotator.annotate();
        self.stats.annotation_time += t3.elapsed().as_nanos();
        // Beautify.
        let t4 = std::time::Instant::now();
        self.beautify(&annotated_tokens);
        self.stats.beautify_time += t4.elapsed().as_nanos();
    }
    //@+node:ekr.20240929074037.7: *3* LB.do_*
    //@+node:ekr.20241002071143.1: *4* tbo.do_ws
    // *** Temporary
    #[allow(unused_variables)]
    fn do_ws(&mut self, kind: &str, value: &str) {
        //! Handle the "ws" pseudo-token.
        //! Put the whitespace only if if ends with backslash-newline.

        // To do.

        // let last_token = self.input_tokens[self.index - 1];
        // let is_newline = kind in ("nl", "newline");
        // if is_newline {
        // self.pending_lws = val;
        // self.pending_ws = "";
        // }
        // else if "\\\n" in val {
        // self.pending_lws = "";
        // self.pending_ws = val;
        // }
        // else {
        // self.pending_ws = val
        // }
    }
    //@+node:ekr.20240929074037.8: *4* LB:Handlers with values
    //@+node:ekr.20240929074037.9: *5* LB.do_Comment
    fn do_Comment(&mut self, tok_value: &str) {
        // print!("{tok_value}");  // Correct.
        // print!("{value} ");  // Wrong!
        self.add_output_string("Comment", tok_value);
    }
    //@+node:ekr.20240929074037.10: *5* LB.do_Complex
    fn do_Complex(&mut self, tok_value: &str) {
        self.add_output_string("Complex", tok_value);
    }
    //@+node:ekr.20240929074037.11: *5* LB.do_Float
    fn do_Float(&mut self, tok_value: &str) {
        self.add_output_string("Float", tok_value);
    }
    //@+node:ekr.20240929074037.12: *5* LB.do_Int
    fn do_Int(&mut self, tok_value: &str) {
        self.add_output_string("Int", tok_value);
    }
    //@+node:ekr.20240929074037.13: *5* LB.do_Name
    fn do_Name(&mut self, tok_value: &str) {
        self.add_output_string("Name", tok_value);
    }
    //@+node:ekr.20240929074037.14: *5* LB.do_String
    fn do_String(&mut self, tok_value: &str) {
        // correct.
        // print!("{tok_value}");

        // incorrect.
        // let quote = if *triple_quoted {"'''"} else {"'"};
        // print!("{:?}:{quote}{value}{quote}", kind);

        self.add_output_string("String", tok_value);
    }
    //@+node:ekr.20240929074037.15: *4* LB:Handlers using lws
    //@+node:ekr.20240929074037.16: *5* LB.do_Dedent
    fn do_Dedent(&mut self, tok_value: &str) {
        self.add_output_string("Dedent", tok_value);
    }
    //@+node:ekr.20240929074037.17: *5* LB.do_Indent
    fn do_Indent(&mut self, tok_value: &str) {
        self.add_output_string("Indent", tok_value);
    }
    //@+node:ekr.20240929074037.18: *5* LB.do_Newline
    fn do_Newline(&mut self) {
        self.add_output_string("Indent", "\n");
    }
    //@+node:ekr.20240929074037.19: *5* LB.do_NonLogicalNewline
    fn do_NonLogicalNewline(&mut self) {
        self.add_output_string("Indent", "\n");
    }
    //@+node:ekr.20240929074037.20: *4* LB:Handlers w/o values
    //@+node:ekr.20240929074037.21: *5* LB.do_Amper
    fn do_Amper(&mut self) {
        self.add_output_string("Amper", "&");
    }
    //@+node:ekr.20240929074037.22: *5* LB.do_AmperEqual
    fn do_AmperEqual(&mut self) {
        self.add_output_string("AmperEqual", "&=");
    }
    //@+node:ekr.20240929074037.23: *5* LB.do_And
    fn do_And(&mut self) {
        self.add_output_string("And", "and");
    }
    //@+node:ekr.20240929074037.24: *5* LB.do_As
    fn do_As(&mut self) {
        self.add_output_string("As", "as");
    }
    //@+node:ekr.20240929074037.25: *5* LB.do_Assert
    fn do_Assert(&mut self) {
        self.add_output_string("Assert", "assert");
    }
    //@+node:ekr.20240929074037.26: *5* LB.do_Async
    fn do_Async(&mut self) {
        self.add_output_string("Async", "async");
    }
    //@+node:ekr.20240929074037.27: *5* LB.do_At
    fn do_At(&mut self) {
        self.add_output_string("At", "@");
    }
    //@+node:ekr.20240929074037.28: *5* LB.do_AtEqual
    fn do_AtEqual(&mut self) {
        self.add_output_string("AtEqual", "@=");
    }
    //@+node:ekr.20240929074037.29: *5* LB.do_Await
    fn do_Await(&mut self) {
        self.add_output_string("Await", "await");
    }
    //@+node:ekr.20240929074037.30: *5* LB.do_Break
    fn do_Break(&mut self) {
        self.add_output_string("Break", "break");
    }
    //@+node:ekr.20240929074037.31: *5* LB.do_Case
    fn do_Case(&mut self) {
        self.add_output_string("Case", "case");
    }
    //@+node:ekr.20240929074037.32: *5* LB.do_CircumFlex
    fn do_CircumFlex(&mut self) {
        self.add_output_string("CircumFlex", "^");
    }
    //@+node:ekr.20240929074037.33: *5* LB.do_CircumflexEqual
    fn do_CircumflexEqual(&mut self) {
        self.add_output_string("CircumflexEqual", "^=");
    }
    //@+node:ekr.20240929074037.34: *5* LB.do_Class
    fn do_Class(&mut self) {
        self.add_output_string("Class", "class");
    }
    //@+node:ekr.20240929074037.35: *5* LB.do_Colon
    fn do_Colon(&mut self) {
        self.add_output_string("Colon", ":");
    }
    //@+node:ekr.20240929074037.36: *5* LB.do_ColonEqual
    fn do_ColonEqual(&mut self) {
        self.add_output_string("ColonEqual", ":=");
    }
    //@+node:ekr.20240929074037.37: *5* LB.do_Comma
    fn do_Comma(&mut self) {
        self.add_output_string("Comma", ",");
    }
    //@+node:ekr.20240929074037.38: *5* LB.do_Continue
    fn do_Continue(&mut self) {
        self.add_output_string("Continue", "continue");
    }
    //@+node:ekr.20240929074037.39: *5* LB.do_Def
    fn do_Def(&mut self) {
        self.add_output_string("Def", "def");
    }
    //@+node:ekr.20240929074037.40: *5* LB.do_Del
    fn do_Del(&mut self) {
        self.add_output_string("Del", "del");
    }
    //@+node:ekr.20240929074037.41: *5* LB.do_Dot
    fn do_Dot(&mut self) {
        self.add_output_string("Dot", ".");
    }
    //@+node:ekr.20240929074037.42: *5* LB.do_DoubleSlash
    fn do_DoubleSlash(&mut self) {
        self.add_output_string("DoubleSlash", "//");
    }
    //@+node:ekr.20240929074037.43: *5* LB.do_DoubleSlashEqual
    fn do_DoubleSlashEqual(&mut self) {
        self.add_output_string("DoubleSlashEqual", "//=");
    }
    //@+node:ekr.20240929074037.44: *5* LB.do_DoubleStar
    fn do_DoubleStar(&mut self) {
        self.add_output_string("DoubleStar", "**");
    }
    //@+node:ekr.20240929074037.45: *5* LB.do_DoubleStarEqual
    fn do_DoubleStarEqual(&mut self) {
        self.add_output_string("DoubleStarEqual", "**=");
    }
    //@+node:ekr.20240929074037.46: *5* LB.do_Elif
    fn do_Elif(&mut self) {
        self.add_output_string("Elif", "elif");
    }
    //@+node:ekr.20240929074037.47: *5* LB.do_Ellipsis
    fn do_Ellipsis(&mut self) {
        self.add_output_string("Ellipsis", "...");
    }
    //@+node:ekr.20240929074037.48: *5* LB.do_Else
    fn do_Else(&mut self) {
        self.add_output_string("Else", "else");
    }
    //@+node:ekr.20240929074037.49: *5* LB.do_EndOfFile
    fn do_EndOfFile(&mut self) {
        self.add_output_string("EndOfFile", "EOF");
    }
    //@+node:ekr.20240929074037.50: *5* LB.do_EqEqual
    fn do_EqEqual(&mut self) {
        self.add_output_string("EqEqual", "==");
    }
    //@+node:ekr.20240929074037.51: *5* LB.do_Equal
    fn do_Equal(&mut self) {
        self.add_output_string("Equal", "=");
    }
    //@+node:ekr.20240929074037.52: *5* LB.do_Except
    fn do_Except(&mut self) {
        self.add_output_string("Except", "except");
    }
    //@+node:ekr.20240929074037.53: *5* LB.do_False
    fn do_False(&mut self) {
        self.add_output_string("False", "False");
    }
    //@+node:ekr.20240929074037.54: *5* LB.do_Finally
    fn do_Finally(&mut self) {
        self.add_output_string("Finally", "finally");
    }
    //@+node:ekr.20240929074037.55: *5* LB.do_For
    fn do_For(&mut self) {
        self.add_output_string("For", "for");
    }
    //@+node:ekr.20240929074037.56: *5* LB.do_From
    fn do_From(&mut self) {
        self.add_output_string("From", "from");
    }
    //@+node:ekr.20240929074037.57: *5* LB.do_Global
    fn do_Global(&mut self) {
        self.add_output_string("Global", "global");
    }
    //@+node:ekr.20240929074037.58: *5* LB.do_Greater
    fn do_Greater(&mut self) {
        self.add_output_string("Greater", ">");
    }
    //@+node:ekr.20240929074037.59: *5* LB.do_GreaterEqual
    fn do_GreaterEqual(&mut self) {
        self.add_output_string("GreaterEqual", ">-");
    }
    //@+node:ekr.20240929074037.60: *5* LB.do_If
    fn do_If(&mut self) {
        self.add_output_string("If", "if");
    }
    //@+node:ekr.20240929074037.61: *5* LB.do_Import
    fn do_Import(&mut self) {
        self.add_output_string("Import", "import");
    }
    //@+node:ekr.20240929074037.62: *5* LB.do_In
    fn do_In(&mut self) {
        self.add_output_string("In", "in");
    }
    //@+node:ekr.20240929074037.63: *5* LB.do_Is
    fn do_Is(&mut self) {
        self.add_output_string("Is", "is");
    }
    //@+node:ekr.20240929074037.64: *5* LB.do_Lambda
    fn do_Lambda(&mut self) {
        self.add_output_string("Lambda", "lambda");
    }
    //@+node:ekr.20240929074037.65: *5* LB.do_Lbrace
    fn do_Lbrace(&mut self) {
        self.add_output_string("Lbrace", "[");
    }
    //@+node:ekr.20240929074037.66: *5* LB.do_LeftShift
    fn do_LeftShift(&mut self) {
        self.add_output_string("LeftShift", "<<");
    }
    //@+node:ekr.20240929074037.67: *5* LB.do_LeftShiftEqual
    fn do_LeftShiftEqual(&mut self) {
        self.add_output_string("LeftShiftEqual", "<<=");
    }
    //@+node:ekr.20240929074037.68: *5* LB.do_Less
    fn do_Less(&mut self) {
        self.add_output_string("Less", "<");
    }
    //@+node:ekr.20240929074037.69: *5* LB.do_LessEqual
    fn do_LessEqual(&mut self) {
        self.add_output_string("LessEqual", "<=");
    }
    //@+node:ekr.20240929074037.70: *5* LB.do_Lpar
    fn do_Lpar(&mut self) {
        self.add_output_string("Lpar", "(");
    }
    //@+node:ekr.20240929074037.71: *5* LB.do_Lsqb
    fn do_Lsqb(&mut self) {
        self.add_output_string("Lsqb", "[");
    }
    //@+node:ekr.20240929074037.72: *5* LB.do_Match
    fn do_Match(&mut self) {
        self.add_output_string("Match", "match");
    }
    //@+node:ekr.20240929074037.73: *5* LB.do_Minus
    fn do_Minus(&mut self) {
        self.add_output_string("Minus", "-");
    }
    //@+node:ekr.20240929074037.74: *5* LB.do_MinusEqual
    fn do_MinusEqual(&mut self) {
        self.add_output_string("MinusEqual", "-=");
    }
    //@+node:ekr.20240929074037.75: *5* LB.do_None
    fn do_None(&mut self) {
        self.add_output_string("None", "None");
    }
    //@+node:ekr.20240929074037.76: *5* LB.do_Nonlocal
    fn do_Nonlocal(&mut self) {
        self.add_output_string("Nonlocal", "nonlocal");
    }
    //@+node:ekr.20240929074037.77: *5* LB.do_Not
    fn do_Not(&mut self) {
        self.add_output_string("Not", "not");
    }
    //@+node:ekr.20240929074037.78: *5* LB.do_NotEqual
    fn do_NotEqual(&mut self) {
        self.add_output_string("NotEqual", "!=");
    }
    //@+node:ekr.20240929074037.79: *5* LB.do_Or
    fn do_Or(&mut self) {
        self.add_output_string("Or", "or");
    }
    //@+node:ekr.20240929074037.80: *5* LB.do_Pass
    fn do_Pass(&mut self) {
        self.add_output_string("Pass", "pass");
    }
    //@+node:ekr.20240929074037.81: *5* LB.do_Percent
    fn do_Percent(&mut self) {
        self.add_output_string("Percent", "%");
    }
    //@+node:ekr.20240929074037.82: *5* LB.do_PercentEqual
    fn do_PercentEqual(&mut self) {
        self.add_output_string("PercentEqual", "%=");
    }
    //@+node:ekr.20240929074037.83: *5* LB.do_Plus
    fn do_Plus(&mut self) {
        self.add_output_string("Plus", "+");
    }
    //@+node:ekr.20240929074037.84: *5* LB.do_PlusEqual
    fn do_PlusEqual(&mut self) {
        self.add_output_string("PlusEqual", "+=");
    }
    //@+node:ekr.20240929074037.85: *5* LB.do_Raise
    fn do_Raise(&mut self) {
        self.add_output_string("Raise", "raise");
    }
    //@+node:ekr.20240929074037.86: *5* LB.do_Rarrow
    fn do_Rarrow(&mut self) {
        self.add_output_string("Rarrow", "->");
    }
    //@+node:ekr.20240929074037.87: *5* LB.do_Rbrace
    fn do_Rbrace(&mut self) {
        self.add_output_string("Rbrace", "]");
    }
    //@+node:ekr.20240929074037.88: *5* LB.do_Return
    fn do_Return(&mut self) {
        self.add_output_string("Return", "return");
    }
    //@+node:ekr.20240929074037.89: *5* LB.do_RightShift
    fn do_RightShift(&mut self) {
        self.add_output_string("RightShift", ">>");
    }
    //@+node:ekr.20240929074037.90: *5* LB.do_RightShiftEqual
    fn do_RightShiftEqual(&mut self) {
        self.add_output_string("RightShiftEqual", ">>=");
    }
    //@+node:ekr.20240929074037.91: *5* LB.do_Rpar
    fn do_Rpar(&mut self) {
        self.add_output_string("Rpar", ")");
    }
    //@+node:ekr.20240929074037.92: *5* LB.do_Rsqb
    fn do_Rsqb(&mut self) {
        self.add_output_string("Rsqb", "]");
    }
    //@+node:ekr.20240929074037.93: *5* LB.do_Semi
    fn do_Semi(&mut self) {
        self.add_output_string("Semi", ";");
    }
    //@+node:ekr.20240929074037.94: *5* LB.do_Slash
    fn do_Slash(&mut self) {
        self.add_output_string("Slash", "/");
    }
    //@+node:ekr.20240929074037.95: *5* LB.do_SlashEqual
    fn do_SlashEqual(&mut self) {
        self.add_output_string("SlashEqual", "/=");
    }
    //@+node:ekr.20240929074037.96: *5* LB.do_Star
    fn do_Star(&mut self) {
        self.add_output_string("Star", "*");
    }
    //@+node:ekr.20240929074037.97: *5* LB.do_StarEqual
    fn do_StarEqual(&mut self) {
        self.add_output_string("StarEqual", "*=");
    }
    //@+node:ekr.20240929074037.98: *5* LB.do_StartExpression
    fn do_StartExpression(&mut self) {
        // self.add_output_string("StartExpression", "");
    }
    //@+node:ekr.20240929074037.99: *5* LB.do_StartInteractive
    fn do_StartInteractive(&mut self) {
        // self.add_output_string("StartModule", "");
    }
    //@+node:ekr.20240929074037.100: *5* LB.do_StarModule
    fn do_StartModule(&mut self) {
        // self.add_output_string("StartModule", "");
        println!("do_StartModule");
    }
    //@+node:ekr.20240929074037.101: *5* LB.do_Tilde
    fn do_Tilde(&mut self) {
        self.add_output_string("Tilde", "~");
    }
    //@+node:ekr.20240929074037.102: *5* LB.do_True
    fn do_True(&mut self) {
        self.add_output_string("True", "True");
    }
    //@+node:ekr.20240929074037.103: *5* LB.do_Try
    fn do_Try(&mut self) {
        self.add_output_string("Try", "try");
    }
    //@+node:ekr.20240929074037.104: *5* LB.do_Type
    fn do_Type(&mut self) {
        self.add_output_string("Type", "type");
    }
    //@+node:ekr.20240929074037.105: *5* LB.do_Vbar
    fn do_Vbar(&mut self) {
        self.add_output_string("Vbar", "|");
    }
    //@+node:ekr.20240929074037.106: *5* LB.do_VbarEqual
    fn do_VbarEqual(&mut self) {
        self.add_output_string("VbarEqual", "|=");
    }
    //@+node:ekr.20240929074037.107: *5* LB.do_While
    fn do_While(&mut self) {
        self.add_output_string("While", "while");
    }
    //@+node:ekr.20240929074037.108: *5* LB.do_With
    fn do_With(&mut self) {
        self.add_output_string("With", "with");
    }
    //@+node:ekr.20240929074037.109: *5* LB.do_Yield
    fn do_Yield(&mut self) {
        self.add_output_string("Yield", "yield");
    }
    //@+node:ekr.20240929074037.110: *3* LB.enabled
    fn enabled(&self, arg: &str) -> bool {
        //! Beautifier::enabled: return true if the given command-line argument is enabled.
        //! Example:  x.enabled("--report");
        return self.args.contains(&arg.to_string());
    }
    //@+node:ekr.20240929074037.111: *3* LB.get_args
    fn get_args(&mut self) {
        //! Beautifier::get_args: Set the args and files_list ivars.
        let args: Vec<String> = env::args().collect();
        let valid_args = vec![
            "--all",
            "--beautified",
            "--diff",
            "-h",
            "--help",
            "--report",
            "--write",
        ];
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                if valid_args.contains(&arg.as_str()) {
                    self.args.push(arg.to_string())
                } else if arg.as_str().starts_with("--") || arg.as_str().starts_with("--") {
                    println!("Ignoring invalid arg: {arg}");
                } else {
                    println!("File: {arg}");
                    self.files_list.push(arg.to_string());
                }
            }
        }
    }
    //@+node:ekr.20240929074037.112: *3* LB.make_input_list
    fn make_input_list<'a>(&mut self, contents: &'a str) -> Vec<InputTok<'a>> {
        //! Return an input_list from the tokens given by the RustPython lex.
        let mut n_tokens: u64 = 0;
        let mut n_ws_tokens: u64 = 0;
        let mut prev_start: usize = 0;
        let mut result: Vec<InputTok> = Vec::new();
        let mut index: u32 = 0;
        for token_tuple in lex(&contents, Mode::Module)
            .map(|tok| tok.expect("Failed to lex"))
            .collect::<Vec<_>>()
        {
            use Tok::*;
            let (token, range) = token_tuple;
            let tok_value = &contents[range];

            // The gem: create a whitespace pseudo-tokens.
            // This code adds maybe about 1 ms when beautifying leoFrame.py.
            // With the gem: 14.1 - 14.5 ms. Without: 13.1 - 13.7 ms.
            let start_i = usize::from(range.start());
            let end_i = usize::from(range.end());
            if start_i > prev_start {
                let ws = &contents[prev_start..start_i];
                result.push(InputTok::new(index, "ws", ws));
                n_ws_tokens += 1
            }
            prev_start = end_i;

            //@+<< Calculate class_name using match token >>
            //@+node:ekr.20241002113506.1: *4* << Calculate class_name using match token >>
            // Variant names are necessary, but otherwise not used.
            #[allow(unused_variables)]
            let class_name = match token {
                // Tokens with values...
                Comment(value) => "Comment",
                Complex { real, imag } => "Complex",
                Float { value } => "Float",
                Int { value } => "Int",
                Name { name } => "Name",
                Tok::String {
                    value,
                    kind,
                    triple_quoted,
                } => "String",

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
            //@-<< Calculate class_name using match token >>
            n_tokens += 1;
            result.push(InputTok::new(index, class_name, tok_value));
            index += 1;
        }
        // Update counts.
        self.stats.n_tokens += n_tokens;
        self.stats.n_ws_tokens += n_ws_tokens;
        return result;
    }
    //@+node:ekr.20240929074037.115: *3* LB.show_args
    fn show_args(&self) {
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
    //@+node:ekr.20240929074037.116: *3* LB.show_help
    fn show_help(&self) {
        //! Beautifier::show_help: print the help messages.
        println!(
            "{}",
            textwrap::dedent(
                "
            Beautify or diff files.

            -h --help:      Print this help message and exit.
            --all:          Beautify all files, even unchanged files.
            --beautified:   Report beautified files individually, even if not written.
            --diff:         Show diffs instead of changing files.
            --report:       Print summary report.
            --write:        Write beautifed files (dry-run mode otherwise).
        "
            )
        );
    }
    //@+node:ekr.20241002163554.1: *3* LB.string_to_static_str (not used)
    fn string_to_static_str(&self, s: String) -> &'static str {
        Box::leak(s.into_boxed_str())
    }

    //@-others
}
//@+node:ekr.20241004112826.1: ** class ParseState
#[allow(dead_code)]
#[derive(Debug)]
struct ParseState {
    //@+<< docstring: ParseState >>
    //@+node:ekr.20241004113118.1: *3* << docstring: ParseState >>
    //@@language rest
    //@+doc
    //
    // A class representing items in the parse state stack.
    //
    // The present states:
    //
    // 'file-start': Ensures the stack stack is never empty.
    //
    // 'decorator': The last '@' was a decorator.
    //
    //     do_op():    push_state('decorator')
    //     do_name():  pops the stack if state.kind == 'decorator'.
    //
    // 'indent': The indentation level for 'class' and 'def' names.
    //
    //     do_name():      push_state('indent', self.level)
    //     do_dendent():   pops the stack once or
    //                     twice if state.value == self.level.
    //
    //@-<< docstring: ParseState >>
    kind: String,
    value: String,
}
//@+node:ekr.20241004165555.1: ** class ScanState 
#[derive(Clone, Debug)]

// *** Python
    // def __init__(self, kind: str, token: InputToken) -> None:
        // self.kind = kind
        // self.token = token
        // self.value: list[int] = []  # Not always used

struct ScanState<'a> {
    // A class representing tbo.pre_scan's scanning state.
    // Valid (kind, value) pairs:
    // kind  Value
    // ====  =====
    // "args" Not used
    // "from" Not used
    // "import" Not used
    // "slice" list of colon indices
    // "dict" list of colon indices

    kind: &'a str,
    token: &'a InputTok<'a>,
    indices: Vec<usize>,  // Empty for most tokens.
}

impl <'a> ScanState<'_> {
    fn new(kind: &'a str, token: &'a InputTok) -> ScanState<'a> {
        ScanState {
            kind: kind,
            token: token,
            indices: Vec::new(),
        }
    }
}
//@+node:ekr.20240929074547.1: ** class Stats
// Allow unused write_time
#[allow(dead_code)]
#[derive(Debug)]
pub struct Stats {
    // Cumulative statistics for all files.
    n_files: u64,     // Number of files.
    n_tokens: u64,    // Number of tokens.
    n_ws_tokens: u64, // Number of pseudo-ws tokens.

    // Timing stat, in microseconds...
    annotation_time: u128,
    beautify_time: u128,
    make_tokens_time: u128,
    read_time: u128,
    write_time: u128,
}

// Calling Stats.report is optional.
#[allow(dead_code)]
impl Stats {
    //@+others
    //@+node:ekr.20241001100954.1: *3*  Stats::new
    pub fn new() -> Stats {
        let x = Stats {
            // Cumulative counts.
            n_files: 0,     // Number of files.
            n_tokens: 0,    // Number of tokens.
            n_ws_tokens: 0, // Number of pseudo-ws tokens.

            // Timing stats, in nanoseconds...
            annotation_time: 0,
            beautify_time: 0,
            make_tokens_time: 0,
            read_time: 0,
            write_time: 0,
        };
        return x;
    }
    //@+node:ekr.20240929080242.1: *3* Stats::fmt_ns
    fn fmt_ns(&mut self, t: u128) -> String {
        //! Convert nanoseconds to fractional milliseconds.
        let ms = t / 1000000;
        let micro = (t % 1000000) / 10000; // 2-places only.
                                           // println!("t: {t:8} ms: {ms:03} micro: {micro:02}");
        return f!("{ms:4}.{micro:02}");
    }

    //@+node:ekr.20240929075236.1: *3* Stats::report
    fn report(&mut self) {
        // Cumulative counts.
        let n_files = self.n_files;
        let n_tokens = self.n_tokens;
        let n_ws_tokens = self.n_ws_tokens;
        // Print cumulative timing stats, in ms.
        let annotation_time = self.fmt_ns(self.annotation_time);
        let beautify_time = self.fmt_ns(self.beautify_time);
        let make_tokens_time = self.fmt_ns(self.make_tokens_time);
        let read_time = self.fmt_ns(self.read_time);
        let write_time = self.fmt_ns(self.write_time);
        let total_time_ns = self.annotation_time
            + self.beautify_time
            + self.make_tokens_time
            + self.read_time
            + self.write_time;
        let total_time = self.fmt_ns(total_time_ns);
        println!("");
        println!("     files: {n_files}, tokens: {n_tokens}, ws tokens: {n_ws_tokens}");
        println!("       read: {read_time:>7} ms");
        println!("make_tokens: {make_tokens_time:>7} ms");
        println!("   annotate: {annotation_time:>7} ms");
        println!("   beautify: {beautify_time:>7} ms");
        println!("      write: {write_time:>7} ms");
        println!("      total: {total_time:>7} ms");
    }
    //@-others
}
//@+node:ekr.20241003093554.1: ** fn: entry
pub fn entry() {
    main();
}
//@+node:ekr.20241005091217.1: ** fn: is_python_keyword (to do)
// def is_python_keyword(self, token: Optional[InputToken]) -> bool:
    // """Return True if token is a 'name' token referring to a Python keyword."""
    // if not token or token.kind != 'name':
        // return False
    // return keyword.iskeyword(token.value) or keyword.issoftkeyword(token.value)
    
// Keywords:
// False      await      else       import     pass
// None       break      except     in         raise
// True       class      finally    is         return
// and        continue   for        lambda     try
// as         def        from       nonlocal   while
// assert     del        global     not        with
// async      elif       if         or         yield

// Soft keywords:
// match, case, type and _

// *** Remove leading underscores.
fn is_python_keyword(_token: &InputTok) -> bool {
    return false;

    // *** Not ready yet.
        // //! Return True if token is a 'name' token referring to a Python keyword.
        // if token.kind != "name" {
            // return false;
        // }
        // // let word = &token.value;  // &String
        // return false;  // ***
}
//@+node:ekr.20241005092549.1: ** fn: is_unary_op_with_prev (to do)
// def is_unary_op_with_prev(self, prev: Optional[InputToken], token: InputToken) -> bool:
    // """
    // Return True if token is a unary op in the context of prev, the previous
    // significant token.
    // """
    // if token.value == '~':  # pragma: no cover
        // return True
    // if prev is None:
        // return True  # pragma: no cover
    // assert token.value in '**-+', repr(token.value)
    // if prev.kind in ('number', 'string'):
        // return_val = False
    // elif prev.kind == 'op' and prev.value in ')]':
         // # An unnecessary test?
        // return_val = False  # pragma: no cover
    // elif prev.kind == 'op' and prev.value in '{([:,':
        // return_val = True
    // elif prev.kind != 'name':
        // # An unnecessary test?
        // return_val = True  # pragma: no cover
    // else:
        // # prev is a'name' token.
        // return self.is_python_keyword(token)
    // return return_val

// *** Remove leading underscores.
fn is_unary_op_with_prev(_prev_token: &InputTok, _token: &InputTok) -> bool {
    return false;  // ***
}
//@+node:ekr.20241003093722.1: ** fn: main
//@@language rust
pub fn main() {
    // Main line of beautifier.
    let mut x = Beautifier::new();
    if true {
        // testing.
        println!("");
        for file_path in [
            "C:\\Repos\\ekr-tbo-in-rust\\test\\test1.py",
            // "C:\\Repos\\leo-editor\\leo\\core\\leoFrame.py",
            // "C:\\Repos\\leo-editor\\leo\\core\\leoApp.py"
        ] {
            x.beautify_one_file(&file_path);
        }
        // x.stats.report();
    } else {
        if x.enabled("--help") || x.enabled("-h") {
            x.show_help();
            return;
        }
        x.show_args();
        x.beautify_all_files();
    }
}
//@-others

//@-leo
