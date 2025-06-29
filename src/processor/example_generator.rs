use cubecl::{cube, prelude::{Array, ArrayArg, Float, ScalarArg, ABSOLUTE_POS_X, ABSOLUTE_POS_Y, CUBE_CLUSTER_DIM_X, CUBE_CLUSTER_DIM_Y}, CubeCount, CubeDim, Runtime,frontend::*};

use crate::model::{skid_color::SKIDColor, skid_image::{SKIDImage, SKIDSizeVector2}};

#[cube(launch)]
pub fn gpu_example_generator(
    width: u32,
    height: u32,
    output: &mut Array<f32>,
) {
    for x in 0..CUBE_CLUSTER_DIM_X {
        let px = ABSOLUTE_POS_X + x;
        for y in 0..CUBE_CLUSTER_DIM_Y {
            let py = ABSOLUTE_POS_Y + y;
            let new_idx = (py * width + px) * 4;
            let border_zone = 1024; // 32 pixels border zone
            let is_border = py % border_zone == 0 || px % border_zone == 0;
            let r_p = px as f32 / f32::cast_from(width);
            let g_p = py as f32 / f32::cast_from(height);
            let b_p = if is_border {
                1.0f32.into()
            } else {
                (px as f32 / f32::cast_from(width)) * 0.5f32
                + (py as f32 / f32::cast_from(height)) * 0.5f32
            };
            output[new_idx] = r_p;
            output[new_idx + 1] = g_p;
            output[new_idx + 2] = 1.0f32 - b_p; // 보간된 색상
            output[new_idx + 3] = 1.0f32; // Alpha channel
        }
    }
}

pub fn launch<T: Runtime>(
    run_device: &T::Device,
    new_size: SKIDSizeVector2,
    thread_count: Option<usize>
) -> SKIDImage {
    let client = T::client(run_device);
    let output_handle = client.empty(new_size.width * new_size.height * 4 * core::mem::size_of::<f32>());
    let thread_count = thread_count.unwrap_or(4) as u32;

    unsafe {
        let new_width = new_size.width as u32;
        let new_height = new_size.height as u32;
        let block_x_o = new_width / thread_count;
        let block_y_o = new_height / thread_count;

        let threads_x = if block_x_o * block_y_o > 1024 {
            new_width / 32
        } else {
            thread_count
        };
        let threads_y = if block_x_o * block_y_o > 1024 {
            new_height / 32
        } else {
            thread_count
        };
        let block_x = (new_width + threads_x - 1) / threads_x;
        let block_y = (new_height + threads_y - 1) / threads_y;

        println!("Launching example generator with runtime: {}x{}", new_width, new_height);
        println!("Threads: {}x{}", threads_x, threads_y);
        println!("Blocks: {}x{}", block_x, block_y);
        gpu_example_generator::launch::<T>(
            &client,
            CubeCount::Static(threads_x as u32, threads_y as u32, 1),
            CubeDim::new(block_x as u32, block_y as u32, 1),
            ScalarArg { elem: new_width },
            ScalarArg { elem: new_height },
            ArrayArg::from_raw_parts::<f32>(
                    &output_handle, 
                    new_size.width * new_size.height, 
                    1
                )
        );
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