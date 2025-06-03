use crate::{model::{skid_color::SKIDColor, skid_image::SKIDImage}, utils::gpu_opt};



#[test]
fn gpu_call_tests() {
    let example_image = SKIDImage::new_with_color(2560, 1440,SKIDColor::new(0.1, 0.5, 0.5, 1.0));
    
    let _result_image = gpu_opt::launch::<cubecl::cuda::CudaRuntime>(
        &Default::default(),
        example_image,
    );
    println!("Result image: {:?}", _result_image.get_size());

}