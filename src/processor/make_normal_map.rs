use std::sync::{Arc, Mutex};

use cubecl::{cube, frontend::CompilationArg, prelude::{Array, ArrayArg, Float, FloatExpand, ScalarArg, UNIT_POS_X, UNIT_POS_Y}, CubeCount, CubeDim, CubeElement, Runtime};
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
    let position_x = UNIT_POS_X;
    let position_y = UNIT_POS_Y;

    let x_left = if position_x > 0 { position_x - 1 } else { (0 as u32).into() };
    let x_right = if position_x < width - 1 { position_x + 1 } else { width - 1 };
    let y_top = if position_y > 0 { position_y - 1 } else { (0 as u32).into() };
    let y_bottom = if position_y < height - 1 { position_y + 1 } else { height - 1 };

    let left_r = input[x_left][position_y][0];
    let left_g = input[x_left][position_y][1];
    let left_b = input[x_left][position_y][2];
    let left_delta = compute_grayscale::<F>(left_r, left_g, left_b);

    let right_r = input[x_right][position_y][0];
    let right_g = input[x_right][position_y][1];
    let right_b = input[x_right][position_y][2];
    let right_delta = compute_grayscale::<F>(right_r, right_g, right_b);

    let top_r = input[position_x][y_top][0];
    let top_g = input[position_x][y_top][1];
    let top_b = input[position_x][y_top][2];
    let top_delta = compute_grayscale::<F>(top_r, top_g, top_b);

    let bottom_r = input[position_x][y_bottom][0];
    let bottom_g = input[position_x][y_bottom][1];
    let bottom_b = input[position_x][y_bottom][2];
    let bottom_delta = compute_grayscale::<F>(bottom_r, bottom_g, bottom_b);

    let dx = (right_delta - left_delta) * x_factor;
    let dy = (bottom_delta - top_delta) * y_factor;

    let normal_x = dx / F::sqrt(dx * dx + dy * dy);
    let normal_y = dy / F::sqrt(dx * dx + dy * dy);
    let normal_z = 1.0;
    let min_scale = F::new(0.);
    let max_scale = F::new(1.);

    let n_r = normal_vector_size::<F>(normal_x, min_scale, max_scale);
    let n_g = normal_vector_size::<F>(normal_y, min_scale, max_scale);
    let n_b = normal_vector_size::<F>(F::new(normal_z), min_scale, max_scale);
    
    let n_a = F::new(1.0);
    let idx = (position_y * width + position_x);
    output[idx * 4 + 0] = n_r;
    output[idx * 4 + 1] = n_g;
    output[idx * 4 + 2] = n_b;
    output[idx * 4 + 3] = n_a;
}

pub fn make_normal_map_base<R:Runtime>(
    runtime: R::Device,
    original_image: &SKIDImage,
) -> SKIDImage {
    launch::<R>(
        &runtime,
        original_image
    )
}


fn launch<T: Runtime>(
    run_device: &T::Device,
    original_image: &SKIDImage,
) -> SKIDImage {
    let client = T::client(run_device);

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
    let pixel_count = input.len() / 4; // Assuming each color has 4 components (RGBA)

    unsafe{
        kernel_make_normal_map::launch_unchecked::<f32, T>(
            &client,
            CubeCount::Static(threads_x as u32, threads_y as u32, 1),
            CubeDim::new(block_x as u32, block_y as u32, 4),
            ArrayArg::from_raw_parts::<f32>(&input_handle, pixel_count, 1),
            ScalarArg::from(cubecl::frontend::ScalarArg { elem: w_u32 }),
            ScalarArg::from(cubecl::frontend::ScalarArg { elem: h_u32 }),
            ScalarArg::from(cubecl::frontend::ScalarArg { elem:0.5 }),
            ScalarArg::from(cubecl::frontend::ScalarArg { elem:0.5 }),
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
