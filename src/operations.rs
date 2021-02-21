use crate::value::Value;
use crate::value::Value::*;

macro_rules! impl_two_op_integer {
    ($f:ident) => {
        pub fn $f(left: Value, right: Value) -> Value {
            match (left, right) {
                (I32(v1), I32(v2)) => I32(v1.$f(v2 as u32)),
                (I64(v1), I64(v2)) => I64(v1.$f(v2 as u32)),
                _ => panic!("Type mismatch during {}", stringify!($f)),
            }
        }
    };
}

macro_rules! impl_two_op_all_numbers {
    ($f:ident, $k:expr) => {
        pub fn $f(left: Value, right: Value) -> Value {
            match (left, right) {
                (I32(v1), I32(v2)) => I32($k(v1, v2) as i32),
                (I64(v1), I64(v2)) => I64($k(v1, v2) as i64),
                (F32(v1), F32(v2)) => F32($k(v1, v2) as u32 as f32),
                (F64(v1), F64(v2)) => F64($k(v1, v2) as u32 as f64),
                _ => panic!("Type mismatch during {}", stringify!($f)),
            }
        }
    };
}

macro_rules! impl_one_op_integer {
    ($f:ident) => {
        pub fn $f(left: Value) -> Value {
            match left {
                I32(v1) => I32(v1.$f() as i32),
                I64(v1) => I64(v1.$f() as i64),
                _ => panic!("Type mismatch during {}", stringify!($f)),
            }
        }
    };
}

macro_rules! impl_one_op_float {
    ($f:ident) => {
        pub fn $f(left: Value) -> Value {
            match left {
                F32(v1) => F32(v1.$f() as f32),
                F64(v1) => F64(v1.$f() as f64),
                _ => panic!("Type mismatch during {}", stringify!($f)),
            }
        }
    };
}

macro_rules! impl_one_op_float_closure {
    ($k:ident, $f:expr) => {
        pub fn $k(left: Value) -> Value {
            match left {
                F32(v1) => F32($f(v1.into()) as f32),
                F64(v1) => F64($f(v1) as f64),
                _ => panic!("Type mismatch during {}", stringify!($f)),
            }
        }
    };
}

macro_rules! impl_two_op_float {
    ($f:ident, @nan $k:expr) => {
        pub fn $f(left: Value, right: Value) -> Value {
            match (left, right) {
                (F32(v1), F32(v2)) if v1.is_nan() || v2.is_nan() => F32(f32::NAN),
                (F64(v1), F64(v2)) if v1.is_nan() || v2.is_nan() => F64(f64::NAN),
                (F32(v1), F32(v2)) => F32($k(v1.into(), v2.into()) as f32),
                (F64(v1), F64(v2)) => F64($k(v1, v2) as f64),
                _ => panic!("Type mismatch during {}", stringify!($f)),
            }
        }
    };
    ($f:ident, $k:expr) => {
        pub fn $f(left: Value, right: Value) -> Value {
            match (left, right) {
                (F32(v1), F32(v2)) => F32($k(v1.into(), v2.into()) as f32),
                (F64(v1), F64(v2)) => F64($k(v1, v2) as f64),
                _ => panic!("Type mismatch during {}", stringify!($f)),
            }
        }
    };
}

macro_rules! impl_trunc_sat_u {
    (@from $from_ty:ident @to $target:ident @but $cast:ident, $ret:ident, $fn:ident) => {
        pub(crate) fn $fn(f: Value) -> Value {
            let val = match f {
                $from_ty(v) => v,
                _ => panic!("Truncation only works on floats"),
            };

            if val.is_nan() {
                return $ret(0);
            }

            if val.is_infinite() {
                if val.is_sign_negative() {
                    return $ret(0);
                } else {
                    return $ret($target::MAX as $cast);
                }
            }

            return $ret(val.trunc() as $target as $cast);
        }
    };
}

macro_rules! impl_trunc_sat_s {
    (@from $bits:ident @to $target:ident, $ret:ident, $fn:ident) => {
        pub(crate) fn $fn(f: Value) -> Value {
            let val = match f {
                F32(v) => v as f64,
                F64(v) => v,
                _ => panic!("Truncation only works on floats"),
            };
            if val.is_nan() {
                return $ret(0);
            }

            if val.is_infinite() {
                if val.is_sign_negative() {
                    return $ret($bits::MIN as $target);
                } else {
                    return $ret($bits::MAX as $target);
                }
            }

            if val < $bits::MIN as f64 {
                return $ret($bits::MIN as $target);
            }
            if val > $bits::MAX as f64 {
                return $ret($bits::MAX as $target);
            }
            return $ret(val.trunc() as $target);
        }
    };
}

impl_two_op_integer!(rotate_left);
impl_two_op_integer!(rotate_right);

impl_one_op_integer!(leading_zeros);
impl_one_op_integer!(trailing_zeros);
impl_one_op_integer!(count_ones);

impl_two_op_all_numbers!(lt, |left, right| left < right);
impl_two_op_all_numbers!(gt, |left, right| left > right);
impl_two_op_all_numbers!(le, |left, right| left <= right);
impl_two_op_all_numbers!(ge, |left, right| left >= right);

impl_one_op_float!(abs);
impl_one_op_float_closure!(neg, |w: f64| -w);
impl_one_op_float!(ceil);
impl_one_op_float!(floor);
//impl_one_op_float!(round);
//impl_one_op_float!(nearest);
impl_one_op_float!(sqrt);
impl_one_op_float!(trunc);

impl_trunc_sat_s!(@from i32 @to i32, I32, trunc_sat_i32_s);
impl_trunc_sat_s!(@from i64 @to i64, I64, trunc_sat_i64_s);
impl_trunc_sat_u!(@from F32 @to u32 @but i32, I32, trunc_sat_from_f32_to_i32_u);
impl_trunc_sat_u!(@from F64 @to u32 @but i32, I32, trunc_sat_from_f64_to_i32_u);
impl_trunc_sat_u!(@from F32 @to u64 @but i64, I64, trunc_sat_from_f32_to_i64_u);
impl_trunc_sat_u!(@from F64 @to u64 @but i64, I64, trunc_sat_from_f64_to_i64_u);

impl_two_op_float!(min, @nan |left: f64, right: f64| left.min(right));
impl_two_op_float!(max, @nan |left: f64, right: f64| left.max(right));

impl_two_op_float!(copysign, |left: f64, right: f64| right.copysign(left));

pub fn eqz(left: Value) -> Value {
    match left {
        I32(v1) => I32((v1 == 0_i32) as i32),
        I64(v1) => I32((v1 == 0_i64) as i32),
        _ => panic!("Type mismatch during eqz"),
    }
}

pub fn nearest(v: Value) -> Value {
    use std::ops::Rem;

    match v {
        F32(v1) if v1.is_nan() => F32(f32::NAN),
        F64(v1) if v1.is_nan() => F64(f64::NAN),
        F32(v1) => {
            let fract = v1.fract().abs();

            let round = v1.round();
            if (fract - 0.5).abs() > f32::EPSILON {
                F32(round)
            } else if (round.rem(2.0) - 1.0).abs() < f32::EPSILON {
                F32(v1.floor())
            } else if (round.rem(2.0) - -1.0).abs() < f32::EPSILON {
                F32(v1.ceil())
            } else {
                F32(round)
            }
        }
        F64(v1) => {
            let fract = v1.fract().abs();

            let round = v1.round();
            if (fract - 0.5).abs() > f64::EPSILON {
                F64(round)
            } else if (round.rem(2.0) - 1.0).abs() < f64::EPSILON {
                F64(v1.floor())
            } else if (round.rem(2.0) - -1.0).abs() < f64::EPSILON {
                F64(v1.ceil())
            } else {
                F64(round)
            }
        }
        _ => panic!("Type mismatch during nearest"),
    }
}

pub fn reinterpret(v: Value) -> Value {
    match v {
        I32(k) => F32(f32::from_bits(k as u32)),
        I64(k) => F64(f64::from_bits(k as u64)),
        F32(k) => I32(i32::from_le(k.to_bits() as i32)),
        F64(k) => I64(i64::from_le(k.to_bits() as i64)),
    }
}

#[macro_export]
macro_rules! convert {
    ($self:expr, $val:ident, $from_ctr:ident, $to_ctr:ident, $to:ident) => {
        match $val {
            $from_ctr(i) => $self.store.stack.push(StackContent::Value($to_ctr(i as $to))),
            x => return Err(anyhow!("Expected $from_ctr on stack but found {:?}", x)),
        }
    };
    ($self:expr, $val:ident, $from_ctr:ident, $to_ctr:ident, $to:ident, $intermediate:ident) => {
        match $val {
            $from_ctr(i) => $self
                .store
                .stack
                .push(StackContent::Value($to_ctr(i as $intermediate as $to))),
            x => return Err(anyhow!("Expected $from_ctr on stack but found {:?}", x)),
        }
    };
}
