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

pub const CONTINUATION_BIT: u8 = 1 << 7;
pub const SIGN_BIT: u8 = 1 << 6;

#[inline]
fn low_bits_of_byte(byte: u8) -> u8 {
    byte & !CONTINUATION_BIT
}

pub fn read_i32_leb128(input: &[u8]) -> (i32, usize) {
    let mut result = 0;
    let mut shift = 0;
    let size = 32;
    let mut byte;
    let mut bytes_read = 0;
    let mut counter = 0;

    loop {
        if counter >= input.len() {
            panic!("not enough data");
        }

        byte = input[counter];
        bytes_read += 1;
        if shift == 31 && byte != 0x00 && byte != 0x7f {
            panic!("Overflow");
        }

        let low_bits = low_bits_of_byte(byte) as i32;
        result |= low_bits << shift;
        shift += 7;

        if byte & CONTINUATION_BIT == 0 {
            break;
        }

        counter += 1;
    }

    if shift < size && (SIGN_BIT & byte) == SIGN_BIT {
        // Sign extend the result.
        result |= !0 << shift;
    }


    (result, bytes_read)
}

pub fn read_i64_leb128(input: &[u8]) -> (i64, usize) {
    let mut result = 0;
    let mut shift = 0;
    let size = 64;
    let mut byte;
    let mut bytes_read = 0;
    let mut counter = 0;

    loop {
        if counter >= input.len() {
            panic!("not enough data");
        }

        byte = input[counter];
        bytes_read += 1;
        if shift == 63 && byte != 0x00 && byte != 0x7f {
            panic!("Overflow");
        }

        let low_bits = low_bits_of_byte(byte) as i64;
        result |= low_bits << shift;
        shift += 7;

        if byte & CONTINUATION_BIT == 0 {
            break;
        }

        counter += 1;
    }

    if shift < size && (SIGN_BIT & byte) == SIGN_BIT {
        // Sign extend the result.
        result |= !0 << shift;
    }


    (result, bytes_read)
}

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
