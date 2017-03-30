mod types;
mod reader;
mod printer;

use types::Mal;
use std::io::{self, Write, BufRead};
use std::env;

fn read(text: &str) -> Mal {
    reader::read_str(text)
}

fn eval(expr: Mal) -> Mal {
    expr
}

fn print(mal: &Mal) -> String {
    printer::pr_str(mal, true)
}

fn rep(text: &str) -> String {
    print(&eval(read(text)))
}


fn main() {
    // If args are given, don't start in interactive mode.
    let args = env::args().skip(1).collect::<Vec<_>>();
    if ! args.is_empty() {
        for arg in args {
            println!("{}", rep(&arg));
            let stdout = io::stdout();
            stdout.lock().flush().unwrap();
        }
        return;
    }
    
    let mut input = String::new();
    loop {
        input.clear();
        
        print!("user> ");
        let stdout = io::stdout();
        stdout.lock().flush().unwrap();
        
        let stdin = io::stdin();
        stdin.lock().read_line(&mut input).unwrap();
        
        println!("{}", rep(&input));
        let stdout = io::stdout();
        stdout.lock().flush().unwrap();
    }
}