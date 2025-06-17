use crate::model::skid_image::{SKIDImage, SKIDSizeVector2};
#[no_mangle]
pub unsafe extern "C" fn skid_image_new(width: usize, height: usize) -> Box<SKIDImage> {
    Box::new(SKIDImage::new(width, height))
}


#[no_mangle]
pub extern "C" fn skid_image_get_size(handle: Box<SKIDImage> ) -> SKIDSizeVector2 {
    handle.get_size()
}