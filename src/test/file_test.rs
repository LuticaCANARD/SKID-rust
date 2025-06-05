use crate::model::skid_color::SKIDColor;
use crate::model::skid_image::SKIDImage;
use crate::utils::file_io::export_to_png;
use crate::utils::{self, gpu_opt};
use std::fs;

const TEST_OUTPUT: &str = "output/test_output.png";

#[test]
pub fn test_file_write() {

    // Create a dummy SKIDImage
    let image = SKIDImage::new_with_color(
        2560, 1440, 
        SKIDColor::new(0., 0.1, 0.9, 1.0)
    );


    let new_image = gpu_opt::launch::<cubecl::cuda::CudaRuntime>(
        &Default::default(),
        image,
    );

    
    // Define the file path
    let file_path = TEST_OUTPUT;

    // Export the image to PNG
    let result = export_to_png(&new_image, file_path,Some(8));
    assert!(result.is_ok(), "Failed to export image: {:?}", result);
}

#[test]
pub fn test_file_read() {

    // Define the file path
    let file_path = TEST_OUTPUT;

    // Check if the file exists
    let exists = fs::metadata(file_path).is_ok();
    assert!(exists, "File does not exist: {}", file_path);

    // Load the image from the file
    let loaded_image = utils::file_io::import_from_png(file_path,None);

    assert!(loaded_image.is_ok(), "Failed to load image from file: {:?}", loaded_image.err());
    let image = loaded_image.unwrap();
    assert_eq!(image.get_size().width, 2560, "Image width does not match expected value");
    assert_eq!(image.get_size().height, 1440, "Image height does not match expected value");
    println!("Image loaded successfully with size: {}x{}", image.get_size().width, image.get_size().height);
}