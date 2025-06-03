use crate::model::{skid_color::SKIDColor};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SKIDSizeVector2 {
    pub width: usize,
    pub height: usize,
}
pub struct SKIDImage {
    width: usize,
    height: usize,
    data: Vec<Vec<SKIDColor>>,
    len: usize,
    size:SKIDSizeVector2,
}

impl SKIDImage {
    /// Creates a new SKIDImage with the specified width and height, initializing all pixels to transparent black.
    pub fn new(width: usize, height: usize) -> Self {
        let data = vec![
            vec![SKIDColor::new(0.0, 0.0, 0.0, 0.0); width as usize];
            height as usize
        ];
        SKIDImage { width, height, data, len: (width * height) as usize, size:
            SKIDSizeVector2 { width, height }
        }
    }
    pub fn new_with_color(width: usize, height: usize, color: SKIDColor) -> Self {
        let data = vec![
            vec![color; width as usize];
            height as usize
        ];
        SKIDImage { width, height, data, len: (width * height) as usize, size:
            SKIDSizeVector2 { width, height }
        }
    }
    pub fn from_1d_data(size:SKIDSizeVector2,  data: Vec<SKIDColor>) -> Self {
        let width = size.width;
        let height = size.height;
        if data.len() != width * height {
            panic!("Data length does not match width and height");
        }
        let mut data_2d = Vec::with_capacity(height);
        for i in 0..height {
            let start = i * width;
            let end = start + width;
            data_2d.push(data[start..end].to_vec());
        }
        SKIDImage { 
            width, height, data: data_2d, 
            len: (width * height) as usize, 
            size: SKIDSizeVector2 { width, height } 
        }
    }
    pub fn from_data_size(size:SKIDSizeVector2, data: Vec<Vec<SKIDColor>>) -> Self {
        SKIDImage::from_data(size.width, size.height, data)
    }
    pub fn from_data(width: usize, height: usize, data: Vec<Vec<SKIDColor>>) -> Self {
        if data.len() != height as usize // Check if data has the correct number of rows
        || data.iter().any(|row| row.len() != width as usize) {
            panic!("Data length does not match width and height");
        }
        SKIDImage { width, height, data, 
            len: (width * height) as usize, 
            size: SKIDSizeVector2 { width, height } }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Option<&SKIDColor> {
        if x < self.width as u32 && y < self.height as u32 {
            Some(&self.data[y as usize][x as usize])
        } else {
            None
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: SKIDColor) -> Option<()> {
        if x < self.width as u32 && y < self.height as u32 {
            self.data[y as usize][x as usize] = color;
            Some(())
        } else {
            None
        }
    }

    pub fn fill(&mut self, color: SKIDColor) {
        for height in &mut self.data {
            for pixel in height.iter_mut() {
                *pixel = color;
            }
        }
    }
    pub fn to_vec(&self) -> Vec<SKIDColor> {
        self.data.iter().flat_map(|row| row.iter().cloned()).collect()
    }
    pub fn len(self) -> usize {
        self.len
    }
    pub fn get_u8_byte_len(&self) -> usize {
        self.len * SKIDColor::SKID_U8_ARRAY_BYTE_SIZE_TOTAL
    }
    pub fn get_size(&self) -> SKIDSizeVector2 {
        self.size
    }
    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }
    pub fn to_byte_array(&self) -> Vec<u8> {
        self.data.iter().flat_map(|row| row.iter().flat_map(|color| color.to_u8_array())).collect()
    }
    pub fn get_data(&self) -> &Vec<Vec<SKIDColor>> {
        &self.data
    }
    pub fn from_raw_bytes(bytes: &[u8]) -> Self {
        if bytes.len() % SKIDColor::SKID_U8_ARRAY_BYTE_SIZE_TOTAL != 0 {
            panic!("Byte array length is not a multiple of SKIDColor::SKID_U8_ARRAY_BYTE_SIZE_TOTAL");
        }
        let pixel_count = bytes.len() / SKIDColor::SKID_U8_ARRAY_BYTE_SIZE_TOTAL;
        let mut data = Vec::with_capacity(pixel_count);
        for chunk in bytes.chunks(SKIDColor::SKID_U8_ARRAY_BYTE_SIZE_TOTAL) {
            data.push(SKIDColor::from_u8_array(chunk.try_into().unwrap()));
        }
        let width = (pixel_count as f64).sqrt() as usize;
        let height = pixel_count / width;
        SKIDImage::from_1d_data(SKIDSizeVector2 { width, height }, data)
    }
}