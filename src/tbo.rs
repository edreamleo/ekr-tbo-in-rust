//@+leo-ver=5-thin
//@+node:ekr.20240928161210.1: * @file src/tbo.rs
// tbo.rs

// From https://docs.rs/rustpython-parser/0.3.1/rustpython_parser/lexer/index.html

// Must be first.
#![allow(dead_code)]
// #![allow(unused_imports)]
// #![allow(unused_variables)]

extern crate rustpython_parser;
use rustpython_parser::{lexer::lex, Mode, Tok}; // text_size::TextRange
use std::env;
use std::fmt;
use std::fs;
use std::path;

//@+others
//@+node:ekr.20241003093554.1: **  pub fn entry
pub fn entry() {
    if false {
        test();
    } else {
        main();
    }
}
//@+node:ekr.20241003094145.1: **  struct TestTok
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct TestTok {
    value: i32,
}
//@+node:ekr.20241004095931.1: ** class AnnotatedInputTok
// Only Clone is valid for String.
#[derive(Clone)]
struct AnnotatedInputTok {
    context: Vec::<String>,
    kind: String,
    value: String,
}

impl AnnotatedInputTok {
    fn new(context: Vec<String>, kind: &str, value: &str) -> AnnotatedInputTok {
        AnnotatedInputTok {
            context: context,
            kind: kind.to_string(),
            value: value.to_string(),
        }
    }
}
//@+node:ekr.20241004110721.1: ** class Annotator
struct Annotator<'a> {
    // The present input token...
    input_tokens: Vec<InputTok<'a>>,
    insignificant_tokens: [String; 6],
    index: u32,  // The index within the tokens array of the token being scanned.
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
    verbatim: bool,  // True: don't beautify.
}

impl Annotator<'_> {
    //@+others
    //@+node:ekr.20241004153742.1: *3* Annotator.new
    fn new(input_tokens: Vec<InputTok>) -> Annotator {
        Annotator {
            curly_brackets_level: 0,
            decorator_seen: false,
            in_arg_list: 0,  // > 0 if in an arg list of a def.
            in_doc_part: false,
            indent_level: 0,
            index: 0,
            input_tokens: input_tokens,
            insignificant_tokens: [
                "comment".to_string(), "dedent".to_string(), "indent".to_string(),
                "newline".to_string(), "nl".to_string(), "ws".to_string(),
            ],
            lws: String::new(),
            paren_level: 0,
            state_stack: Vec::new(),
            square_brackets_stack: Vec::new(),
            verbatim: false, 
        }
    }
    //@+node:ekr.20241004153802.1: *3* Annotator.pre_pass
    fn pre_scan(&mut self) {
        //! Scan the entire file in one iterative pass, adding context to a few
        //! kinds of tokens as follows:
        //!
        //! Token   Possible Contexts (or None)
        //! =====   ===========================
        //! ":"     "annotation", "dict", "complex-slice", "simple-slice"
        //! "="     "annotation", "initializer"
        //! "*"     "arg"
        //! "**"    "arg"
        //! "."     "import"

        // The main loop.
        let mut in_import = false;
        // Avoid Option complications by creating a dummy token and scan state.
        let dummy_token = InputTok::new("dummy", "");
        let dummy_state = ScanState::new("dummy", &dummy_token);
        let mut scan_stack: Vec<ScanState> = Vec::new();
        scan_stack.push(dummy_state);
        let mut prev_token = InputTok::new("dummy", "");
        let mut i = 0;
        for token in &self.input_tokens {
            let (kind, value) = (token.kind, token.value);
            if kind == "newline" {
                //@+<< pre-scan newline tokens >>
                //@+node:ekr.20241004154345.2: *4* << pre-scan newline tokens >>
                // "import" and "from x import" statements may span lines.
                // "ws" tokens represent continued lines like this:   ws: " \\\n    "
                if in_import && scan_stack.len() == 0 {
                    in_import = false;
                }
                //@-<< pre-scan newline tokens >>
            }
            else if kind == "op" {
                //@+<< pre-scan op tokens >>
                //@+node:ekr.20241004154345.3: *4* << pre-scan op tokens >>
                // top_state: Optional[fScanState] = scan_stack[-1] if scan_stack else None
                // The scan_stack always contains at least a dummy state.
                let top_state = &mut scan_stack[scan_stack.len() - 1].clone();

                // Handle "[" and "]".
                if value == "[" {
                    scan_stack.push(ScanState::new("slice", &token));
                }
                else if  value == "]" {
                    assert!(top_state.kind == "slice");
                    self.finish_slice(i, top_state);
                    scan_stack.pop();
                }
                // Handle "{" and "}".
                if value == "{" {
                    scan_stack.push(ScanState::new("dict", &token));
                }
                else if value == "}" {
                    assert!(top_state.kind == "dict");
                    self.finish_dict(i, top_state);
                    scan_stack.pop();
                }
                // Handle "(" and ")"
                else if value == "(" {
                    let state_kind: &str;
                    if self.is_python_keyword(&prev_token) || prev_token.kind != "name" {
                        state_kind = "(";
                    }
                    else {
                        state_kind = "arg";
                    }
                    scan_stack.push(ScanState::new(state_kind, &token));
                }
                else if value == ")" {
                    assert!(["arg", "("].contains(&top_state.kind));
                    if top_state.kind == "arg" {
                        self.finish_arg(i, top_state);
                    }
                    scan_stack.pop();
                }
                // Handle interior tokens in "arg" and "slice" states.
                if top_state.kind != "dummy" {
                    if value == ":" && ["dict", "slice"].contains(&top_state.kind) {
                        top_state.indices.push(i);
                    }
                    else if top_state.kind == "arg" && ["**", "*", "=", ":", ","].contains(&value) {
                        top_state.indices.push(i);
                    }
                }
                // Handle "." and "(" tokens inside "import" and "from" statements.
                if in_import && ["(", "."].contains(&value) {
                    self.set_context(i, "import");
                }
                //@-<< pre-scan op tokens >>
            }
            else if kind == "name" {
                //@+<< pre-scan name tokens >>
                //@+node:ekr.20241004154345.4: *4* << pre-scan name tokens >>
                let prev_is_yield = prev_token.kind == "name" && prev_token.value == "yield";
                // if ["from", "import"].contains(value) && !prev_is_yield {
                if !prev_is_yield && (value == "from" || value == "import") {
                    // "import" and "from x import" statements should be at the outer level.
                    assert!(scan_stack.len() == 1 && scan_stack[0].kind == "dummy");
                    in_import = true;
                }
                //@-<< pre-scan name tokens >>
            }
            // Remember the previous significant token.
            if !self.insignificant_tokens.contains(&kind.to_string()) { 
                prev_token = token.clone();
            }
            i += 1;
        }
        // Sanity check.
        if scan_stack.len() > 0 {
            println!("pre_scan: non-empty scan_stack");
        }
    }
    //@+node:ekr.20241004154345.5: *3* Annotator.finish_arg
    fn finish_arg(&self, end: usize, state: &ScanState) {
        //! Set context for all ":" when scanning from "(" to ")".

        // Sanity checks.
        if state.kind == "dummy" {
            return;
        }
        assert!(state.kind == "arg");
        assert!(state.token.value == "(");
        let indices = &state.indices;
        if indices.len() == 0 {
            return;
        }

        // *** let mut i1 = token.index;
        let i1 = 0;  // *** add mut later.
        // assert i1 < end, (i1, end)

        // Compute the context for each *separate* "=" token.
        let mut equal_context = "initializer";
        for i in indices {
            let token = &self.input_tokens[*i];
            assert!(token.kind == "op");
            if token.value == "," {
                equal_context = "initializer";
            }
            else if token.value == ":" {
                equal_context = "annotation";
            }
            else if token.value == "=" {
                self.set_context(*i, equal_context);
                equal_context = "initializer";
            }
        }
        // Set the context of all outer-level ":", "*", and "**" tokens.
        let mut prev_token = &InputTok::new("dummy", "");
        for i in i1..end {
            let token = &self.input_tokens[i];
            if !self.insignificant_tokens.contains(&token.kind.to_string()) {
                if token.kind == "op" {
                    // if ["*", "**"].contains(token.value) {
                    if token.value == "*" || token.value == "**" {
                        if self.is_unary_op_with_prev(&prev_token, &token) {
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
    //@+node:ekr.20241004154345.6: *3* Annotator.finish_slice
    fn finish_slice(&self, end: usize, state: &ScanState) {
        //! Set context for all ":" when scanning from "[" to "]".

        // Sanity checks.
        assert!(state.kind == "slice");
        
        let token = state.token;
        assert!(token.value == "[");
        
        let colons = &state.indices;
        
        // *** let mut i1 = token.index;
        let i1 = 0;
        // assert i1 < end, (i1, end)

        // Do nothing if there are no ":" tokens in the slice.
        if colons.len() == 0 {
            return;
        }

        // Compute final context by scanning the tokens.
        let mut final_context = "simple-slice";
        let mut inter_colon_tokens = 0;
        let mut prev: &InputTok = &token;
        for i in i1 + 1..end - 1 {
            let token = &self.input_tokens[i];
            let (kind, value) = (token.kind, token.value);
            if !self.insignificant_tokens.contains(&kind.to_string()) {
                if kind == "op" {
                    if *value == *"." {
                        // Ignore "." tokens and any preceding "name" token.
                        if prev.kind == "name" {
                            inter_colon_tokens -= 1;
                        }
                    }
                    else if *value == *":" {
                        inter_colon_tokens = 0;
                    }
                    else if *value == *"-" || *value == *"+" {
                        // Ignore unary "-" or "+" tokens.
                        if !self.is_unary_op_with_prev(&prev, &token) {
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
                prev = &token;
            }
        }
        // Set the context of all outer-level ":" tokens.
        for i in colons {
            self.set_context(*i, final_context);
        }    
    }
    //@+node:ekr.20241004154345.7: *3* Annotator.finish_dict
    // ***
    #[allow(unused_variables)]
    fn finish_dict(&self, end: usize, state: &ScanState) {
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
    //@+node:ekr.20241005091217.1: *3* Annotator.is_python_keyword (to do)
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

    fn is_python_keyword(&self, token: &InputTok) -> bool {  // *** Temp.
        //! Return True if token is a 'name' token referring to a Python keyword.
        if token.kind != "name" {
            return false;
        }
        // let word = &token.value;  // &String
        return false;  // ***
    }
    //@+node:ekr.20241005092549.1: *3* Annotator.is_unary_op_with_prev (to do)
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

    fn is_unary_op_with_prev(&self, _prev_token: &InputTok, _token: &InputTok) -> bool {  // *** Temp. _
        return false;  // ***
    }
    //@+node:ekr.20241004163018.1: *3* Annotator.set_context
    fn set_context(&self, _i: usize, _context: &str) {  // *** temp.
        //! Set self.input_tokens[i].context, but only if it does not already exist!
        //! See the docstring for pre_scan for details.

        // *** Rewrite.

        // let trace = false;  // Do not delete the trace below.
        // let valid_contexts = [
            // "annotation", "arg", "complex-slice", "simple-slice",
            // "dict", "import", "initializer",
        // ];
        // if !valid_contexts.contain(context) {
            // // self.oops(f"Unexpected context! {context!r}")
            // println!("Unexpected context! {context:?}");
        // }
        // let token = self.input_tokens[i];
        // if trace {  // Do not delete.
            // let token_kind = token.kind;
            // let token_val = token.show_val(12);
            // let token_s = f!("<{token_kind}: {token_val}>");
            // let ignore_s = if token.context { "Ignore" } else { "      "};
            // println!("{i:3} {ignore_s} token: {token_s} context: {context}");
        // }
        // *** Rewrite
        // if token.context.len() == 0 {  // **
            // token.context.push(context);
        // }
    }
    //@-others
}
//@+node:ekr.20240929024648.120: ** class InputTok
#[derive(Clone, Debug)]
struct InputTok<'a> {
    kind: &'a str,
    value: &'a str,
}

impl <'a> InputTok<'_> {
    fn new(kind: &'a str, value: &'a str) -> InputTok<'a> {
        InputTok {
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
    //@+node:ekr.20240929074037.114: *3*  LB::new
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
    //@+node:ekr.20240929074037.2: *3* LB::add_output_string
    #[allow(unused_variables)]
    fn add_output_string(&mut self, kind: &str, value: &str) {
        //! Add value to the output list.
        //! kind is for debugging.
        if !value.is_empty() {
            self.output_list.push(value.to_string())
        }
    }
    //@+node:ekr.20241004095735.1: *3* LB::annotate_tokens (** finish)
    fn annotate_tokens(&mut self, input_list: &Vec<InputTok>) -> Vec::<AnnotatedInputTok> {
        //! Do the prepass, returning tokens annotated with context.
        let mut result = Vec::new();
        for token in input_list {
            let context = Vec::new();
            let annotated_tok = AnnotatedInputTok::new(context, &token.kind, &token.value);
            result.push(annotated_tok)
        }
        return result;
    }
    //@+node:ekr.20240929074037.113: *3* LB::beautify
    fn beautify(&mut self, annotated_list: &Vec<AnnotatedInputTok>) -> String {
        //! Beautify the input_tokens, creating the output String.
        for input_token in annotated_list {
            //@+<< LB: beautify: dispatch on input_token.kind >>
            //@+node:ekr.20241002062655.1: *4* << LB: beautify: dispatch on input_token.kind >>
            let kind = input_token.kind.as_str();
            let value = input_token.kind.as_str();
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
            //@-<< LB: beautify: dispatch on input_token.kind >>
        }
        // return ''.join(self.output_list);
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
    //@+node:ekr.20240929074037.4: *3* LB::beautify_all_files
    pub fn beautify_all_files(&mut self) {
        // for file_name in self.files_list.clone() {
        for file_name in self.files_list.clone() {
            self.beautify_one_file(&file_name);
        }
    }

    //@+node:ekr.20240929074037.5: *3* LB::beautify_one_file
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
        let annotated_tokens = self.annotate_tokens(&input_tokens);
        self.stats.annotation_time += t3.elapsed().as_nanos();
        // Beautify.
        let t4 = std::time::Instant::now();
        self.beautify(&annotated_tokens);
        self.stats.beautify_time += t4.elapsed().as_nanos();
    }
    //@+node:ekr.20240929074037.7: *3* LB::do_*
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
    //@+node:ekr.20240929074037.112: *3* LB::make_input_list
    fn make_input_list<'a>(&mut self, contents: &'a str) -> Vec<InputTok<'a>> {
        //! Return an input_list from the tokens given by the RustPython lex.
        let mut n_tokens: u64 = 0;
        let mut n_ws_tokens: u64 = 0;
        let mut prev_start: usize = 0;
        let mut result: Vec<InputTok> = Vec::new();
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
                result.push(InputTok::new("ws", ws));
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
            result.push(InputTok::new(class_name, tok_value));
        }
        // Update counts.
        self.stats.n_tokens += n_tokens;
        self.stats.n_ws_tokens += n_ws_tokens;
        return result;
    }
    //@+node:ekr.20240929074037.115: *3* LB::show_args
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
    //@+node:ekr.20240929074037.116: *3* LB::show_help
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
    //@+node:ekr.20241002163554.1: *3* LB::string_to_static_str
    fn string_to_static_str(&self, s: String) -> &'static str {
        Box::leak(s.into_boxed_str())
    }

    //@-others
}
//@+node:ekr.20241004112826.1: ** class ParseState
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

// #[allow(dead_code)]
// #[allow(non_snake_case)]
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
        println!(" annotation: {annotation_time:>7} ms");
        println!("   beautify: {beautify_time:>7} ms");
        println!("      write: {write_time:>7} ms");
        println!("      total: {total_time:>7} ms");
    }
    //@-others
}
//@+node:ekr.20241003093722.1: ** fn main
pub fn main() {
    // Main line of beautifier.
    let mut x = Beautifier::new();
    if true {
        // testing.
        println!("");
        for file_path in [
            "C:\\Repos\\leo-editor\\leo\\core\\leoFrame.py",
            // "C:\\Repos\\leo-editor\\leo\\core\\leoApp.py"
        ] {
            x.beautify_one_file(&file_path);
        }
        x.stats.report();
    } else {
        if x.enabled("--help") || x.enabled("-h") {
            x.show_help();
            return;
        }
        x.show_args();
        x.beautify_all_files();
    }
}
//@+node:ekr.20241001093308.1: ** fn test & helpers
fn test() {
    test_vec();
    test_struct();
}
//@+node:ekr.20241003094218.2: *3* fn test_struct
fn test_struct() {
    //! Test code for Vec.
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
    println!("");
    for z in &v {
        // or just v.
        println!("{z:?}");
    }
    let tok = &v[0]; // v[0] fails.
    println!("\ntok: {tok:?}");

    // A data race happens when these three behaviors occur:

    // - Two or more pointers access the same data at the same time.
    // - At least one of the pointers is being used to write to the data.
    // - There’s no mechanism being used to synchronize access to the data.

    // So This fails
    // {
    // let tok = v[0];
    // println!("{tok:?}");
    // }
}

fn push_struct(v: &mut Vec<TestTok>, val: i32) {
    let mut tok = TestTok { value: 0 };
    tok.value = val; // To test mutability.
    v.push(tok);
}
//@+node:ekr.20241003094218.1: *3* fn test_vec & push_vec
fn test_vec() {
    //! Test code for Vec.
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
