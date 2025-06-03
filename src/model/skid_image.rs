use crate::model::skid_color::SKIDColor;

pub struct SKIDImage {
    pub width: u32,
    pub height: u32,
    pub data: Vec<SKIDColor>,
}

impl SKIDImage {
    pub fn new(width: u32, height: u32) -> Self {
        let data = vec![SKIDColor::new(0, 0, 0, 0); (width * height) as usize];
        SKIDImage { width, height, data }
    }
    pub fn from_data(width: u32, height: u32, data: Vec<SKIDColor>) -> Self {
        if data.len() != (width * height) as usize {
            panic!("Data length does not match width and height");
        }
        SKIDImage { width, height, data }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Option<&SKIDColor> {
        if x < self.width && y < self.height {
            Some(&self.data[(y * self.width + x) as usize])
        } else {
            None
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: SKIDColor) -> Option<()> {
        if x < self.width && y < self.height {
            self.data[(y * self.width + x) as usize] = color;
            Some(())
        } else {
            None
        }
    }

    pub fn fill(&mut self, color: SKIDColor) {
        for pixel in &mut self.data {
            *pixel = color.clone();
        }
    }
}