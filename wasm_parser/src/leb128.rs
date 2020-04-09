macro_rules! impl_read_unsigned_leb128 {
    ($fn_name:ident, $int_ty:ident) => {
        #[inline]
        #[allow(dead_code)]
        pub fn $fn_name(slice: &[u8]) -> ($int_ty, usize) {
            let mut result = 0;
            let mut shift = 0;
            let mut position = 0;
            loop {
                let byte = slice[position];
                position += 1;
                if (byte & 0x80) == 0 {
                    result |= (byte as $int_ty) << shift;
                    return (result, position);
                } else {
                    result |= ((byte & 0x7F) as $int_ty) << shift;
                }
                shift += 7;
            }
        }
    };
}

macro_rules! impl_write_unsigned_leb128 {
    ($fn_name:ident, $int_ty:ident) => {
        #[inline]
        #[allow(dead_code)]
        pub fn $fn_name(out: &mut Vec<u8>, mut value: $int_ty) {
            loop {
                if value < 0x80 {
                    out.push(value as u8);
                    break;
                } else {
                    out.push(((value & 0x7f) | 0x80) as u8);
                    value >>= 7;
                }
            }
        }
    };
}

impl_read_unsigned_leb128!(read_u8_leb128, u8);
impl_read_unsigned_leb128!(read_u16_leb128, u16);
impl_read_unsigned_leb128!(read_u32_leb128, u32);
impl_read_unsigned_leb128!(read_u64_leb128, u64);
impl_read_unsigned_leb128!(read_u128_leb128, u128);

impl_write_unsigned_leb128!(write_u8_leb128, u8);
impl_write_unsigned_leb128!(write_u16_leb128, u16);
impl_write_unsigned_leb128!(write_u32_leb128, u32);
impl_write_unsigned_leb128!(write_u64_leb128, u64);

/*
pub fn read_signed_i32_leb128(data: &[u8], start_position: usize) -> (i32, usize) {
    let mut result = 0;
    let mut shift = 0;
    let mut position = start_position;
    let mut byte;

    loop {
        byte = data[position];
        position += 1;
        result |= i128::from(byte & 0x7F) << shift;
        shift += 7;

        if (byte & 0x80) == 0 {
            break;
        }
    }

    if (shift < 64) && ((byte & 0x40) != 0) {
        // sign extend
        result |= -(1 << shift);
    }

    (result, position - start_position)
}*/
