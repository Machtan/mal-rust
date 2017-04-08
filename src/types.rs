use errors::*;
use std::ops;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::clone;
use std::cmp;
use env::Env;

#[derive(Debug, Clone)]
pub enum Mal {
    List(MalList),
    Arr(MalArr),
    Num(f64),
    Sym(Symbol),
    Str(String),
    Bool(bool),
    Kw(Keyword),
    Map(MalMap),
    Fn(MalFunc),
    Nil,
}
impl Mal {
    pub fn type_name(&self) -> &'static str {
        use self::Mal::*;
        match *self {
            List(_) => "list",
            Arr(_) => "array",
            Num(_) => "number",
            Sym(_) => "symbol",
            Str(_) => "string",
            Bool(_) => "boolean",
            Kw(_) => "keyword",
            Map(_) => "hashmap",
            Fn(_) => "function",
            Nil => "nil",
         }
    }
    
    fn conv_err<S: Into<String>, T>(&self, expected: S, got: &Mal) -> Result<T> {
        Err(ErrorKind::TypeError {
            expected: expected.into(),
            got: got.type_name().into()
        }.into())
    }
    
    pub fn number(&self) -> Result<f64> {
        match *self {
            Mal::Num(val) => Ok(val),
            ref other => self.conv_err("number", other),
        }
    }
    
    pub fn as_list_or_array(&mut self) -> Result<&mut VecDeque<Mal>> {
        match *self {
            Mal::List(ref mut list) => Ok(list.inner()),
            Mal::Arr(ref mut arr) => Ok(arr.inner()),
            ref other => self.conv_err("array or list", other),
        }
    }
    
    pub fn as_function(&mut self) -> Result<&mut MalFunc> {
        match *self {
            Mal::Fn(ref mut func) => Ok(func),
            ref other => self.conv_err("function", other),
        }
    }
    
    pub fn list(self) -> Result<MalList> {
        match self {
            Mal::List(list) => Ok(list),
            ref other => self.conv_err("list", other),
        }
    }
    
    pub fn symbol(self) -> Result<Symbol> {
        match self {
            Mal::Sym(symbol) => Ok(symbol),
            ref other => self.conv_err("symbol", other),
        }
    }
    
    pub fn is_truesy(&self) -> bool {
        match *self {
            Mal::Nil => false,
            Mal::Bool(false) => false,
            _ => true,
        }
    }
}
impl cmp::PartialEq for Mal {
    fn eq(&self, other: &Mal) -> bool {
        use self::Mal::*;
        match (self, other) {
            (&List(ref list),    &Arr(ref arr)) => list.items == arr.items,
            (&Arr(ref arr),      &List(ref list)) => arr.items == list.items,
            
            (&List(ref val), &List(ref oval)) => val == oval,
            (&Arr(ref val),  &Arr(ref oval))  => val == oval,
            (&Num(ref val),  &Num(ref oval))  => val == oval,
            (&Sym(ref val),  &Sym(ref oval))  => val == oval,
            (&Str(ref val),  &Str(ref oval))  => val == oval,
            (&Bool(ref val), &Bool(ref oval)) => val == oval,
            (&Kw(ref val),   &Kw(ref oval))   => val == oval,
            (&Map(ref val),  &Map(ref oval))  => val == oval,
            (&Fn(ref val),   &Fn(ref oval))   => val == oval,
            (&Nil, &Nil) => true,
            _ => false
        }
    }
}

impl From<Symbol> for Mal {
    fn from(value: Symbol) -> Mal {
        Mal::Sym(value)
    }
}

impl From<bool> for Mal {
    fn from(value: bool) -> Mal {
        Mal::Bool(value)
    }
}

impl From<f64> for Mal {
    fn from(value: f64) -> Mal {
        Mal::Num(value)
    }
}

impl From<MalList> for Mal {
    fn from(value: MalList) -> Mal {
        Mal::List(value)
    }
}

impl From<MalMap> for Mal {
    fn from(value: MalMap) -> Mal {
        Mal::Map(value)
    }
}

impl From<MalArr> for Mal {
    fn from(value: MalArr) -> Mal {
        Mal::Arr(value)
    }
}

impl From<String> for Mal {
    fn from(value: String) -> Mal {
        Mal::Str(value)
    }
}

impl From<MalFunc> for Mal {
    fn from(value: MalFunc) -> Mal {
        Mal::Fn(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol {
    pub(crate) inner: String,
}

impl Symbol {
    #[inline]
    pub fn new<S: Into<String>>(value: S) -> Symbol {
        Symbol { inner: value.into() }
    }
    
    #[inline]
    pub fn text(&self) -> &str {
        &self.inner
    }
    
    #[inline]
    pub fn into_string(self) -> String {
        self.inner
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Keyword {
    pub(crate) sym: String,
}

impl Keyword {
    #[inline]
    pub fn new<S: Into<String>>(symbol: S) -> Keyword {
        Keyword { sym: symbol.into() }
    }
    
    #[inline]
    pub fn symbol(&self) -> &str {
        &self.sym
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MalList {
    pub(crate) items: VecDeque<Mal>,
}
impl MalList {
    #[inline]
    pub fn new() -> MalList {
        MalList { items: VecDeque::new() }
    }
    
    #[inline]
    pub fn inner(&mut self) -> &mut VecDeque<Mal> {
        &mut self.items
    }
}

impl ops::Deref for MalList {
    type Target = VecDeque<Mal>;
    
    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl ops::DerefMut for MalList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MalArr {
    pub(crate) items: VecDeque<Mal>,
}

impl MalArr {
    #[inline]
    pub fn new() -> MalArr {
        MalArr { items: VecDeque::new() }
    }
    
    #[inline]
    pub fn inner(&mut self) -> &mut VecDeque<Mal> {
        &mut self.items
    }
}

impl ops::Deref for MalArr {
    type Target = VecDeque<Mal>;
    
    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl ops::DerefMut for MalArr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MapKey {
    Str(String),
    Kw(Keyword),
}

impl From<String> for MapKey {
    fn from(value: String) -> MapKey {
        MapKey::Str(value)
    }
}

impl From<Keyword> for MapKey {
    fn from(value: Keyword) -> MapKey {
        MapKey::Kw(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MalMap {
    pub(crate) inner: HashMap<MapKey, Mal>,
}

impl MalMap {
    #[inline]
    pub fn new() -> MalMap {
        MalMap { inner: HashMap::new() }
    }
    
    #[inline]
    pub fn insert<K: Into<MapKey>, V: Into<Mal>>(&mut self, key: K, value: V) -> Option<Mal> {
        self.inner.insert(key.into(), value.into())
    }
}

impl ops::Deref for MalMap {
    type Target = HashMap<MapKey, Mal>;
    
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl ops::DerefMut for MalMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub type NativeFunc = fn(&mut MalList) -> Result<Mal>;

pub enum MalFunc {
    Native(&'static str, NativeFunc),
    /// args, closed environment, body
    Closure(VecDeque<Symbol>, Env, Box<Mal>),
    /// name, args, closed env, body
    /// What would be a 'function' in another language.
    NamedClosure(Symbol, VecDeque<Symbol>, Env, Box<Mal>),
}

impl fmt::Debug for MalFunc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::MalFunc::*;
        match *self {
            Native(name, _) => {
                write!(f, "MalFunc::Native {{ \"{}\" }}", name)
            }
            Closure(ref args, ref _env, ref body) => {
                write!(f, "MalFunc::Closure {{ ({:?}) => {:?} }}", args, body)
            }
            NamedClosure(ref name, ref args, ref _env, ref body) => {
                write!(f, "MalFunc::NamedClosure {{ {}({:?}) => {:?} }}", name.text(), args, body)
            }
        }
    }
}

impl clone::Clone for MalFunc {
    fn clone(&self) -> Self {
        match *self {
            MalFunc::Native(name, func) => {
                MalFunc::Native(name, func)
            }
            MalFunc::Closure(ref args, ref env, ref body) => {
                MalFunc::Closure(args.clone(), env.clone(), body.clone())
            }
            MalFunc::NamedClosure(ref name, ref args, ref env, ref body) => {
                MalFunc::NamedClosure(name.clone(), args.clone(), env.clone(), body.clone())
            }
        }
    }
}

impl cmp::PartialEq for MalFunc {
    fn eq(&self, other: &MalFunc) -> bool {
        use self::MalFunc::*;
        match (self, other) {
            (&Native(name, _), &Native(oname, _)) => {
                oname == name
            }
            (&Closure(ref args, ref env, ref body), &Closure(ref oargs, ref oenv, ref obody)) => {
                oargs == args && obody == body && oenv == env
            }
            (&NamedClosure(ref name, ref args, ref env, ref body), &NamedClosure(ref oname, ref oargs, ref oenv, ref obody)) => {
                oname == name && oargs == args && obody == body && oenv == env
            }
            _ => false,
        }
    }
}

