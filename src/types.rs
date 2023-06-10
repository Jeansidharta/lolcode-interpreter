use std::cell::RefCell;
use std::collections::HashMap;

use lolcode_ast::parser::expression::ASTType;

pub enum RuntimeError {
    IdentifierNotFound,
    CannotSRSNonYarn,

    GenericError,

    InvalidType,
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

#[derive(Debug, Clone, PartialEq)]
pub struct Bukkit(pub HashMap<String, Value>);

impl Default for Bukkit {
    fn default() -> Self {
        Bukkit(HashMap::new())
    }
}

#[derive(Debug, Clone)]
pub struct Scope<'a> {
    pub parent_scope: Option<&'a Scope<'a>>,
    pub variables: RefCell<HashMap<String, Value>>,
    pub it: RefCell<Value>,
}

impl Default for Scope<'_> {
    fn default() -> Self {
        Self {
            parent_scope: None,
            variables: RefCell::new(HashMap::new()),
            it: RefCell::new(Value::Noob),
        }
    }
}

impl<'a> Scope<'a> {
    pub fn from_parent(parent: &'a Scope) -> Scope<'a> {
        let mut scope = Scope::default();
        scope.parent_scope = Some(parent);
        scope
    }

    pub fn child(&'a self) -> Scope<'a> {
        Scope::from_parent(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Yarn(String),
    Numbr(i32),
    Numbar(f32),
    Troof(bool),
    Noob,
    Bukkit(Bukkit),
}

impl std::fmt::Display for Bukkit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for value in self.0.values().into_iter() {
            write!(f, "{}", value)?;
        }
        Ok(())
    }
}

impl From<ASTType> for Value {
    fn from(value: ASTType) -> Self {
        (&value).into()
    }
}

impl From<&ASTType> for Value {
    fn from(value: &ASTType) -> Self {
        match value {
            ASTType::Yarn => Value::Yarn(String::default()),
            ASTType::Bukkit => Value::Bukkit(Bukkit::default()),
            ASTType::Numbr => Value::Numbr(0),
            ASTType::Numbar => Value::Numbar(0.0),
            ASTType::Troof => Value::Troof(false),
            ASTType::Noob => Value::Noob,
        }
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::Numbr(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Value::Numbar(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::Yarn(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Troof(value)
    }
}

impl Value {
    pub fn to_boolean(&self) -> bool {
        match self {
            Value::Noob => false,
            Value::Yarn(yarn) => yarn != "",
            Value::Numbr(val) => *val != 0,
            Value::Numbar(val) => *val != 0.0,
            Value::Troof(val) => *val,
            Value::Bukkit(_) => true,
        }
    }

    pub fn not(&self) -> bool {
        !self.to_boolean()
    }

    pub fn and(&self, other: Value) -> bool {
        self.to_boolean() && other.to_boolean()
    }
}
