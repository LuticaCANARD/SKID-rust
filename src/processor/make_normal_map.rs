use std::sync::{Arc, Mutex};

use cubecl::{cube, frontend::CompilationArg, prelude::{index, le, Array, ArrayArg, Float, FloatExpand, ScalarArg, ABSOLUTE_POS, ABSOLUTE_POS_X, ABSOLUTE_POS_Y, CUBE_CLUSTER_DIM_X, CUBE_CLUSTER_DIM_Y, CUBE_COUNT_Y, UNIT_POS_PLANE, UNIT_POS_X, UNIT_POS_Y}, terminate, CubeCount, CubeDim, CubeElement, Runtime};
use crate::{model::{skid_color::SKIDColor, skid_image::SKIDImage}, utils::graphic_fn::{compute_grayscale, normal_vector_size}};

#[cube(launch_unchecked)]
fn kernel_make_normal_map<F: Float>(
    input: &Array<F>,
    width: u32,
    height: u32,
    x_factor: F,
    y_factor: F,
    output: &mut Array<F>,
) {
    for x in 0..CUBE_CLUSTER_DIM_X {
        let px = ABSOLUTE_POS_X + x;
        for y in 0..CUBE_CLUSTER_DIM_Y {
            let py = ABSOLUTE_POS_Y + y;
            let idx = py * width + px;

            let up_y = if py > 0 { py - 1 } else { height - 1 };
            let up_r = input[up_y* width + px][0]; // R
            let up_g = input[up_y* width + px][1]; // G
            let up_b = input[up_y* width + px][2]; // B
            let d_up = compute_grayscale::<F>(up_r, up_g, up_b);

            let down_y = if py < height - 1 { py + 1 } else { 0u32.into() };
            let down_r = input[down_y * width + px][0]; // R
            let down_g = input[down_y * width + px][1]; // G
            let down_b = input[down_y * width + px][2]; // B
            let d_down = compute_grayscale::<F>(down_r, down_g, down_b);

            let left_x = if px > 0 { px - 1 } else { width - 1 };
            let left_r = input[py * width + left_x][0]; // R
            let left_g = input[py * width + left_x][1]; // G
            let left_b = input[py * width + left_x][2]; // B
            let d_left = compute_grayscale::<F>(left_r, left_g, left_b);

            let right_x = if px < width - 1 { px + 1 } else { 0u32.into() };
            let right_r = input[py * width + right_x][0]; // R
            let right_g = input[py * width + right_x][1]; // G
            let right_b = input[py * width + right_x][2]; // B
            let d_right = compute_grayscale::<F>(right_r, right_g, right_b);

            let d_x = (d_right - d_left) * x_factor;
            let d_y = (d_down - d_up) * y_factor;
            let d_z = F::new(1.0); // Assuming a constant depth value of 1.0

            let normal_x = d_x;
            let normal_y = d_y;
            let normal_z = d_z;

            let normal_length = F::sqrt(normal_x * normal_x + normal_y * normal_y + normal_z * normal_z);
            if normal_length == F::new(0.0) {
                terminate!(F::new(0.0)); // Avoid division by zero
            }
            let normalized_x = normal_x / normal_length;
            let normalized_y = normal_y / normal_length;
            let normalized_z = normal_z / normal_length;

            let n_r = normal_vector_size::<F>(normalized_x , F::new(-0.), F::new(1.0));
            let n_g = normal_vector_size::<F>(normalized_y , F::new(-0.), F::new(1.0));
            let n_b = normal_vector_size::<F>(normalized_z , F::new(-0.), F::new(1.0));
            output[idx*4+0] = n_r; // R
            output[idx*4+1] = n_g; // G
            output[idx*4+2] = n_b; // B
            output[idx*4+3] = F::new(1.0); // A
        }
    }
}

pub fn make_normal_map_base<R:Runtime>(
    runtime: R::Device,
    original_image: &SKIDImage,
    x_factor: Option<f32>,
    y_factor: Option<f32>,
) -> SKIDImage {
    launch::<R>(
        &runtime,
        original_image,
        x_factor,
        y_factor,
    )
}


fn launch<T: Runtime>(
    run_device: &T::Device,
    original_image: &SKIDImage,
    x_factor: Option<f32>,
    y_factor: Option<f32>,
) -> SKIDImage {
    let client = T::client(run_device);


    let x_factor = x_factor.unwrap_or(0.5);
    let y_factor = y_factor.unwrap_or(0.5);
    let w_size = original_image.get_size().width;
    let h_size = original_image.get_size().height;
    let w_u32 = w_size as u32;
    let h_u32 = h_size as u32;
    let (x_count, y_count, _z_count) = T::max_cube_count();
    let block_x = (w_u32 + x_count - 1) / x_count;
    let threads_x = if w_u32 < x_count { w_u32 } else { x_count };
    let block_y = (h_u32 + y_count - 1) / y_count;
    let threads_y = if h_u32 < y_count { h_u32 } else { y_count };


    let input = original_image.get_1d_data_as_f32();
    let input_handle = client.create(bytemuck::cast_slice(&input));

    let output_handle = client.empty(input.len() * core::mem::size_of::<f32>());
    let pixel_count = input.len() / 4 ; // Assuming each color has 4 components (RGBA)
    
    unsafe{
        kernel_make_normal_map::launch_unchecked::<f32, T>(
            &client,
            CubeCount::Static(threads_x as u32, threads_y as u32, 1),
            CubeDim::new(block_x as u32, block_y as u32, 1),
            ArrayArg::from_raw_parts::<f32>(&input_handle, pixel_count, 4),
            ScalarArg::from(cubecl::frontend::ScalarArg { elem: w_u32 }),
            ScalarArg::from(cubecl::frontend::ScalarArg { elem: h_u32 }),
            ScalarArg::from(cubecl::frontend::ScalarArg { elem:x_factor }),
            ScalarArg::from(cubecl::frontend::ScalarArg { elem:y_factor }),
            ArrayArg::from_raw_parts::<f32>(&output_handle, pixel_count, 1),
        )
    };
    let bytes = client.read_one(output_handle.binding());
    let output = f32::from_bytes(&bytes);
    let output_colors: Vec<SKIDColor> = output.chunks(4)
        .map(|chunk| SKIDColor::from_f32_array(chunk.try_into().unwrap()))
        .collect();
    SKIDImage::from_1d_data(
        original_image.get_size(), 
        output_colors
    )
}
