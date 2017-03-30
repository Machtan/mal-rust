use std::iter::Peekable;
use std::str::Chars;
use types::{MalList, Mal};

// Tokens
/* 
[\s,]*
(
      ~@
    | [\[\]{}()'`~^@] 
    | "(?:\\.|[^\\"])*"
    | ;.* 
    | [^\s\[\]{}('"`,;)]*
)
*/

#[derive(Debug, Clone)]
pub enum Token {
    Tadpole, // ~@
    LBrack, // [
    RBrack, // ]
    LCurly, // {
    RCurly, // }
    LParen, // (
    RParen, // )
    Apo, // '
    Tilde, // ~
    Hat, // ^
    At, // @
    Str(String), // "with \" escapes"
    SemiCTrail(String), // ;.*
    Ident(String), // A sequence of non-ws and non-specials
}

fn is_special_char(ch: char) -> bool {
    match ch {
        '~' | '@' | '[' | ']' | '(' | ')' | '{' | '}' | '\'' | '"' | '^' | ';' | ',' => true,
        ch => ch.is_whitespace(),
    }
}

pub struct Reader<'a> {
    chars: Peekable<Chars<'a>>,
    token: Option<Token>,
}

impl<'a> Reader<'a> {
    pub fn new(text: &'a str) -> Reader<'a> {
        Reader {
            chars: text.chars().peekable(),
            token: None,
        }
    }
    
    pub fn peek(&mut self) -> Option<Token> {
        if let Some(ref token) = self.token {
            Some(token.clone())
        } else {
            self.token = self.next();
            self.token.clone()
        }
    }
    
    fn eat_or(&mut self, pat: &str, ifpat: Token, ifnot: Token) -> Token {
        let mut chars = self.chars.clone();
        let mut other = pat.chars();
        loop {
            match (chars.next(), other.next()) {
                (Some(a), Some(b)) => {
                    if a == b {
                        continue;
                    } else {
                        return ifnot;
                    }
                }
                (None, None) => break,
                (None, Some(_)) | (Some(_), None) => return ifnot,
            }
        }
        self.chars = chars;
        ifpat
    }
    
    fn eat_whitespace(&mut self) {
        loop {
            let is_ws = self.chars.peek().map_or(false, |&c| c == ',' || c.is_whitespace());
            if is_ws {
                self.chars.next();
            } else {
                break;
            }
        }
    }
    
    fn read_string(&mut self) -> Result<Token, String> {
        let mut string = String::new();
        let mut escaped = false;
        while let Some(ch) = self.chars.next() {
            if ! escaped {
                match ch {
                    '"' => return Ok(Token::Str(string)),
                    '\\' => {
                        escaped = true;
                    }
                    ch => string.push(ch),
                }
            } else {
                match ch {
                    '"' => string.push('"'),
                    '\\' => string.push('\\'),
                    'n' => string.push('\n'),
                    ch => {
                        string.push('\\');
                        string.push(ch);
                    }
                }
                escaped = false;
            }
        }
        Err(format!("Unterminated string: '\"{}'", string))
    }
    
    fn trail(&mut self) -> String {
        let mut trail = String::new();
        while let Some(ch) = self.chars.next() {
            trail.push(ch);
        }
        trail
    }
    
    pub fn next(&mut self) -> Option<Token> {
        use self::Token::*;
        if self.token.is_some() {
            return self.token.take();
        }
        self.eat_whitespace();
        let ch = if let Some(ch) = self.chars.next() {
            ch
        } else {
            return None;
        };
        Some(match ch {
            '~' => self.eat_or("@", Tadpole, Tilde),
            '(' => LParen,
            ')' => RParen,
            '[' => LBrack,
            ']' => RBrack,
            '{' => LCurly,
            '}' => RCurly,
            '\'' => Apo,
            '^' => Hat,
            '@' => At,
            ';' => SemiCTrail(self.trail()),
            '"' => self.read_string().expect("Could not read string"),
            ch => {
                let mut ident = String::new();
                ident.push(ch);
                while let Some(&next) = self.chars.peek() {
                    if ! is_special_char(next) {
                        ident.push(next);
                        self.chars.next();
                    } else {
                        break;
                    }
                }
                Ident(ident)
            },
        })
    }
}

pub fn read_str(text: &str) -> Mal {
    let mut reader = Reader::new(text);
    read_form(&mut reader)
}

pub fn read_list(reader: &mut Reader) -> Mal {
    let mut list = Vec::new();
    loop {
        match reader.peek().expect("Could not read at list") {
            Token::RParen => {
                reader.next();
                return MalList::new(list).into();
            }
            _ => {
                list.push(read_form(reader));
            }
        }
    }
}

pub fn read_atom(ident: String) -> Mal {
    let first = ident.chars().nth(0).unwrap();
    match first {
        '-' | '+' => {
            if let Some(ch) = ident.chars().nth(1) {
                match ch {
                    '0' ... '9' => Mal::Number(ident.parse().expect("Invalid number")),
                    _ => Mal::Symbol(ident),
                }
            } else {
                Mal::Symbol(ident)
            }
        }
        '0' ... '9' => Mal::Number(ident.parse().expect("Invalid number")),
        _ => {
            match ident.as_str() {
                "true" => Mal::Bool(true),
                "false" => Mal::Bool(false),
                "nil" => Mal::Nil,
                _ => Mal::Symbol(ident),
            }
        }
    }
}

pub fn read_form(reader: &mut Reader) -> Mal {
    use self::Token::*;
    match reader.next().expect("read_form: No more tokens") {
        LParen => {
            read_list(reader).into()
        }
        Ident(ident) => {
            let atom = read_atom(ident);
            atom
        }
        Str(string) => {
            Mal::Str(string)
        }
        _ => unimplemented!(),
    }
}
