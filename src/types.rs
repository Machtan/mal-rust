use std::ops;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Mal {
    List(MalList),
    Arr(MalArr),
    Number(i32),
    Symbol(String),
    Str(String),
    Bool(bool),
    Kw(Keyword),
    Map(MalMap),
    Nil,
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
    items: Vec<Mal>,
}
impl MalList {
    pub fn new(items: Vec<Mal>) -> MalList {
        MalList { items }
    }
}

impl From<MalList> for Mal {
    fn from(value: MalList) -> Mal {
        Mal::List(value)
    }
}

impl ops::Deref for MalList {
    type Target = Vec<Mal>;
    
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
    items: Vec<Mal>,
}
impl MalArr {
    pub fn new(items: Vec<Mal>) -> MalArr {
        MalArr { items }
    }
}

impl From<MalArr> for Mal {
    fn from(value: MalArr) -> Mal {
        Mal::Arr(value)
    }
}

impl ops::Deref for MalArr {
    type Target = Vec<Mal>;
    
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
    
    pub fn insert<K: Into<MapKey>>(&mut self, key: K, value: Mal) -> Option<Mal> {
        self.inner.insert(key.into(), value)
    }
}

impl From<MalMap> for Mal {
    fn from(value: MalMap) -> Mal {
        Mal::Map(value)
    }
}
