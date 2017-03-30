use std::iter::Peekable;
use std::str::Chars;
use types::{MalList, Mal, Keyword, MalArr, MalMap, MapKey};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Tadpole, // ~@
    BrackOpen, // [
    BrackClose, // ]
    CurlOpen, // {
    CurlClose, // }
    ParOpen, // (
    ParClose, // )
    Apo, // '
    BackTick, // `
    Tilde, // ~
    Hat, // ^
    At, // @
    Str(String), // "with \" escapes"
    SemiCTrail(String), // ;.*
    Ident(String), // A sequence of non-ws and non-specials
}

fn is_special_char(ch: char) -> bool {
    match ch {
        '~' | '@' | '[' | ']' | '(' | ')' | '{' | '}' | '\'' | '`' | '"' | '^' | ';' | ',' => true,
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
    
    pub fn peek(&mut self) -> Option<&Token> {
        if let Some(ref token) = self.token {
            Some(token)
        } else {
            self.token = self.next();
            self.token.as_ref()
        }
    }
    
    fn eat_or(&mut self, pat: &str, ifpat: Token, ifnot: Token) -> Token {
        let mut chars = self.chars.clone();
        let mut other = pat.chars();
        let mut shared_chars = 0;
        loop {
            match (chars.next(), other.next()) {
                (Some(a), Some(b)) => {
                    if a == b {
                        shared_chars += 1;
                    } else {
                        return ifnot;
                    }
                }
                (None, Some(_)) => return ifnot,
                (_, None) => break,   
            }
        }
        for _ in 0..shared_chars {
            self.chars.next();
        }
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
            '(' => ParOpen,
            ')' => ParClose,
            '[' => BrackOpen,
            ']' => BrackClose,
            '{' => CurlOpen,
            '}' => CurlClose,
            '\'' => Apo,
            '^' => Hat,
            '@' => At,
            '`' => BackTick,
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

pub fn read_list(reader: &mut Reader, end_token: Token) -> Vec<Mal> {
    let mut list = Vec::new();
    loop {
        if *reader.peek().expect("Could not read at list") == end_token {
            reader.next();
            return list;
        } else {
            list.push(read_form(reader));
        }
    }
}

pub fn read_atom(mut ident: String) -> Mal {
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
        ':' => {
            ident.remove(0);
            Mal::Kw(Keyword::new(ident))
        }
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

pub fn read_hash_map(reader: &mut Reader) -> Mal {
    let mut map = MalMap::new();
    loop {
        if *reader.peek().expect("Could not read at list") == Token::CurlClose {
            reader.next();
            return map.into();
        } else {
            let key: MapKey = match read_form(reader) {
                Mal::Str(string) => string.into(),
                Mal::Kw(kw) => kw.into(),
                other => {
                    panic!("Invalid key type gotten: {:?}", other);
                }
            };
            let value = read_form(reader);
            map.insert(key, value);
        }
    }
}

pub fn read_form(reader: &mut Reader) -> Mal {
    use self::Token::*;
    match reader.next().expect("read_form: No more tokens") {
        ParOpen => {
            MalList::new(read_list(reader, ParClose)).into()
        }
        BrackOpen => {
            MalArr::new(read_list(reader, BrackClose)).into()
        }
        CurlOpen => {
            read_hash_map(reader)
        }
        Ident(ident) => {
            let atom = read_atom(ident);
            atom
        }
        Apo => {
            let quoted = read_form(reader);
            MalList::new(vec![Mal::Symbol("quote".into()), quoted]).into()
        }
        BackTick => {
            let quoted = read_form(reader);
            MalList::new(vec![Mal::Symbol("quasiquote".into()), quoted]).into()
        }
        Tilde => {
            let unquoted = read_form(reader);
            MalList::new(vec![Mal::Symbol("unquote".into()), unquoted]).into()
        }
        Tadpole => {
            let unquoted = read_form(reader);
            MalList::new(vec![Mal::Symbol("splice-unquote".into()), unquoted]).into()
        }
        At => {
            let derefed = read_form(reader);
            MalList::new(vec![Mal::Symbol("deref".into()), derefed]).into()
        }
        Str(string) => {
            Mal::Str(string)
        }
        Hat => {
            let meta = read_form(reader);
            let target = read_form(reader);
            MalList::new(vec![Mal::Symbol("with-meta".into()), target, meta]).into()
        }
        other => panic!("Unsuported token: {:?}", other),
    }
}
