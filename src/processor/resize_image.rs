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
    let width_f = F::cast_from(width);
    let height_f = F::cast_from(height);
    let new_width_f = F::cast_from(new_width);
    let new_height_f = F::cast_from(new_height);
    let width_u = width as u32;

    for x in 0..CUBE_CLUSTER_DIM_X {
        let px = ABSOLUTE_POS_X + x;
        for y in 0..CUBE_CLUSTER_DIM_Y {
            let py = ABSOLUTE_POS_Y + y;

            if px < new_width && py < new_height {
                // Calculate corresponding coordinates in the original image
                let original_x_f = (F::cast_from(px) + F::new(0.5)) * width_f / new_width_f - F::new(0.5);
                let original_y_f = (F::cast_from(py) + F::new(0.5)) * height_f / new_height_f - F::new(0.5);

                // Get integer coordinates and fractional parts for interpolation
                let x0 = F::floor(original_x_f);
                let y0 = F::floor(original_y_f);
                let tx = original_x_f - x0;
                let ty = original_y_f - y0;

                let x0_u = u32::cast_from(x0);
                let y0_u = u32::cast_from(y0);
                
                // Clamp coordinates to be within bounds
                let x1_u = if x0_u + 1 < width - 1 {
                    x0_u + 1
                } else {
                    width - 1
                };
                let y1_u = if y0_u + 1 < height - 1 {
                    y0_u + 1
                } else {
                    height - 1
                };
                let x0_clamped_u = x0_u;
                let y0_clamped_u = y0_u;

                // Get the four surrounding pixels (c00, c10, c01, c11)
                let c00_idx = y0_clamped_u * width_u + x0_clamped_u;
                let c10_idx = y0_clamped_u * width_u + x1_u;
                let c01_idx = y1_u * width_u + x0_clamped_u;
                let c11_idx = y1_u * width_u + x1_u;

                let new_idx = (py * new_width + px) * 4;

                // Interpolate for each channel (R, G, B, A)
                for i in 0..4 {
                    let c00 = input[c00_idx][i];
                    let c10 = input[c10_idx][i];
                    let c01 = input[c01_idx][i];
                    let c11 = input[c11_idx][i];

                    // Lerp horizontally
                    let top_lerp = c00 * (F::new(1.0) - tx) + c10 * tx;
                    let bottom_lerp = c01 * (F::new(1.0) - tx) + c11 * tx;

                    // Lerp vertically
                    let final_val = top_lerp * (F::new(1.0) - ty) + bottom_lerp * ty;
                    output[new_idx + i] = final_val;
                }
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
    //new_width < width && new_height < height 인 경우에만 작동합니다.
    let width_f = F::cast_from(width);
    let height_f = F::cast_from(height);
    let new_width_f = F::cast_from(new_width);
    let new_height_f = F::cast_from(new_height);
    let width_u = width as u32;
    let height_u = height as u32;

    for x in 0..CUBE_CLUSTER_DIM_X {
        let px = ABSOLUTE_POS_X + x;
        for y in 0..CUBE_CLUSTER_DIM_Y {
            let py = ABSOLUTE_POS_Y + y;
            if px < new_width && py < new_height {
                let new_idx = (py * new_width + px) * 4;

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

    let thread_x_o = if new_width < max_thread_x { new_width } else { max_thread_x };
    let thread_y_o = if new_height < max_thread_y { new_height } else { max_thread_y };

    let thread_x = if thread_x_o * thread_y_o > 1024 {
        new_width / 32
    } else {
        thread_count as u32
    };
    let thread_y = if thread_x_o * thread_y_o > 1024 {
        new_height / 32
    } else {
        thread_count as u32
    };
    let block_x = (new_width as u32) / thread_x;
    let block_y = (new_height as u32) / thread_y;

    println!("Launching resize with runtime: {}x{}", new_width, new_height);
    println!("Threads: {}x{}", thread_x, thread_y);
    println!("Blocks: {}x{}", block_x, block_y);
    if new_width > original_image.get_size().width as u32 {
        unsafe {
            resize_scaleup_kernel::launch_unchecked::<f32, T>(
                &client,
                CubeCount::Static(thread_x as u32, thread_y as u32, 1),
                CubeDim::new(block_x as u32, block_y as u32, 1),
                ArrayArg::from_raw_parts::<f32>(
                    &input_handle,
                    pixel_count, 
                    4
                ),
                ScalarArg { elem: original_image.get_size().width as u32 },
                ScalarArg { elem: original_image.get_size().height as u32 },
                ScalarArg { elem: new_width },
                ScalarArg { elem: new_height },
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
                CubeCount::Static(thread_x as u32, thread_y as u32, 1),
                CubeDim::new(block_x as u32, block_y as u32, 1),
                ArrayArg::from_raw_parts::<f32>(
                    &input_handle,
                    pixel_count, 
                    4
                ),
                ScalarArg { elem: original_image.get_size().width as u32 },
                ScalarArg { elem: original_image.get_size().height as u32 },
                ScalarArg { elem: new_width },
                ScalarArg { elem: new_height },
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