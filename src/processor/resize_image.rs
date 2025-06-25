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
fn bilinear_interpolate<F: Float>(
    input: &Array<F>,
    x: u32,
    y: u32,
    x_lerp: u32,
    y_lerp: u32,
    output: &mut Array<F>,
) {
    let width = input.len() as u32 / 4; // Assuming input is a
    let height = input.len() as u32 / (width * 4);
    let x0 = x;
    let y0 = y;
    let x1 = if x + 1 < width { x + 1 } else { x };
    let y1 = if y + 1 < height { y + 1 } else { y };
    let x0_idx = (y0 * width + x0);
    let x1_idx = (y0 * width + x1);
    let y0_idx = (y1 * width + x0);
    let y1_idx = (y1 * width + x1);
    let x0_r = input[x0_idx][0];
    let x0_g = input[x0_idx][1];
    let x0_b = input[x0_idx][2];
    let x0_a = input[x0_idx][3];
    let x1_r = input[x1_idx][0];
    let x1_g = input[x1_idx][1];
    let x1_b = input[x1_idx][2];
    let x1_a = input[x1_idx][3];
    let y0_r = input[y0_idx][0];
    let y0_g = input[y0_idx][1];
    let y0_b = input[y0_idx][2];
    let y0_a = input[y0_idx][3];
    let y1_r = input[y1_idx][0];
    let y1_g = input[y1_idx][1];
    let y1_b = input[y1_idx][2];
    let y1_a = input[y1_idx][3];
    let x_lerp_f = F::cast_from(x_lerp) / F::cast_from(width);
    let y_lerp_f = F::cast_from(y_lerp) / F::cast_from(height);
    let r = x0_r * (F::new(1.0) - x_lerp_f) * (F::new(1.0) - y_lerp_f)
        + x1_r * x_lerp_f * (F::new(1.0) - y_lerp_f)
        + y0_r * (F::new(1.0) - x_lerp_f) * y_lerp_f
        + y1_r * x_lerp_f * y_lerp_f;
    let g = x0_g * (F::new(1.0) - x_lerp_f) * (F::new(1.0) - y_lerp_f)
        + x1_g * x_lerp_f * (F::new(1.0) - y_lerp_f)
        + y0_g * (F::new(1.0) - x_lerp_f) * y_lerp_f
        + y1_g * x_lerp_f * y_lerp_f;
    let b = x0_b * (F::new(1.0) - x_lerp_f) * (F::new(1.0) - y_lerp_f)
        + x1_b * x_lerp_f * (F::new(1.0) - y_lerp_f)
        + y0_b * (F::new(1.0) - x_lerp_f) * y_lerp_f
        + y1_b * x_lerp_f * y_lerp_f;
    let a = x0_a * (F::new(1.0) - x_lerp_f) * (F::new(1.0) - y_lerp_f)
        + x1_a * x_lerp_f * (F::new(1.0) - y_lerp_f)
        + y0_a * (F::new(1.0) - x_lerp_f) * y_lerp_f
        + y1_a * x_lerp_f * y_lerp_f;
    output[(y_lerp * width + x_lerp) * 4] = r;
    output[(y_lerp * width + x_lerp) * 4 + 1] = g;
    output[(y_lerp * width + x_lerp) * 4 + 2] = b;
    output[(y_lerp * width + x_lerp) * 4 + 3] = a;
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
    let width_scale = new_width as f32 / width as f32;
    let height_scale = new_height as f32 / height as f32; 
    for x in 0..CUBE_CLUSTER_DIM_X {
        let px = ABSOLUTE_POS_X + x; // 자기 순회 x좌표...
        for y in 0..CUBE_CLUSTER_DIM_Y {
            let py = ABSOLUTE_POS_Y + y; // 자기 순회 y좌표...

            let output_index = (py * new_width + px) * 4; // 출력 이미지에서의 인덱스

            for x_delta in -1..=1 {
                for y_delta in -1..=1 {
                    // Ensure we don't go out of bounds
                    let x_lerp = ((px as f32 + x_delta as f32) * width_scale) as u32;
                    let y_lerp = ((py as f32 + y_delta as f32) * height_scale) as u32;
                    // let input_index = (px * width + py);
                    // Perform bilinear interpolation
                    bilinear_interpolate::<F>(
                        input,
                        px + x_delta as u32,
                        py + y_delta as u32,
                        x_lerp,
                        y_lerp,
                        output
                    );
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
    let block_x = (original_image.get_size().width as u32) / thread_x;
    let block_y = (original_image.get_size().height as u32) / thread_y;

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