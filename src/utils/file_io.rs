use crate::model::skid_image::SKIDSizeVector2;
use crate::model::{skid_color::SKIDColor, skid_image::SKIDImage};
use image::{ImageFormat, Rgba};
use std::fs::File;
use std::io::{BufWriter,BufReader};
use std::sync::{Arc, Mutex};
use std::thread;
use image::codecs::png::{PngEncoder, CompressionType, FilterType};
use std::path::Path;


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
    // 최적화 방향:
    // 1. 각 스레드가 자신의 결과를 별도의 버퍼에 저장하고, 마지막에 합치기만 하도록 Mutex 사용 최소화
    // 2. Arc<Mutex<>> 대신 Arc<Vec<...>>로 각 스레드가 독립적으로 작업
    // 3. clone 대신 참조만 사용 (가능하다면)
    // 4. 불필요한 unwrap_or, get 등 제거

    let file = File::create(file_path).map_err(|e| e.to_string())?;
    let mut writer = BufWriter::new(file);
    let size = image.get_size();
    let width = size.width;
    let height = size.height;
    let num_threads = thread_count.unwrap_or(4);
    let rows_per_thread = (height + num_threads - 1) / num_threads;

    // 최종 픽셀 데이터를 담을 2차원 벡터 (스레드별로 분할)
    let origin_image = Arc::new(image.get_data().clone());

    let mut handles = Vec::new();
    let start = std::time::Instant::now();
    for thread_idx in 0..num_threads {
        let origin_image = Arc::clone(&origin_image);
        let start_row = thread_idx * rows_per_thread;
        let end_row = ((thread_idx + 1) * rows_per_thread).min(height);

        // 각 스레드별 버퍼를 미리 준비
        let rows_count = end_row.saturating_sub(start_row);
        let mut local_buffer = vec![vec![[0u16; 4]; width]; rows_count];

        let handle = thread::spawn(move || {
            for (i, y) in (start_row..end_row).enumerate() {
                let fallback_row = vec![DEFAULT_COLOR; width];

                let now_row = origin_image.get(y).unwrap_or(&fallback_row);
                for x in 0..width {
                    let color = now_row.get(x).unwrap_or(&DEFAULT_COLOR);
                    local_buffer[i][x] = [
                        (color.r * 65535.0) as u16,
                        (color.g * 65535.0) as u16,
                        (color.b * 65535.0) as u16,
                        (color.a * 65535.0) as u16,
                    ];
                }
            }
            local_buffer
        });
        handles.push(handle);
    }
    println!("Thread spawn time: {:?}", start.elapsed());
    let start = std::time::Instant::now();
    // 스레드 결과 합치기
    let mut rows: Vec<Vec<[u16; 4]>> = Vec::with_capacity(height);
    for handle in handles {
        let buffer = handle.join().unwrap();
        rows.extend(buffer);
    }

    // 1차원 u16 벡터로 변환
    let flat: Vec<u16> = rows.into_iter()
        .flat_map(|row| row.into_iter().flat_map(|px| px))
        .collect();

    
    let img: image::ImageBuffer<Rgba<u16>, _> =
        image::ImageBuffer::from_raw(width as u32, height as u32, flat)
            .ok_or("Failed to create image buffer")?;
    println!("Image buffer creation time: {:?}", start.elapsed());
    let start = std::time::Instant::now();
    // Write the image to the file
    img.write_to(&mut writer, ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    println!("Image write time: {:?}", start.elapsed());
    Ok(())

}

pub fn export_rgba_channels_to_png(
    image: &SKIDImage,
    base_file_path: &str,
) -> Result<(), String> {

    let size = image.get_size();
    let width = size.width;
    let height = size.height;
    let data = image.get_data();

    let channels = ["r", "g", "b", "a"];
    let mut handles = Vec::new();
    let dir_path = Path::new(base_file_path);
    println!("Creating directory: {}", dir_path.display());
    std::fs::create_dir_all(dir_path)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    println!("Exporting channels to PNG files...");
    for (i, &ch) in channels.iter().enumerate() {
        let data = data.clone();
        let file_path = format!(
            "{}/{}.png",
            dir_path.display(),
            ch
        );
        println!("Exporting channel {} to {}", ch, file_path);
        let handle = thread::spawn(move || {
            let mut flat: Vec<u8> = Vec::with_capacity(width * height);
            for row in &data {
                for color in row {
                    let v = match i {
                        0 => color.r,
                        1 => color.g,
                        2 => color.b,
                        3 => color.a,
                        _ => 0.0,
                    };
                    flat.push((v.clamp(0.0, 1.0) * 255.0) as u8);
                }
            }
            let img: image::GrayImage =
                image::ImageBuffer::from_vec(width as u32, height as u32, flat)
                    .ok_or("Failed to create channel image buffer")?;
            let file = File::create(&file_path).map_err(|e| e.to_string())?;
            let mut writer = BufWriter::new(file);
            img.write_to(&mut writer, ImageFormat::Png)
                .map_err(|e| e.to_string())
        });
        handles.push(handle);
    }
    export_to_png(image, &format!("{}/combined.png", dir_path.display()), None)?;

    for handle in handles {
        handle.join().map_err(|_| "Thread join failed".to_string())??;
    }
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
