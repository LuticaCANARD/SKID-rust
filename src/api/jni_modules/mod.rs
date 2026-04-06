/// JNI(JVM Native Interface) 호환 FFI 모듈.
///
/// `jni` 크레이트를 사용하여 타입 안전한 JNI 상호작용을 제공한다.
/// 기존 C# FFI와 동일한 핸들 기반 IMAGE_HANDLES를 공유하여,
/// C#과 JVM이 동일 프로세스 내에서 같은 이미지 핸들에 접근할 수 있다.
///
/// JNI 함수 명명 규칙: Java_<패키지>_<클래스>_<메서드>
/// 패키지: dev.lutica.skid → dev_lutica_skid
/// 클래스: SKIDNative

#[cfg(feature = "use_jni")]
mod impl_jni {
    use jni::JNIEnv;
    use jni::objects::{JClass, JFloatArray};
    use jni::sys::{jlong, jint, jfloat, jfloatArray};

    use crate::model::skid_color::SKIDColor;
    use crate::model::skid_image::{SKIDImage, SKIDSizeVector2};
    use crate::processor;
    use crate::api::ffi_modules::{IMAGE_HANDLES, new_handle_id};

    /// float[] 배열로부터 SKIDImage를 생성하고 핸들(long)을 반환한다.
    ///
    /// Java 시그니처: `native long createFromF32Array(float[] data, int width, int height);`
    #[no_mangle]
    pub extern "system" fn Java_dev_lutica_skid_SKIDNative_createFromF32Array(
        mut env: JNIEnv,
        _class: JClass,
        data: JFloatArray,
        width: jint,
        height: jint,
    ) -> jlong {
        let expected = (width as usize) * (height as usize) * 4;

        let len = match env.get_array_length(&data) {
            Ok(l) => l as usize,
            Err(_) => return 0,
        };
        if len < expected {
            return 0;
        }

        let mut buf = vec![0.0f32; expected];
        if env.get_float_array_region(&data, 0, &mut buf).is_err() {
            return 0;
        }

        let colors: Vec<SKIDColor> = buf
            .chunks_exact(4)
            .map(|c| SKIDColor::from_f32_array([c[0], c[1], c[2], c[3]]))
            .collect();

        let image = SKIDImage::from_1d_data(
            SKIDSizeVector2 {
                width: width as usize,
                height: height as usize,
            },
            colors,
        );

        let handle = new_handle_id();
        IMAGE_HANDLES.write().unwrap().insert(handle, Box::new(image));
        handle as jlong
    }

    /// 핸들 해제 (메모리 반환).
    ///
    /// Java 시그니처: `native void free(long handle);`
    #[no_mangle]
    pub extern "system" fn Java_dev_lutica_skid_SKIDNative_free(
        _env: JNIEnv,
        _class: JClass,
        handle: jlong,
    ) {
        if handle > 0 {
            IMAGE_HANDLES.write().unwrap().remove(&(handle as u64));
        }
    }

    /// 이미지 크기를 반환한다.
    /// 반환값: 상위 32비트 = width, 하위 32비트 = height
    ///
    /// Java 시그니처: `native long getSize(long handle);`
    #[no_mangle]
    pub extern "system" fn Java_dev_lutica_skid_SKIDNative_getSize(
        _env: JNIEnv,
        _class: JClass,
        handle: jlong,
    ) -> jlong {
        let handles = IMAGE_HANDLES.read().unwrap();
        if let Some(image) = handles.get(&(handle as u64)) {
            let size = image.get_size();
            ((size.width as i64) << 32) | (size.height as i64 & 0xFFFF_FFFF)
        } else {
            0
        }
    }

    /// 이미지 데이터를 float[] 배열로 반환한다.
    ///
    /// Java 시그니처: `native float[] getDataAsF32Array(long handle);`
    #[no_mangle]
    pub extern "system" fn Java_dev_lutica_skid_SKIDNative_getDataAsF32Array(
        mut env: JNIEnv,
        _class: JClass,
        handle: jlong,
    ) -> jfloatArray {
        let handles = IMAGE_HANDLES.read().unwrap();
        if let Some(image) = handles.get(&(handle as u64)) {
            let data = image.get_1d_data_as_f32();
            let arr = match env.new_float_array(data.len() as i32) {
                Ok(a) => a,
                Err(_) => return std::ptr::null_mut(),
            };
            if env.set_float_array_region(&arr, 0, &data).is_err() {
                return std::ptr::null_mut();
            }
            arr.into_raw()
        } else {
            std::ptr::null_mut()
        }
    }

    /// 이미지를 리사이즈하고 새 핸들을 반환한다.
    ///
    /// Java 시그니처: `native long resize(long handle, int newWidth, int newHeight);`
    #[no_mangle]
    pub extern "system" fn Java_dev_lutica_skid_SKIDNative_resize(
        _env: JNIEnv,
        _class: JClass,
        handle: jlong,
        new_width: jint,
        new_height: jint,
    ) -> jlong {
        use cubecl::wgpu::WgpuRuntime;
        use crate::api::ffi_modules::DEFAULT_WGPU_DEVICE;

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
        new_handle as jlong
    }

    /// 높이맵에서 노멀맵을 생성하고 새 핸들을 반환한다.
    ///
    /// Java 시그니처: `native long generateNormalMap(long handle, float xFactor, float yFactor);`
    #[no_mangle]
    pub extern "system" fn Java_dev_lutica_skid_SKIDNative_generateNormalMap(
        _env: JNIEnv,
        _class: JClass,
        handle: jlong,
        x_factor: jfloat,
        y_factor: jfloat,
    ) -> jlong {
        use cubecl::wgpu::WgpuRuntime;
        use crate::api::ffi_modules::DEFAULT_WGPU_DEVICE;

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
        new_handle as jlong
    }
}
