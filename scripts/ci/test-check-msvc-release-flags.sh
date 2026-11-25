#!/usr/bin/env bash
# Proves check-msvc-release-flags.sh (#59) on release target folders made here, holding only
# the flag lines of the CMake caches measured on 2026-11-24/25: a fixed build passes; the
# broken build, a build with only the C++ line broken, O2 only inside another word, a broken
# build beside an older fixed one, and a folder with no cache fail.
set -euo pipefail
check="$(dirname "$0")/check-msvc-release-flags.sh"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
fixed='-MD -O2 -Ob2 -DNDEBUG'
broken=' -nologo -MD -Brepro -W0'
# target <name> <C flags> <C++ flags> [hash]: a whisper-rs-sys cache in a release target folder.
target() {
  local out="$tmp/$1/build/whisper-rs-sys-${4:-0123456789abcdef}/out/build"
  mkdir -p "$out"
  printf 'CMAKE_C_FLAGS_RELEASE:STRING=%s\nCMAKE_CXX_FLAGS_RELEASE:STRING=%s\n' "$2" "$3" > "$out/CMakeCache.txt"
}
target fixed "$fixed" "$fixed"
target broken "$broken" "$broken"
target cxx-broken "$fixed" "$broken"
target o2-in-a-word '-MD -DNO2 -W0' '-MD -O2x -W0'
# The newest cache decides: an older optimised one (say, restored from a cache) must not hide it.
target stale "$fixed" "$fixed"
touch -d 2000-01-01 "$tmp"/stale/build/whisper-rs-sys-0123456789abcdef/out/build/CMakeCache.txt
target stale "$broken" "$broken" fedcba9876543210
mkdir -p "$tmp/no-cache/build"
failed=0
# expect <pass|fail> <name>
expect() {
  if bash "$check" "$tmp/$2" > "$tmp/out" 2>&1; then got=pass; else got=fail; fi
  if [ "$got" = fail ] && ! grep -q '^::error::' "$tmp/out"; then got="fail without ::error::"; fi
  if [ "$got" = "$1" ]; then
    echo "ok   $2: $got"
  else
    echo "FAIL $2: $got, expected $1"
    cat "$tmp/out"
    failed=1
  fi
}
expect pass fixed
expect fail broken
expect fail cxx-broken
expect fail o2-in-a-word
expect fail stale
expect fail no-cache
exit "$failed"
