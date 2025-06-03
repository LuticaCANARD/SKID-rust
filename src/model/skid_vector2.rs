#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SKIDVector2 {
    pub x: f32,
    pub y: f32,
}

impl SKIDVector2 {
    pub fn new(x: f32, y: f32) -> Self {
        SKIDVector2 { x, y }
    }

    pub fn dot(&self, other: &SKIDVector2) -> f32 {
        self.x * other.x + self.y * other.y
    }

}

impl std::ops::Add for SKIDVector2 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        SKIDVector2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl std::ops::Sub for SKIDVector2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        SKIDVector2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl std::ops::Mul<f32> for SKIDVector2 {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        SKIDVector2 {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl std::ops::Div<f32> for SKIDVector2 {
    type Output = Self;

    fn div(self, scalar: f32) -> Self {
        if scalar == 0.0 {
            panic!("Division by zero in SKIDVector2");
        }
        SKIDVector2 {
            x: self.x / scalar,
            y: self.y / scalar
        }
    }
}

impl std::ops::Neg for SKIDVector2 {
    type Output = Self;

    fn neg(self) -> Self {
        SKIDVector2 {
            x: -self.x,
            y: -self.y,

        }
    }
}
