use types::{Mal, MalList};
use env::Env;
use printer;
use errors::*;

/// Returns the core environment.
pub fn core_env() -> Env {
    let mut env = Env::new();
    env.add_native_func("+", add).unwrap();
    env.add_native_func("-", sub).unwrap();
    env.add_native_func("*", mul).unwrap();
    env.add_native_func("/", div).unwrap();
    env.add_native_func("list", list).unwrap();
    env.add_native_func("list?", listp).unwrap();
    env.add_native_func("empty?", emptyp).unwrap();
    env.add_native_func("count", count).unwrap();
    env.add_native_func("=", eq).unwrap();
    // TODO: favor names like 'gt' (arrows seem misleading in lisp-syntax) 
    env.add_native_func("<", lt).unwrap(); 
    env.add_native_func("<=", le).unwrap();
    env.add_native_func(">", gt).unwrap();
    env.add_native_func(">=", ge).unwrap();
    env.add_native_func("pr-str", pr_str).unwrap();
    env.add_native_func("str", str_).unwrap();
    env.add_native_func("prn", prn).unwrap();
    env.add_native_func("println", println).unwrap();
    env
}

fn println(args: &mut MalList) -> Result<Mal> {
    if let Mal::Str(string) = str_(args).unwrap() {
        println!("{}", string);
        Ok(Mal::Nil)
    } else {
        unreachable!();
    }
}

fn prn(args: &mut MalList) -> Result<Mal> {
    if let Mal::Str(string) = pr_str(args).unwrap() {
        println!("{}", string);
        Ok(Mal::Nil)
    } else {
        unreachable!();
    }
}

fn str_(args: &mut MalList) -> Result<Mal> {
    if args.is_empty() {
        return Ok(String::from("").into());
    }
    let first = args.pop_front().unwrap();
    let mut string = printer::pr_str(&first, false);
    for arg in args.drain(..) {
        printer::pr_str_into(&arg, &mut string, false);
    }
    Ok(string.into())
}

fn pr_str(args: &mut MalList) -> Result<Mal> {
    if args.is_empty() {
        return Ok(String::from("").into());
    }
    let first = args.pop_front().unwrap();
    let mut string = printer::pr_str(&first, true);
    for arg in args.drain(..) {
        string.push(' ');
        printer::pr_str_into(&arg, &mut string, true);
    }
    Ok(string.into())
}

fn ge(args: &mut MalList) -> Result<Mal> {
    if args.len() < 2 {
        bail!("'>=' takes 2 or more arguments, got {}", args.len());
    }
    let mut prev = args.pop_front().unwrap().number()?;
    while ! args.is_empty() {
        let test = args.pop_front().unwrap().number()?;
        if prev < test {
            return Ok(false.into());
        }
        prev = test;
    }
    Ok(true.into())
}

fn gt(args: &mut MalList) -> Result<Mal> {
    if args.len() < 2 {
        bail!("'>' takes 2 or more arguments, got {}", args.len());
    }
    let mut prev = args.pop_front().unwrap().number()?;
    while ! args.is_empty() {
        let test = args.pop_front().unwrap().number()?;
        if prev <= test {
            return Ok(false.into());
        }
        prev = test;
    }
    Ok(true.into())
}

fn le(args: &mut MalList) -> Result<Mal> {
    if args.len() < 2 {
        bail!("'<=' takes 2 or more arguments, got {}", args.len());
    }
    let mut prev = args.pop_front().unwrap().number()?;
    while ! args.is_empty() {
        let test = args.pop_front().unwrap().number()?;
        if prev > test {
            return Ok(false.into());
        }
        prev = test;
    }
    Ok(true.into())
}

fn lt(args: &mut MalList) -> Result<Mal> {
    if args.len() < 2 {
        bail!("'<' takes 2 or more arguments, got {}", args.len());
    }
    let mut prev = args.pop_front().unwrap().number()?;
    while ! args.is_empty() {
        let test = args.pop_front().unwrap().number()?;
        if prev >= test {
            return Ok(false.into());
        }
        prev = test;
    }
    Ok(true.into())
}


fn eq(args: &mut MalList) -> Result<Mal> {
    if args.len() < 2 {
        bail!("'=' takes 2 or more arguments, got {}", args.len());
    }
    let first = args.pop_front().unwrap();
    Ok(args.drain(..).all(|arg| arg == first).into())
}

fn assert_nargs(name: &str, nargs: usize, args: &MalList) -> Result<()> {
    if args.len() != nargs {
        bail!("'{}' takes {} arguments, found {}", name, nargs, args.len());
    }
    Ok(())
}

fn emptyp(args: &mut MalList) -> Result<Mal> {
    assert_nargs("empty?", 1, args)?;
    let arg = args.pop_front().unwrap();
    match arg {
        Mal::List(ref list) => Ok(list.is_empty().into()),
        Mal::Arr(ref arr) => Ok(arr.is_empty().into()),
        Mal::Map(ref map) => Ok(map.is_empty().into()),
        ref other => bail!("'empty?' takes a collection type, found {}", other.type_name()),
    }
}

fn count(args: &mut MalList) -> Result<Mal> {
    assert_nargs("count", 1, args)?;
    let arg = args.pop_front().unwrap();
    match arg {
        Mal::List(ref list) => Ok((list.len() as f64).into()),
        Mal::Arr(ref arr) => Ok((arr.len() as f64).into()),
        Mal::Map(ref map) => Ok((map.len() as f64).into()),
        Mal::Nil => Ok(0.0f64.into()),
        ref other => bail!("'count' takes a collection type (or nil), found {}", other.type_name()),
    }
}

fn listp(args: &mut MalList) -> Result<Mal> {
    assert_nargs("list?", 1, args)?;
    let arg = args.pop_front().unwrap();
    if let Mal::List(_) = arg {
        Ok(Mal::Bool(true))
    } else {
        Ok(Mal::Bool(false))
    }
}

fn list(args: &mut MalList) -> Result<Mal> {
    Ok(args.clone().into())
}

fn add(args: &mut MalList) -> Result<Mal> {
    if args.len() < 2 {
        bail!("'+' requires at least 2 arguments!");
    }
    let mut sum = 0.0;
    for arg in args.iter() {
        sum += arg.number()?;
    }
    Ok(Mal::Num(sum))
}

fn sub(args: &mut MalList) -> Result<Mal> {
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

fn mul(args: &mut MalList) -> Result<Mal> {
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

fn div(args: &mut MalList) -> Result<Mal> {
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
