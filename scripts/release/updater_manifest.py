#!/usr/bin/env python3
"""Write the updater's latest.json from the .sig files of a draft release (ADR-020 § 6).

One job runs this after every platform has uploaded its installers and their updater
signatures, so no two jobs edit latest.json at once, and the Windows signatures are
the ones made after Authenticode signing. Standard library only.

  python scripts/release/updater_manifest.py --version 0.1.0 --tag v0.1.0 \
      --repo nategroks/voxxa --sigs <dir of *.sig> --out latest.json

Platform keys follow tauri-plugin-updater 2.x: "{os}-{arch}-{bundle}" first, then the
bare "{os}-{arch}" for the preferred bundle (NSIS on Windows, AppImage on Linux).
"""
import argparse
import datetime
import json
import os
import re
import sys
import urllib.parse

# (suffix of the asset name, os, bundle), most preferred bundle first per os.
BUNDLES = [
    ("-setup.exe", "windows", "nsis"),
    (".msi", "windows", "msi"),
    (".app.tar.gz", "darwin", "app"),
    (".AppImage", "linux", "appimage"),
    (".deb", "linux", "deb"),
    (".rpm", "linux", "rpm"),
]
ARCHES = [("aarch64", "aarch64"), ("arm64", "aarch64"), ("x86_64", "x86_64"), ("x64", "x86_64"), ("amd64", "x86_64")]


def platform_of(asset):
    """(os, arch, bundle) for an updater asset name, or None if it is not one."""
    for suffix, os_name, bundle in BUNDLES:
        if asset.endswith(suffix):
            words = re.split(r"[_.\-]", asset[: -len(suffix)].lower())
            arch = next((a for word, a in ARCHES if word in words), None)
            return (os_name, arch, bundle) if arch else None
    return None


def build(version, tag, repo, sigs, pub_date, notes=""):
    """latest.json as a dict. `sigs` maps an asset name (without .sig) to its signature text."""
    if not re.fullmatch(r"\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?", version):
        raise ValueError(f"not a release version: {version!r}")
    platforms, preferred, rank = {}, {}, {b: i for i, (_, _, b) in enumerate(BUNDLES)}
    for asset in sorted(sigs):
        where = platform_of(asset)
        if where is None:
            print(f"skipped (not an updater bundle this script knows): {asset}", file=sys.stderr)
            continue
        os_name, arch, bundle = where
        key = f"{os_name}-{arch}-{bundle}"
        if key in platforms:
            raise ValueError(f"two assets for {key}: {platforms[key]['url']} and {asset}")
        entry = {
            "signature": sigs[asset].strip(),
            "url": f"https://github.com/{repo}/releases/download/{urllib.parse.quote(tag)}/{urllib.parse.quote(asset)}",
        }
        platforms[key] = entry
        bare = f"{os_name}-{arch}"
        if bare not in preferred or rank[bundle] < rank[preferred[bare][0]]:
            preferred[bare] = (bundle, entry)
    for bare, (_, entry) in preferred.items():
        platforms[bare] = entry
    if not platforms:
        raise ValueError("no updater signatures found")
    return {"version": version, "notes": notes, "pub_date": pub_date, "platforms": dict(sorted(platforms.items()))}


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--version", required=True)
    ap.add_argument("--tag", required=True)
    ap.add_argument("--repo", required=True)
    ap.add_argument("--sigs", required=True, help="folder holding <asset>.sig files")
    ap.add_argument("--out", required=True)
    ap.add_argument("--notes", default="")
    args = ap.parse_args()
    sigs = {}
    for name in os.listdir(args.sigs):
        if name.endswith(".sig"):
            with open(os.path.join(args.sigs, name), encoding="utf-8") as fh:
                sigs[name[: -len(".sig")]] = fh.read()
    now = datetime.datetime.now(datetime.timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")
    manifest = build(args.version, args.tag, args.repo, sigs, now, args.notes)
    with open(args.out, "w", encoding="utf-8", newline="\n") as fh:
        json.dump(manifest, fh, indent=2)
        fh.write("\n")
    for key, entry in manifest["platforms"].items():
        print(f"{key}: {entry['url']}")


if __name__ == "__main__":
    main()
