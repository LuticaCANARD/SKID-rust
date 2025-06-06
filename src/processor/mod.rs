pub mod image_sync_action;
pub mod make_normal_map;
pub mod resize_image;


pub enum ProcessorError {
    ImageSyncError(String),
    // Add other error variants as needed
}