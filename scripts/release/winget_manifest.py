#!/usr/bin/env python3
"""Fill the winget manifest template in packaging/winget/ for one release (ADR-020 § 6).

  python scripts/release/winget_manifest.py --version 0.1.0 --tag v0.1.0 \
      --installer <path to Voxxa_0.1.0_x64-setup.exe> --out <folder> [--repo nategroks/voxxa]

The installer URL is the release's download URL for the installer's file name, and its
SHA-256 is computed from the file given, so run it on the final (signed) installer.
The build workflow keeps the result as an artifact; winget.yml submits it only after the
founder publishes a full release. Standard library only.
"""
import argparse
import datetime
import hashlib
import os
import re
import urllib.parse

TEMPLATE_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "packaging", "winget")
PLACEHOLDER = re.compile(r"__[A-Z0-9_]+__")


def fill(text, values):
    """Replace every __NAME__ in `text`; refuse unknown or leftover placeholders."""
    def one(m):
        if m.group(0) not in values:
            raise ValueError(f"no value for {m.group(0)}")
        return values[m.group(0)]
    out = PLACEHOLDER.sub(one, text)
    if PLACEHOLDER.search(out):
        raise ValueError("a value contains a placeholder")
    return out


def values_for(version, tag, repo, installer_name, sha256, release_date):
    if not re.fullmatch(r"\d+\.\d+\.\d+", version):
        raise ValueError(f"not an app version (MAJOR.MINOR.PATCH): {version!r}")
    # An rc tag (v0.1.0-rc.1) builds the app version 0.1.0: the manifest is then a preview
    # artifact only; winget.yml submits full releases only.
    if tag != f"v{version}" and not tag.startswith(f"v{version}-"):
        raise ValueError(f"tag {tag!r} does not match version {version!r}")
    if not re.fullmatch(r"[0-9A-F]{64}", sha256):
        raise ValueError(f"not an upper-case SHA-256: {sha256!r}")
    if not installer_name.endswith("-setup.exe"):
        raise ValueError(f"not the NSIS installer: {installer_name!r}")
    url = f"https://github.com/{repo}/releases/download/{urllib.parse.quote(tag)}/{urllib.parse.quote(installer_name)}"
    return {"__VERSION__": version, "__TAG__": tag, "__INSTALLER_URL__": url, "__INSTALLER_SHA256__": sha256, "__RELEASE_DATE__": release_date}


def sha256_of(path):
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest().upper()


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--version", required=True)
    ap.add_argument("--tag", required=True)
    ap.add_argument("--installer", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--repo", default="nategroks/voxxa")
    ap.add_argument("--date", default=datetime.date.today().isoformat())
    args = ap.parse_args()
    values = values_for(args.version, args.tag, args.repo, os.path.basename(args.installer), sha256_of(args.installer), args.date)
    os.makedirs(args.out, exist_ok=True)
    for name in sorted(os.listdir(TEMPLATE_DIR)):
        if not name.endswith(".yaml"):
            continue
        with open(os.path.join(TEMPLATE_DIR, name), encoding="utf-8") as fh:
            text = fill(fh.read(), values)
        with open(os.path.join(args.out, name), "w", encoding="utf-8", newline="\n") as fh:
            fh.write(text)
        print(f"wrote {name}")
    print(f"InstallerUrl: {values['__INSTALLER_URL__']}\nInstallerSha256: {values['__INSTALLER_SHA256__']}")


if __name__ == "__main__":
    main()
