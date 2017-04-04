extern crate mal;
#[macro_use]
extern crate error_chain;

use mal::{Mal, Env, MalList};
use std::io::{self, Write, BufRead};
use std::env;
use mal::errors::*;
use std::iter;


fn read(text: &str) -> mal::Result<Mal> {
    mal::read_str(text)
}

/// Resolves symbols to their environment values.
fn eval_ast(expr: &mut Mal, env: &mut Env) -> mal::Result<()> {
    use mal::Mal::*;
    let mut new_val = None;
    match *expr {
        Sym(ref ident) => {
            new_val = Some(env.get(&ident)?);
            
        },
        List(ref mut list) => {
            eval_list(list, env)?;
        }
        Arr(ref mut arr) => {
            for item in arr.iter_mut() {
                eval(item, env)?;
            }
        }
        Map(ref mut map) => {
            for (_, item) in map.iter_mut() {
                eval(item, env)?;
            }
        }
        _ => {},
    }
    if let Some(val) = new_val {
        *expr = val;
    }
    Ok(())
}

fn eval_list(list: &mut MalList, env: &mut Env) -> mal::Result<()> {
    for item in list.iter_mut() {
        eval(item, env)?;
    }
    Ok(())
}

fn assert_arg_len(name: &str, nargs: usize, args: &MalList) -> mal::Result<()> {
    if args.len() != nargs {
        bail!("'{}' takes {} arguments, found {}", name, nargs, args.len());
    }
    Ok(())
}

fn apply(list: &mut MalList, env: &mut Env) -> mal::Result<Mal> {
    let first = list.pop_front().unwrap().symbol()?;
    match first.text() {
        "def!" => {
            assert_arg_len("def!", 2, list)?;
            let sym = list.pop_front().unwrap().symbol()
                .chain_err(|| "def!: Invalid first argument")?;
            let mut val = list.pop_front().unwrap();
            eval(&mut val, env)?;
            env.set(sym, val.clone());
            Ok(val)
        }
        "let*" => {
            assert_arg_len("let*", 2, list)?;
            let mut bindings = list.pop_front().unwrap().list()
                .chain_err(|| "let*: Invalid list of bindings")?;
            if (bindings.len() % 2) != 0 {
                bail!("let*: odd number of elements in binding list");
            }
            
            env.with_new_scope(|env| {
                while ! bindings.is_empty() {
                    let sym = bindings.pop_front().unwrap().symbol()
                        .chain_err(|| "let*: Invalid variable")?;
                    let mut val = bindings.pop_front().unwrap();
                    eval(&mut val, env)?;
                    env.set(sym, val);
                }
                let mut expr = list.pop_front().unwrap();
                eval(&mut expr, env)?;
                Ok(expr)
            })
        }
        _ => {
            let func = env.get(&first)?;
            env.with_new_scope(|env| {
                eval_list(list, env)
            })?;
            func.call(list)
        }
    }
}

/// Evaluates list forms.
fn eval(expr: &mut Mal, env: &mut Env) -> mal::Result<()> {
    let mut new_val = None;
    match *expr {
        Mal::List(ref mut list) => {
            if ! list.is_empty() {
                new_val = Some(apply(list, env)?);
            }
        }
        _ => eval_ast(expr, env)?,
    }
    if let Some(val) = new_val {
        *expr = val;
    }
    Ok(())
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

fn main() {
    let mut env = mal::core_env();
    
    // If args are given, don't start in interactive mode.
    let args = env::args().skip(1).collect::<Vec<_>>();
    if ! args.is_empty() {
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
