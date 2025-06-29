use cubecl::{cube, prelude::*};

use crate::model::skid_image::{SKIDImage, SKIDSizeVector2};


pub fn launch_image_rotation(
    run_device: &impl Runtime,
    rotation_radius: f32,
    thread_count: Option<usize>,
) -> SKIDImage {
    // Implementation for launching image rotation
    todo!("Implement the launch logic for image rotation");
}



#[cube(launch_unchecked)]
fn kernel_image_rotation<F: Float>(
    input: Array<F>,
    width: u32,
    height: u32,
    rotation_radius: f32, 
    output: &mut Array<F>,
) {
    // Kernel implementation for image rotation
    let center_x = F::cast_from(width) / F::new(2.0);
    let center_y = F::cast_from(height) / F::new(2.0);
    let radius = F::cast_from(rotation_radius);
    let cos_angle = F::new(0.0);
    let sin_angle = F::new(0.0);
    let width_i = (width) as i32;
    let height_i = (height) as i32;

    // Perform the rotation

    for y in 0..height {
        let py = ABSOLUTE_POS_Y + y;
        for x in 0..width {
            let px = ABSOLUTE_POS_X + x;
            let src_x = F::cast_from(px) - center_x;
            let src_y = F::cast_from(py) - center_y;

            let rotated_x = src_x * cos_angle - src_y * sin_angle;
            let rotated_y = src_x * sin_angle + src_y * cos_angle;

            let cal_rotated_x = F::cast_from(rotated_x) + center_x;
            let cal_rotated_y = F::cast_from(rotated_y) + center_y;

            let cal_src_x = i32::cast_from(F::floor(cal_rotated_x));
            let cal_src_y = i32::cast_from(F::floor(cal_rotated_y));

            if cal_src_x >= 0i32 && cal_src_x < width_i && cal_src_y >= 0i32 && cal_src_y < height_i {
                let output_idx = (y * width + x) * 4; // Assuming each pixel has 4 components (RGBA)
                let input_idx = (cal_src_y * width_i + cal_src_x) as u32;
                
                let in_r = input[input_idx][0];
                let in_g = input[input_idx][1];
                let in_b = input[input_idx][2];
                let in_a = input[input_idx][3];
                output[output_idx] = in_r; // R
                output[output_idx + 1] = in_g; // G
                output[output_idx + 2] = in_b; // B
                output[output_idx + 3] = in_a; // A

            }
        }
    }
}