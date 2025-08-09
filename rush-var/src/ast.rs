/// Represents a word that can contain text, parameter expansions, command substitutions, or arithmetic substitutions
#[derive(Debug, Clone, PartialEq)]
pub enum Word {
    Text(String),
    Param(Box<ParamExpr>),
    CmdSubst(String),   // $(...) literal (not executed)
    ArithSubst(String), // $((...)) literal (not executed)
}

/// Zsh parameter expansion flags
#[derive(Debug, Clone, PartialEq)]
pub struct ZshFlag {
    pub kind: ZFlag,
    pub args: Vec<String>,
}

/// Supported zsh flags
#[derive(Debug, Clone, PartialEq)]
pub enum ZFlag {
    /// (U) - uppercase
    U,
    /// (L) - lowercase  
    L,
    /// (C) - capitalize
    C,
    /// (q) - shell quote
    Q,
    /// (Q) - remove shell quotes
    Unquote,
    /// (f) - split on newlines
    F,
    /// (z) - shell word splitting
    Z,
    /// (Z) - extended shell word splitting with options
    ZExt,
    /// (u) - unique
    Unique,
    /// (o) - sort ascending
    O,
    /// (O) - sort descending  
    ODesc,
    /// (k) - keys of associative array
    K,
    /// (v) - values of associative array
    V,
    /// (t) - type of parameter
    T,
    /// (l:width::pad:) - left pad
    L2 { width: String, fill: String, pad: String },
    /// (r:width::pad:) - right pad
    R2 { width: String, fill: String, pad: String },
    /// (j:sep:) - join array elements
    J { sep: String },
    /// (s:sep:) - split string  
    S { sep: String },
    /// (V) - display with escape sequences visible
    VDisplay,
    /// (P) - indirection
    P,
    /// (e) - re-expand result (security risk)
    E,
}

/// Default/assign/alt/error operations  
#[derive(Debug, Clone, PartialEq)]
pub enum DefaultOp {
    Dash,   // -
    Assign, // =
    Plus,   // +
    QMark,  // ?
}

/// Prefix/suffix removal operations
#[derive(Debug, Clone, PartialEq)]
pub enum RemoveOp {
    Prefix { long: bool }, // # or ##
    Suffix { long: bool }, // % or %%
}

/// Replace operation scopes
#[derive(Debug, Clone, PartialEq)]
pub enum ReplaceScope {
    First,        // /
    Global,       // //
    AnchorPrefix, // /#
    AnchorSuffix, // /%
}

/// Array/string indexing
#[derive(Debug, Clone, PartialEq)]
pub enum Index {
    None,
    One(i64),           // name[i] - numeric index
    Key(String),        // name[key] - string key for associative arrays
    Slice(i64, i64),    // name[i,j]
    StrSlice(i64, i64), // for string [offset,length] form
}

/// Parameter target (name, special char, or positional)
#[derive(Debug, Clone, PartialEq)]
pub struct Target {
    pub name: String,            // plain name
    pub special: Option<char>,   // '?', '-', '$', '!', '*', '@', '#', '0'-'9' etc.
    pub positional: Option<u32>, // for $1, $2, etc.
}

/// Main parameter expression AST
#[derive(Debug, Clone, PartialEq)]
pub enum ParamExpr {
    /// Basic reference: $name, ${name}, ${name[index]}
    Ref { target: Target, index: Index },

    /// Length: ${#name}
    Length { inner: Box<ParamExpr> },

    /// Default/assign/alt/error: ${name:-word}, ${name:=word}, etc.
    Defaulting {
        inner: Box<ParamExpr>,
        colon: bool,
        op: DefaultOp,
        word: Vec<Word>,
    },

    /// Prefix/suffix removal: ${name#pat}, ${name%pat}
    Remove {
        inner: Box<ParamExpr>,
        op: RemoveOp,
        pattern: Vec<Word>,
    },

    /// Replace: ${name/pat/repl}, ${name//pat/repl}
    Replace {
        inner: Box<ParamExpr>,
        scope: ReplaceScope,
        pat: Vec<Word>,
        repl: Vec<Word>,
    },

    /// Substring: ${name:offset:length}
    Substring {
        inner: Box<ParamExpr>,
        offset: i64,
        len: Option<i64>,
    },

    /// Indirection: ${!name}, ${(P)name}
    Indirection { inner: Box<ParamExpr>, style: IndirectStyle },

    /// Zsh flags: ${(flags)name}
    ZshFlags { flags: Vec<ZshFlag>, inner: Box<ParamExpr> },

    /// Path modifiers: ${name:h:t:r:e}
    Modifiers { inner: Box<ParamExpr>, mods: Vec<PathMod> },
}

/// Indirection styles
#[derive(Debug, Clone, PartialEq)]
pub enum IndirectStyle {
    BashBang, // ${!name}
    ZshP,     // ${(P)name}
}

/// Path modifier types
#[derive(Debug, Clone, PartialEq)]
pub enum PathMod {
    H,      // :h - dirname
    T,      // :t - basename
    R,      // :r - root (remove extension)
    E,      // :e - extension
    A,      // :A - realpath (absolute)
    LowerA, // :a - realpath (resolve symlinks)
}

/// Error types
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    BadSubstitution(String),
    Unsupported(String),
    Eval(String),
    IndexOutOfBounds(String),
    InvalidPattern(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BadSubstitution(msg) => write!(f, "bad substitution: {}", msg),
            Error::Unsupported(msg) => write!(f, "unsupported: {}", msg),
            Error::Eval(msg) => write!(f, "evaluation error: {}", msg),
            Error::IndexOutOfBounds(msg) => write!(f, "index out of bounds: {}", msg),
            Error::InvalidPattern(msg) => write!(f, "invalid pattern: {}", msg),
        }
    }
}

impl std::error::Error for Error {}
