# 퍼포먼스 측정 방안

## 현재 상태

- 벤치마크 인프라 **전무** (criterion 없음, `benches/` 없음, `[dev-dependencies]` 없음)
- 수동 `Instant::now()` 측정만 존재 (`gpu_call_tests.rs`, `file_io.rs`)
- 크레이트 타입: `cdylib` only → **criterion 벤치마크를 직접 쓸 수 없음** (`rlib` 또는 `lib` 필요)

## 제약 조건

1. **`cdylib` only 문제**: Criterion은 `rlib`에서만 동작. `crate-type`에 `"lib"` 추가 필요
2. **GPU 런타임**: 벤치마크 실행 환경에 GPU(또는 WGPU CPU 폴백) 필요
3. **워밍업**: 첫 GPU 커널 실행은 셰이더 컴파일 포함 → 워밍업 필수

---

## 측정 대상 (4개 영역)

### 영역 1: GPU 작업 단위 성능 (Criterion 벤치마크)

**대상 함수:**
- `processor::resize_image::resize_image::<WgpuRuntime>()` — 업스케일
- `processor::make_normal_map::make_normal_map_base::<WgpuRuntime>()` — 노멀맵 생성
- `processor::example_generator::launch::<WgpuRuntime>()` — 이미지 생성

**측정 방법:**
- Criterion 벤치마크 그룹, 해상도별 파라미터화
  - 입력: 256×256, 720p, 1080p, 4K
  - 출력: 2× 업스케일 (resize), 동일 해상도 (normal map)
- GPU 워밍업: `criterion::measurement_time(Duration::from_secs(10))` + `warm_up_time(5s)`

**측정 항목:**
- 전체 wall time (함수 진입~반환)
- 처리량: pixels/sec

### 영역 2: 데이터 변환 오버헤드

**대상:**
- `SKIDImage::clone()` — Vec<Vec<SKIDColor>> 깊은 복사
- `SKIDImage::get_1d_data_as_f32()` — 2D→1D f32 변환
- `SKIDImage::from_1d_data()` — 1D→2D 재구축
- `SKIDColor::from_f32_array()` / `to_f32_array()` — 픽셀 변환

**측정 방법:**
- Criterion 벤치마크, 해상도별 파라미터화
- 순수 CPU 작업이므로 GPU 없이 실행 가능

**측정 항목:**
- wall time
- 메모리 대역폭 활용률 (bytes_processed / time vs 이론적 대역폭)

### 영역 3: RwLock 경합 (멀티스레드 시뮬레이션)

**대상:**
- FFI 핸들 기반 워크플로: `create → resize → get_data → free`

**측정 방법:**
- `std::thread::spawn`으로 동시 접근 시뮬레이션
- 시나리오:
  - **A) 읽기 경합**: N 스레드가 동시에 `skid_image_get_size()` 호출
  - **B) 읽기-쓰기 경합**: N-1 스레드가 `get_size()` + 1 스레드가 `resize()` (쓰기 락)
  - **C) 쓰기 경합**: N 스레드가 동시에 `resize()` 호출
- N = 1, 2, 4, 8, 16
- 측정: `Instant::now()` 기반, 각 스레드의 대기시간 + 전체 완료시간

**측정 항목:**
- 스레드당 평균/p99 대기시간
- 전체 처리량 (ops/sec)
- 락 경합으로 인한 스케일링 저하율

### 영역 4: 엔드투엔드 FFI 파이프라인

**대상:**
- C# FFI 호출 시뮬레이션: `create_from_f32_array → resize → get_data_as_f32_array → free`

**측정 방법:**
- Rust 내부에서 FFI 함수를 직접 호출하는 통합 벤치마크
- 해상도별 파라미터화

**측정 항목:**
- 전체 파이프라인 wall time
- 각 단계별 비중 (%) — 데이터 변환 vs GPU vs 락 대기

---

## 구현 계획

### Step 1: Cargo.toml 수정

```toml
[lib]
crate-type = ["cdylib", "lib"]  # "lib" 추가 → criterion이 rlib로 링크 가능

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "gpu_benchmarks"
harness = false

[[bench]]
name = "data_conversion_benchmarks"
harness = false

[[bench]]
name = "lock_contention_benchmarks"
harness = false
```

### Step 2: `benches/gpu_benchmarks.rs` — 영역 1

- `resize_benchmark` 그룹: 256→512, 720p→1440p, 1080p→2160p
- `normal_map_benchmark` 그룹: 256, 720p, 1080p, 4K
- WgpuDevice::Cpu 폴백으로 CI 환경에서도 실행 가능

### Step 3: `benches/data_conversion_benchmarks.rs` — 영역 2

- `clone_benchmark` 그룹: 해상도별 SKIDImage::clone() 비용
- `get_1d_data_as_f32_benchmark`: 2D→1D 변환
- `from_1d_data_benchmark`: 1D→2D 변환
- GPU 불필요, 모든 환경에서 실행 가능

### Step 4: `benches/lock_contention_benchmarks.rs` — 영역 3 + 4

- `concurrent_read_benchmark`: N 스레드 동시 읽기
- `concurrent_rw_benchmark`: 읽기/쓰기 혼합
- `e2e_pipeline_benchmark`: FFI 전체 파이프라인
- 커스텀 측정 로직 (criterion iter 내부에서 spawn + join)

### Step 5: 결과 분석 및 보고

- `cargo bench` 실행 후 `target/criterion/` HTML 리포트 생성
- PERFORMANCE.md에 측정 결과 섹션 추가
- 회귀 감지: 이전 결과 대비 자동 비교 (criterion 내장)
