use types::Mal;
use std::fmt::Write;

fn pr_str_into(mal: &Mal, string: &mut String, print_readably: bool) {
    use types::Mal::*;
    match *mal {
        Number(num) => write!(string, "{}", num).unwrap(),
        Symbol(ref sym) => string.push_str(sym),
        Bool(true) => string.push_str("true"),
        Bool(false) => string.push_str("false"),
        Nil => string.push_str("nil"),
        Str(ref s) => {
            if ! print_readably {
                write!(string, "\"{}\"", s).unwrap();
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
    }
}

pub fn pr_str(mal: &Mal, print_readably: bool) -> String {
    let mut string = String::new();
    pr_str_into(mal, &mut string, print_readably);
    //println!("pr_str({:?}) -> {:?}", mal, string);
    string
}

