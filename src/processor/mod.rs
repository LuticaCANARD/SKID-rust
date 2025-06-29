pub mod image_sync_action;
pub mod make_normal_map;
pub mod resize_image;
pub mod example_generator;
pub mod image_synthesis_action;
pub mod image_rotation_action;


pub enum ProcessorError {
    ImageSyncError(String),
    // Add other error variants as needed
}