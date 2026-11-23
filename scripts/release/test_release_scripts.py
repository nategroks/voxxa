"""Unit tests for the release helper scripts (ADR-020 § 6). No network, no secrets.

  python -m unittest discover -s scripts/release -v
"""
import hashlib
import os
import re
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import updater_manifest as um  # noqa: E402
import winget_manifest as wm  # noqa: E402

REPO = "nategroks/voxxa"
SIGS = {
    "Voxxa_0.1.0_x64-setup.exe": "sig-nsis\n",
    "Voxxa_0.1.0_x64_en-US.msi": "sig-msi",
    "Voxxa_aarch64.app.tar.gz": "sig-mac-arm",
    "Voxxa_x64.app.tar.gz": "sig-mac-intel",
    "Voxxa_0.1.0_amd64.AppImage": "sig-appimage",
    "Voxxa_0.1.0_amd64.deb": "sig-deb",
}


class UpdaterManifest(unittest.TestCase):
    def build(self, sigs=SIGS, version="0.1.0", tag="v0.1.0"):
        return um.build(version, tag, REPO, sigs, "2026-11-23T12:00:00Z")

    def test_every_platform_key_and_its_preferred_bare_key(self):
        p = self.build()["platforms"]
        self.assertEqual(sorted(p), [
            "darwin-aarch64", "darwin-aarch64-app", "darwin-x86_64", "darwin-x86_64-app",
            "linux-x86_64", "linux-x86_64-appimage", "linux-x86_64-deb",
            "windows-x86_64", "windows-x86_64-msi", "windows-x86_64-nsis",
        ])
        self.assertEqual(p["windows-x86_64"], p["windows-x86_64-nsis"], "NSIS is the Windows default")
        self.assertEqual(p["linux-x86_64"], p["linux-x86_64-appimage"])
        self.assertEqual(p["windows-x86_64-nsis"]["signature"], "sig-nsis", "trailing newline stripped")

    def test_urls_point_at_the_tagged_release(self):
        p = self.build(tag="v0.1.0-rc.1")["platforms"]
        self.assertEqual(p["windows-x86_64-msi"]["url"],
                         "https://github.com/nategroks/voxxa/releases/download/v0.1.0-rc.1/Voxxa_0.1.0_x64_en-US.msi")

    def test_msi_only_is_still_the_windows_default(self):
        p = self.build({"Voxxa_0.1.0_x64_en-US.msi": "s"})["platforms"]
        self.assertEqual(sorted(p), ["windows-x86_64", "windows-x86_64-msi"])

    def test_unknown_assets_are_skipped_and_nothing_is_an_error(self):
        p = self.build({**SIGS, "Voxxa_0.1.0_x64-setup.exe.asc": "x", "notes.txt": "x"})["platforms"]
        self.assertEqual(len(p), 10)
        with self.assertRaises(ValueError):
            self.build({"notes.txt": "x"})
        with self.assertRaises(ValueError):
            self.build({"Voxxa.app.tar.gz": "no arch in the name"})

    def test_two_assets_for_one_key_is_an_error(self):
        with self.assertRaises(ValueError):
            self.build({"Voxxa_0.1.0_x64-setup.exe": "a", "Other_0.1.0_x64-setup.exe": "b"})

    def test_main_fails_when_a_built_platform_has_no_update(self):
        def run(sigs):
            with tempfile.TemporaryDirectory() as d:
                for name, text in sigs.items():
                    with open(os.path.join(d, name + ".sig"), "w", encoding="utf-8") as fh:
                        fh.write(text)
                out = os.path.join(d, "latest.json")
                um.main(["--version", "0.1.0", "--tag", "v0.1.0", "--repo", REPO, "--sigs", d, "--out", out])
                return os.path.exists(out)
        self.assertTrue(run(SIGS))
        # An Intel Mac bundle whose name the script does not recognise: skipped, then refused.
        renamed = {**{k: v for k, v in SIGS.items() if k != "Voxxa_x64.app.tar.gz"}, "Voxxa_universal.app.tar.gz": "s"}
        self.assertEqual(um.missing(self.build(renamed)), ["darwin-x86_64"])
        with self.assertRaises(SystemExit) as stop:
            run(renamed)
        self.assertEqual(stop.exception.code, 1)

    def test_the_version_must_be_semver(self):
        for bad in ["v0.1.0", "0.1", "latest", ""]:
            with self.assertRaises(ValueError, msg=bad):
                self.build(version=bad)


class WingetManifest(unittest.TestCase):
    SHA = "A" * 64

    def values(self, version="0.1.0", tag="v0.1.0", name="Voxxa_0.1.0_x64-setup.exe", sha=SHA):
        return wm.values_for(version, tag, REPO, name, sha, "2026-11-23")

    def test_the_templates_fill_completely(self):
        values = self.values()
        names = sorted(n for n in os.listdir(wm.TEMPLATE_DIR) if n.endswith(".yaml"))
        self.assertEqual(names, ["nategroks.Voxxa.installer.yaml", "nategroks.Voxxa.locale.en-US.yaml", "nategroks.Voxxa.yaml"])
        for name in names:
            with open(os.path.join(wm.TEMPLATE_DIR, name), encoding="utf-8") as fh:
                text = wm.fill(fh.read(), values)
            self.assertNotRegex(text, r"__[A-Z0-9_]+__", name)
            self.assertIn("PackageIdentifier: nategroks.Voxxa\n", text, name)
            self.assertIn("PackageVersion: 0.1.0\n", text, name)
            self.assertIn("ManifestVersion: 1.6.0\n", text, name)
            if "installer" in name:
                self.assertIn("InstallerUrl: https://github.com/nategroks/voxxa/releases/download/v0.1.0/Voxxa_0.1.0_x64-setup.exe\n", text)
                self.assertIn(f"InstallerSha256: {self.SHA}\n", text)
                self.assertIn("Scope: user\n", text)
                self.assertIn("InstallerType: nullsoft\n", text)

    def test_an_rc_tag_fills_a_preview_of_its_version(self):
        self.assertIn("/v0.1.0-rc.1/", self.values(tag="v0.1.0-rc.1")["__INSTALLER_URL__"])

    def test_refusals(self):
        for kw in [dict(version="0.1.0-rc.1", tag="v0.1.0-rc.1"), dict(tag="v0.2.0"), dict(tag="0.1.0"),
                   dict(sha="a" * 64), dict(sha="A" * 63), dict(name="Voxxa_0.1.0_x64_en-US.msi")]:
            with self.assertRaises(ValueError, msg=str(kw)):
                self.values(**kw)
        with self.assertRaises(ValueError):
            wm.fill("x: __UNKNOWN__", self.values())

    def test_the_hash_is_of_the_file_given(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "Voxxa_0.1.0_x64-setup.exe")
            with open(path, "wb") as fh:
                fh.write(b"placeholder installer bytes")
            sha = wm.sha256_of(path)
        self.assertTrue(re.fullmatch(r"[0-9A-F]{64}", sha))
        self.assertEqual(sha, hashlib.sha256(b"placeholder installer bytes").hexdigest().upper())


if __name__ == "__main__":
    unittest.main()
