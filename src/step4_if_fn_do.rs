
extern crate mal;
#[macro_use]
extern crate error_chain;

mod eval;

use mal::{Mal, Env, MalFunc, Symbol, MalList};
use std::io::{self, Write, BufRead};
use std::env;
use std::iter;
use eval::eval;

fn read(text: &str) -> mal::Result<Mal> {
    mal::read_str(text)
}

fn print(mal: &Mal) -> String {
    mal::pr_str(mal, true)
}

fn rep(text: &str, env: &mut Env) -> mal::Result<String> {
    let mut val = read(text)?;
    eval(&mut val, env)?;
    let text = print(&val);
    Ok(text)
}


fn print_err(e: &mal::Error) {
    use ::std::io::Write;
    let stderr = &mut ::std::io::stderr();
    let errmsg = "Error writing to stderr";
    let indent = 2;

    writeln!(stderr, "error: {}", e).expect(errmsg);

    for (i, e) in e.iter().skip(1).enumerate() {
        write!(stderr, "{}", iter::repeat(" ").take(indent + i * indent).collect::<String>()).expect(errmsg);
        writeln!(stderr, "caused by: {}", e).expect(errmsg);
    }

    // The backtrace is not always generated. Try to run this example
    // with `RUST_BACKTRACE=1`.
    /*if let Some(backtrace) = e.backtrace() {
        writeln!(stderr, "backtrace: {:?}", backtrace).expect(errmsg);
    }*/
}

fn nop(_args: &mut MalList) -> mal::Result<Mal> {
    Ok(Mal::Nil)
}

const MAL_DEFS: &'static str = "
(def! not (fn* (a) (if a false true)))
";

fn main() {
    let mut env = mal::core_env();
    for line in MAL_DEFS.lines() {
        if line == "" { continue; }
        let mut defs = read(MAL_DEFS).expect("Could not read def");
        eval(&mut defs, &mut env).expect("Could not eval def");
    }
    
    // If args are given, don't start in interactive mode.
    let args = env::args().skip(1).collect::<Vec<_>>();
    if ! args.is_empty() {
        // Overwrite the print functions to avoid bad output!
        let nopfunc = MalFunc::Native("nop", nop);
        env.set(Symbol::new("prn"), nopfunc.clone());
        env.set(Symbol::new("println"), nopfunc.clone());
        
        for arg in args {
            match rep(&arg, &mut env) {
                Ok(res) => {
                    println!("{}", res);
                    let stdout = io::stdout();
                    stdout.lock().flush().unwrap();
                }
                Err(ref e) => {
                    print_err(e);
                    ::std::process::exit(1);
                }
            }
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
        
        match rep(&input, &mut env) {
            Ok(string) => {
                println!("{}", string);
                let stdout = io::stdout();
                stdout.lock().flush().unwrap();
            }
            Err(ref e) => {
                print_err(e);
            }
        }
    }
}
