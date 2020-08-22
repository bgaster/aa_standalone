use serde::{Deserialize};
use serde_repr::{Deserialize_repr};

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Value {
    VInt(i32),
    VFloat(f32),
    VString(String),
    VPair((u8,u8)),
    VVU8(Vec<u8>),
}

impl From<Value> for i32 {
    fn from(v: Value) -> Self {
        match v {
            Value::VFloat(f) => f as i32,
            Value::VInt(i) => i,
            _ => 0,
        }
    } 
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Self::VFloat(f) => f.to_string(),
            Self::VInt(i) => i.to_string(),
            Self::VString(s) => s.clone(),
            Self::VPair((x,y)) => {
                "[".to_string() + &x.to_string() + "," + &y.to_string() + "]"
            }
            Self::VVU8(v) => {
                let mut s = "[".to_string();
                for (i, u) in v.iter().enumerate() {
                    s = s + &((*u) as i32).to_string();
                    if i + 1 != v.len() {
                        s.push(',')
                    }
                }
                s.push(']');
                s
            },
        }
    }
}

pub type Index = u32;