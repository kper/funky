use wasm_bindgen::prelude::*;

/*
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn add(a: usize, b: usize) -> usize {
    a + b
}
*/

#[wasm_bindgen]
pub fn my_loop() -> u32{
    let mut sum = 0 as u32;
    let v : Vec<u32> = vec![1, 2, 3];

    for i in v {
        sum += i;
    }

    return sum;
}
