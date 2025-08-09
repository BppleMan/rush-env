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
                map.values().map(|s| s.as_str()).collect::<Vec<_>>().join(" ")
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
    pub pid: String,             // $$
    pub last_status: String,     // $?
    pub shell_opts: String,      // $-
    pub last_bg_pid: String,     // $!
    pub shell_name: String,      // $0
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
    pub fn set_assoc<K: Into<String>, K2: Into<String>, V: Into<String>>(&mut self, name: K, pairs: Vec<(K2, V)>) {
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
        self.vars.get(name).is_some_and(|v| !v.is_empty())
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
        self.vars.keys().filter(|name| name.starts_with(prefix)).cloned().collect()
    }

    /// Get all variable names
    pub fn get_all_names(&self) -> Vec<String> {
        self.vars.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_value_creation() {
        // Test scalar creation
        let val = Value::scalar("hello");
        assert_eq!(val, Value::Scalar("hello".to_string()));

        // Test array creation
        let val = Value::array(vec!["a", "b", "c"]);
        assert_eq!(val, Value::Array(vec!["a".to_string(), "b".to_string(), "c".to_string()]));

        // Test associative array creation
        let val = Value::assoc(vec![("key1", "val1"), ("key2", "val2")]);
        let mut expected_map = BTreeMap::new();
        expected_map.insert("key1".to_string(), "val1".to_string());
        expected_map.insert("key2".to_string(), "val2".to_string());
        assert_eq!(val, Value::Assoc(expected_map));
    }

    #[test]
    fn test_value_is_empty() {
        // Empty scalar
        assert!(Value::Scalar("".to_string()).is_empty());
        assert!(!Value::Scalar("test".to_string()).is_empty());

        // Empty array
        assert!(Value::Array(vec![]).is_empty());
        assert!(!Value::Array(vec!["item".to_string()]).is_empty());

        // Empty associative array
        assert!(Value::Assoc(BTreeMap::new()).is_empty());
        let mut map = BTreeMap::new();
        map.insert("key".to_string(), "value".to_string());
        assert!(!Value::Assoc(map).is_empty());
    }

    #[test]
    fn test_value_to_scalar() {
        // Scalar
        let val = Value::Scalar("hello".to_string());
        assert_eq!(val.to_scalar(), "hello");

        // Array
        let val = Value::Array(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
        assert_eq!(val.to_scalar(), "a b c");

        // Empty array
        let val = Value::Array(vec![]);
        assert_eq!(val.to_scalar(), "");

        // Associative array
        let mut map = BTreeMap::new();
        map.insert("key1".to_string(), "val1".to_string());
        map.insert("key2".to_string(), "val2".to_string());
        let val = Value::Assoc(map);
        assert_eq!(val.to_scalar(), "val1 val2"); // BTreeMap is ordered
    }

    #[test]
    fn test_value_to_array() {
        // Scalar - 修正：to_array() 应该返回单个元素数组而不是拆分
        let val = Value::Scalar("hello world".to_string());
        assert_eq!(val.to_array(), vec!["hello world"]);

        // Empty scalar - 空字符串仍然是一个元素
        let val = Value::Scalar("".to_string());
        assert_eq!(val.to_array(), vec![""]); // 应该返回包含空字符串的数组

        // Array
        let val = Value::Array(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
        assert_eq!(val.to_array(), vec!["a", "b", "c"]);

        // Associative array
        let mut map = BTreeMap::new();
        map.insert("key1".to_string(), "val1".to_string());
        map.insert("key2".to_string(), "val2".to_string());
        let val = Value::Assoc(map);
        assert_eq!(val.to_array(), vec!["val1", "val2"]);
    }

    #[test]
    fn test_env_creation() {
        let env = Env::new();
        assert!(env.vars.is_empty());
        assert_eq!(env.pid, "12345");
        assert_eq!(env.last_status, "0");
        assert_eq!(env.shell_opts, "himBH");
        assert_eq!(env.last_bg_pid, "");
        assert_eq!(env.shell_name, "zsh");
        assert!(env.positional.is_empty());
    }

    #[test]
    fn test_env_set_get_scalar() {
        let mut env = Env::new();

        // Set and get scalar
        env.set_scalar("TEST", "value");
        let val = env.get("TEST").unwrap();
        assert_eq!(val, &Value::Scalar("value".to_string()));

        // Test overwrite
        env.set_scalar("TEST", "new_value");
        let val = env.get("TEST").unwrap();
        assert_eq!(val, &Value::Scalar("new_value".to_string()));

        // Test non-existent variable
        assert!(env.get("NONEXISTENT").is_none());
    }

    #[test]
    fn test_env_set_get_array() {
        let mut env = Env::new();

        // Set and get array
        env.set_array("ARR", vec!["a", "b", "c"]);
        let val = env.get("ARR").unwrap();
        assert_eq!(val, &Value::Array(vec!["a".to_string(), "b".to_string(), "c".to_string()]));

        // Test empty array
        env.set_array("EMPTY_ARR", Vec::<&str>::new());
        let val = env.get("EMPTY_ARR").unwrap();
        assert_eq!(val, &Value::Array(vec![]));
    }

    #[test]
    fn test_env_set_get_assoc() {
        let mut env = Env::new();

        // Set and get associative array
        env.set_assoc("MAP", vec![("key1", "val1"), ("key2", "val2")]);
        let val = env.get("MAP").unwrap();
        let mut expected_map = BTreeMap::new();
        expected_map.insert("key1".to_string(), "val1".to_string());
        expected_map.insert("key2".to_string(), "val2".to_string());
        assert_eq!(val, &Value::Assoc(expected_map));

        // Test empty associative array
        env.set_assoc("EMPTY_MAP", Vec::<(&str, &str)>::new());
        let val = env.get("EMPTY_MAP").unwrap();
        assert_eq!(val, &Value::Assoc(BTreeMap::new()));
    }

    #[test]
    fn test_env_is_set() {
        let mut env = Env::new();

        // Variable doesn't exist
        assert!(!env.is_set("TEST"));

        // Variable exists with value
        env.set_scalar("TEST", "value");
        assert!(env.is_set("TEST"));

        // Variable exists but is empty
        env.set_scalar("EMPTY", "");
        assert!(env.is_set("EMPTY"));

        // Empty array
        env.set_array("EMPTY_ARR", Vec::<&str>::new());
        assert!(env.is_set("EMPTY_ARR"));
    }

    #[test]
    fn test_env_unset() {
        let mut env = Env::new();

        // Set then unset
        env.set_scalar("TEST", "value");
        assert!(env.is_set("TEST"));

        env.unset("TEST");
        assert!(!env.is_set("TEST"));
        assert!(env.get("TEST").is_none());

        // Unset non-existent variable (should not panic)
        env.unset("NONEXISTENT");
    }

    #[test]
    fn test_env_special_parameters() {
        let mut env = Env::new();

        // Test default special parameters
        assert_eq!(env.get_special('$'), Some("12345".to_string())); // PID
        assert_eq!(env.get_special('?'), Some("0".to_string()));
        assert_eq!(env.get_special('-'), Some("himBH".to_string()));
        assert_eq!(env.get_special('!'), Some("".to_string()));
        assert_eq!(env.get_special('0'), Some("zsh".to_string()));
        assert_eq!(env.get_special('#'), Some("0".to_string()));
        assert_eq!(env.get_special('*'), Some("".to_string()));
        assert_eq!(env.get_special('@'), Some("".to_string()));

        // Test unknown special parameter
        assert_eq!(env.get_special('x'), None);

        // Test with positional parameters
        env.set_positional(vec!["arg1", "arg2", "arg3"]);
        assert_eq!(env.get_special('#'), Some("3".to_string()));
        assert_eq!(env.get_special('*'), Some("arg1 arg2 arg3".to_string()));
        assert_eq!(env.get_special('@'), Some("arg1 arg2 arg3".to_string()));
    }

    #[test]
    fn test_env_positional_parameters() {
        let mut env = Env::new();

        // Initially no positional parameters
        assert_eq!(env.get_positional(0), Some("zsh".to_string())); // $0 is shell name
        assert_eq!(env.get_positional(1), None);

        // Set positional parameters
        env.set_positional(vec!["arg1", "arg2", "arg3"]);
        assert_eq!(env.get_positional(0), Some("zsh".to_string()));
        assert_eq!(env.get_positional(1), Some("arg1".to_string()));
        assert_eq!(env.get_positional(2), Some("arg2".to_string()));
        assert_eq!(env.get_positional(3), Some("arg3".to_string()));
        assert_eq!(env.get_positional(4), None);

        // Test large index
        assert_eq!(env.get_positional(100), None);
    }

    #[test]
    fn test_env_get_names_with_prefix() {
        let mut env = Env::new();

        // Set various variables
        env.set_scalar("TEST_1", "val1");
        env.set_scalar("TEST_2", "val2");
        env.set_scalar("OTHER", "other");
        env.set_scalar("TEST_PREFIX", "prefix");

        // Get names with prefix
        let mut names = env.get_names_with_prefix("TEST_");
        names.sort(); // BTreeMap should be ordered, but let's be explicit
        assert_eq!(names, vec!["TEST_1", "TEST_2", "TEST_PREFIX"]);

        // Non-existent prefix
        let names = env.get_names_with_prefix("NONEXISTENT");
        assert!(names.is_empty());

        // Empty prefix should return all names
        let names = env.get_names_with_prefix("");
        assert_eq!(names.len(), 4);
    }

    #[test]
    fn test_env_get_all_names() {
        let mut env = Env::new();

        // Empty environment
        let names = env.get_all_names();
        assert!(names.is_empty());

        // Add some variables
        env.set_scalar("VAR1", "val1");
        env.set_scalar("VAR2", "val2");
        env.set_array("ARR", vec!["a", "b"]);

        let mut names = env.get_all_names();
        names.sort();
        assert_eq!(names, vec!["ARR", "VAR1", "VAR2"]);
    }

    #[test]
    fn test_env_edge_cases() {
        let mut env = Env::new();

        // Test with empty string keys
        env.set_scalar("", "empty_key");
        assert_eq!(env.get(""), Some(&Value::Scalar("empty_key".to_string())));

        // Test with unicode keys and values
        env.set_scalar("UNICODE_KEY_éñ中文", "unicode_value_éñ中文");
        assert_eq!(
            env.get("UNICODE_KEY_éñ中文"),
            Some(&Value::Scalar("unicode_value_éñ中文".to_string()))
        );

        // Test very long key/value
        let long_key = "x".repeat(1000);
        let long_value = "y".repeat(10000);
        env.set_scalar(&long_key, &long_value);
        assert_eq!(env.get(&long_key), Some(&Value::Scalar(long_value)));

        // Test special characters in keys/values
        env.set_scalar("KEY_WITH_$PECIAL_CH@R$", "value_with_$pecial_ch@r$");
        assert_eq!(
            env.get("KEY_WITH_$PECIAL_CH@R$"),
            Some(&Value::Scalar("value_with_$pecial_ch@r$".to_string()))
        );
    }

    #[test]
    fn test_value_edge_cases() {
        // Test scalar with newlines - to_array() 应该返回单个元素
        let val = Value::Scalar("line1\nline2\nline3".to_string());
        assert_eq!(val.to_array(), vec!["line1\nline2\nline3"]);

        // Test scalar with multiple spaces - to_array() 应该返回单个元素
        let val = Value::Scalar("a    b     c".to_string());
        let array = val.to_array();
        assert_eq!(array, vec!["a    b     c"]);

        // Test array with empty strings
        let val = Value::Array(vec!["".to_string(), "a".to_string(), "".to_string()]);
        assert_eq!(val.to_scalar(), " a ");
        assert_eq!(val.to_array().len(), 3);

        // Test associative array with empty values
        let mut map = BTreeMap::new();
        map.insert("key1".to_string(), "".to_string());
        map.insert("key2".to_string(), "value".to_string());
        let val = Value::Assoc(map);
        assert_eq!(val.to_scalar(), " value"); // Empty value first due to sorting
        assert_eq!(val.to_array().len(), 2);
    }

    #[test]
    fn test_env_modification_patterns() {
        let mut env = Env::new();

        // Test converting between value types
        env.set_scalar("VAR", "scalar_value");
        assert!(matches!(env.get("VAR"), Some(Value::Scalar(_))));

        // Override with array
        env.set_array("VAR", vec!["a", "b"]);
        assert!(matches!(env.get("VAR"), Some(Value::Array(_))));

        // Override with associative array
        env.set_assoc("VAR", vec![("key", "value")]);
        assert!(matches!(env.get("VAR"), Some(Value::Assoc(_))));

        // Back to scalar
        env.set_scalar("VAR", "back_to_scalar");
        assert!(matches!(env.get("VAR"), Some(Value::Scalar(_))));
        if let Some(Value::Scalar(s)) = env.get("VAR") {
            assert_eq!(s, "back_to_scalar");
        }
    }
}
