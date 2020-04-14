use wasm_parser::core::*;

type IResult<T> = Result<T, &'static str>;

#[derive(Debug, PartialEq, Eq)]
pub enum Either<T: PartialEq + Eq, G: PartialEq + Eq> {
    A(T),
    B(G),
}

// Leading question: Should validation return errors or panic?

/// k is the range
/// k must be between `n` and `m`
pub fn check_limits(limit: &Limits, k: u32) -> bool {
    match limit {
        Limits::Zero(n) => &k > n,
        Limits::One(n, m) => &k > n && m > &k && n < m,
    }
}

pub fn get_ty_of_blocktype(blocktype: BlockType, types: Vec<FuncType>) -> IResult<FuncType> {
    use std::convert::TryInto;

    let w = match blocktype {
        BlockType::ValueType(v) => get_ty_of_valuetype(v),
        BlockType::Empty => get_ty_of_valuetype_empty(),
        BlockType::S33(v) => get_ty_of_typeidx(types, v.try_into().unwrap()).unwrap(), //TODO make this safe
    };

    Ok(w)
}

// If there exists a `typeidx` in `types`, then `typeidx` has its type.
fn get_ty_of_typeidx(types: Vec<FuncType>, typeidx: usize) -> IResult<FuncType> {
    if let Some(t) = types.get(typeidx) {
        return Ok(FuncType {
            param_types: t.param_types.clone(),
            return_types: t.return_types.clone(),
        });
    }

    Err("No function with this index")
}

/// The valuetype has the type `[] -> [valtype]`
fn get_ty_of_valuetype(val: ValueType) -> FuncType {
    match val {
        ValueType::F32 => FuncType {
            param_types: vec![],
            return_types: vec![ValueType::F32],
        },
        ValueType::F64 => FuncType {
            param_types: vec![],
            return_types: vec![ValueType::F32],
        },
        ValueType::I32 => FuncType {
            param_types: vec![],
            return_types: vec![ValueType::I32],
        },
        ValueType::I64 => FuncType {
            param_types: vec![],
            return_types: vec![ValueType::I64],
        },
    }
}

/// The ty has the type `[] -> []`
fn get_ty_of_valuetype_empty() -> FuncType {
    FuncType {
        param_types: vec![],
        return_types: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_limits() {
        let l = Limits::One(10, 20);
        let l2 = Limits::Zero(10);

        assert!(check_limits(&l, 15));
        assert!(check_limits(&l2, 15));

        assert_eq!(false, check_limits(&l, 9));
        assert_eq!(false, check_limits(&l2, 9));
        assert_eq!(false, check_limits(&l, 21));
    }

    #[test]
    fn test_typeidx() {
        let types = vec![FuncType {
            param_types: vec![ValueType::F32, ValueType::F64],
            return_types: vec![ValueType::I64],
        }];

        let ty = FuncType {
            param_types: vec![ValueType::F32, ValueType::F64],
            return_types: vec![ValueType::I64],
        };

        assert_eq!(ty, get_ty_of_typeidx(types, 0).unwrap());
    }

    #[test]
    fn test_blocktype_funcidx() {
        let types = vec![FuncType {
            param_types: vec![ValueType::F32, ValueType::F64],
            return_types: vec![ValueType::I64],
        }];

        let ty = FuncType {
            param_types: vec![ValueType::F32, ValueType::F64],
            return_types: vec![ValueType::I64],
        };

        let bty = get_ty_of_blocktype(BlockType::S33(0), types).unwrap();

        assert_eq!(ty, bty);
    }

    #[test]
    fn test_blocktype_valuetype() {
        let types = vec![];

        let ty = FuncType {
            param_types: vec![],
            return_types: vec![ValueType::I64],
        };

        let bty = get_ty_of_blocktype(BlockType::ValueType(ValueType::I64), types).unwrap();

        assert_eq!(ty, bty);
    }
}
