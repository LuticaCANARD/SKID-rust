/// JNI(JVM Native Interface) 호환 FFI 모듈.
///
/// 기존 C# FFI와 동일한 핸들 기반 IMAGE_HANDLES를 공유하여,
/// C#과 JVM이 동일 프로세스 내에서 같은 이미지 핸들에 접근할 수 있다.
///
/// JNI 함수 명명 규칙: Java_<패키지>_<클래스>_<메서드>
/// 패키지: dev.lutica.skid → dev_lutica_skid
/// 클래스: SKIDNative

use std::os::raw::c_void;

use crate::model::skid_color::SKIDColor;
use crate::model::skid_image::{SKIDImage, SKIDSizeVector2};
use crate::processor;

// JNI 타입 정의 (jni-sys 크레이트 없이 직접 정의)
type JNIEnv = *mut c_void;
type JClass = *mut c_void;
type JFloatArray = *mut c_void;
type JLong = i64;
type JInt = i32;
type JFloat = f32;

// JNI 함수 테이블 오프셋을 통한 배열 접근 헬퍼
// JNIEnv는 실제로 함수 포인터 테이블의 이중 포인터이다.
// 안전을 위해 raw pointer 기반으로 최소한의 접근만 구현.

/// JNI 배열에서 f32 슬라이스를 가져오는 헬퍼
unsafe fn get_float_array_elements(
    env: JNIEnv,
    array: JFloatArray,
) -> (*mut f32, i32) {
    // JNIEnv → JNINativeInterface_ 함수 테이블
    let env_ptr = env as *mut *const *const c_void;
    let vtable = *env_ptr as *const *const c_void;

    // GetArrayLength는 vtable[171] (0-indexed)
    let get_array_length: extern "C" fn(JNIEnv, JFloatArray) -> i32 =
        std::mem::transmute(*vtable.add(171));
    let len = get_array_length(env, array);

    // GetFloatArrayElements는 vtable[188]
    let get_float_array_elements: extern "C" fn(JNIEnv, JFloatArray, *mut u8) -> *mut f32 =
        std::mem::transmute(*vtable.add(188));
    let ptr = get_float_array_elements(env, array, std::ptr::null_mut());

    (ptr, len)
}

/// JNI float 배열 요소 해제
unsafe fn release_float_array_elements(
    env: JNIEnv,
    array: JFloatArray,
    elems: *mut f32,
    mode: i32,
) {
    let env_ptr = env as *mut *const *const c_void;
    let vtable = *env_ptr as *const *const c_void;

    // ReleaseFloatArrayElements는 vtable[192]
    let release: extern "C" fn(JNIEnv, JFloatArray, *mut f32, i32) =
        std::mem::transmute(*vtable.add(192));
    release(env, array, elems, mode);
}

/// 새 JNI float 배열 생성
unsafe fn new_float_array(env: JNIEnv, len: i32) -> JFloatArray {
    let env_ptr = env as *mut *const *const c_void;
    let vtable = *env_ptr as *const *const c_void;

    // NewFloatArray는 vtable[175]
    let new_arr: extern "C" fn(JNIEnv, i32) -> JFloatArray =
        std::mem::transmute(*vtable.add(175));
    new_arr(env, len)
}

/// JNI float 배열에 데이터 복사
unsafe fn set_float_array_region(
    env: JNIEnv,
    array: JFloatArray,
    start: i32,
    len: i32,
    buf: *const f32,
) {
    let env_ptr = env as *mut *const *const c_void;
    let vtable = *env_ptr as *const *const c_void;

    // SetFloatArrayRegion는 vtable[203]
    let set_region: extern "C" fn(JNIEnv, JFloatArray, i32, i32, *const f32) =
        std::mem::transmute(*vtable.add(203));
    set_region(env, array, start, len, buf);
}

// ─── 핸들 관리: 기존 C# FFI의 IMAGE_HANDLES를 재사용 ───

use super::ffi_modules::{IMAGE_HANDLES, new_handle_id};

// ═══════════════════════════════════════════════════════════
// JNI 함수 구현
// ═══════════════════════════════════════════════════════════

/// float[] 배열로부터 SKIDImage를 생성하고 핸들(long)을 반환한다.
///
/// Java 시그니처: `native long createFromF32Array(float[] data, int width, int height);`
#[no_mangle]
pub unsafe extern "C" fn Java_dev_lutica_skid_SKIDNative_createFromF32Array(
    env: JNIEnv,
    _class: JClass,
    data: JFloatArray,
    width: JInt,
    height: JInt,
) -> JLong {
    if data.is_null() {
        return 0;
    }

    let (ptr, len) = get_float_array_elements(env, data);
    if ptr.is_null() {
        return 0;
    }

    let expected = (width as usize) * (height as usize) * 4;
    if (len as usize) < expected {
        release_float_array_elements(env, data, ptr, 0);
        return 0;
    }

    let slice = std::slice::from_raw_parts(ptr, expected);
    let colors: Vec<SKIDColor> = slice
        .chunks_exact(4)
        .map(|c| SKIDColor::from_f32_array([c[0], c[1], c[2], c[3]]))
        .collect();

    release_float_array_elements(env, data, ptr, 0);

    let image = SKIDImage::from_1d_data(
        SKIDSizeVector2 {
            width: width as usize,
            height: height as usize,
        },
        colors,
    );

    let handle = new_handle_id();
    IMAGE_HANDLES.write().unwrap().insert(handle, Box::new(image));
    handle as JLong
}

/// 핸들 해제 (메모리 반환).
///
/// Java 시그니처: `native void free(long handle);`
#[no_mangle]
pub unsafe extern "C" fn Java_dev_lutica_skid_SKIDNative_free(
    _env: JNIEnv,
    _class: JClass,
    handle: JLong,
) {
    if handle > 0 {
        IMAGE_HANDLES.write().unwrap().remove(&(handle as u64));
    }
}

/// 이미지 크기를 [width, height] int 배열로 반환한다.
/// 반환값: width, height 순서로 long 패킹 (상위 32비트: width, 하위 32비트: height)
///
/// Java 시그니처: `native long getSize(long handle);`
#[no_mangle]
pub unsafe extern "C" fn Java_dev_lutica_skid_SKIDNative_getSize(
    _env: JNIEnv,
    _class: JClass,
    handle: JLong,
) -> JLong {
    let handles = IMAGE_HANDLES.read().unwrap();
    if let Some(image) = handles.get(&(handle as u64)) {
        let size = image.get_size();
        // 상위 32비트: width, 하위 32비트: height
        ((size.width as i64) << 32) | (size.height as i64 & 0xFFFF_FFFF)
    } else {
        0
    }
}

/// 이미지 데이터를 float[] 배열로 반환한다.
///
/// Java 시그니처: `native float[] getDataAsF32Array(long handle);`
#[no_mangle]
pub unsafe extern "C" fn Java_dev_lutica_skid_SKIDNative_getDataAsF32Array(
    env: JNIEnv,
    _class: JClass,
    handle: JLong,
) -> JFloatArray {
    let handles = IMAGE_HANDLES.read().unwrap();
    if let Some(image) = handles.get(&(handle as u64)) {
        let data = image.get_1d_data_as_f32();
        let arr = new_float_array(env, data.len() as i32);
        if !arr.is_null() {
            set_float_array_region(env, arr, 0, data.len() as i32, data.as_ptr());
        }
        arr
    } else {
        std::ptr::null_mut()
    }
}

/// 이미지를 리사이즈하고 새 핸들을 반환한다.
///
/// Java 시그니처: `native long resize(long handle, int newWidth, int newHeight);`
#[no_mangle]
pub unsafe extern "C" fn Java_dev_lutica_skid_SKIDNative_resize(
    _env: JNIEnv,
    _class: JClass,
    handle: JLong,
    new_width: JInt,
    new_height: JInt,
) -> JLong {
    use cubecl::wgpu::WgpuRuntime;
    use super::ffi_modules::DEFAULT_WGPU_DEVICE;

    let device = &*DEFAULT_WGPU_DEVICE;

    // 1) 읽기 락: clone 후 즉시 해제
    let image_clone = {
        let handles = IMAGE_HANDLES.read().unwrap();
        match handles.get(&(handle as u64)) {
            Some(image) => image.clone(),
            None => return 0,
        }
    };

    // 2) 락 없이 GPU 작업
    let new_size = SKIDSizeVector2 {
        width: new_width as usize,
        height: new_height as usize,
    };
    let resized = processor::resize_image::resize_image::<WgpuRuntime>(
        device,
        &image_clone,
        new_size,
        None,
    );

    // 3) 쓰기 락: 결과 저장
    let new_handle = new_handle_id();
    IMAGE_HANDLES.write().unwrap().insert(new_handle, Box::new(resized));
    new_handle as JLong
}

/// 높이맵에서 노멀맵을 생성하고 새 핸들을 반환한다.
///
/// Java 시그니처: `native long generateNormalMap(long handle, float xFactor, float yFactor);`
#[no_mangle]
pub unsafe extern "C" fn Java_dev_lutica_skid_SKIDNative_generateNormalMap(
    _env: JNIEnv,
    _class: JClass,
    handle: JLong,
    x_factor: JFloat,
    y_factor: JFloat,
) -> JLong {
    use cubecl::wgpu::WgpuRuntime;
    use super::ffi_modules::DEFAULT_WGPU_DEVICE;

    let device = &*DEFAULT_WGPU_DEVICE;

    // 1) 읽기 락: clone 후 즉시 해제
    let image_clone = {
        let handles = IMAGE_HANDLES.read().unwrap();
        match handles.get(&(handle as u64)) {
            Some(image) => image.clone(),
            None => return 0,
        }
    };

    // 2) 락 없이 GPU 작업
    let result = processor::make_normal_map::make_normal_map_base::<WgpuRuntime>(
        device.clone(),
        &image_clone,
        Some(x_factor),
        Some(y_factor),
    );

    // 3) 쓰기 락: 결과 저장
    let new_handle = new_handle_id();
    IMAGE_HANDLES.write().unwrap().insert(new_handle, Box::new(result));
    new_handle as JLong
}
