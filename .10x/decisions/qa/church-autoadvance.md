# QA — church-autoadvance

## 2026-10-24 — Windows verification of Increment 0 (local Windows session, handoff task 1)

Machine: Windows 11 Pro 10.0.26300, AMD Ryzen 3 1200 (4 cores, no SMT), fresh toolchain
installed via winget: Node 24.22.1, rustc/cargo 1.99.1 (stable-x86_64-pc-windows-msvc),
CMake 4.4.4, VS 2022 Build Tools (MSVC 14.44.35207, VCTools workload), LLVM (see below).
Branch: `claude/trusting-newton-v71ebu` @ d976698.

### Finding 1 — whisper-rs-sys 0.11.1 + LLVM ≥ 23 produces an empty `whisper_full_params`
- Symptom: `cargo clippy` / `cargo test` fail with ~60 × `E0609: no field … on type
  whisper_full_params` inside `whisper-rs-0.13.2/src/whisper_params.rs`; the generated
  `bindings.rs` has `pub struct whisper_full_params { pub _address: u8 }` with a layout test
  that expects 264 bytes.
- Not the header: clang 23.1.4 parses `wrapper.h` with zero diagnostics and its AST has
  every field of the struct.
- Isolated repro (scratch crate, same `wrapper.h` and clang args as whisper-rs-sys's
  build.rs, `allowlist_type("whisper_full_params")`):

  | bindgen | libclang | Result |
  |---|---|---|
  | 0.69.5 (pinned by whisper-rs-sys 0.11.x as `^0.69`) | 23.1.4 | empty struct (bug) |
  | 0.72.1 | 23.1.4 | all fields |
  | 0.69.5 | 20.1.8 | all fields |

- `WHISPER_DONT_GENERATE_BINDINGS=1` (bundled bindings) is **not** a workaround on Windows:
  those bindings were generated for a 64-bit LP64 target and fail their own layout
  asserts on MSVC (`attempt to compute 12_usize - 16_usize, which would overflow`, ×3).
- Gotcha: changing that env var does not rerun whisper-rs-sys's build script
  (it only watches `wrapper.h`); `cargo clean -p whisper-rs-sys` is required between tries.
- Unexplained: CI's `windows-latest` (image windows-2025-vs2026 20261021, "LLVM version:
  23.1.1") passes. Either 23.1.1 vs 23.1.4 differs in a way bindgen 0.69 depends on, or CI
  got its bindings another way. Not chased further.
- Local workaround used: LLVM 20.1.8 (`winget install LLVM.LLVM --version 20.1.8`).
- **Recommended fix (cloud owns `src-tauri/`):** move to whisper-rs ≥ 0.14.2
  (whisper-rs-sys ≥ 0.12.1 requires bindgen `^0.71`; whisper-rs 0.16.0 / -sys 0.15.0
  requires `^0.72`), ideally as part of P1.2 since the recognizer moves behind a trait
  there anyway. Until then, `docs/` or CONTRIBUTING should say "LLVM ≤ 22 on Windows".

### Finding 2 — `.github/workflows/build.yml` is invalid YAML (cloud owns workflows)
- Line 105: `releaseName: 'Voxxa ${{ github.ref_name || inputs.version || 'dev' }}'` nests
  single quotes inside a single-quoted scalar. GitHub cannot parse the file, so it reports
  a 0-second "workflow file issue" failure on every push (runs 40149187244, 40148590077)
  and the tag-triggered release build has never run.
- Same file: upload glob `bundle/**/*.nsis` matches nothing (NSIS output is
  `bundle/nsis/*-setup.exe`); `macos-13` runners are retired.

### Results (LLVM 20.1.8, clean whisper-rs-sys build)

| Step | Result | Time on Ryzen 3 1200 |
|---|---|---|
| `npm ci && npm run build` | ok | 0.1 min |
| `cargo fmt --check` | ok | <0.1 min |
| `cargo clippy --all-targets -- -D warnings` | ok | 1.1 min (whisper.cpp already built; cold ≈ 5.5 min) |
| `cargo test` | ok, 0 tests in 3 targets (as expected before P1.1) | 1.7 min |
| `npx tauri build` | ok | 4.8 min |

Artefacts: `bundle/nsis/Voxxa_0.1.0_x64-setup.exe` (3.3 MB) and
`bundle/msi/Voxxa_0.1.0_x64_en-US.msi` (4.6 MB), both unsigned.
Tauri warns that identifier `com.voxxa.app` ends in `.app` (conflicts with macOS bundle
extension); worth changing before the first public release, since the identifier also
names the settings folder and is costly to change later.

### Install test (non-admin, WebView2, tray, Models tab, SmartScreen)
(pending — needs a standard user account and the founder at the screen)
