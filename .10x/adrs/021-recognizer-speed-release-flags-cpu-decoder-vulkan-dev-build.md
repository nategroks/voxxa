# ADR-021: Recognizer speed: fix the release-build flags (#59), tune the CPU decoder first, prove Vulkan as a booth-only dev build

**Status:** Accepted § 1 (founder, FA-01, 2026-11-25); §§ 2-3 Proposed · **Date:** 2026-11-24 · **Feature:** church-autoadvance · **Reversibility:** Easy (two CI env lines, a check script, a test, recognizer settings behind configuration, an opt-in Cargo feature) · **Owner:** nategroks · **Author:** 10x-Team synthesiser (Principal Architect + Engineering Manager + Staff Engineer), from the 2026-11-24 strategy draft (not yet committed); § 1 edited in V0 to match what was built

Amends nothing. Implements the speed half of the strategy's latency attribute (`.10x/specs/2026-10-24-software-strategy.md:32`).

## Context
- **#59 already tracks the symptom**: Windows installers built by `build.yml` run whisper.cpp unoptimised, about 6.5 s per 1 s hop (#59, Q4-01). The cause was found on 2026-11-24: on Windows MSVC the `cmake` crate 0.1.58 writes `CMAKE_C/CXX_FLAGS_RELEASE` without optimisation flags (with no generator named, it builds them from the `cc` flags minus every `-O`/`/O` word, `cmake-0.1.58/src/lib.rs:723-726`, `:752-767`), and whisper-rs-sys always builds the Release configuration (`whisper-rs-sys-0.11.1/build.rs:158`). Tiny on the 2:00 private song P1: 461 s of compute broken vs 130 s fixed. The fix, verified on the founder's PC: env `CMAKE_C_FLAGS_RELEASE` and `CMAKE_CXX_FLAGS_RELEASE` = `-MD -O2 -Ob2 -DNDEBUG` (whisper-rs-sys forwards every `CMAKE_*` env var as a `-D`, `build.rs:235-243`, and the crate leaves a variable that is already defined alone).
- **Calibration (orchestrator, the founder's PC, 2026-11-24/25):** the broken release build's whisper-rs-sys `out/build/CMakeCache.txt` has `CMAKE_C_FLAGS_RELEASE:STRING= -nologo -MD -Brepro -W0` (C++ the same) and `CMAKE_BUILD_TYPE` Release; the fixed build has `CMAKE_C_FLAGS_RELEASE:STRING=-MD -O2 -Ob2 -DNDEBUG` (C++ the same). A check for `-O2` in those two lines tells them apart.
- `build.yml` sets `CMAKE_PROJECT_INCLUDE` workflow-wide (`.github/workflows/build.yml:42-43`) and caches release builds under a fixed key `release-portable-ggml` (`:165` and `:354` at 866b51e). whisper-rs-sys 0.11.1 reruns its build script only when `VULKAN_SDK`, `HIP_PATH`, `AMDGPU_TARGETS` or its `wrapper.h` change (`build.rs:79`, `:102`, `:180`, `:189`, `:200`), so a cached unoptimised build can survive a flag change. Swatinem/rust-cache v2.9.3 keys each cache by job id, runner OS, and a hash of every non-empty `CARGO*`, `CC*`, `CFLAGS*`, `CXX*`, `CMAKE*` and `RUST*` variable, and that hash is part of its restore key (`add-rust-environment-hash-key` defaults to true; its `src/config.ts` and `action.yml`).
- `build.yml` runs only for a tag push or a manual run naming a tag, and always makes a draft release; a new rc tag needs the founder's approval.
- `ci.yml` and `full-eval.yml` build debug only. whisper-rs-sys then sets `CMAKE_BUILD_TYPE` RelWithDebInfo (`build.rs:228-233`), so the crate's override lands on `CMAKE_*_FLAGS_RELWITHDEBINFO` (also without `/O2`). The Visual Studio generator does not use that line (on MSVC with no generator set, the crate always names a Visual Studio generator, `lib.rs:645-653`): the crate builds `--config Release` (`lib.rs:879`), whose line keeps CMake's `/MD /O2 /Ob2 /DNDEBUG`. Debug target folders on the booth PC show it (the Release `ggml.vcxproj` has `<Optimization>MaxSpeed`), so debug builds are optimised and only release builds need the fix.
- `-Ob2` is not a gcc/clang flag, and `-MD` means "write a dependency file" there; the env must never be set repo-wide. Git Bash rewrites `/MD`-style flags as paths; the dash style avoids that.
- `tests/release_config.rs` already pins `build.yml` (handoff, "Things to keep from Q10").
- The recognizer creates a new Whisper state for every window and sets no `temperature_inc`, `max_tokens`, `n_threads` or `audio_ctx` (`src-tauri/src/transcription/mod.rs:179-198`). Tiny measures 1.3 s per pass where its encoder should cost about 0.1 s on this CPU [est., the 2026-11-24 strategy draft (not yet committed)], so decoding (temperature fallbacks, long decodes) likely dominates **[UNVERIFIED]**.
- Release CPU timings with the flags fixed (the 2026-11-24 strategy draft (not yet committed)): Tiny 1.3–1.6 s/pass, Small 3.4 s (6 s encoder), Medium 10.5 s. Live needs < 1 s per 1 s hop. Only Small and Medium were clean on P1 at a fixed follower setting.
- whisper-rs 0.13.2 (locked) already has a `vulkan` feature (docs.rs, per the 2026-11-24 strategy draft (not yet committed)). 0.16.0 is three breaking pre-1.0 minors away. A recognizer change forces re-recorded fixture logs.
- The booth PC has a GTX 960 2 GB (Maxwell), unused. Small F16 needs about 0.85 GB; Medium does not fit 2 GB (whisper.cpp README figures, per the 2026-11-24 strategy draft (not yet committed)).

## Decision
1. **#59 fix by environment, in `build.yml`'s Windows job only.** The job's `env:` sets `CMAKE_C_FLAGS_RELEASE` and `CMAKE_CXX_FLAGS_RELEASE` to `-MD -O2 -Ob2 -DNDEBUG`. Never at workflow level, never in another job, never in a `.cargo/config.toml`. `ci.yml` and `full-eval.yml` get no such step: they build debug only, which is already optimised and where these two variables would change nothing (a Windows debug build compiles the Release line, which already has CMake's `/O2`; a Linux or macOS debug build never reads it). Guards in the same PR:
   - `scripts/ci/check-msvc-release-flags.sh <release target folder>` reads the newest `whisper-rs-sys-*/out/build/CMakeCache.txt` and fails with `::error::` when there is none, or when either `CMAKE_*_FLAGS_RELEASE` line lacks `-O2`. The Windows job runs it right after "Build the installers". It is calibrated on the measured broken and fixed caches (Context); no `.vcxproj` fallback is needed.
   - `scripts/ci/test-check-msvc-release-flags.sh` runs on every PR, in `ci.yml`'s `budgets` job and in the `check` job's Windows leg (the CRLF checkout and Git Bash of the release job). A fixed cache passes, alone or beside an older broken one. These fail: the broken cache; only the C or only the C++ line broken; `-O1`; `-Od`; `O2` inside a word, at either end; a broken cache beside an older fixed one; a cache from another crate only; no cache. The fixtures hold only flag lines (the measured ones, plus made-up negative cases), laid out like a target folder.
   - `tests/release_config.rs` fails if either env line leaves the Windows job's `env:` or appears anywhere else in `build.yml`, if the check step is missing, conditional or not right after the installer build (no step between, named or not), if the Windows cache key reverts, or if another workflow or a `.cargo/config(.toml)` names either variable.
   - The Windows job's rust-cache key becomes `release-portable-ggml-o2`. The two new variables already change that key through rust-cache's env hash, so no old cache is restored; the new name says so in the file. The macOS/Linux `build` job keeps `release-portable-ggml`: its key carries another job id and OS, so it never holds an MSVC build. The `ci.yml` and `full-eval.yml` keys stay: they hold debug builds only.
   - `scripts/dev/msvc-release-flags.ps1` sets the two variables for one PowerShell window (dot-sourced; `powershell -File` would lose them), and `docs/RELEASING.md` and `fixtures/README.md` add `cargo clean --manifest-path src-tauri/Cargo.toml -p whisper-rs-sys --release` once, then the same check script on the local build folder.
   - **Proof still open:** the check runs on GitHub only in a tag build, so its first real run is the next founder-approved rc draft; that run's Windows job must show it passing.
   The CMake `CACHE … FORCE` include and a repo `.cargo/config.toml` are not used (unverified).
2. **CPU decoder settings before any GPU work.** Behind recognizer configuration, default unchanged until the V6 gate: one reused `WhisperState` per recognizer; `temperature_inc = 0`; `max_tokens = 48`; `n_threads = physical cores`; `audio_ctx = 300` for Small and larger (Tiny keeps the default). Each pass logs encode ms, decode ms and the fallback count (numbers only). **Decision rule:** Small ≤ 1.0 s/pass → CPU with a 1 s hop; ≤ 1.5 s → test a 1.5 s hop; otherwise the GPU is on the critical path.
3. **Vulkan as an opt-in Cargo feature, booth-only dev build.** `gpu-vulkan = ["whisper-rs/vulkan"]` on whisper-rs 0.13.2; never in `default`; never in installers until a no-loader start test passes and the bake-off (Q21) is done. The context sets `use_gpu` explicitly and the session start line logs backend and device name. Target: Small, 6 s encoder, reused state, ≤ 0.8 s per pass (p95 ≤ 1.0 s), VRAM ≤ 1.2 GB, no visible stutter in Presenter.
4. **No whisper-rs bump** in this increment.

## Alternatives Considered
| Alternative | Pros | Cons | Why Not |
|---|---|---|---|
| CMake `if(MSVC) … CACHE FORCE` in `ggml-portable.cmake` + repo-root `.cargo/config.toml` (the 2026-11-24 strategy draft, not yet committed) | one place for CI and local builds | whether it beats the crate's `-D` is unverified; adds a new config file every cargo run reads | the env route is the one verified fix; revisit only if the env route fails |
| Env vars in `[env]` of `.cargo/config.toml` | covers local builds | no per-target form; breaks gcc/clang builds | wrong on macOS/Linux |
| GPU first (Vulkan before the CPU decoder settings) | Small comfortably live | needs the SDK, a long booth build, a second artefact, Maxwell efficiency unknown | the CPU settings are a cheaper lever and may remove the GPU from the critical path |
| whisper-rs 0.16 with the GPU work | newer ggml-vulkan | three breaking minors; fixture re-records; API delta unverified | not needed: 0.13.2 has `vulkan` |
| CUDA 12.x build | likely fastest on NVIDIA | NVIDIA-only; hundreds of MB of DLLs; licence question inside GPL-3.0; CUDA 13 dropped Maxwell | Vulkan is vendor-neutral; CUDA only as a one-time benchmark if Vulkan disappoints |

## Consequences
### Positive
- Installer builds run Whisper at the speed every number assumes; Q11 can test a fast build.
- A Windows release build without `-O2` now fails its job instead of shipping a slow installer.
- The cheapest speed lever is measured before hardware work.
- A GPU result tells the founder what hardware other churches need (founder item FA-23 in the 2026-11-24 strategy draft, not yet committed).
### Negative
- Local release builds still need the founder's shell script; CI does not protect them.
- Decoder settings change what Whisper outputs; the follower must be re-checked on cached windows (V3 acceptance) before any default changes.
### Risks
- `temperature_inc = 0` removes the guard against repetition loops; `max_tokens` caps it, and the follower result must hold.
- The GTX 960 may also drive the projector; GPU load could stutter Presenter (measured in V4).
- A Vulkan build may not start without `vulkan-1.dll` **[UNVERIFIED]**: that is why it stays a dev build.

## Dependencies
- Depends on: ADR-014 (streaming hop), ADR-015 (eval gate), ADR-020 (draft releases).
- Constrains: ADR-022, proposed in the 2026-11-24 strategy draft (not yet committed): the live configuration the follower is gated on.
- Plan steps (the 2026-11-24 strategy draft, not yet committed): V0 (#59), V3 (CPU decoder), V4 (Vulkan proof).

## Review trigger
Small ≤ 1.0 s/pass on CPU (GPU drops to "nice to have"); the Vulkan proof fails its bar (then consider the whisper-rs bump or CUDA benchmark); a security advisory on whisper-rs; the CMakeCache check fails on a runner image update.
