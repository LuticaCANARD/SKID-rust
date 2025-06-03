use crate::model::skid_image::SKIDImage;


// SKIDImage -> SKIDImage
pub enum ImageOpts{
    MakeNormalMap(SKIDImage),
    MakeHeightMap(SKIDImage),
    MakeNormalMapFromHeightMap(SKIDImage),
    MakeHeightMapFromNormalMap(SKIDImage),
    MakeNormalMapFromHeightMapWithScale(SKIDImage, f32),
    Blend(SKIDImage,SKIDImage, f32),
    BlendAdd(SKIDImage,SKIDImage, f32),
    BlendSubtract(SKIDImage,SKIDImage, f32),
    BlendMultiply(SKIDImage,SKIDImage, f32),
    BlendDivide(SKIDImage,SKIDImage, f32),
}
