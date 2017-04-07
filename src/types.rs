use errors::*;
use std::ops;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::clone;

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

impl<'a> From<&'a str> for Mal {
    fn from(value: &'a str) -> Mal {
        if value.starts_with(":") {
            Mal::Kw(Keyword::new(&value[1..]))
        } else {
            Mal::Sym(Symbol::new(value))
        }
    }
}

impl From<Symbol> for Mal {
    fn from(value: Symbol) -> Mal {
        Mal::Sym(value)
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

#[derive(Debug, Clone)]
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

impl From<MalList> for Mal {
    fn from(value: MalList) -> Mal {
        Mal::List(value)
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

#[derive(Debug, Clone)]
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

impl From<MalArr> for Mal {
    fn from(value: MalArr) -> Mal {
        Mal::Arr(value)
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

#[derive(Debug, Clone)]
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

impl From<MalMap> for Mal {
    fn from(value: MalMap) -> Mal {
        Mal::Map(value)
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

pub type NativeFunc = fn(&MalList) -> Result<Mal>;

pub enum MalFunc {
    Native(&'static str, NativeFunc),
    Defined(VecDeque<Symbol>, Box<Mal>),
}

impl fmt::Debug for MalFunc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::MalFunc::*;
        match *self {
            Native(name, _) => {
                write!(f, "MalFunc::Native {{ \"{}\" }}", name)
            }
            Defined(ref args, ref body) => {
                write!(f, "MalFunc::Defined({:?}){{ {:?} }}", args, body)
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
            MalFunc::Defined(ref args, ref body) => {
                MalFunc::Defined(args.clone(), body.clone())
            }
        }
    }
}

impl From<MalFunc> for Mal {
    fn from(value: MalFunc) -> Mal {
        Mal::Fn(value)
    }
}

