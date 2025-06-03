#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SKIDColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl SKIDColor {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        SKIDColor { r, g, b, a }
    }
    pub fn to_u32(&self) -> u32 {
        ((self.r as u32) << 24) | ((self.g as u32) << 16) | ((self.b as u32) << 8) | (self.a as u32)
    }
    pub fn from_u32(color: u32) -> Self {
        SKIDColor {
            r: (color >> 24) as u8,
            g: (color >> 16) as u8,
            b: (color >> 8) as u8,
            a: color as u8,
        }
    }
    
}

impl std::ops::Add for SKIDColor {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        SKIDColor {
            r: self.r.saturating_add(other.r),
            g: self.g.saturating_add(other.g),
            b: self.b.saturating_add(other.b),
            a: self.a.saturating_add(other.a),
        }
    }
}
impl std::ops::Sub for SKIDColor {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        SKIDColor {
            r: self.r.saturating_sub(other.r),
            g: self.g.saturating_sub(other.g),
            b: self.b.saturating_sub(other.b),
            a: self.a.saturating_sub(other.a),
        }
    }
}

impl std::ops::Mul<SKIDColor> for SKIDColor {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        SKIDColor {
            r: self.r.saturating_mul(other.r),
            g: self.g.saturating_mul(other.g),
            b: self.b.saturating_mul(other.b),
            a: self.a.saturating_mul(other.a),
        }
    }
}

impl std::ops::Div<SKIDColor> for SKIDColor {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        SKIDColor {
            r: if other.r == 0 { 0 } else { self.r / other.r },
            g: if other.g == 0 { 0 } else { self.g / other.g },
            b: if other.b == 0 { 0 } else { self.b / other.b },
            a: if other.a == 0 { 0 } else { self.a / other.a },
        }
    }
}


impl std::ops::Mul<u8> for SKIDColor {
    type Output = Self;

    fn mul(self, scalar: u8) -> Self {
        SKIDColor {
            r: self.r.saturating_mul(scalar),
            g: self.g.saturating_mul(scalar),
            b: self.b.saturating_mul(scalar),
            a: self.a.saturating_mul(scalar),
        }
    }
}

impl std::ops::Mul<f32> for SKIDColor {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        SKIDColor {
            r: (self.r as f32 * scalar).min(255.0) as u8,
            g: (self.g as f32 * scalar).min(255.0) as u8,
            b: (self.b as f32 * scalar).min(255.0) as u8,
            a: (self.a as f32 * scalar).min(255.0) as u8,
        }
    }
}

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