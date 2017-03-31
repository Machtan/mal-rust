use errors::*;
use std::iter::Peekable;
use std::str::CharIndices;
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

pub struct Token {
    pub kind: TokenKind,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
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

pub struct Lexer<'a> {
    text: &'a str,
    pos: usize,
    chars: Peekable<CharIndices<'a>>,
    next_token: Option<Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(text: &'a str) -> Lexer<'a> {
        Lexer {
            text: text,
            pos: 0,
            chars: text.char_indices().peekable(),
            next_token: None,
        }
    }
    
    pub fn peek(&mut self) -> Option<&Token> {
        if let Some(ref token) = self.next_token {
            Some(token)
        } else {
            match self.next() {
                Ok(token) => {
                    self.next_token = Some(token);
                    self.next_token.as_ref()
                }
                Err(_) => None,
            }
        }
    }
    
    fn err<T, S: Into<String>>(&self, msg: S) -> Result<T> {
        Err(ErrorKind::Lexer { 
            source: String::from(self.text), 
            pos: self.pos, 
            msg: msg.into()
        }.into())
    }
    
    fn send_if_next(&mut self, pat: &str, ifpat: TokenKind, ifnot: TokenKind) -> Result<Token> {
        let mut chars = self.chars.clone();
        let mut other = pat.chars();
        let mut pat_chars = 0;
        loop {
            match (chars.next(), other.next()) {
                (Some((_, a)), Some(b)) => {
                    if a != b {
                        return self.send_token(ifnot);
                    }
                }
                (_, None) => break,
                _ => return self.send_token(ifnot),
            }
            pat_chars += 1;
        }
        for _ in 0..pat_chars {
            self.chars.next();
        }
        self.send_token(ifpat)
    }
    
    /// Advances the lexer by a character.
    fn advance(&mut self) -> Result<()> {
        if let Some((i, _)) = self.chars.next() {
            self.pos = i;
            Ok(())
        } else {
            self.pos = self.text.len();
            self.err("Unxepected EOF")
        }
    }
    
    fn eat_whitespace(&mut self) {
        loop {
            let is_ws = self.chars.peek().map_or(false, |&(_, c)| c == ',' || c.is_whitespace());
            if is_ws {
                self.advance().unwrap();
            } else {
                break;
            }
        }
    }
    
    fn read_string(&mut self) -> Result<Token> {
        let mut string = String::new();
        let mut escaped = false;
        while let Some((_, ch)) = self.chars.next() {
            if ! escaped {
                match ch {
                    '"' => return self.send_token(TokenKind::Str(string)),
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
                        return self.err(format!("Invalid escape character: {:?}", ch));
                    }
                }
                escaped = false;
            }
        }
        self.err(format!("Unterminated string: '\"{}'", string))
    }
    
    #[inline]
    fn peek_is(&mut self, ch: char) -> bool {
        if let Some(&(_, peek)) = self.chars.peek() {
            peek == ch
        } else {
            false
        }
    }
    
    fn trail(&mut self) -> Result<String> {
        let mut trail = String::new();
        while let Some((_, ch)) = self.chars.next() {
            if ch == '\r' {
                if self.peek_is('\n') {
                    self.chars.next();
                    return Ok(trail);
                } else {
                    return self.err("Carriage return without newline!");
                }
            } else if ch == '\n' {
                self.chars.next();
                return Ok(trail);
            } else {
                trail.push(ch);
            }
        }
        Ok(trail)
    }
    
    #[inline]
    pub fn has_next(&mut self) -> bool {
        self.next_token.is_some() || self.chars.peek().is_some()
    }
    
    #[inline]
    fn end(&mut self) -> usize {
        if let Some(&(i, _)) = self.chars.peek() {
            i
        } else {
            self.text.len()
        }
    }
    
    #[inline]
    fn send_token(&mut self, kind: TokenKind) -> Result<Token> {
        Ok(Token { kind: kind, start: self.pos, end: self.end() })
    }
    
    pub fn next(&mut self) -> Result<Token> {
        use self::TokenKind::*;
        if self.next_token.is_some() {
            return Ok(self.next_token.take().unwrap());
        }
        self.eat_whitespace();
        let ch = if let Some((_, ch)) = self.chars.next() {
            ch
        } else {
            return self.err("Unexpected EOF");
        };
        match ch {
            '~' => self.send_if_next("@", Tadpole, Tilde),
            '(' => self.send_token(ParOpen),
            ')' => self.send_token(ParClose),
            '[' => self.send_token(BrackOpen),
            ']' => self.send_token(BrackClose),
            '{' => self.send_token(CurlOpen),
            '}' => self.send_token(CurlClose),
            '\'' => self.send_token(Apo),
            '^' => self.send_token(Hat),
            '@' => self.send_token(At),
            '`' => self.send_token(BackTick),
            ';' => {
                let trail = self.trail()?;
                self.send_token(SemiCTrail(trail))
            }
            '"' => self.read_string(),
            ch => {
                let mut ident = String::new();
                ident.push(ch);
                while let Some(&(_, next)) = self.chars.peek() {
                    if ! is_special_char(next) {
                        ident.push(next);
                        self.chars.next();
                    } else {
                        break;
                    }
                }
                self.send_token(Ident(ident))
            },
        }
    }
}

pub fn read_str(text: &str) -> Result<Mal> {
    let mut lexer = Lexer::new(text);
    read_form(&mut lexer)
}

fn read_list(lexer: &mut Lexer, end_token: TokenKind, start: usize) -> Result<Vec<Mal>> {
    let mut list = Vec::new();
    loop {
        if lexer.peek().ok_or_else(|| Error::from("Unclosed list"))?.kind == end_token {
            lexer.next().unwrap();
            return Ok(list);
        } else {
            list.push(read_form(lexer)?);
        }
    }
}

pub fn read_atom(mut ident: String) -> Result<Mal> {
    let first = ident.chars().nth(0).unwrap();
    Ok(match first {
        '-' | '+' => {
            if let Some(ch) = ident.chars().nth(1) {
                match ch {
                    '0' ... '9' => Mal::Number(ident.parse().chain_err(|| "Could not parse number")?),
                    _ => Mal::Symbol(ident),
                }
            } else {
                Mal::Symbol(ident)
            }
        }
        '0' ... '9' => Mal::Number(ident.parse().chain_err(|| "Could not parse number")?),
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
    })
}

pub fn read_hash_map(lexer: &mut Lexer) -> Result<Mal> {
    let mut map = MalMap::new();
    loop {
        if lexer.peek().ok_or_else(|| Error::from("unclosed hash map"))?.kind == TokenKind::CurlClose {
            lexer.next().unwrap();
            return Ok(map.into());
        } else {
            let key: MapKey = match read_form(lexer)? {
                Mal::Str(string) => string.into(),
                Mal::Kw(kw) => kw.into(),
                other_token => {
                    bail!("Invalid key type for hashmap: {:?}", other_token);
                }
            };
            let value = read_form(lexer)?;
            map.insert(key, value);
        }
    }
}

pub fn read_form(lexer: &mut Lexer) -> Result<Mal> {
    use self::TokenKind::*;
    let token = lexer.next()?;
    Ok(match token.kind {
        ParOpen => {
            MalList::new(read_list(lexer, ParClose, token.start)?).into()
        }
        BrackOpen => {
            MalArr::new(read_list(lexer, BrackClose, token.start)?).into()
        }
        CurlOpen => {
            read_hash_map(lexer)?
        }
        Ident(ident) => {
            let atom = read_atom(ident)?;
            atom
        }
        Apo => {
            let quoted = read_form(lexer)?;
            MalList::new(vec![Mal::Symbol("quote".into()), quoted]).into()
        }
        BackTick => {
            let quoted = read_form(lexer)?;
            MalList::new(vec![Mal::Symbol("quasiquote".into()), quoted]).into()
        }
        Tilde => {
            let unquoted = read_form(lexer)?;
            MalList::new(vec![Mal::Symbol("unquote".into()), unquoted]).into()
        }
        Tadpole => {
            let unquoted = read_form(lexer)?;
            MalList::new(vec![Mal::Symbol("splice-unquote".into()), unquoted]).into()
        }
        At => {
            let derefed = read_form(lexer)?;
            MalList::new(vec![Mal::Symbol("deref".into()), derefed]).into()
        }
        Str(string) => {
            Mal::Str(string)
        }
        Hat => {
            let meta = read_form(lexer)?;
            let target = read_form(lexer)?;
            MalList::new(vec![Mal::Symbol("with-meta".into()), target, meta]).into()
        }
        other => panic!("Unsuported token: {:?}", other),
    })
}
