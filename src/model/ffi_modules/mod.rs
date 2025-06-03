use crate::model::{skid_color::SKIDColor, skid_vector3::SKIDVector3};


#[unsafe(no_mangle)]
pub extern "C" fn skid_color_new(r: u8, g: u8, b: u8, a: u8) -> SKIDColor {
    SKIDColor::new(r, g, b, a)
}

#[unsafe(no_mangle)]
pub extern "C" fn skid_color_to_u32(color: SKIDColor) -> u32 {
    color.to_u32() // 내부적으로 SKIDColor의 to_u32 메서드 사용
}

#[unsafe(no_mangle)]
pub extern "C" fn skid_color_from_u32(color_val: u32) -> SKIDColor {
    SKIDColor::from_u32(color_val)
}

// 연산자 오버로딩에 대한 FFI 함수들
#[unsafe(no_mangle)]
pub extern "C" fn skid_color_add(c1: SKIDColor, c2: SKIDColor) -> SKIDColor {
    c1 + c2 // Rust의 Add 트레잇 구현 사용
}

#[unsafe(no_mangle)]
pub extern "C" fn skid_color_sub(c1: SKIDColor, c2: SKIDColor) -> SKIDColor {
    c1 - c2 // Rust의 Sub 트레잇 구현 사용
}

#[unsafe(no_mangle)]
pub extern "C" fn skid_color_mul_color(c1: SKIDColor, c2: SKIDColor) -> SKIDColor {
    c1 * c2 // Rust의 Mul<SKIDColor> 트레잇 구현 사용
}

#[unsafe(no_mangle)]
pub extern "C" fn skid_color_div_color(c1: SKIDColor, c2: SKIDColor) -> SKIDColor {
    c1 / c2 // Rust의 Div<SKIDColor> 트레잇 구현 사용
}

#[unsafe(no_mangle)]
pub extern "C" fn skid_color_mul_u8(color: SKIDColor, scalar: u8) -> SKIDColor {
    color * scalar // Rust의 Mul<u8> 트레잇 구현 사용
}

#[unsafe(no_mangle)]
pub extern "C" fn skid_color_mul_f32(color: SKIDColor, scalar: f32) -> SKIDColor {
    color * scalar // Rust의 Mul<f32> 트레잇 구현 사용
}



#[unsafe(no_mangle)]
pub extern "C" fn skid_vector3_new(x: f32, y: f32, z: f32) -> SKIDVector3 {
    SKIDVector3::new(x, y, z)
}

#[unsafe(no_mangle)]
pub extern "C" fn skid_vector3_dot(v1: SKIDVector3, v2: SKIDVector3) -> f32 {
    v1.dot(&v2)
}

#[unsafe(no_mangle)]
pub extern "C" fn skid_vector3_cross(v1: SKIDVector3, v2: SKIDVector3) -> SKIDVector3 {
    v1.cross(&v2)
}

#[unsafe(no_mangle)]
pub extern "C" fn skid_vector3_add(v1: SKIDVector3, v2: SKIDVector3) -> SKIDVector3 {
    v1 + v2 // Rust 내부의 Add 트레잇 사용
}

#[unsafe(no_mangle)]
pub extern "C" fn skid_vector3_sub(v1: SKIDVector3, v2: SKIDVector3) -> SKIDVector3 {
    v1 - v2 // Rust 내부의 Sub 트레잇 사용
}

#[unsafe(no_mangle)]
pub extern "C" fn skid_vector3_mul_f32(v: SKIDVector3, scalar: f32) -> SKIDVector3 {
    v * scalar // Rust 내부의 Mul<f32> 트레잇 사용
}

#[unsafe(no_mangle)]
pub extern "C" fn skid_vector3_div_f32(v: SKIDVector3, scalar: f32) -> SKIDVector3 {
    if scalar == 0.0 {
        return SKIDVector3::new(0.0, 0.0, 0.0);
    }
    v / scalar // Rust 내부의 Div<f32> 트레잇 사용
}

#[unsafe(no_mangle)]
pub extern "C" fn skid_vector3_neg(v: SKIDVector3) -> SKIDVector3 {
    -v // Rust 내부의 Neg 트레잇 사용
}