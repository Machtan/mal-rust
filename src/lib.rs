#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;

pub mod types;
pub mod env;
#[macro_use]
pub mod macros;
pub mod reader;
pub mod printer;

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

fn add(args: &MalList) -> Result<Mal> {
    if args.len() < 2 {
        bail!("'+' requires at least 2 arguments!");
    }
    let mut sum = 0.0;
    for arg in args.iter() {
        sum += arg.number()?;
    }
    Ok(Mal::Num(sum))
}

fn sub(args: &MalList) -> Result<Mal> {
    if args.len() < 2 {
        bail!("'-' requires at least 2 arguments!")
    }
    let mut vals = args.iter();
    let mut sum = vals.next().unwrap().number()?;
    for arg in vals {
        sum -= arg.number()?;
    }
    Ok(Mal::Num(sum))
}

fn mul(args: &MalList) -> Result<Mal> {
    if args.len() < 2 {
        bail!("'*' requires at least 2 arguments!")
    }
    let mut vals = args.iter();
    let mut sum = vals.next().unwrap().number()?;
    for arg in vals {
        sum *= arg.number()?;
    }
    Ok(Mal::Num(sum))
}

fn div(args: &MalList) -> Result<Mal> {
    if args.len() < 2 {
        bail!("'/' requires at least 2 arguments!")
    }
    let mut vals = args.iter();
    let mut sum = vals.next().unwrap().number()?;
    for arg in vals {
        let num = arg.number()?;
        if num == 0.0 {
            bail!("Division by 0");
        }
        sum /= num;
    }
    Ok(Mal::Num(sum))
}

pub fn core_env() -> Env {
    let mut env = Env::new();
    env.add_native_func("+", add).unwrap();
    env.add_native_func("-", sub).unwrap();
    env.add_native_func("*", mul).unwrap();
    env.add_native_func("/", div).unwrap();
    env
}
