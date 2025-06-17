use cubecl::{cube, prelude::Float, terminate};
#[cube]
pub fn compute_grayscale<F:Float>(r: F, g: F, b: F) -> F {
    F::new(0.299) * r 
    + F::new(0.587) * g 
    + F::new(0.114) * b
}
#[cube]
pub fn compute_luminance<F:Float>(r: F, g: F, b: F) -> F {
    F::new(0.2126) * r 
    + F::new(0.7152) * g 
    + F::new(0.0722) * b
}
#[cube]
pub fn normalize<F:Float>(value: F, min: F, max: F) -> F {
    if max - min == F::new(0.) {
        terminate!(F::new(0.)); //0.0; // Avoid division by zero
    }
    (value - min) / (max - min)
}
#[cube]
pub fn denormalize<F:Float>(value: F, min: F, max: F) -> F {
    if max - min == F::new(0.) {
        terminate!(0.0); //) 0.0; // Avoid division by zero
    }
    value * (max - min) + min
}

#[cube]
pub fn normal_vector_size<F:Float>(v:F,min:F,max:F) -> F {
    let mid = (max + min) / F::new(2.);
    v * mid + mid
}