/*extern crate mal;

use mal::{Mal, Env, MalList, MalArr, MalMap};
use std::io::{self, Write, BufRead};
use std::env;

// TODO: Cow the Env.

fn read(text: &str) -> mal::Result<Mal> {
    mal::read_str(text)
}

/// Resolves symbols to their bound environment values.
fn eval_ast(expr: Mal, env: Env) -> mal::Result<Mal> {
    use mal::Mal::*;
    match expr {
        Sym(ident) => Ok(env.get(&ident)?),
        List(list) => {
            let mut new_list = MalList::new();
            for item in list.iter() {
                let val = eval(item.clone(), env.clone())?;
                new_list.push_back(val);
            }
            Ok(new_list.into())
        }
        Arr(arr) => {
            let mut new_arr = MalArr::new();
            for item in arr.iter() {
                let val = eval(item.clone(), env.clone())?;
                new_arr.push_back(val);
            }
            Ok(new_arr.into())
        }
        Map(map) => {
            let mut new_map = MalMap::new();
            for (key, item) in map.iter() {
                let val = eval(item.clone(), env.clone())?;
                new_map.insert(key.clone(), val);
            }
            Ok(new_map.into())
        }
        other => Ok(other),
    }
}

/// Evaluates list forms.
fn eval(expr: Mal, env: Env) -> mal::Result<Mal> {
    match expr {
        Mal::List(list) => {
            if list.is_empty() {
                Ok(list.into())
            } else {
                let mut evaled = eval_ast(list.into(), env.clone())?.list().unwrap();
                let first = evaled.pop_front().unwrap();
                first.call(evaled)
            }
        }
        other => eval_ast(other, env.clone())
    }
}

fn print(mal: &Mal) -> String {
    mal::pr_str(mal, true)
}

fn rep(text: &str, env: Env) -> mal::Result<String> {
    Ok(print(&eval(read(text)?, env)?))
}


fn print_err(e: &mal::Error) {
    use ::std::io::Write;
    let stderr = &mut ::std::io::stderr();
    let errmsg = "Error writing to stderr";

    writeln!(stderr, "error: {}", e).expect(errmsg);

    for e in e.iter().skip(1) {
        writeln!(stderr, "caused by: {}", e).expect(errmsg);
    }

    // The backtrace is not always generated. Try to run this example
    // with `RUST_BACKTRACE=1`.
    /*if let Some(backtrace) = e.backtrace() {
        writeln!(stderr, "backtrace: {:?}", backtrace).expect(errmsg);
    }*/
}

fn main() {
    let env = mal::core_env();
    
    // If args are given, don't start in interactive mode.
    let args = env::args().skip(1).collect::<Vec<_>>();
    if ! args.is_empty() {
        for arg in args {
            match rep(&arg, env.clone()) {
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
        
        match rep(&input, env.clone()) {
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
}*/
fn main() {}
