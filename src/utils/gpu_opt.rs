use cubecl::prelude::*;

use crate::model::{skid_color::SKIDColor, skid_image::SKIDImage};

#[cube(launch_unchecked)]
fn norm_test<F: Float>(input: &Array<F>, output_a: &mut Array<F>) {
    if ABSOLUTE_POS < input.len() {
        output_a[ABSOLUTE_POS] = F::normalize(input[ABSOLUTE_POS]);
    }
}

pub fn launch<R: Runtime>(
    device: &R::Device,
    image_input: SKIDImage
) -> SKIDImage {
    let client = R::client(device);
    let input = image_input.get_1d_data_as_f32();
    let input_handle = client.create(bytemuck::cast_slice(&input));
    let pixel_count = input.len() / 4; // Assuming each color has 4 components (RGBA)

    let width = image_input.get_size().width;
    let w_u32 = width as u32;
    let height = image_input.get_size().height;
    let h_u32 = height as u32;
    let output_a_handle = client.empty(input.len() * core::mem::size_of::<f32>());
    let (max_thread_x, max_thread_y, _max_thread_z) = R::max_cube_count();
    let block_x = (w_u32 + max_thread_x - 1) / max_thread_x;
    let threads_x = if w_u32 < max_thread_x { w_u32  } else { max_thread_x };

    let block_y = (h_u32 + max_thread_y - 1) / max_thread_y;
    let threads_y = if h_u32 < max_thread_y { h_u32 } else { max_thread_y };

    println!("Launching normalize with runtime: {}", input.len());
    unsafe {
        norm_test::launch_unchecked::<f32, R>(
            &client,
            CubeCount::Static(threads_x as u32, threads_y as u32, 1),
            CubeDim::new(block_x as u32, block_y as u32, 1),
            ArrayArg::from_raw_parts::<f32>(&input_handle, pixel_count, 4),
            ArrayArg::from_raw_parts::<f32>(&output_a_handle, pixel_count, 4),
        )
    };

    let bytes = client.read_one(output_a_handle.binding());
    let output = f32::from_bytes(&bytes);

    let output_colors: Vec<SKIDColor> = output.chunks(4)
        .map(|chunk| SKIDColor::from_f32_array(chunk.try_into().unwrap()))
        .collect();
    SKIDImage::from_1d_data(
        image_input.get_size(), 
        output_colors
    )
}