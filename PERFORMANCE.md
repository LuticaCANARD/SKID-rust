# 성능 분석 및 개선 제안

## 치명적 (Critical) 문제

### 1. `Vec<Vec<SKIDColor>>` 내부 저장 구조 — `skid_image.rs:22`

```rust
data: Vec<Vec<SKIDColor>>,  // 행마다 별도 힙 할당
```

**문제:** 행(row)마다 개별 힙 할당 → 캐시 지역성 파괴. 1080p 이미지에서 1,080번의 힙 할당 + 2단계 포인터 역참조가 매 픽셀 접근마다 발생.

**해결:** `Vec<SKIDColor>` 1D 평탄 배열로 교체하고, `data[y * width + x]`로 접근.

---

### 2. `import_from_png` 픽셀당 Mutex 잠금 — `file_io.rs:214`

```rust
let mut data = pixel_data.lock().unwrap();  // 매 픽셀마다 lock
data[y][x as usize] = color;
```

**문제:** 1080p 이미지 기준 ~2,073,600번의 lock/unlock. 멀티스레드의 이점이 완전히 상쇄됨.

**해결:** 스레드별 로컬 버퍼에 쓴 후 마지막에 합치기 (export 쪽의 `get_u16_color_vectors` 패턴처럼).

---

### 3. 전역 단일 `Mutex<HashMap>` 핸들 관리 — `api/ffi_modules/mod.rs:95`

```rust
static IMAGE_HANDLES: Lazy<Mutex<HashMap<u64, Box<SKIDImage>>>> = ...;
```

**문제:** 모든 FFI 호출이 하나의 Mutex를 공유. 이미지 A를 읽는 동안 이미지 B 접근 불가. C#에서 멀티스레드 호출 시 직렬화됨.

**해결:** `RwLock<HashMap<u64, ...>>`으로 교체하여 읽기 동시성 확보. 또는 슬랩/슬롯맵 사용.

---

### 4. `resize_scaledown_kernel` 미구현 — `resize_image.rs:101-127`

```rust
if px < new_width && py < new_height {
    let new_idx = (py * new_width + px) * 4;
    // ← 본문 없음
}
```

**문제:** 다운스케일 시 출력이 초기화되지 않은 메모리. 실제로 검은 이미지나 쓰레기 값 반환.

---

## 높음 (High) 문제

### 5. FFI 경계에서 전체 이미지 복제 — `api/ffi_modules/mod.rs:114-118`

```rust
let colors: Vec<SKIDColor> = data_slice.chunks_exact(4)
    .map(|chunk| SKIDColor::from_f32_array(chunk.try_into().unwrap()))
    .collect();                                    // 복사 1: f32[] → Vec<SKIDColor>
let image = SKIDImage::from_1d_data(..., colors);  // 복사 2: Vec → Vec<Vec<>>
```

**문제:** C# → Rust 호출 시 매번 2중 복제. 1080p RGBA: ~32MB 할당.

---

### 6. ~~GPU 호출마다 디바이스 재생성~~ — **해결됨**

`DEFAULT_WGPU_DEVICE` 싱글턴으로 교체 완료.
참고: CubeCL `ComputeRuntime`이 내부적으로 `client()`를 디바이스 키별로 캐싱하므로,
`WgpuDevice::default()` 자체는 열거형 생성일 뿐 실제 GPU 초기화 비용은 없었음.
그러나 명시적 싱글턴이 의도를 더 명확히 전달하고 HashMap 룩업도 제거.

---

### 7. PNG 내보내기 시 전체 이미지 clone — `file_io.rs:88`

```rust
Arc::new(image.get_data().clone()),  // 전체 Vec<Vec<>> 깊은 복사
```

**문제:** 불변 참조만 필요한 상황에서 전체 이미지 데이터 복제.

**해결:** `get_data()`가 `&Vec<Vec<>>` 반환하므로 수명(lifetime) 기반 공유 가능.

---

### 8. GPU 커널 내 중첩 루프 — `resize_image.rs:36-95`

```rust
for x in 0..CUBE_CLUSTER_DIM_X {
    for y in 0..CUBE_CLUSTER_DIM_Y {
        // 각 스레드가 여러 픽셀을 순차 처리
    }
}
```

**문제:** GPU의 병렬성을 활용하지 못함. 1 스레드 = 1 픽셀이 이상적.

---

### 9. println! 디버그 출력 — `resize_image.rs:163-165`

```rust
println!("Launching resize with runtime: {}x{}", new_width, new_height);
```

**문제:** `println!`은 글로벌 stdout 락을 사용. 프로덕션 코드에서 GPU 런치 직전에 I/O 대기 발생.

---

## 중간 (Medium) 문제

### 10. `SKIDColor::Sub` 알파 채널 버그 — `skid_color.rs:73`

```rust
a: self.a - other.g  // ← other.a 여야 함
```

**문제:** 알파 빼기 연산이 green 채널을 사용. 블렌딩 결과가 잘못됨.

---

### 11. 중복 크기 필드 — `skid_image.rs:19-25`

```rust
width: usize,           // 중복
height: usize,          // 중복
len: usize,             // width * height와 동일
size: SKIDSizeVector2,  // width, height와 동일
```

**문제:** 4개 필드가 같은 정보. 불일치 위험 + 캐시 라인 낭비.

---

### 12. `get_1d_data()`와 `to_vec()` 중복 — `skid_image.rs:102-104, 136-138`

```rust
pub fn to_vec(&self) -> Vec<SKIDColor> { ... }       // 동일 로직
pub fn get_1d_data(&self) -> Vec<SKIDColor> { ... }   // 동일 로직
```

**문제:** 같은 기능의 메서드 2개. 매 호출마다 전체 이미지 클론.
