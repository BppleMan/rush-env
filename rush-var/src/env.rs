use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Scalar(String),
    Array(Vec<String>),
    Assoc(BTreeMap<String, String>),
}

#[derive(Debug, Default, Clone)]
pub struct Env {
    vars: HashMap<String, Value>,
}

impl Env {
    pub fn new() -> Self {
        Self { vars: HashMap::new() }
    }

    pub fn set_scalar(&mut self, key: impl Into<String>, val: impl Into<String>) {
        self.vars.insert(key.into(), Value::Scalar(val.into()));
    }

    pub fn set_array(&mut self, key: impl Into<String>, val: Vec<String>) {
        self.vars.insert(key.into(), Value::Array(val));
    }

    pub fn set_assoc(&mut self, key: impl Into<String>, val: BTreeMap<String, String>) {
        self.vars.insert(key.into(), Value::Assoc(val));
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.vars.get(key)
    }

    pub fn get_scalar(&self, key: &str) -> Option<&str> {
        match self.vars.get(key) {
            Some(Value::Scalar(s)) => Some(s.as_str()),
            _ => None,
        }
    }
}
