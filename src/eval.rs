use mal::{self, Mal, MalList, Env, MalFunc, Symbol};
use mal::errors::*;
use std::collections::VecDeque;

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

/// Evaluates list forms.
pub fn eval(expr: &mut Mal, env: &mut Env) -> mal::Result<()> {
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

fn assert_arg_len(name: &str, nargs: usize, args: &MalList) -> mal::Result<()> {
    if args.len() != nargs {
        bail!("'{}' takes {} arguments, found {}", name, nargs, args.len());
    }
    Ok(())
}

/// Evaluates the expression inside the given list.
fn apply(list: &mut MalList, env: &mut Env) -> mal::Result<Mal> {
    let mut first = list.pop_front().unwrap();
    match first {
        Mal::Sym(sym) => {
            apply_symbol(sym, list, env)
        }
        _ => {
            apply_function(&mut first, list, env)
        }
    }
}

/// Resolves a list starting with a symbol to either a special form,
/// or a function that is called.
fn apply_symbol(symbol: Symbol, list: &mut MalList, env: &mut Env) -> mal::Result<Mal> {
    match symbol.text() {
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
            let mut value = list.pop_front().unwrap();
            let bindings = value.as_list_or_array()
                .chain_err(|| "let*: Invalid set of bindings")?;
            if (bindings.len() % 2) != 0 {
                bail!("let*: odd number of elements in binding list");
            }
            
            env.with_new_scope(|env| {
                while ! bindings.is_empty() {
                    let sym = bindings.pop_front().unwrap().symbol()
                        .chain_err(|| "let*: Invalid binding variable")?;
                    let mut val = bindings.pop_front().unwrap();
                    eval(&mut val, env)?;
                    env.set(sym, val);
                }
                let mut expr = list.pop_front().unwrap();
                eval(&mut expr, env)?;
                Ok(expr)
            })
        }
        "fn*" => {
            assert_arg_len("fn*", 2, list)?;
            let mut args = list.pop_front().unwrap();
            let arg_ref = args.as_list_or_array()
                .chain_err(|| "fn*: Invalid argument list")?;
            let mut arg_list = VecDeque::new();
            while ! arg_ref.is_empty() {
                arg_list.push_back(
                    arg_ref.pop_front().unwrap().symbol()
                        .chain_err(|| "fn*: Invalid argument list")?
                );
            }
            let body = list.pop_front().unwrap();
            Ok(Mal::Fn(MalFunc::Defined(arg_list, Box::new(body))))
        }
        "do" => { // TODO: Is 'do' actually a new scope?
            env.with_new_scope(|env| {
                let mut res = Mal::Nil;
                for arg in list.drain(..) {
                    res = arg;
                    eval(&mut res, env)?;
                }
                Ok(res)
            })
        }
        "if" => {
            if ! (list.len() == 2 || list.len() == 3) {
                bail!("'if' takes 2 or 3 arguments, found {}", list.len());
            }
            let has_else = list.len() == 3;
            let mut condition = list.pop_front().unwrap();
            eval(&mut condition, env)?;
            
            let mut if_body = list.pop_front().unwrap();
            if condition.is_truesy() {
                eval(&mut if_body, env)?;
                Ok(if_body)
            } else {
                if has_else {
                    let mut else_body = list.pop_front().unwrap();
                    eval(&mut else_body, env)?;
                    Ok(else_body)
                } else {
                    Ok(Mal::Nil)
                }
            }
        }
        _ => {
            let mut func = env.get(&symbol)?;
            apply_function(&mut func, list, env)
        }
    }
}

/// Resolves the given value to a function and calls it.
fn apply_function(func: &mut Mal, args: &mut MalList, env: &mut Env) -> mal::Result<Mal> {
    eval(func, env)?;
    let function = func.as_function()?;
    env.with_new_scope(|env| {
        eval_list(args, env)
    })?;
    match *function {
        MalFunc::Defined(ref arg_names, ref body) => {
            assert_arg_len("#<function>", arg_names.len(), args)?;
            // Clone the body to let eval modify it.
            let mut body: Mal = (**body).clone();
            env.with_new_scope(|env| {
                for name in arg_names.iter() {
                    let symbol = name.clone();
                    let value = args.pop_front().unwrap();
                    env.set(symbol, value);
                }
                eval(&mut body, env)
            })?;
            Ok(body)
        }
        MalFunc::Native(_, ref mut func) => {
            func(args)
        }
    }
}
