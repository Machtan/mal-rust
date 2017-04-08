use types::{Mal, MapKey, MalFunc};
use std::fmt::Write;

fn pr_malstr_into(s: &str, string: &mut String, print_readably: bool) {
    if ! print_readably {
        string.push_str(s);
    } else {
        string.push('"');
        for ch in s.chars() {
            match ch {
                '"' => {
                    string.push('\\');
                    string.push('"');
                }
                '\\' => {
                    string.push('\\');
                    string.push('\\');
                }
                '\n' => {
                    string.push('\\');
                    string.push('n');
                }
                ch => string.push(ch),
            }
        }
        string.push('"');
    }
}

pub fn pr_str_into(mal: &Mal, string: &mut String, print_readably: bool) {
    use types::Mal::*;
    match *mal {
        Num(num) => write!(string, "{}", num).unwrap(),
        Sym(ref sym) => string.push_str(sym.text()),
        Bool(true) => string.push_str("true"),
        Bool(false) => string.push_str("false"),
        Nil => string.push_str("nil"),
        Kw(ref keyword) => {
            string.push(':');
            string.push_str(keyword.symbol());
        }
        Str(ref s) => {
            pr_malstr_into(s, string, print_readably);
        }
        Fn(ref f) => {
            match *f {
                MalFunc::Native(name, _) => string.push_str(name),
                MalFunc::Closure(ref args, ref _env, ref body) |
                MalFunc::NamedClosure(_, ref args, ref _env, ref body) => {
                    if ! print_readably {
                        string.push_str("#<function>");
                    } else {
                        string.push_str("(fn* (");
                        let len = args.len();
                        if len != 0 {
                            let last = len - 1;
                            for (i, sym) in args.iter().enumerate() {
                                string.push_str(sym.text());
                                if i != last {
                                    string.push(' ');
                                }
                            }
                        }
                        string.push_str(") ");
                        pr_str_into(body, string, print_readably);
                        string.push(')');
                    }
                }
            }
        }
        List(ref list) => {
            string.push('(');
            let len = list.len();
            if len != 0 {
                let last = len - 1;
                for (i, item) in list.iter().enumerate() {
                    pr_str_into(item, string, print_readably);
                    if i != last {
                        string.push(' ');
                    }
                }
            }
            string.push(')');
        }
        Arr(ref arr) => {
            string.push('[');
            let len = arr.len();
            if len != 0 {
                let last = len - 1;
                for (i, item) in arr.iter().enumerate() {
                    pr_str_into(item, string, print_readably);
                    if i != last {
                        string.push(' ');
                    }
                }
            }
            string.push(']');
        }
        Map(ref map) => {
            string.push('{');
            for (k, v) in map.inner.iter() {
                match *k {
                    MapKey::Str(ref s) => pr_malstr_into(s, string, print_readably),
                    MapKey::Kw(ref kw) => {
                        string.push(':');
                        string.push_str(kw.symbol());
                    }
                }
                string.push(' ');
                pr_str_into(v, string, print_readably);
            }
            string.push('}');
        }
    }
}

pub fn pr_str(mal: &Mal, print_readably: bool) -> String {
    let mut string = String::new();
    pr_str_into(mal, &mut string, print_readably);
    //println!("pr_str({:?}) -> {:?}", mal, string);
    string
}

