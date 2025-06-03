use crate::model::skid_image::SKIDImage;


pub fn make_normal_map_base(
    original_image: &SKIDImage,
) -> SKIDImage {
    // Placeholder for the actual implementation
    // This function should create a normal map from the original image
    // For now, we will just return a new SKIDImage with the same dimensions
    let size = original_image.get_size();
    SKIDImage::new(size.width, size.height)
    
}

