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