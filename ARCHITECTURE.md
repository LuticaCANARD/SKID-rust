# LuticaSKID - 프로젝트 구조 문서

## 개요

LuticaSKID는 Rust로 작성된 고성능 GPU 가속 이미지 처리 라이브러리입니다. CubeCL 프레임워크를 사용하여 GPU 커널을 실행하며, csbindgen을 통해 C# 애플리케이션에서 FFI로 호출할 수 있는 네이티브 라이브러리(`cdylib`)를 빌드합니다.

```
C# 애플리케이션 (프론트엔드)
    ↓  csbindgen FFI
api/ (C 호출 가능 인터페이스)
    ↓
processor/ (CubeCL GPU 커널)
    ↓
cubecl 런타임 (WGPU / CUDA / HIP)
    ↓
GPU 디바이스
```

## 디렉터리 구조

```
SKID-rust/
├── Cargo.toml                  # 패키지 매니페스트 및 의존성
├── Cargo.lock                  # 의존성 잠금 파일
├── build.rs                    # C# 바인딩 자동 생성 빌드 스크립트
├── README.md                   # 프로젝트 소개
├── ARCHITECTURE.md             # 이 문서
├── .github/
│   └── instructions/
│       └── skid-int.instructions.md  # 개발 가이드라인
├── dotnet/
│   ├── LuticaSKIDBinder.cs               # C# 바인딩 래퍼
│   └── LuticaSKIDBinderToCSharp.g.cs     # 자동 생성된 C# FFI 코드
└── src/
    ├── lib.rs                  # 라이브러리 진입점
    ├── api/                    # C# FFI 인터페이스 레이어
    │   ├── mod.rs
    │   ├── image_opts.rs       # 이미지 처리 연산 열거형
    │   └── ffi_modules/
    │       └── mod.rs          # #[no_mangle] FFI 함수들
    ├── model/                  # 핵심 데이터 구조체
    │   ├── mod.rs
    │   ├── skid_color.rs       # RGBA f32 색상 타입
    │   ├── skid_image.rs       # 2D 이미지 컨테이너
    │   ├── skid_vector2.rs     # 2D 벡터
    │   ├── skid_vector3.rs     # 3D 벡터 (dot/cross 포함)
    │   └── ffi_modules/
    │       └── skid_image_ffi.rs  # 색상/벡터 산술 FFI 래퍼
    ├── processor/              # GPU 이미지 처리 커널
    │   ├── mod.rs
    │   ├── make_normal_map.rs       # 높이맵 → 노멀맵 변환
    │   ├── resize_image.rs          # 바이리니어 보간 리사이즈
    │   ├── example_generator.rs     # 절차적 테스트 패턴 생성
    │   ├── image_rotation_action.rs # 이미지 회전 (TODO)
    │   ├── image_synthesis_action.rs # 이미지 합성 (TODO)
    │   └── image_sync_action.rs     # 이미지 블렌드 타입 별칭
    ├── utils/                  # 유틸리티 함수
    │   ├── mod.rs
    │   ├── gpu_opt.rs          # GPU 정규화 커널
    │   ├── graphic_fn.rs       # GPU 측 수학 함수 (#[cube])
    │   └── file_io.rs          # PNG 파일 입출력 (멀티스레드)
    └── test/                   # 테스트 모듈
        ├── mod.rs
        ├── gpu_call_tests.rs   # GPU 커널 테스트
        ├── file_test.rs        # PNG I/O 테스트
        └── structs_calls.rs    # 구조체 테스트
```

## 핵심 모듈 설명

### model/ - 데이터 구조체

| 파일 | 설명 |
|------|------|
| `skid_color.rs` | RGBA `f32` 기반 색상 구조체. 사칙연산, `f32[]` ↔ `u8[]` 변환 지원 |
| `skid_image.rs` | `Vec<Vec<SKIDColor>>` 기반 2D 이미지. `get_pixel`, `set_pixel`, `to_f32_array` 등 제공 |
| `skid_vector2.rs` | 2D 부동소수점 벡터. 사칙연산 |
| `skid_vector3.rs` | 3D 부동소수점 벡터. dot product, cross product 포함 |
| `ffi_modules/skid_image_ffi.rs` | 위 구조체들의 C 호출 가능 `#[no_mangle]` 래퍼 (47개 함수) |

### processor/ - GPU 커널

| 파일 | 설명 |
|------|------|
| `make_normal_map.rs` | Sobel 유사 엣지 탐지로 높이맵에서 노멀맵을 생성. 상하좌우 픽셀 샘플링 → 법선 벡터 계산 |
| `resize_image.rs` | 바이리니어 보간 기반 업/다운 스케일링. Gather 방식으로 인접 4픽셀 보간 |
| `example_generator.rs` | 테스트용 절차적 그래디언트 패턴 생성 (5120×2880 등) |
| `image_rotation_action.rs` | 이미지 회전 (미완성) |
| `image_synthesis_action.rs` | 이미지 합성 (미완성) |

### api/ - FFI 인터페이스

| 파일 | 설명 |
|------|------|
| `ffi_modules/mod.rs` | C# 호출 가능 인터페이스. 이미지 핸들 관리(`HashMap<i32, Arc<Mutex<SKIDImage>>>`), GPU 디바이스 선택, 리사이즈/노멀맵 생성 등 |
| `image_opts.rs` | `ImageOpts` 열거형 - 사용 가능한 이미지 처리 연산 정의 |

### utils/ - 유틸리티

| 파일 | 설명 |
|------|------|
| `gpu_opt.rs` | CubeCL 기반 GPU 정규화 연산 |
| `graphic_fn.rs` | GPU 측 수학 함수 - grayscale(BT.601), luminance(BT.709), normalize, denormalize |
| `file_io.rs` | PNG 입출력. 멀티스레드 내보내기, 채널별 분리 내보내기, u16 정밀도 지원 |

## GPU 메모리 레이아웃

모든 이미지는 인터리브된 1D `f32` 배열로 GPU에 전달됩니다:

```
[R₀, G₀, B₀, A₀, R₁, G₁, B₁, A₁, ...]
```

- **CubeCL 배열 접근**: `input[pixel_index][channel_index]` (2D 뷰)
- **출력 쓰기**: `output[pixel_index * 4 + channel_index]` (1D 뷰)
- **타입 변환**: `F::cast_from()` → f32 ↔ u32/i32, `as` → u32 ↔ i32

## 주요 의존성

| 의존성 | 버전 | 용도 |
|--------|------|------|
| `cubecl` | 0.5.0 | GPU 컴퓨트 프레임워크 (WGPU, CUDA, HIP 백엔드) |
| `bytemuck` | 1 | 메모리 레이아웃 유틸리티 |
| `image` | 0.25.6 | PNG 파일 I/O (선택적) |
| `once_cell` | 1.21.3 | 지연 정적 초기화 |
| `csbindgen` | 1.8.0 | C# FFI 바인딩 자동 생성 (빌드 전용) |

## Feature 플래그

| 플래그 | 설명 |
|--------|------|
| `use_wgpu` | WGPU 백엔드 (Vulkan/Metal/DX12) - **기본 활성화** |
| `use_cuda` | CUDA 백엔드 - **기본 활성화** |
| `use_image` | PNG 파일 I/O - **기본 활성화** |
| `use_wgpu_msl` | WGPU Metal Shading Language 백엔드 |
| `use_wgpu_spriv` | WGPU SPIR-V 백엔드 |
| `use_hip` | AMD HIP 백엔드 |

## 빌드 출력

- **네이티브 라이브러리**: `skid_rust_backend.dll` (Windows) / `.so` (Linux) / `.dylib` (macOS)
- **C# 바인딩**: `dotnet/LuticaSKIDBinderToCSharp.g.cs` (빌드 시 자동 생성)
