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
fn resize_scaleup_kernel<F: Float>(
    input: &Array<F>,
    width: u32,
    height: u32,
    new_width: u32,
    new_height: u32,
    output: &mut Array<F>,
) {
    // TODO : 스플라인 보간 구현
    for x in 0..CUBE_CLUSTER_DIM_X {
        let px = ABSOLUTE_POS_X + x;
        for y in 0..CUBE_CLUSTER_DIM_Y {
            let py = ABSOLUTE_POS_Y + y;
            // Calculate the corresponding coordinates in the original image
            let original_x = (px as f32 + 0.5) * (width as f32 / new_width as f32) - 0.5;
            let original_y = (py as f32 + 0.5) * (height as f32 / new_height as f32) - 0.5;

            let x_floor = original_x as i32;
            let y_floor = original_y as i32;

            let mut final_r = F::new(0.0);
            let mut final_g = F::new(0.0);
            let mut final_b = F::new(0.0);
            let mut final_a = F::new(0.0);

            // Iterate over the 4x4 neighborhood
            for i in -1..=2 {
                for j in -1..=2 {
                    let sample_x = (x_floor + i) as u32;
                    let sample_y = (y_floor + j) as u32;

                    // Check if the sampled pixel is within the original image bounds
                    if sample_x < width && sample_y < height {
                        let idx = (sample_y * width + sample_x) * 4;
                        let pixel_r = input[idx];
                        let pixel_g = input[idx + 1];
                        let pixel_b = input[idx + 2];
                        let pixel_a = input[idx + 3];

                        let weight_x = (original_x - (sample_x as f32)) * (original_x - (sample_x as f32));
                        let weight_y = (original_y - (sample_y as f32)) * (original_y - (sample_y as f32));
                        let weight= F::cast_from((1.0 - weight_x) * (1.0 - weight_y));
                        final_r += pixel_r * weight;
                        final_g += pixel_g * weight;
                        final_b += pixel_b * weight;
                        final_a += pixel_a * weight;
                    }
                }
            }

            let new_idx = (py * new_width + px) * 4;
            output[new_idx] = final_r;
            output[new_idx + 1] = final_g;
            output[new_idx + 2] = final_b;
            output[new_idx + 3] = final_a; // 보간된 알파 채널
        }
    }
}

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
            let idx = py * width + px;

            // Calculate the corresponding pixel in the new image
            let new_x = (px * new_width) / width;
            let new_y = (py * new_height) / height;
            let new_idx = new_y * new_width + new_x;

            // Copy the pixel value from input to output
            output[new_idx] = input[idx];
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