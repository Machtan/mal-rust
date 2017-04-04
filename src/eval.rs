use mal::{self, Mal, MalList, Env};
use mal::errors::*;

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
        _ => {
            let func = env.get(&first)?;
            env.with_new_scope(|env| {
                eval_list(list, env)
            })?;
            func.call(list)
        }
    }
}
