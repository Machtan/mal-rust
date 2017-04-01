use errors::*;
use std::ops;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub enum Mal {
    List(MalList),
    Arr(MalArr),
    Num(f64),
    Sym(String),
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
    
    pub fn number(&self) -> Result<f64> {
        match *self {
            Mal::Num(val) => Ok(val),
            ref other => Err(ErrorKind::TypeError{
                expected: String::from("number"),
                got: other.type_name().into()
            }.into()),
        }
    }
    
    pub fn list(self) -> Result<MalList> {
        match self {
            Mal::List(list) => Ok(list),
            ref other => Err(ErrorKind::TypeError{
                expected: String::from("list"),
                got: other.type_name().into()
            }.into()),
        }
    }
    
    pub fn call(&self, args: MalList) -> Result<Mal> {
        match *self {
            Mal::Fn(MalFunc::Native(_, func)) => func(args),
            ref other => bail!("Attempted to call value of type '{}'", other.type_name()),
        }
    }
}

impl<'a> From<&'a str> for Mal {
    fn from(value: &'a str) -> Mal {
        if value.starts_with(":") {
            Mal::Kw(Keyword::new(&value[1..]))
        } else {
            Mal::Sym(String::from(value))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Keyword {
    pub(crate) sym: String,
}

impl Keyword {
    pub fn new<S: Into<String>>(symbol: S) -> Keyword {
        Keyword { sym: symbol.into() }
    }
    
    pub fn symbol(&self) -> &str {
        &self.sym
    }
}

#[derive(Debug, Clone)]
pub struct MalList {
    items: VecDeque<Mal>,
}
impl MalList {
    pub fn new() -> MalList {
        MalList { items: VecDeque::new() }
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
    items: VecDeque<Mal>,
}
impl MalArr {
    pub fn new() -> MalArr {
        MalArr { items: VecDeque::new() }
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
    pub fn new() -> MalMap {
        MalMap { inner: HashMap::new() }
    }
    
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

pub type NativeFunc = fn(MalList) -> Result<Mal>;

#[derive(Debug, Clone)]
pub enum MalFunc {
    Native(&'static str, NativeFunc),
}

impl From<MalFunc> for Mal {
    fn from(value: MalFunc) -> Mal {
        Mal::Fn(value)
    }
}

#[derive(Debug, Clone)]
pub struct Env {
    pub(crate) map: HashMap<String, Mal>,
}

impl Env {
    pub fn new() -> Env {
        Env { map: HashMap::new() }
    }
    
    pub fn get(&self, ident: &str) -> Result<Mal> {
        if let Some(mal) = self.map.get(ident) {
            Ok(mal.clone())
        } else {
            bail!("Unknown variable: '{}'", ident);
        }
    }
    
    pub fn add_native_func(&mut self, name: &'static str, func: NativeFunc) -> Result<()> {
        if self.map.contains_key(name) {
            bail!("Native function '{}' declared twice!", name);
        }
        self.map.insert(name.into(), MalFunc::Native(name, func).into());
        Ok(())
    }
}

impl ops::Deref for Env {
    type Target = HashMap<String, Mal>;
    
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl ops::DerefMut for Env {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

