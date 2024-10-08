
# ekr-tbo-in-rust

The project transliterates Leo's Token Based beautifier (TBO) from Python to Rust.

### Status

- The Rust code compiles correctly.
- *This project is too slow to replace Leo's Python beautifier.*
   The present code is only about 10% faster than the Python code!
- *The project contains significant logic errors.*
   In particular, no context is ever added to any annotated input token.
   Fixing these errors is beyond the scope of the preset work.

### Repos

ekr-tbo-in-rust: https://github.com/edreamleo/ekr-tbo-in-rust
ruff_python_parser: https://github.com/astral-sh/ruff/tree/main/crates/ruff_python_parser/src
lexer.rs: https://github.com/astral-sh/ruff/blob/main/crates/ruff_python_parser/src/lexer.rs

### Statistics

This project is too slow to replace Leo's Python beautifier.
As shown below, the culprit is `LB.make_input_tokens`.
As a result, the project can not be significantly faster than Leo's python beautifier!

**Python stats, with extra tracing code in tbo.init_tokens_from_file**:

```
python -c "import leo.core.leoTokens" --all --report leo\core\leoFrame.py
tbo: 0.03 sec. dirty: 0   checked: 1   beautified: 0   in leo\core\leoFrame.py

       read:   0.28 ms
make_tokens:  29.45 ms
      total:  29.73 ms
```  
**Rust, with an empty loop in make_input_tokens:**
```
leoFrame.py

     files: 1, tokens: 14619, ws tokens: 5156
       read:    0.5 ms
make_tokens:   10.7 ms  Empty loop
   beautify:    7.3 ms
      write:    0.0 ms
      total:   18.5 ms
```
**Rust, with latest code**:
```
leoFrame.py

annotate: self.input_tokens.len(): 19775
annotate: self.index_dict: 0

     files: 1, tokens: 14619, ws tokens: 5156
       read:    0.53 ms
make_tokens:   11.75 ms
   annotate:    6.69 ms
   beautify:    4.11 ms
      write:    0.00 ms
      total:   23.10 ms
```
**Notes**:
- The latest Rust code is only about 10% faster than the Python code.
- The index dict contains no entries, so no annotated tokens contain any context.
  This is a serious logic error. Fixing it is beyond the scope of this project.

