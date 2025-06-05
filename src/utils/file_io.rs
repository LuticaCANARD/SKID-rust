use crate::model::skid_image::SKIDSizeVector2;
use crate::model::{skid_color::SKIDColor, skid_image::SKIDImage};
use image::{ImageFormat, Rgba};
use std::fs::File;
use std::io::{BufWriter,BufReader};
use std::sync::{Arc, Mutex};
use std::thread;

// Define a static default color to avoid temporary borrow issues
static DEFAULT_COLOR: SKIDColor = SKIDColor {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 0.0,
};

pub fn export_to_png(
    image: &SKIDImage,
    file_path: &str,
    thread_count: Option<usize>,
) -> Result<(), String> {
    
    let file = File::create(file_path).map_err(|e| e.to_string())?;
    let ref mut writer = BufWriter::new(file);
    let size = image.get_size();
    let width = size.width;
    let height = size.height;
    let num_threads = if let Some(count) = thread_count {count} 
    else {4};// 기본값으로 4개의 스레드를 사용
    let rows_per_thread = (height + num_threads - 1) / num_threads;
    let default_row = Arc::new(vec![DEFAULT_COLOR; size.width]);
    // 미리 2차원 벡터를 준비
    let rows = Arc::new(Mutex::new(vec![vec![[0u16; 4]; width]; height]));
    let mut handles = Vec::new();
    let origin_image = Arc::new(image.get_data().clone());
    for thread_idx in 0..num_threads {
        let rows = Arc::clone(&rows); // 각 스레드에 Arc clone 전달
        let start_row = thread_idx * rows_per_thread;
        let end_row = ((thread_idx + 1) * rows_per_thread).min(height);
        let origin_image = Arc::clone(&origin_image); // 각 스레드에 Arc clone 전달
        let default_row = Arc::clone(&default_row);
        let handle = thread::spawn(move || {
            for y in start_row..end_row {
            let value = origin_image.get(y);
            let now_row = value.unwrap_or(&default_row);
                for x in 0..width {
                    let color = now_row.get(x).unwrap_or(&DEFAULT_COLOR);
                    let generated_r = (color.r * 65535.0) as u16;
                    let generated_g = (color.g * 65535.0) as u16;
                    let generated_b = (color.b * 65535.0) as u16;
                    let generated_a = (color.a * 65535.0) as u16;
                    // 각 스레드에서 lock을 얻어야 함
                    let mut rows_guard = rows.lock().unwrap();
                    rows_guard[y][x] = [generated_r, generated_g, generated_b, generated_a];
                }
            }
        });
        handles.push(handle);
}
    for handle in handles {
        handle.join().unwrap();
    }

    // 1차원 u16 벡터로 변환
    let rows = Arc::try_unwrap(rows)
        .map_err(|_| "Arc unwrap failed: 다른 스레드에서 rows를 아직 참조 중입니다.".to_string())?
        .into_inner()
        .map_err(|_| "Mutex 해제 실패".to_string())?;

    let flat: Vec<u16> = rows.into_iter()
        .flat_map(|row| row.into_iter().flat_map(|px| px))
        .collect();

    let img: image::ImageBuffer<Rgba<u16>, _> =
        image::ImageBuffer::from_raw(width as u32, height as u32, flat)
            .ok_or("Failed to create image buffer")?;

    img.write_to(writer, ImageFormat::Png)
        .map_err(|e| e.to_string())?;

    Ok(())

}


pub fn import_from_png(file_path: &str,thread_count:Option<usize>) -> Result<SKIDImage, String> {
    let thread_count = thread_count.unwrap_or(4); // 기본값으로 4개의 스레드를 사용
    // Open the file
    let file = File::open(file_path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);

    // Load the image
    let img = image::load(reader, ImageFormat::Png)
        .map_err(|e| e.to_string())?
        .to_rgba16();

    // Get the dimensions
    let (width, height) = img.dimensions();

    let pixel_data =  Arc::new(Mutex::new(vec![
        vec![SKIDColor::new( 0.0, 0.0, 0.0, 0.0 ); width as usize]; 
        height as usize
    ]));

    let row_per_thread = ((height + thread_count as u32 - 1) / thread_count as u32) as usize;
    let img = Arc::new(img);

    // Fill the SKIDImage with pixel data


    let mut handles = Vec::new();
    for thread_idx in 0..thread_count {
        let pixel_data = Arc::clone(&pixel_data);
        let img = Arc::clone(&img);
        let start_row = thread_idx * row_per_thread ;
        let end_row = ((thread_idx + 1) * row_per_thread).min(height as usize);

        let handle = thread::spawn(move || {
            for y in start_row..end_row {
                for x in 0..width {
                    let pixel = img.get_pixel(x as u32, y as u32);
                    let Rgba([r, g, b, a]) = *pixel;
                    let color = SKIDColor::new(
                        r as f32 / 65535.0,
                        g as f32 / 65535.0,
                        b as f32 / 65535.0,
                        a as f32 / 65535.0,
                    );
                    let mut data = pixel_data.lock().unwrap();
                    data[y][x as usize] = color;
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    let pixel_data = Arc::try_unwrap(pixel_data)
        .map_err(|_| "Arc unwrap failed".to_string())?
        .into_inner()
        .map_err(|_| "Mutex unlock failed".to_string())?;


    let skid_image = SKIDImage::from_data_size(
        SKIDSizeVector2 { 
            width: width as usize, 
            height: height as usize 
        },
        pixel_data,
    );


    Ok(skid_image)
}
