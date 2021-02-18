use crate::value::Value::*;
use log::trace;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Shl, Shr, Sub};
use wasm_parser::core::*;
use std::fmt;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Value {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

pub type Arity = u32;

impl Into<ValueType> for Value {
    fn into(self) -> ValueType {
        match self {
            Value::I32(_) => ValueType::I32,
            Value::I64(_) => ValueType::I64,
            Value::F32(_) => ValueType::F32,
            Value::F64(_) => ValueType::F64,
        }
    }
}

impl Value {
    pub fn convert(self, vt: ValueType) -> Value {
        trace!("Convert {:?} to {:?}", self, vt);
        match (self, vt) {
            (Value::I32(v), ValueType::I64) => Value::I64(v as i64),
            (Value::I32(v), ValueType::F32) => Value::F32(v as f32),
            (Value::I32(v), ValueType::F64) => Value::F64(v as f64),
            (Value::I64(v), ValueType::I32) => Value::I32(v as i32),
            (Value::I64(v), ValueType::F32) => Value::F32(v as f32),
            (Value::I64(v), ValueType::F64) => Value::F64(v as f64),
            (Value::F32(v), ValueType::F64) => Value::F64(v as f64),
            (Value::F32(v), ValueType::I32) => Value::I32(v as i32),
            (Value::F32(v), ValueType::I64) => Value::I64(v as i64),
            (Value::F64(v), ValueType::F32) => Value::F32(v as f32),
            (Value::F64(v), ValueType::I32) => Value::I32(v as i32),
            (Value::F64(v), ValueType::I64) => Value::I64(v as i64),
            _ => self,
        }
    }

    pub fn signum(&self) -> f32 {
        match self {
            Value::F32(k) => k.signum() as f32,
            Value::F64(k) => k.signum() as f32,
            Value::I32(k) => k.signum() as f32,
            Value::I64(k) => k.signum() as f32,
        }
    }

    pub fn is_f32(&self) -> bool {
        matches!(self, Value::F32(_))
    }

    pub fn is_f64(&self) -> bool {
        matches!(self, Value::F64(_))
    }
}

impl Add for Value {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        match (self, other) {
            (I32(v1), I32(v2)) => I32(v1.wrapping_add(v2)),
            (I64(v1), I64(v2)) => I64(v1.wrapping_add(v2)),
            (F32(v1), F32(v2)) => F32(v1 + v2),
            (F64(v1), F64(v2)) => F64(v1 + v2),
            _ => panic!("Type missmatch during addition"),
        }
    }
}

impl Sub for Value {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        match (self, other) {
            (I32(v1), I32(v2)) => I32(v1.wrapping_sub(v2)),
            (I64(v1), I64(v2)) => I64(v1.wrapping_sub(v2)),
            (F32(v1), F32(v2)) => F32(v1 - v2),
            (F64(v1), F64(v2)) => F64(v1 - v2),
            _ => panic!("Type missmatch during subtraction"),
        }
    }
}

impl Mul for Value {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        match (self, other) {
            (I32(v1), I32(v2)) => I32(v1.wrapping_mul(v2)),
            (I64(v1), I64(v2)) => I64(v1.wrapping_mul(v2)),
            (F32(v1), F32(v2)) => F32(v1 * v2),
            (F64(v1), F64(v2)) => F64(v1 * v2),
            _ => panic!("Type missmatch during subtraction"),
        }
    }
}

impl Div for Value {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        match (self, other) {
            (I32(v1), I32(v2)) => I32(v1.wrapping_div(v2)),
            (I64(v1), I64(v2)) => I64(v1.wrapping_div(v2)),
            (F32(v1), F32(v2)) => F32(v1 / v2),
            (F64(v1), F64(v2)) => F64(v1 / v2),
            _ => panic!("Type missmatch during division"),
        }
    }
}

impl BitAnd for Value {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        match (self, other) {
            (I32(v1), I32(v2)) => I32(v1 & v2),
            (I64(v1), I64(v2)) => I64(v1 & v2),
            _ => panic!("Type missmatch during bitand"),
        }
    }
}

impl BitOr for Value {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        match (self, other) {
            (I32(v1), I32(v2)) => I32(v1 | v2),
            (I64(v1), I64(v2)) => I64(v1 | v2),
            _ => panic!("Type missmatch during bitor"),
        }
    }
}

impl BitXor for Value {
    type Output = Self;
    fn bitxor(self, other: Self) -> Self {
        match (self, other) {
            (I32(v1), I32(v2)) => I32(v1 ^ v2),
            (I64(v1), I64(v2)) => I64(v1 ^ v2),
            _ => panic!("Type missmatch during bitxor"),
        }
    }
}

impl Shl for Value {
    type Output = Self;
    fn shl(self, other: Self) -> Self {
        match (self, other) {
            (I32(v1), I32(v2)) => I32(v1.wrapping_shl(v2 as u32)),
            (I64(v1), I64(v2)) => I64(v1.wrapping_shl(v2 as u32)),
            _ => panic!("Type missmatch during shift left"),
        }
    }
}

impl Shr for Value {
    type Output = Self;
    fn shr(self, other: Self) -> Self {
        match (self, other) {
            (I32(v1), I32(v2)) => I32(v1.wrapping_shr(v2 as u32)),
            (I64(v1), I64(v2)) => I64(v1.wrapping_shr(v2 as u32)),
            _ => panic!("Type missmatch during shift right"),
        }
    }
}

impl Rem for Value {
    type Output = Self;
    fn rem(self, other: Self) -> Self {
        match (self, other) {
            (I32(v1), I32(v2)) => I32(v1.wrapping_rem(v2)),
            (I64(v1), I64(v2)) => I64(v1.wrapping_rem(v2)),
            (F32(v1), F32(v2)) => F32(v1 % v2),
            (F64(v1), F64(v2)) => F64(v1 % v2),
            _ => panic!("Type missmatch during remainder"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}