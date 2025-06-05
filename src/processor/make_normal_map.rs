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


            let n_r = input[idx][0]; // R
            let n_g = input[idx][1]; // G
            let n_b = input[idx][2]; // B
            output[idx*4+0] = n_r; // R
            output[idx*4+1] = n_g; // G
            output[idx*4+2] = n_b; // B
            output[idx*4+3] = input[idx][3]; // A
        }
    }
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
    let pixel_count = input.len() / 4 ; // Assuming each color has 4 components (RGBA)
    
    unsafe{
        kernel_make_normal_map::launch_unchecked::<f32, T>(
            &client,
            CubeCount::Static(threads_x as u32, threads_y as u32, 1),
            CubeDim::new(block_x as u32, block_y as u32, 1),
            ArrayArg::from_raw_parts::<f32>(&input_handle, pixel_count, 4),
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
