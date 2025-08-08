#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefaultOp {
    Dash,
    Assign,
    Plus,
    QMark,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoveOp {
    Prefix { long: bool },
    Suffix { long: bool },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplaceScope {
    First,
    Global,
    AnchorPrefix,
    AnchorSuffix,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamExpr {
    Ref { target: Target },
    Length { inner: Box<ParamExpr> },
    Defaulting {
        inner: Box<ParamExpr>,
        colon: bool,
        op: DefaultOp,
        word: String,
    },
    Remove {
        inner: Box<ParamExpr>,
        op: RemoveOp,
        pattern: String,
    },
    Replace {
        inner: Box<ParamExpr>,
        scope: ReplaceScope,
        pat: String,
        repl: String,
    },
    Substring {
        inner: Box<ParamExpr>,
        offset: i64,
        len: Option<i64>,
    },
    Indirection { inner: Box<ParamExpr> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    BadSubstitution(String),
    Unsupported(String),
    Eval(String),
}
