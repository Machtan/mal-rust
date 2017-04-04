use std::collections::HashMap;
use types::{Mal, NativeFunc, MalFunc, Symbol};
use errors::*;

#[derive(Debug, Clone)]
pub enum EnvChange {
    /// A new scope was entered
    NewScopeEntered,
    /// A new binding was added
    BindingAdded(Symbol),
    /// The binding of symbol was replaced, and the old value was 'Mal'
    BindingReplaced(Symbol, Mal)
}

#[derive(Debug, Clone)]
pub struct Env {
    map: HashMap<Symbol, Mal>,
    history: Vec<EnvChange>,
}

impl Env {
    pub fn new() -> Env {
        Env { map: HashMap::new(), history: Vec::new() }
    }
    
    pub fn with_new_scope<F, R>(&mut self, mut func: F) -> R where F: FnMut(&mut Env) -> R {
        use self::EnvChange::*;
        self.history.push(NewScopeEntered);
        let res = func(self);
        // Revert back to previous state.
        loop {
            match self.history.pop().expect("Broken env invariant!") {
                NewScopeEntered => break,
                BindingAdded(ident) => {
                    self.map.remove(&ident).expect("Broken env invariant");
                }
                BindingReplaced(sym, val) => {
                    self.map.insert(sym, val);
                }
            }
        }
        res
    }
    
    pub fn get(&self, ident: &Symbol) -> Result<Mal> {
        if let Some(mal) = self.map.get(ident) {
            Ok(mal.clone())
        } else {
            bail!("Unknown variable: '{}'", ident.text());
        }
    }
    
    pub fn set<K: Into<Symbol>, V: Into<Mal>>(&mut self, ident: K, value: V) {
        let symbol = ident.into();
        match self.map.insert(symbol.clone(), value.into()) {
            None => self.history.push(EnvChange::BindingAdded(symbol)),
            Some(old_value) => self.history.push(EnvChange::BindingReplaced(symbol, old_value))
        }
    }
    
    pub fn add_native_func(&mut self, name: &'static str, func: NativeFunc) -> Result<()> {
        let symbol = Symbol::new(name);
        if self.map.contains_key(&symbol) {
            bail!("Native function '{}' declared twice!", name);
        }
        self.map.insert(symbol, MalFunc::Native(name, func).into());
        Ok(())
    }
}
