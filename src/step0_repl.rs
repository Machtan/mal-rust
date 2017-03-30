use std::io::{self, Write, BufRead};
use std::env;

fn read(text: &str) -> &str {
    text
}

fn eval(text: &str) -> &str {
    text
}

fn print(text: &str) -> &str {
    text
}

fn rep(text: &str) -> &str {
    print(eval(read(text)))
}


fn main() {
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
        print!("{}", rep(&input));
    }
}