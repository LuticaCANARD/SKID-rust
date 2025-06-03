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
    let input: &Vec<Vec<SKIDColor>> = image_input.get_data();
    
    let input_flat: Vec<f32> = input.iter()
        .flat_map(|row| row.iter().flat_map(|color| color.to_f32_array()))
        .collect();
    let input_handle = client.create(bytemuck::cast_slice(&input_flat));
    let pixel_count = input_flat.len() / 4; // Assuming each color has 4 components (RGBA)

    let width = image_input.get_size().width;
    let height = image_input.get_size().height;
    let output_a_handle = client.empty(input_flat.len() * core::mem::size_of::<f32>());
    let handle_count = 1024; // Number of handles to use for the kernel
    let max_threads = R::max_cube_count();
    let block_x = (width + max_threads.0 as usize - 1) / max_threads.0 as usize;
    let threads_x = if width < max_threads.0 as usize { width } else { max_threads.0 as usize };

    let block_y = (height + max_threads.1 as usize - 1) / max_threads.1 as usize;
    let threads_y = if height < max_threads.1 as usize { height } else { max_threads.1 as usize };

    println!("Launching normalize with runtime: {}", input_flat.len());
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