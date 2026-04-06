#[cfg(feature = "use_cuda")]
use cubecl::Runtime;

use crate::model::{skid_image::{SKIDImage, SKIDSizeVector2}, skid_color::SKIDColor};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Mutex, RwLock};

#[cfg(feature = "use_wgpu")]
use cubecl::wgpu::WgpuDevice;
#[cfg(feature = "use_cuda")]
use cubecl::cuda::CudaDevice;

// ─── GPU 디바이스 싱글턴 ───
// WgpuDevice 자체는 열거형 변수일 뿐이지만, CubeCL 내부의 ComputeRuntime이
// client()를 통해 디바이스별 클라이언트를 캐싱한다.
// 싱글턴으로 관리하여 동일 디바이스가 항상 동일 캐시 키로 조회되도록 보장.
#[cfg(feature = "use_wgpu")]
pub static DEFAULT_WGPU_DEVICE: Lazy<WgpuDevice> = Lazy::new(WgpuDevice::default);

#[cfg(feature = "use_cuda")]
pub static DEFAULT_CUDA_DEVICE: Lazy<CudaDevice> = Lazy::new(CudaDevice::default);

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

// lib.rs 또는 ffi.rs


use crate::processor;

// SKIDImage 인스턴스를 저장할 전역 핸들 관리자 (JNI 모듈과 공유)
pub static IMAGE_HANDLES: Lazy<RwLock<HashMap<u64, Box<SKIDImage>>>> = Lazy::new(Default::default);
// 고유 핸들 ID를 생성하기 위한 카운터
static LAST_HANDLE_ID: Lazy<Mutex<u64>> = Lazy::new(|| Mutex::new(0));

// 새 핸들 ID를 생성하는 헬퍼 함수 (JNI 모듈과 공유)
pub fn new_handle_id() -> u64 {
    let mut id = LAST_HANDLE_ID.lock().unwrap();
    *id += 1;
    *id
}

/// C#에서 float[] 배열을 받아 SKIDImage를 생성하고 핸들을 반환합니다.
#[no_mangle]
pub extern "C" fn skid_image_create_from_f32_array(
    bytes: *const f32,
    width: usize,
    height: usize,
) -> u64 {
    if bytes.is_null() { return 0; }
    let data_slice = unsafe { std::slice::from_raw_parts(bytes, width * height * 4) };
    let colors: Vec<SKIDColor> = data_slice.chunks_exact(4)
        .map(|chunk| SKIDColor::from_f32_array(chunk.try_into().unwrap()))
        .collect();
    let image = SKIDImage::from_1d_data(SKIDSizeVector2 { width, height }, colors);

    let handle_id = new_handle_id();
    IMAGE_HANDLES.write().unwrap().insert(handle_id, Box::new(image));
    handle_id
}

/// 핸들을 사용하여 이미지의 메모리를 해제합니다. (매우 중요!)
#[no_mangle]
pub extern "C" fn skid_image_free(handle: u64) {
    if handle != 0 {
        IMAGE_HANDLES.write().unwrap().remove(&handle);
    }
}

/// 핸들을 사용하여 이미지 크기를 가져옵니다.
#[no_mangle]
pub extern "C" fn skid_image_get_size(handle: u64, out_size: *mut SKIDSizeVector2) -> i32 {
    if out_size.is_null() { return -1; }
    let handles = IMAGE_HANDLES.read().unwrap();
    if let Some(image) = handles.get(&handle) {
        unsafe { *out_size = image.get_size(); }
        0
    } else {
        -2 // Invalid handle
    }
}

/// 핸들을 사용하여 이미지 데이터를 C#의 float[] 배열로 복사합니다.
#[no_mangle]
pub extern "C" fn skid_image_get_data_as_f32_array(
    handle: u64,
    out_bytes: *mut f32,
    buffer_len: usize,
) -> i32 {
    if out_bytes.is_null() { return -1; }
    let handles = IMAGE_HANDLES.read().unwrap();
    if let Some(image) = handles.get(&handle) {
        let image_data = image.get_1d_data_as_f32();
        if image_data.len() > buffer_len {
            return -2; // Buffer too small
        }
        unsafe {
            std::ptr::copy_nonoverlapping(image_data.as_ptr(), out_bytes, image_data.len());
        }
        0 // Success
    } else {
        -3 // Invalid handle
    }
}

/// 이미지 리사이즈 함수 (핸들 기반)
///
/// 락 점유 최소화 패턴:
///   1. read lock → clone → drop lock  (락 점유: clone 비용만큼)
///   2. GPU 작업 수행                   (락 없음)
///   3. write lock → insert → drop lock (락 점유: HashMap insert만큼)
#[no_mangle]
pub extern "C" fn skid_image_resize(
    handle: u64,
    new_width: usize,
    new_height: usize,
) -> u64 {
    use cubecl::wgpu::WgpuRuntime;

    let device = &*DEFAULT_WGPU_DEVICE;

    // 1) 읽기 락: 이미지 clone 후 즉시 해제
    let image_clone = {
        let handles = IMAGE_HANDLES.read().unwrap();
        match handles.get(&handle) {
            Some(image) => image.clone(),
            None => return 0,
        }
        // handles(RwLockReadGuard) 여기서 drop
    };

    // 2) 락 없이 GPU 작업 수행
    let new_size = SKIDSizeVector2 { width: new_width, height: new_height };
    let resized_image = processor::resize_image::resize_image::<WgpuRuntime>(
        device,
        &image_clone,
        new_size,
        None,
    );

    // 3) 쓰기 락: 결과 저장 후 즉시 해제
    let new_handle = new_handle_id();
    IMAGE_HANDLES.write().unwrap().insert(new_handle, Box::new(resized_image));
    new_handle
}