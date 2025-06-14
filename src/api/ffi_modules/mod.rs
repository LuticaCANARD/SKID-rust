#[cfg(feature = "use_cuda")]
use cubecl::Runtime;

use crate::model::skid_image::SKIDImage;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct CalcDevice {
    pub(crate) device_id: u32,
    pub(crate) device_name: &'static str
}

// #[no_mangle]
// extern "C" fn skid_get_calc_device() -> Vec<CalcDevice> {
//     use cubecl::wgpu::{WgpuDevice, WgpuRuntime};
    
// }

#[repr(C)]
struct NormalMapOptions {
    pub(crate) x_factor: Option<f32>,
    pub(crate) y_factor: Option<f32>,
    pub(crate) make_by_gpu: bool,
    pub(crate) gpu_option: Option<CalcDevice>,
}
#[no_mangle]
extern "C" fn skid_generate_normal_map(
    input_image: SKIDImage,
    options: NormalMapOptions,
) -> SKIDImage {
    let x_factor = options.x_factor;
    let y_factor: Option<f32> = options.y_factor;
    let make_by_gpu = options.make_by_gpu;

    if let Some(gpu) = options.gpu_option {
        // Use GPU processing if available and requested
        #[cfg(feature = "use_wgpu")]
        {
            use cubecl::wgpu::{WgpuDevice, WgpuRuntime};
            let gpu_device = WgpuDevice::IntegratedGpu(gpu.device_id as usize); // Assuming the device_id corresponds to an integrated GPU
            return crate::processor::make_normal_map::make_normal_map_base::<WgpuRuntime>(
                gpu_device,
                &input_image,
                x_factor,
                y_factor,
            )
        }
        #[cfg(feature = "use_cuda")]
        {
            use cubecl::cuda::{CudaDevice, CudaRuntime};
            let gpu_device = CudaDevice::new(gpu.device_id as usize);
            return crate::processor::make_normal_map::make_normal_map_base::<CudaRuntime>(
                gpu_device,
                &input_image,
                x_factor,
                y_factor,
            )
        }
        #[cfg(feature = "use_hip")]
        {
            use cubecl::hip::{HipDevice, HipRuntime};
            let gpu_device = HipDevice::new(gpu.device_id as usize);
            return crate::processor::make_normal_map::make_normal_map_base::<HipRuntime>(
                gpu_device,
                &input_image,
                x_factor,
                y_factor,
            )
        }
        
    } else {
        use cubecl::wgpu::{WgpuDevice, WgpuRuntime};

        // Default to CPU processing if no GPU is specified
        let cpu_device = WgpuDevice::Cpu; // Assuming 0 is the CPU device ID
        return crate::processor::make_normal_map::make_normal_map_base::<WgpuRuntime>(
            cpu_device,
            &input_image,
            x_factor,
            y_factor,
        )

    }
}

