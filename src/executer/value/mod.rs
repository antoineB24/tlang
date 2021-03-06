

use std::{ops::Range, fmt, hash::Hash, collections::HashMap, rc::Rc};

use super::*;



macro_rules! build_enum {
    ($name:ident, $($variant:ident),*) => {
        enum $name {
            $($variant),*
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Ident(pub String);

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Type {
    Int, 
    String,
    Bool,
    List,
    Func,
    Range,
    Enum,
    FieldEnum(String),
    Struct(String),
    FieldStruct(String),
    None
}

pub struct Function(pub Rc<dyn Fn(HashMap<String, Var>, Vm) -> Result<Value, Error>>);

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Function {
        name: String,
        func: Function,
        args: Vec<String>,
    },
    DefStruct {
        name: String,
        fields: Vec<Ident>,
        function: HashMap<String, Value>
    },
    CallStruct {
        name: String,
        fields: HashMap<Ident, Value>,
    },
    List(Vec<Value>),
    Range(Range<isize>),
    Enum {
        variants: Vec<String>,
    },
    EnumCall {
        name: String,
        field: String,
    },
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Var {
    pub value: Value,
    pub mutable: bool,
    pub type_: Type,

}

impl Clone for Function {
    fn clone(&self) -> Self {
        Function(self.0.clone())
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Function")
    }
}



impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        &self.0 as *const _ == &other.0 as *const _
    }
}
impl Hash for Function {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (&self.0 as *const _ as usize).hash(state);
    }
}

impl Eq for Function {

}

impl Value {
    pub fn add(&self, other: &Value) -> Result<Value, Error> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            _ => Err(Error::CannotAdd(CannotAddError {
                left: self.to_string(),
                right: other.to_string(),
            })),
        }
    }
    pub fn sub(&self, other: &Value) -> Result<Value, Error> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
            _ => Err(Error::CannotSub(CannotSubError {
                left: self.to_string(),
                right: other.to_string(),
            })),
        }
    }

    pub fn mul(&self, other: &Value) -> Result<Value, Error> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
            _ => Err(Error::CannotMul(CannotMulError {
                left: self.to_string(),
                right: other.to_string(),
            })),
        }
    }

    pub fn div(&self, other: &Value) -> Result<Value, Error> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a / b)),
            _ => Err(Error::CannotDiv(CannotDivError {
                left: self.to_string(),
                right: other.to_string(),
            })),
        }
    }

    pub fn modulo(&self, other: &Value) -> Result<Value, Error> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a % b)),
            _ => Err(Error::CannotMod(CannotModError {
                left: self.to_string(),
                right: other.to_string(),
            })),
        }
    }

    pub fn eq(&self, other: &Value) -> Result<Value, Error> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a == b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a == b)),
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a == b)),
            _ => Err(Error::CannotCompare(CannotCompareError {
                left: self.to_string(),
                right: other.to_string(),
            })),
        }
    }

    pub fn neq(&self, other: &Value) -> Result<Value, Error> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a != b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a != b)),
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a != b)),
            _ => Err(Error::CannotCompare(CannotCompareError {
                left: self.to_string(),
                right: other.to_string(),
            })),
        }
    }

    pub fn gt(&self, other: &Value) -> Result<Value, Error> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(*a > *b)),
            _ => Err(Error::CannotCompare(CannotCompareError {
                left: self.to_string(),
                right: other.to_string(),
            })),
        }
    }

    pub fn lt(&self, other: &Value) -> Result<Value, Error> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(*a < *b)),
            _ => Err(Error::CannotCompare(CannotCompareError {
                left: self.to_string(),
                right: other.to_string(),
            })),
        }
    }

    pub fn ge(&self, other: &Value) -> Result<Value, Error> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(*a >= *b)),
            _ => Err(Error::CannotCompare(CannotCompareError {
                left: self.to_string(),
                right: other.to_string(),
            })),
        }
    }

    pub fn le(&self, other: &Value) -> Result<Value, Error> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(*a <= *b)),
            _ => Err(Error::CannotCompare(CannotCompareError {
                left: self.to_string(),
                right: other.to_string(),
            })),
        }
    }

    pub fn and(&self, other: &Value) -> Result<Value, Error> {
        match (self, other) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a && *b)),
            _ => Err(Error::CannotCompare(CannotCompareError {
                left: self.to_string(),
                right: other.to_string(),
            })),
        }
    }

    pub fn or(&self, other: &Value) -> Result<Value, Error> {
        match (self, other) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a || *b)),
            _ => Err(Error::CannotCompare(CannotCompareError {
                left: self.to_string(),
                right: other.to_string(),
            })),
        }
    }

    pub fn display_value(&self) -> String {
        match self {
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
            Value::Bool(b) => b.to_string(),
            Value::Function { .. } => "function".to_string(),
            Value::List(list) => {
                let mut s = String::new();
                s.push_str("[");
                for (i, item) in list.iter().enumerate() {
                    if i > 0 {
                        s.push_str(", ");
                    }
                    s.push_str(&item.display_value());
                }
                s.push_str("]");
                s
            }
            Value::Range(_) => "range".to_string(),
            Value::None => "None".to_string(),
            Value::DefStruct { .. } => todo!(),
            Value::CallStruct { .. } => todo!(),
            Value::Enum { .. } => todo!(),
            Value::EnumCall { .. } => todo!()

        }
    }

    pub fn get_type(&self) -> Type {
        match self {
            Value::Number(_) => Type::Int,
            Value::String(_) => Type::String,
            Value::Bool(_) => Type::Bool,
            Value::Function { .. } => Type::Func,
            Value::List(_) => Type::List,
            Value::Range(_) => Type::Range,
            Value::CallStruct { name , ..} => Type::FieldStruct(name.clone()),
            Value::DefStruct { name, .. } => Type::Struct(name.clone()),
            Value::None => Type::None,
            Value::Enum { .. } => Type::Enum,
            Value::EnumCall { name, .. } => Type::FieldEnum(name.clone()),
        }
    }
}



impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_value())
    }
}
