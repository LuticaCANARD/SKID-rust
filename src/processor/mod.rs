pub mod image_sync_action;
pub mod make_normal_map;
pub mod resize_image;
pub mod example_generator;


pub enum ProcessorError {
    ImageSyncError(String),
    // Add other error variants as needed
}