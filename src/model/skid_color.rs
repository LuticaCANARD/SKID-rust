#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SKIDColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}



impl SKIDColor {
    pub const SKID_U8_ARRAY_BYTE_SIZE:usize = 4; // 4 bytes per channel (RGBA)
    pub const SKID_ARRAY_RESOLUTION:usize = 1; // 1 byte per channel
    pub const SKID_U8_ARRAY_BYTE_SIZE_TOTAL:usize = Self::SKID_U8_ARRAY_BYTE_SIZE * Self::SKID_ARRAY_RESOLUTION; // 4 channels * 1 byte per channel (RGBA)
    pub const SKID_F32_ARRAY_BYTE_SIZE:usize = 16; // 4 channels * 4 bytes per channel (RGBA)
    
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        SKIDColor { r, g, b, a }
    }
    pub fn to_f32_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
    pub fn from_f32_array(arr: [f32; Self::SKID_U8_ARRAY_BYTE_SIZE]) -> Self {
        SKIDColor {
            r: arr[0],
            g: arr[1],
            b: arr[2],
            a: arr[3],
        }
    }

    pub fn to_u8_array(&self) -> [u8; Self::SKID_U8_ARRAY_BYTE_SIZE_TOTAL] {
        [
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            (self.a * 255.0) as u8,
        ]
    }
    pub fn from_u8_array(arr: [u8; Self::SKID_U8_ARRAY_BYTE_SIZE_TOTAL]) -> Self {
        SKIDColor {
            r: arr[0] as f32 / 255.0,
            g: arr[1] as f32 / 255.0,
            b: arr[2] as f32 / 255.0,
            a: arr[3] as f32 / 255.0,
        }
    }
}

impl std::ops::Add for SKIDColor {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        SKIDColor {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
            a: self.a + other.a,
        }
    }
}
impl std::ops::Sub for SKIDColor {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        SKIDColor {
            r: self.r - other.r,
            g: self.g - other.g,
            b: self.b - other.b,
            a: self.a - other.g
        }
    }
}

impl std::ops::Mul<SKIDColor> for SKIDColor {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        SKIDColor {
            r: self.r * other.r,
            g: self.g * other.g,
            b: self.b * other.b,
            a: self.a * other.a
        }
    }
}

impl std::ops::Div<SKIDColor> for SKIDColor {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        SKIDColor {
            r: if other.r == 0.0 { 0.0 } else { self.r / other.r },
            g: if other.g == 0.0 { 0.0 } else { self.g / other.g },
            b: if other.b == 0.0 { 0.0 } else { self.b / other.b },
            a: if other.a == 0.0 { 0.0 } else { self.a / other.a },
        }
    }
}


impl std::ops::Mul<f32> for SKIDColor {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        SKIDColor {
            r: self.r * scalar,
            g: self.g * scalar,
            b: self.b * scalar,
            a: self.a * scalar,
        }
    }
}