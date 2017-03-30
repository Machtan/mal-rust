use std::ops;

#[derive(Debug)]
pub enum Mal {
    List(MalList),
    Number(i32),
    Symbol(String),
    Str(String),
    Bool(bool),
    Nil,
}

#[derive(Debug)]
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
