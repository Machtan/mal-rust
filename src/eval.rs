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
        Arr(ref mut arr) => {
            for item in arr.iter_mut() {
                env.with_new_scope(|env| {
                    eval(item, env)
                })?;
            }
        }
        Map(ref mut map) => {
            for (_, item) in map.iter_mut() {
                env.with_new_scope(|env| {
                    eval(item, env)
                })?;
            }
        }
        List(_) => {
            unreachable!();
            //eval_list(list, env)?;
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
        // List members should not share definitions.
        env.with_new_scope(|env| {
            eval(item, env)
        })?;
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
            // Make closures 'specially' able to refer to what they're bound to.
            if let Mal::Fn(MalFunc::Closure(args, env, body)) = val {
                val = MalFunc::NamedClosure(sym.clone(), args, env, body).into();
            }
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
            let mut next_is_vararg = false;
            let mut has_vararg = false;
            for arg in arg_ref.drain(..) {
                let sym = arg.symbol().chain_err(|| "fn*: Invalid argument list")?;
                if sym.text() == "&" {
                    if has_vararg {
                        bail!("fn*: & (vararg operator) used twice!");
                    } else {
                        has_vararg = true;
                        next_is_vararg = true;
                    }
                } else {
                    if next_is_vararg {
                        next_is_vararg = false;
                    } else if has_vararg {
                        bail!("fn*: Got more than one argument after '&'");
                    }
                }
                arg_list.push_back(sym);

            }
            let body = list.pop_front().unwrap();
            Ok(Mal::Fn(MalFunc::Closure(arg_list, env.clone(), Box::new(body))))
        }
        "do" => { // TODO: Is 'do' actually a new scope? Apparently not.
            let mut res = Mal::Nil;
            for arg in list.drain(..) {
                res = arg;
                eval(&mut res, env)?;
            }
            Ok(res)
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
    use mal::types::MalFunc::*;
    eval(func, env)?;
    let function = func.as_function()?;
    env.with_new_scope(|env| {
        // Evaluate the argument values in the outer env.
        eval_list(args, env)
    })?;
    match *function {
        // NOTE: This should've been cloned at env.get, so safe to modify.
        Closure(ref mut arg_names, ref mut closure_env, ref mut body) => {
            apply_closure(None, arg_names, args, closure_env, body)
        }
        NamedClosure(ref mut name, ref mut arg_names, 
                ref mut closure_env, ref mut body) => {
            let self_reference = NamedClosure(name.clone(), arg_names.clone(), 
                closure_env.clone(), body.clone());
            closure_env.set(name.clone(), self_reference);
            apply_closure(Some(name.text()), arg_names, args, closure_env, body)
        }
        Native(_, ref mut func) => {
            func(args)
        }
    }
}

fn apply_closure(name: Option<&str>, arg_names: &mut VecDeque<Symbol>, args: &mut MalList, 
        closure_env: &mut Env, body: &mut Mal) -> mal::Result<Mal> {
    
    let takes_varargs = arg_names.iter().any(|arg| arg.text() == "&");
    if ! takes_varargs {
        assert_arg_len(name.unwrap_or("#<function>"), arg_names.len(), args)?;
    } else {
        let nargs = arg_names.len() - 2;
        if args.len() < nargs {
            bail!("'{}' takes {} or more arguments, found {}!", 
                name.unwrap_or("#<function>"), nargs, args.len());
        }
    }
    
    // Bind the arguments in the closure env.
    let mut is_vararg = false;
    for name in arg_names.drain(..) {
        let symbol = name.clone();
        if symbol.text() == "&" {
            is_vararg = true;
            continue;
        }
        if ! is_vararg {
            let value = args.pop_front().unwrap();
            closure_env.set(symbol, value);
        } else {
            closure_env.set(symbol, args.clone());
            args.clear();
        }
    }

    eval(body, closure_env)?;
    // Clone the reduced result for less garbage.
    Ok(body.clone())
}
