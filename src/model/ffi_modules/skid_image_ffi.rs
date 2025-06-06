use crate::model::skid_image::{SKIDImage, SKIDSizeVector2};

#[repr(C)]
pub struct SKIDImageHandle {
    pub(crate) ptr: *mut SKIDImage,
}

#[no_mangle]
pub unsafe extern "C" fn skid_image_new(width: usize, height: usize) -> SKIDImageHandle {
    let boxed = Box::new(SKIDImage::new(width, height));
    SKIDImageHandle {
        ptr: Box::into_raw(boxed),
    }
}
#[no_mangle]
pub extern "C" fn skid_image_free(handle: SKIDImageHandle) {
    if !handle.ptr.is_null() {
        unsafe { let _ = Box::from_raw(handle.ptr); }
    }
}

#[no_mangle]
pub extern "C" fn skid_image_get_size(handle: SKIDImageHandle) -> SKIDSizeVector2 {
    if handle.ptr.is_null() {
        panic!("SKIDImageHandle is null");
    }
    unsafe { &*handle.ptr }.get_size()
}


