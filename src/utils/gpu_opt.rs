use cubecl::{comptime, cube, prelude::{Array, ArrayArg, Erf, Float, Line, ScalarArg, Sequence, ABSOLUTE_POS}, Runtime};

use crate::model::{skid_color::SKIDColor, skid_image::SKIDImage};
use cubecl::frontend::CompilationArg;
use cubecl::prelude::FloatExpand;
#[cube(launch_unchecked)]
fn action<F:Float>(
    input:&Array<Line<F>>, 
    input_value:F,
    output:&mut Array<Line<F>>
) {
    if ABSOLUTE_POS < input.len() {

        output[ABSOLUTE_POS] = action_add_const(input[ABSOLUTE_POS],
            input_value
        );
    }
}

#[cube]
fn action_add_const<F: Float>(x: Line<F>,constant:F) -> Line<F> {
    let line = Line::<F>::new(constant);
    x + line
}
/**
 * pub fn launch<R: Runtime>(device: &R::Device) {
    let client = R::client(device);
    let input = &[-1., 0., 1., 5.];
    let vectorization = 4;
    let output_handle = client.empty(input.len() * core::mem::size_of::<f32>());
    let input_handle = client.create(f32::as_bytes(input));

    unsafe {
        gelu_array::launch_unchecked::<f32, R>(
            &client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new(input.len() as u32 / vectorization, 1, 1),
            ArrayArg::from_raw_parts::<f32>(&input_handle, input.len(), vectorization as u8),
            ArrayArg::from_raw_parts::<f32>(&output_handle, input.len(), vectorization as u8),
        )
    };

    let bytes = client.read_one(output_handle.binding());
    let output = f32::from_bytes(&bytes);

    // Should be [-0.1587,  0.0000,  0.8413,  5.0000]
    println!("Executed gelu with runtime {:?} => {output:?}", R::name());
}
 */
pub fn launch_apu_action_for_add_const<R:Runtime>(
    device: &R::Device,
    input_image: SKIDImage,
    input_value: f32
) -> SKIDImage {
    use crate::utils::gpu_opt::action::launch_unchecked;
    let client = R::client(device);
    let input_data = input_image.to_byte_array();
    let output_handle = client.empty(input_image.get_u8_byte_len()); 
    let input_handle = client.create(&input_data);
    let image_size = input_image.get_size();
    let height = image_size.height as u32;
    let width = image_size.width as u32;
    let input_len = input_image.len();
    unsafe {
        launch_unchecked::<f32, R>(
            &client,
            cubecl::CubeCount::Static(1, 1, 1),
            cubecl::CubeDim::new(width, height, 1),
            ArrayArg::from_raw_parts::<u32>(
                &input_handle, 
                input_len.clone(), 
                SKIDColor::SKID_U8_ARRAY_RESOLUTION as u8
            ),
            ScalarArg::from(cubecl::frontend::ScalarArg { elem: input_value }),
            ArrayArg::from_raw_parts::<u32>(
                &output_handle, 
                input_len.clone(), 
                SKIDColor::SKID_U8_ARRAY_RESOLUTION as u8
            ),
            
        )
    };

    let data = client.read_one(output_handle.binding());
    let data = data.chunks_exact(SKIDColor::SKID_U8_ARRAY_BYTE_SIZE_TOTAL)
        .map(|chunk| 
            SKIDColor::from_u8_array(chunk.try_into().unwrap())
        )
        .collect::<Vec<SKIDColor>>();
    SKIDImage::from_1d_data(image_size.clone(), data)
}

