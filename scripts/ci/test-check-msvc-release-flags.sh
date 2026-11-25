#!/usr/bin/env bash
# Proves check-msvc-release-flags.sh (#59) on release target folders made here. They hold
# only flag lines: the two measured on 2026-11-24/25 (fixed and broken), plus made-up
# negative cases. A fixed build passes, and so does a fixed build beside an older broken
# one. These fail: the broken build; only the C line broken; only the C++ line broken; -O1
# and -Od; O2 inside a word, at either end; a broken build beside an older fixed one; a
# cache from another crate only; no cache.
set -euo pipefail
check="$(dirname "$0")/check-msvc-release-flags.sh"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
fixed='-MD -O2 -Ob2 -DNDEBUG'
broken=' -nologo -MD -Brepro -W0'
first=whisper-rs-sys-0123456789abcdef
last=whisper-rs-sys-fedcba9876543210
# cache <name> <C flags> <C++ flags> [crate build folder]: a CMake cache in a release target folder.
cache() {
  local out="$tmp/$1/build/${4:-$first}/out/build"
  mkdir -p "$out"
  printf 'CMAKE_C_FLAGS_RELEASE:STRING=%s\nCMAKE_CXX_FLAGS_RELEASE:STRING=%s\n' "$2" "$3" > "$out/CMakeCache.txt"
}
# old <name> [crate build folder]: make that cache older than every other one.
old() { touch -d 2000-01-01 "$tmp/$1/build/${2:-$first}/out/build/CMakeCache.txt"; }
cache fixed "$fixed" "$fixed"
cache broken "$broken" "$broken"
cache c-broken "$broken" "$fixed"
cache cxx-broken "$fixed" "$broken"
cache o1 '-MD -O1 -DNDEBUG' '-MD -O1 -DNDEBUG'
cache od '-MD -Od -DNDEBUG' '-MD -Od -DNDEBUG'
cache o2-suffix "$fixed" '-MD -O2x -W0'
cache o2-prefix '-MD -DX-O2 -W0' "$fixed"
# The newest cache decides, not the name: an older one (say, restored from a cache) must
# not hide it. The newer cache sorts first by name in one case and last in the other.
cache stale "$fixed" "$fixed" "$last"
old stale "$last"
cache stale "$broken" "$broken"
cache fresh "$broken" "$broken"
old fresh
cache fresh "$fixed" "$fixed" "$last"
cache other-crate "$fixed" "$fixed" other-sys-0123456789abcdef
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
expect pass fresh
for name in broken c-broken cxx-broken o1 od o2-suffix o2-prefix stale other-crate no-cache; do
  expect fail "$name"
done
exit "$failed"
