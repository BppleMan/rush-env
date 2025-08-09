use std::collections::BTreeMap;

/// Represents a shell variable value
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Scalar(String),
    Array(Vec<String>),
    Assoc(BTreeMap<String, String>),
}

impl Value {
    /// Create a scalar value
    pub fn scalar<S: Into<String>>(s: S) -> Self {
        Value::Scalar(s.into())
    }
    
    /// Create an array value
    pub fn array<S: Into<String>>(items: Vec<S>) -> Self {
        Value::Array(items.into_iter().map(|s| s.into()).collect())
    }
    
    /// Create an associative array value
    pub fn assoc<K: Into<String>, V: Into<String>>(map: Vec<(K, V)>) -> Self {
        Value::Assoc(map.into_iter().map(|(k, v)| (k.into(), v.into())).collect())
    }
    
    /// Check if value is unset (None case handled by Env)
    pub fn is_empty(&self) -> bool {
        match self {
            Value::Scalar(s) => s.is_empty(),
            Value::Array(arr) => arr.is_empty(),
            Value::Assoc(map) => map.is_empty(),
        }
    }
    
    /// Convert to scalar string representation
    pub fn to_scalar(&self) -> String {
        match self {
            Value::Scalar(s) => s.clone(),
            Value::Array(arr) => arr.join(" "),
            Value::Assoc(map) => {
                // For assoc arrays, return space-separated values by default
                map.values().cloned().collect::<Vec<_>>().join(" ")
            }
        }
    }
    
    /// Get array representation 
    pub fn to_array(&self) -> Vec<String> {
        match self {
            Value::Scalar(s) => vec![s.clone()],
            Value::Array(arr) => arr.clone(),
            Value::Assoc(map) => map.values().cloned().collect(),
        }
    }
    
    /// Get length
    pub fn len(&self) -> usize {
        match self {
            Value::Scalar(s) => s.len(),
            Value::Array(arr) => arr.len(),
            Value::Assoc(map) => map.len(),
        }
    }
    
    /// Get keys (for associative arrays)
    pub fn keys(&self) -> Vec<String> {
        match self {
            Value::Assoc(map) => map.keys().cloned().collect(),
            _ => vec![],
        }
    }
    
    /// Get values (for associative arrays)
    pub fn values(&self) -> Vec<String> {
        match self {
            Value::Assoc(map) => map.values().cloned().collect(),
            Value::Array(arr) => arr.clone(),
            Value::Scalar(s) => vec![s.clone()],
        }
    }
}

/// Shell environment containing variables and special parameters
#[derive(Debug, Clone)]
pub struct Env {
    vars: BTreeMap<String, Value>,
    // Special shell variables
    pub pid: String,        // $$
    pub last_status: String, // $?
    pub shell_opts: String, // $-
    pub last_bg_pid: String, // $!
    pub shell_name: String, // $0
    pub positional: Vec<String>, // $1, $2, etc.
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

impl Env {
    /// Create a new empty environment
    pub fn new() -> Self {
        Self {
            vars: BTreeMap::new(),
            pid: "12345".to_string(),
            last_status: "0".to_string(),
            shell_opts: "himBH".to_string(),
            last_bg_pid: "".to_string(),
            shell_name: "zsh".to_string(),
            positional: vec![],
        }
    }
    
    /// Set a variable
    pub fn set<K: Into<String>>(&mut self, name: K, value: Value) {
        self.vars.insert(name.into(), value);
    }
    
    /// Set a scalar variable
    pub fn set_scalar<K: Into<String>, V: Into<String>>(&mut self, name: K, value: V) {
        self.vars.insert(name.into(), Value::scalar(value));
    }
    
    /// Set an array variable
    pub fn set_array<K: Into<String>, V: Into<String>>(&mut self, name: K, values: Vec<V>) {
        self.vars.insert(name.into(), Value::array(values));
    }
    
    /// Set an associative array variable
    pub fn set_assoc<K: Into<String>, K2: Into<String>, V: Into<String>>(
        &mut self, 
        name: K, 
        pairs: Vec<(K2, V)>
    ) {
        self.vars.insert(name.into(), Value::assoc(pairs));
    }
    
    /// Get a variable (None if unset)
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.vars.get(name)
    }
    
    /// Check if a variable is set
    pub fn is_set(&self, name: &str) -> bool {
        self.vars.contains_key(name)
    }
    
    /// Check if a variable is set and non-empty
    pub fn is_set_and_non_empty(&self, name: &str) -> bool {
        self.vars.get(name).map_or(false, |v| !v.is_empty())
    }
    
    /// Unset a variable
    pub fn unset(&mut self, name: &str) {
        self.vars.remove(name);
    }
    
    /// Get special parameter value
    pub fn get_special(&self, ch: char) -> Option<String> {
        match ch {
            '$' => Some(self.pid.clone()),
            '?' => Some(self.last_status.clone()),
            '-' => Some(self.shell_opts.clone()),
            '!' => Some(self.last_bg_pid.clone()),
            '0' => Some(self.shell_name.clone()),
            '#' => Some(self.positional.len().to_string()),
            '*' => Some(self.positional.join(" ")),
            '@' => Some(self.positional.join(" ")), // Simplified for now
            _ => None,
        }
    }
    
    /// Get positional parameter ($1, $2, etc.)
    pub fn get_positional(&self, n: u32) -> Option<String> {
        if n == 0 {
            Some(self.shell_name.clone())
        } else {
            self.positional.get((n - 1) as usize).cloned()
        }
    }
    
    /// Set positional parameters
    pub fn set_positional<S: Into<String>>(&mut self, args: Vec<S>) {
        self.positional = args.into_iter().map(|s| s.into()).collect();
    }
    
    /// Get all variable names starting with prefix
    pub fn get_names_with_prefix(&self, prefix: &str) -> Vec<String> {
        self.vars
            .keys()
            .filter(|name| name.starts_with(prefix))
            .cloned()
            .collect()
    }
    
    /// Get all variable names
    pub fn get_all_names(&self) -> Vec<String> {
        self.vars.keys().cloned().collect()
    }
}
