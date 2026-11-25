# #59 / ADR-021 section 1: local Windows release builds of whisper.cpp with /O2.
# The cmake crate 0.1.58 writes the MSVC release flags without optimisation, so a release
# build runs Whisper about 3.5 times slower. This sets both flags for THIS PowerShell window
# only. Dot-source it in the window you build from; `powershell -File` would set them in a
# child process that exits at once:
#   Set-ExecutionPolicy -Scope Process Bypass
#   . .\scripts\dev\msvc-release-flags.ps1
# Once, before the first release build with it (whisper-rs-sys never rebuilds by itself):
#   cargo clean --manifest-path src-tauri/Cargo.toml -p whisper-rs-sys --release
# Windows (MSVC) only: -Ob2 breaks gcc and clang.
$env:CMAKE_C_FLAGS_RELEASE = '-MD -O2 -Ob2 -DNDEBUG'
$env:CMAKE_CXX_FLAGS_RELEASE = '-MD -O2 -Ob2 -DNDEBUG'
"CMAKE_C_FLAGS_RELEASE=$env:CMAKE_C_FLAGS_RELEASE"
"CMAKE_CXX_FLAGS_RELEASE=$env:CMAKE_CXX_FLAGS_RELEASE"
