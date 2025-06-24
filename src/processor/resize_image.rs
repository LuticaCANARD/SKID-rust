use cubecl::{cube, frontend::*, terminate, CubeCount, CubeDim, CubeElement, Runtime, prelude::*,Kernel};

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

#[cube(launch_unchecked)]
pub fn resize_scaleup_kernel<F: Float>(
    input: &Array<F>,
    width: u32,
    height: u32,
    new_width: u32,
    new_height: u32,
    output: &mut Array<F>,
) {
    let i_width = width as i32;
    let i_height = height as i32;

    // TODO : 스플라인 보간 구현
    for x in 0..CUBE_CLUSTER_DIM_X {
        let px = ABSOLUTE_POS_X + x;
        for y in 0..CUBE_CLUSTER_DIM_Y {
            let py = ABSOLUTE_POS_Y + y;
            if px < new_width && py < new_height {
                // Calculate the corresponding coordinates in the original image using the generic float type F
                let original_x = (F::cast_from(px) + F::new(0.5)) * (F::cast_from(width) / F::cast_from(new_width)) - F::new(0.5);
                let original_y = (F::cast_from(py) + F::new(0.5)) * (F::cast_from(height) / F::cast_from(new_height)) - F::new(0.5);

                // Use `cast_from` for direct, idiomatic type conversion within the kernel.
                let x_floor = i32::cast_from(original_x);
                let y_floor = i32::cast_from(original_y);

                let mut final_r = F::new(0.0);
                let mut final_g = F::new(0.0);
                let mut final_b = F::new(0.0);
                let mut final_a = F::new(0.0);

                // Iterate over the 4x4 neighborhood
                for i in -1..=2 {
                    for j in -1..=2 {
                        let sample_x = (x_floor + i) as i32;
                        let sample_y = (y_floor + j) as i32;

                        // Check if the sampled pixel is within the original image bounds (0 <= coord < size)
                        if sample_x >= 0 && sample_x < i_width && sample_y >= 0 && sample_y < i_height {
                            // Cast to u32 for array indexing to fix the compile error
                            let u_sample_x = u32::cast_from(sample_x);
                            let u_sample_y = u32::cast_from(sample_y);
                            let idx = (u_sample_y * width + u_sample_x) * 4;

                            let pixel_r = input[idx];
                            let pixel_g = input[idx + 1];
                            let pixel_b = input[idx + 2];
                            let pixel_a = input[idx + 3];

                            let weight_x = (original_x - F::cast_from(sample_x)) * (original_x - F::cast_from(
                                sample_x
                            ));
                            let weight_y = (original_y - F::cast_from(sample_y)) * (original_y - F::cast_from(
                                sample_y
                            ));
                            let weight = (F::new(1.0) - weight_x) * (F::new(1.0) - weight_y);

                            final_r = final_r + F::cast_from(pixel_r * weight);
                            final_g = final_g + F::cast_from(pixel_g * weight);
                            final_b = final_b + F::cast_from(pixel_b * weight);
                            final_a = final_a + F::cast_from(pixel_a * weight);
                        }
                    }
                }
                let new_idx = (py * new_width + px) * 4;
                output[new_idx] = F::cast_from(px as f32 / new_width as f32);
                output[new_idx + 1] = F::cast_from(py as f32 / new_height as f32);
                output[new_idx + 2] = F::new(1.0 as f32);
                output[new_idx + 3] = F::new(1.0 as f32); // 보간된 알파 채널
            }
        }
    }
}

// This kernel uses a "gather" approach, iterating over output pixels.
// This avoids race conditions from multiple threads writing to the same location.
#[cube(launch_unchecked)]
fn resize_scaledown_kernel<F: Float>(
    input: &Array<F>,
    width: u32,
    height: u32,
    new_width: u32,
    new_height: u32,
    output: &mut Array<F>,
) {
    for x in 0..CUBE_CLUSTER_DIM_X {
        let px = ABSOLUTE_POS_X + x;
        for y in 0..CUBE_CLUSTER_DIM_Y {
            let py = ABSOLUTE_POS_Y + y;
            if px < new_width && py < new_height {
                // Calculate corresponding coordinates in the original (larger) image
                // using nearest-neighbor.
                let original_x = (px * width) / new_width;
                let original_y = (py * height) / new_height;

                let original_idx = (original_y * width + original_x) * 4;
                let new_idx = (py * new_width + px) * 4;

                // Copy the pixel value (all 4 channels) from input to output
                for i in 0..4 {
                    output[new_idx + i] = input[original_idx + i];
                }
            }
        }
    }
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

    if new_width > original_image.get_size().width as u32 {
        unsafe {
            resize_scaleup_kernel::launch_unchecked::<f32, T>(
                &client,
                CubeCount::Static(threads_x as u32, threads_y as u32, 1),
                CubeDim::new(block_x as u32, block_y as u32, 1),
                ArrayArg::from_raw_parts::<f32>(
                    &input_handle,
                    pixel_count, 
                    4
                ),
                ScalarArg::from(cubecl::frontend::ScalarArg { elem: original_image.get_size().width as u32}),
                ScalarArg::from(cubecl::frontend::ScalarArg { elem: original_image.get_size().height as u32}),
                ScalarArg::from(cubecl::frontend::ScalarArg { elem: new_width}),
                ScalarArg::from(cubecl::frontend::ScalarArg { elem: new_height}),
                ArrayArg::from_raw_parts::<f32>(
                    &output_handle, 
                    new_size.width * new_size.height, 
                    1
                )
            );
        }
    } else {
        unsafe {
            resize_scaledown_kernel::launch_unchecked::<f32, T>(
                &client,
                CubeCount::Static(threads_x as u32, threads_y as u32, 1),
                CubeDim::new(block_x as u32, block_y as u32, 1),
                ArrayArg::from_raw_parts::<f32>(
                    &input_handle,
                    pixel_count, 
                    4
                ),
                ScalarArg::from(cubecl::frontend::ScalarArg { elem: original_image.get_size().width as u32}),
                ScalarArg::from(cubecl::frontend::ScalarArg { elem: original_image.get_size().height as u32}),
                ScalarArg::from(cubecl::frontend::ScalarArg { elem: new_width}),
                ScalarArg::from(cubecl::frontend::ScalarArg { elem: new_height}),
                ArrayArg::from_raw_parts::<f32>(
                    &output_handle, 
                    new_size.width * new_size.height, 
                    1
                )
            );
        }
    }


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