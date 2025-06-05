use cubecl::Runtime;
use cubecl::prelude::{Array, ArrayArg, Float, FloatExpand, ScalarArg};
use cubecl::{cube, CubeCount, CubeDim, CubeElement};

use crate::model::{skid_color::SKIDColor, skid_image::{SKIDImage, SKIDSizeVector2}};

pub fn resize_image<R:Runtime>(
    runtime: &R::Device,
    image: &SKIDImage,
    new_size:SKIDSizeVector2,
    thread_count: Option<usize>
) -> SKIDImage {
    launch::<R>(
        runtime,
        image,
        new_size,
        thread_count
    )
}


fn launch<T: Runtime>(
    run_device: &T::Device,
    original_image: &SKIDImage,
    new_size: SKIDSizeVector2,
    thread_count: Option<usize>
) -> SKIDImage {
    let client = T::client(run_device);
    let thread_count = thread_count.unwrap_or(4);
    let input = original_image.get_1d_data_as_f32();
    
    let new_width = new_size.width as u32;
    let new_height = new_size.height as u32;
    let output_handle = client.empty(new_size.width * new_size.height * 4 * core::mem::size_of::<f32>());
    let pixel_count = input.len() / 4; // Assuming each color has 4 components (RGBA)
    let input_handle = client.create(bytemuck::cast_slice(&input));
    let (max_thread_x, max_thread_y, _max_thread_z) = T::max_cube_count();
    let block_x = (new_width + max_thread_x - 1) / max_thread_x;
    let threads_x = if new_width < max_thread_x { new_width } else { max_thread_x };
    let block_y = (new_height + max_thread_y - 1) / max_thread_y;
    let threads_y = if new_height < max_thread_y { new_height } else { max_thread_y };


    println!("Launching resize with runtime: {}x{}", new_width, new_height);
    unsafe {

    }

    
    // Launch the resize kernel here (not implemented in this snippet)
    
    let bytes = client.read_one(output_handle.binding());
    let output = f32::from_bytes(&bytes);
    
    let output_colors: Vec<SKIDColor> = output.chunks(4)
        .map(|chunk| SKIDColor::from_f32_array(chunk.try_into().unwrap()))
        .collect();
    
    SKIDImage::from_1d_data(
        new_size, 
        output_colors
    )
}