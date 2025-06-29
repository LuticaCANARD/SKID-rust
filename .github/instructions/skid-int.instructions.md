---
applyTo: '**'
---

# SKID-rust : 이미지 처리 백엔드
##  프로젝트 개요
> 핵심 아키텍처: C# (프론트엔드) → FFI → Rust (백엔드 로직) → CubeCL → GPU (연산)
- 이 프로젝트는 Rust로 작성된 고성능 이미지 처리 라이브러리(skid_rust_backend)입니다. 
- 주요 연산은 `cubecl` 프레임워크를 통해 GPU에서 병렬 처리되어 성능을 극대화합니다.
- `csbindgen` 을 사용하여 `C#(.NET)` 애플리케이션에서 네이티브 라이브러리처럼 호출할 수 있도록 `FFI(Foreign Function Interface)`를 제공하는 것이 특징입니다.

## 주요 데이터 구조
- SKIDImage (이미지 데이터와 크기)
- SKIDColor (RGBA, f32)

## 기술 스택 및 GPU 메모리 레이아웃
- 언어: Rust
- GPU 컴퓨팅: cubecl (WGPU, CUDA 등 다양한 백엔드를 지원하는 GPGPU 프레임워크)
- 상호운용성: csbindgen (Rust-C# FFI 자동 생성)
### GPU 메모리 레이아웃:
- 모든 이미지는 GPU로 전송될 때 Interleaved RGBA, Row-Major 형식의 1차원 f32 배열로 변환됩니다.
> 메모리상 표현: [R, G, B, A, R, G, B, A, ...]
- cubecl 커널에서 입력값에 접근할 때는 이 1차원 배열을 (픽셀 인덱스, 채널 인덱스) 2단계로 접근합니다 (예: `input[pixel_idx][channel_idx]`).
- 출력값에 쓰기 위해 접근할 때에는 1차원으로 좁혀서 접근합니다.(예: `output[pixel_idx * 4 + channel_idx]`)
> channel_idx는 0(R), 1(G), 2(B), 3(A)로 고정되어 있습니다.
- `f32`, `i32`, `u32`, `bool` 타입을 사용합니다.
- 이때, `f32` -> `u32`/`i32` 로 변환할 때는 `u32::cast_from`/`i32::cast_from` 을 써야 하고, `u32` <-> `i32` 변환은 as 키워드를 사용합니다.
- array의 index에 접근 할 때에는 `usize`가 아닌, `u32`를 사용합니다.
