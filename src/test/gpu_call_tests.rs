use std::time::Instant;

use cubecl::wgpu::{Dx12, Vulkan};

use crate::{model::{skid_color::SKIDColor, skid_image::{SKIDImage, SKIDSizeVector2}}, processor::{example_generator, make_normal_map, resize_image::{self, resize_image}}, utils::{file_io, gpu_opt}};



#[test]
fn gpu_call_tests() {
    let example_image = SKIDImage::new_with_color(2560, 1440,SKIDColor::new(0.1, 0.5, 0.5, 1.0));
    
    let _result_image = gpu_opt::launch::<cubecl::cuda::CudaRuntime>(
        &Default::default(),
        example_image,
    );
    println!("Result image: {:?}", _result_image.get_size());

}

#[test]
fn gpu_normap_tests() {
    
    let example_image = file_io::import_from_png("output/test_input.png", Some(4))
        .expect("Failed to load image from file");
    println!("Loaded image: {:?}", example_image.get_size());
    let device = cubecl::wgpu::WgpuDevice::DiscreteGpu(0);
    
    cubecl::wgpu::init_setup::<Vulkan>(
        &device, cubecl::wgpu::RuntimeOptions{
            ..Default::default()
        }
    );

    let result_image = make_normal_map::make_normal_map_base::<cubecl::wgpu::WgpuRuntime>(
        device,
        &example_image,
        Some(1.),
        Some(1.),
    );

    file_io::export_to_png(
        &result_image,
        "output/normal_map_output2.png",
        Some(8),
    ).expect("Failed to export normal map image");
    println!("Result image: {:?}", result_image.get_size());
    println!("Normal map first pixel color: {:?}", result_image.get_pixel(0, 0));

}

#[test]
fn gpu_upscale_tests() {
    let example_image = file_io::import_from_png("output/test_input.png", Some(4))
        .expect("Failed to load image from file");
    println!("Loaded image: {:?}", example_image.get_size());
    
    let device = cubecl::wgpu::WgpuDevice::DiscreteGpu(0);
    
    cubecl::wgpu::init_setup::<Vulkan>(
        &device, cubecl::wgpu::RuntimeOptions{
            ..Default::default()
        }
    );

    let result_image = resize_image::resize_image::<cubecl::wgpu::WgpuRuntime>(
        &device,
        &example_image,
        SKIDSizeVector2 {
            width: 5120,
            height: 2880,
        },
        Some(4),
    );

    file_io::export_to_png(
        &result_image,
        "output/resize_output2.png",
        Some(8),
    ).expect("Failed to export resized image");
    println!("Result image: {:?}", result_image.get_size());
}

#[test]
fn gpu_example_generator_tests() {

    let start = Instant::now();
    let _result_image = example_generator::launch::<cubecl::wgpu::WgpuRuntime>(
        &Default::default(),
        SKIDSizeVector2 {
            width: 5120,
            height: 2880,

        },
        Some(2),
    );
    let duration = start.elapsed();
    println!("Example generator took: {:?}", duration);
    println!("Result image: {:?}", _result_image.get_size());
    let start_file = Instant::now();
    file_io::export_rgba_channels_to_png(
        &_result_image,
        "output/example_generator_output",
        // Some(8),
    ).expect("Failed to export example generator image");
    let duration_file = start_file.elapsed();
    println!("File export took: {:?}", duration_file);
}