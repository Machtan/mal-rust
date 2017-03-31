#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;

pub mod types;
pub mod reader;
pub mod printer;

mod errors {
    fn linepos(src: &str, pos: usize) -> (usize, usize) {
        let mut line_start = 0;
        let mut lineno = 1;
        for (i, ch) in src.char_indices() {
            if ch == '\n' {
                line_start = i + 1;
                lineno += 1;
            }
            if i == pos {
                let col = (&src[line_start..pos]).chars().count() + 1;
                return (lineno, col);
            }
        }
        let col = (&src[line_start..]).chars().count() + 1;
        return (lineno, col);
    }
    
    fn linepos_str(src: &str, pos: usize) -> String {
        let (line, col) = linepos(src, pos);
        format!("{}:{}", line, col)
    }
    
    error_chain! {
        errors {
            Lexer { pos: usize, source: String, msg: String } {
                display("Lexer error: {}| {}", linepos_str(&source, *pos), msg)
            }
        }
    }
}

pub use errors::*;
pub use types::{Mal, MalList, MalArr, MalMap, Keyword};
pub use reader::read_str;
pub use printer::pr_str;


