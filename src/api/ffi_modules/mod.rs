use crate::model::skid_image::SKIDImage;

#[repr(C)]
pub enum ImageOptsTag {
    MakeNormalMap,
    MakeHeightMap,
    MakeNormalMapFromHeightMap,
    MakeHeightMapFromNormalMap,
    MakeNormalMapFromHeightMapWithScale,
    Blend,
    BlendAdd,
    BlendSubtract,
    BlendMultiply,
    BlendDivide,
}

#[repr(C)]
pub struct BlendArgs {
    pub img1: *mut SKIDImage,
    pub img2: *mut SKIDImage,
    pub value: f32,
}

#[repr(C)]
pub struct ImageOptArgs {
    pub img: *mut SKIDImage,
    pub value: f32,
}

#[repr(C)]
pub union ImageOptsData {
    pub img: *mut SKIDImage,
    pub blend: std::mem::ManuallyDrop<BlendArgs>,
    pub img_with_value: std::mem::ManuallyDrop<ImageOptArgs>,
}

#[repr(C)]
pub struct ImageOptsFFI {
    pub tag: ImageOptsTag,
    pub data: ImageOptsData,
}