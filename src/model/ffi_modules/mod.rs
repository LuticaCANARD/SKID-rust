use crate::model::{skid_color::SKIDColor, skid_vector3::SKIDVector3};


#[no_mangle]
pub extern "C" fn skid_color_new(r: f32, g: f32, b: f32, a: f32) -> SKIDColor {
    SKIDColor::new(r, g, b, a)
}

#[no_mangle]
pub extern "C" fn skid_color_to_f32_array(color: SKIDColor, out_array: *mut f32) {
    assert!(!out_array.is_null());
    let arr = color.to_f32_array();
    unsafe {
        std::ptr::copy_nonoverlapping(arr.as_ptr(), out_array, 4);
    }
}

#[no_mangle]
pub extern "C" fn skid_color_from_f32_array(color_val: *const f32) -> SKIDColor {
    assert!(!color_val.is_null());
    let slice = unsafe { std::slice::from_raw_parts(color_val, 4) };
    SKIDColor::from_f32_array(slice.try_into().unwrap())
}

// 연산자 오버로딩에 대한 FFI 함수들
#[no_mangle]
pub extern "C" fn skid_color_add(c1: SKIDColor, c2: SKIDColor) -> SKIDColor {
    c1 + c2 // Rust의 Add 트레잇 구현 사용
}

#[no_mangle]
pub extern "C" fn skid_color_sub(c1: SKIDColor, c2: SKIDColor) -> SKIDColor {
    c1 - c2 // Rust의 Sub 트레잇 구현 사용
}

#[no_mangle]
pub extern "C" fn skid_color_mul_color(c1: SKIDColor, c2: SKIDColor) -> SKIDColor {
    c1 * c2 // Rust의 Mul<SKIDColor> 트레잇 구현 사용
}

#[no_mangle]
pub extern "C" fn skid_color_div_color(c1: SKIDColor, c2: SKIDColor) -> SKIDColor {
    c1 / c2 // Rust의 Div<SKIDColor> 트레잇 구현 사용
}

#[no_mangle]
pub extern "C" fn skid_color_mul_f32(color: SKIDColor, scalar: f32) -> SKIDColor {
    color * scalar // Rust의 Mul<f32> 트레잇 구현 사용
}

#[no_mangle]
pub extern "C" fn skid_vector3_new(x: f32, y: f32, z: f32) -> SKIDVector3 {
    SKIDVector3::new(x, y, z)
}

#[no_mangle]
pub extern "C" fn skid_vector3_dot(v1: SKIDVector3, v2: SKIDVector3) -> f32 {
    v1.dot(&v2)
}

#[no_mangle]
pub extern "C" fn skid_vector3_cross(v1: SKIDVector3, v2: SKIDVector3) -> SKIDVector3 {
    v1.cross(&v2)
}

#[no_mangle]
pub extern "C" fn skid_vector3_add(v1: SKIDVector3, v2: SKIDVector3) -> SKIDVector3 {
    v1 + v2 // Rust 내부의 Add 트레잇 사용
}

#[no_mangle]
pub extern "C" fn skid_vector3_sub(v1: SKIDVector3, v2: SKIDVector3) -> SKIDVector3 {
    v1 - v2 // Rust 내부의 Sub 트레잇 사용
}

#[no_mangle]
pub extern "C" fn skid_vector3_mul_f32(v: SKIDVector3, scalar: f32) -> SKIDVector3 {
    v * scalar // Rust 내부의 Mul<f32> 트레잇 사용
}

#[no_mangle]
pub extern "C" fn skid_vector3_div_f32(v: SKIDVector3, scalar: f32) -> SKIDVector3 {
    if scalar == 0.0 {
        return SKIDVector3::new(0.0, 0.0, 0.0);
    }
    v / scalar // Rust 내부의 Div<f32> 트레잇 사용
}

#[no_mangle]
pub extern "C" fn skid_vector3_neg(v: SKIDVector3) -> SKIDVector3 {
    -v // Rust 내부의 Neg 트레잇 사용
}

pub mod skid_image_ffi;