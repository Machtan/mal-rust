#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;

pub mod types;
pub mod env;
#[macro_use]
pub mod macros;
pub mod reader;
pub mod printer;
pub mod core;

pub mod errors {
    fn linepos(src: &str, pos: usize) -> (usize, usize) {
        let mut line_start = 0;
        let mut lineno = 1;
        for (i, ch) in src.char_indices() {
            if i == pos {
                let col = (&src[line_start..pos]).chars().count() + 1;
                return (lineno, col);
            }
            if ch == '\n' {
                line_start = i + 1;
                lineno += 1;
            }
        }
        let col = (&src[line_start..]).chars().count() + 1;
        return (lineno, col);
    }
    
    fn linepos_str(src: &str, pos: usize) -> String {
        //println!("linepos_str(src: {:?}, pos: {})", src, pos);
        let (line, col) = linepos(src, pos);
        format!("{}:{}", line, col)
    }
    
    error_chain! {
        errors {
            Lexer { pos: usize, source: String, msg: String } {
                display("Lexer: {}| {}", linepos_str(&source, *pos), msg)
            }
            Reader { pos: usize, source: String, msg: String } {
                display("Reader: {}| {}", linepos_str(&source, *pos), msg)
            }
            TypeError { expected: String, got: String } {
                display("Type error: Expected {}, got {}", expected, got)
            }
        }
    }
}

pub use errors::*;
pub use types::{Mal, MalList, MalArr, MalMap, Keyword, Symbol, MalFunc};
pub use env::Env;
pub use reader::read_str;
pub use printer::pr_str;
pub use core::core_env;
