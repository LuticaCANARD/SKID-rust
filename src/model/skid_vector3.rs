#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SKIDVector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl SKIDVector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        SKIDVector3 { x, y, z }
    }

    pub fn dot(&self, other: &SKIDVector3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &SKIDVector3) -> Self {
        SKIDVector3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
}

impl std::ops::Add for SKIDVector3 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        SKIDVector3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}
impl std::ops::Sub for SKIDVector3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        SKIDVector3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl std::ops::Mul<f32> for SKIDVector3 {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        SKIDVector3 {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl std::ops::Div<f32> for SKIDVector3 {
    type Output = Self;

    fn div(self, scalar: f32) -> Self {
        if scalar == 0.0 {
            panic!("Division by zero in SKIDVector3");
        }
        SKIDVector3 {
            x: self.x / scalar,
            y: self.y / scalar,
            z: self.z / scalar,
        }
    }
}

impl std::ops::Neg for SKIDVector3 {
    type Output = Self;

    fn neg(self) -> Self {
        SKIDVector3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
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