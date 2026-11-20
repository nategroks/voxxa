# CI only (ci.yml, full-eval.yml set CMAKE_PROJECT_INCLUDE to this file; whisper-rs-sys
# passes CMAKE_* variables to CMake). whisper.cpp defaults GGML_NATIVE on, which builds
# for the CPU of the machine that compiles it; a build cached on one runner then crashes
# with an illegal instruction on another. Off gives the portable AVX/AVX2/FMA/F16C build.
set(GGML_NATIVE OFF CACHE BOOL "ggml: enable -march=native flag" FORCE)
