#!/usr/bin/env bash
# #59 / ADR-021 section 1: fail unless whisper.cpp was built with -O2. On Windows MSVC the
# cmake crate 0.1.58 writes CMAKE_C/CXX_FLAGS_RELEASE without optimisation, so the build
# needs both set in the environment (build.yml's Windows job, scripts/dev/msvc-release-flags.ps1).
# Reads the newest whisper-rs-sys CMakeCache.txt under a release target folder; no cache fails.
# Usage: bash scripts/ci/check-msvc-release-flags.sh src-tauri/target/x86_64-pc-windows-msvc/release
set -euo pipefail
dir=${1:?usage: check-msvc-release-flags.sh <release target folder>}
cache=$(ls -t "$dir"/build/whisper-rs-sys-*/out/build/CMakeCache.txt 2>/dev/null | head -n 1 || true)
if [ -z "$cache" ]; then
  echo "::error::no whisper-rs-sys CMakeCache.txt under $dir/build, so the release flags cannot be checked (#59)"
  exit 1
fi
echo "Checking $cache"
bad=0
for var in CMAKE_C_FLAGS_RELEASE CMAKE_CXX_FLAGS_RELEASE; do
  line=$(grep "^$var:" "$cache" || true)
  echo "${line:-$var: missing}"
  if ! [[ " ${line#*=} " =~ [[:space:]][-/]O2[[:space:]] ]]; then
    echo "::error::$var has no -O2: whisper.cpp is built unoptimised (#59)"
    bad=1
  fi
done
exit "$bad"
