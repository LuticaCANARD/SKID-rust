use crate::model::skid_image::SKIDSizeVector2;
use crate::model::{skid_color::SKIDColor, skid_image::SKIDImage};
use image::{ImageFormat, Rgba};
use std::fs::File;
use std::io::BufWriter;

pub fn export_to_png(
    image: &SKIDImage,
    file_path: &str,
) -> Result<(), String> {
    // Create a file to write the PNG image
    let file = File::create(file_path).map_err(|e| e.to_string())?;
    let ref mut writer = BufWriter::new(file);

    let img_data = image.get_data();
    let size = image.get_size();

    // Create an image buffer with the correct dimensions
    let mut img: image::ImageBuffer<Rgba<u16>, _> = image::ImageBuffer::new(size.width as u32, size.height as u32);
    img.enumerate_pixels_mut().for_each(|(x, y, pixel)| {
        let color = image.get_pixel(x, y);

        if color.is_none() {
            // If the pixel is out of bounds, set it to transparent black
            *pixel = Rgba([0,0,0,0]);
            return;
        }
        let color = color.unwrap();
        let generated_r = (color.r * 65535.0) as u16; // Scale to u16 range
        let generated_g = (color.g * 65535.0) as u16; // Scale to u16 range
        let generated_b = (color.b * 65535.0) as u16; // Scale to u16 range
        let generated_a = (color.a * 65535.0) as u16; // Scale to u16 range

        *pixel = Rgba([generated_r , generated_g, generated_b,generated_a]);
    });

    // Write the image as PNG
    img.write_to(writer, ImageFormat::Png)
        .map_err(|e| e.to_string())?;

    Ok(())
}


pub fn import_from_png(file_path: &str) -> Result<SKIDImage, String> {
    use image::{ImageBuffer, Rgba};
    use std::fs::File;
    use std::io::BufReader;

    // Open the file
    let file = File::open(file_path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);

    // Load the image
    let img = image::load(reader, ImageFormat::Png)
        .map_err(|e| e.to_string())?
        .to_rgba16();

    // Get the dimensions
    let (width, height) = img.dimensions();

    let mut pixel_data:Vec<Vec<SKIDColor>> = vec![
        vec![SKIDColor::new( 0.0, 0.0, 0.0, 0.0); width as usize]; 
        height as usize
    ];
    // Fill the SKIDImage with pixel data
    for (x, y, pixel) in img.enumerate_pixels() {
        let Rgba([r, g, b, a]) = *pixel;
        let color = SKIDColor::new(
            r as f32 / 65535.0,
            g as f32 / 65535.0,
            b as f32 / 65535.0,
            a as f32 / 65535.0,
        );

        // Set the pixel in the SKIDImage
        pixel_data[y as usize][x as usize] = color;
    }

    Ok(
        SKIDImage::from_data_size(
            SKIDSizeVector2 { 
                width: width as usize, 
                height: height as usize 
            },
            pixel_data,
        )
    )
}
