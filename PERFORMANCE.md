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

---

## 부록: RwLock 점유시간 분석

### 변경 전 (GPU 작업 중 락 점유)

`skid_image_resize` 호출 시 락 점유 구간:

```
read lock 획득
  ├─ HashMap::get()                          ~50ns
  ├─ get_1d_data_as_f32()  [CPU 변환]        ~2–8ms (1080p 기준)
  ├─ client.create() [GPU 버퍼 업로드]        ~0.5–2ms
  ├─ kernel launch + GPU 실행                ~5–50ms (해상도/GPU에 따라)
  ├─ client.read_one() [GPU→CPU 다운로드]     ~1–5ms
  ├─ f32→SKIDColor 변환 + from_1d_data()     ~2–8ms
  └─ HashMap::insert()                       ~100ns
read lock 해제 (→ write lock으로 승격 불가, 실제로는 drop 후 write)
```

**총 락 점유시간: ~10–73ms (1080p 기준)**

이 기간 동안 다른 스레드의 `getSize`, `getData`, `free` 등 모든 읽기/쓰기가 차단됨.

### 변경 후 (clone 패턴)

```
read lock 획득
  ├─ HashMap::get()                          ~50ns
  ├─ SKIDImage::clone()                      ~2–6ms (1080p, Vec<Vec<>> 깊은 복사)
read lock 해제                                ← 여기서 즉시 해제

[락 없음] GPU 작업 수행                        ~8–63ms
  ├─ get_1d_data_as_f32()
  ├─ GPU 버퍼 업로드/커널 실행/다운로드
  └─ 결과 SKIDImage 생성

write lock 획득
  ├─ HashMap::insert()                       ~100ns
write lock 해제
```

**읽기 락 점유: ~2–6ms (clone 비용)**
**쓰기 락 점유: ~100ns**

### 개선 효과 계산

| 해상도 | 변경 전 락 점유 | 변경 후 읽기 락 | 변경 후 쓰기 락 | 동시성 차단 감소율 |
|--------|-----------------|-----------------|-----------------|-------------------|
| 720p (1280×720) | ~8ms | ~1.5ms | ~100ns | **~81%** |
| 1080p (1920×1080) | ~25ms | ~4ms | ~100ns | **~84%** |
| 4K (3840×2160) | ~73ms | ~15ms | ~100ns | **~79%** |

**핵심:** clone 비용(~2–6ms)은 GPU 작업(~8–63ms)보다 항상 작으므로, 락 없이
GPU 작업을 수행하는 것이 전체 스루풋에서 명확한 이득.

### clone 비용 상세 (Vec<Vec<SKIDColor>>)

1080p 이미지 = 1920 × 1080 픽셀 = 2,073,600 `SKIDColor` (각 16바이트)

```
메모리 할당:
  - 외부 Vec: 1개 (1080 포인터)
  - 내부 Vec: 1,080개 (각 1920 × 16바이트 = 30,720바이트)
  - 총 복사량: 1080 × 30,720 = ~31.6 MB

clone 비용 내역:
  - 힙 할당: 1,081회 (outer Vec 1 + inner Vec 1080)
  - memcpy: ~31.6 MB
  - 예상 시간: ~3–5ms (메모리 대역폭 ~10 GB/s 기준)
```

### 향후 개선 가능 사항

1. **`Vec<Vec<>>` → `Vec<SKIDColor>` 평탄화**: clone 시 할당 1회 + memcpy 1회로 축소. 예상 clone 시간 ~1ms.
2. **`Arc<SKIDImage>` 기반 핸들**: clone 대신 참조 카운팅으로 O(1) 공유. 락 점유 ~50ns로 감소.
3. **lock-free concurrent map**: `dashmap` 등으로 RwLock 자체를 제거.
