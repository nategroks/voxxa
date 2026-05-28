# Voxxa on Linux

Voxxa runs on Linux as an AppImage or `.deb`. There are two Linux-specific
quirks worth knowing about up front.

## Wayland vs X11 — keystroke fallback

When Voxxa's active presenter driver is **Keystroke (any app)** — the default,
and the only option for EasyWorship / MediaShout / SongShow Plus / WorshipTools
Presenter / VideoPsalm — Voxxa needs to synthesize key presses for whatever
window the operator has focused.

- **X11**: works out of the box via XTest. No configuration needed.
- **Wayland**: synthetic input requires `ydotool` + a running `ydotoold`
  daemon with your user in the `input` group. Without this, Voxxa's
  keystrokes silently disappear.

**Recommendation:** If your worship laptop runs Wayland (the default on
Fedora 34+, Ubuntu 22.10+, Linux Mint 22), switch to a driver that uses
an API instead — ProPresenter REST, FreeShow, OpenLP, OpenSong, or
Proclaim all bypass the input-injection problem entirely. Use Voxxa's
Settings → Connection panel with the **Auto-detect** button to find one.

If you must use the keystroke fallback under Wayland:

```bash
sudo apt install ydotool          # Debian/Ubuntu
sudo dnf install ydotool          # Fedora
sudo usermod -aG input "$USER"    # Re-login after this
systemctl --user enable --now ydotoold
```

Then start Voxxa from a terminal so it inherits the `ydotool` socket
environment. As an alternative, log into an X11 session at the display
manager — `XSESSION=xfce` or similar — and `XTest` will work without
any of the above.

## OpenLP under Wayland

This is a separate problem from Voxxa's keystroke story. From OpenLP's
release notes for 3.0 onward:

> OpenLP at present does not behave well under Wayland so the recommendation
> is to run under X11. If you can't run in X11 (or prefer to run it in
> XWayland), you should start it with `QT_QPA_PLATFORM` environment variable
> set to `xcb`, although the Main View from the Web Remote will not work.

For Voxxa, this means:

- The Voxxa → OpenLP REST connection (port 4316) works fine either way —
  it's HTTP, no display-server interaction.
- But OpenLP's *own* main view (the audience-facing slide window) is what
  has Wayland problems. If you're using OpenLP, run OpenLP itself on X11
  or under `QT_QPA_PLATFORM=xcb`. Voxxa doesn't care.

## Audio capture backends

Voxxa uses `cpal` for audio capture, which transparently supports
PulseAudio, PipeWire (PulseAudio compat layer), ALSA, and JACK. PipeWire
is the default on:

- Fedora 34+ (April 2021)
- Ubuntu 22.10+ (per OMG Ubuntu: "PipeWire is now shipping as the default
  audio server in Ubuntu 22.10 daily builds")
- Linux Mint 22

**Note:** Ubuntu 22.04 LTS still uses PulseAudio for audio (PipeWire is
only used for video/screensharing there). If you target a mix of these,
test both backends — `cpal` abstracts the call surface but timing and
latency characteristics differ.

## mDNS auto-discovery

The Auto-detect button browses `_pro7stagedsply._tcp.local.` via mDNS.
Most distros have Avahi running by default; if not:

```bash
sudo apt install avahi-daemon libnss-mdns
sudo systemctl enable --now avahi-daemon
```

Firewall: if you're running `ufw` or a stricter alternative, allow
inbound UDP 5353 so mDNS responses can come back:

```bash
sudo ufw allow 5353/udp
```

## Microphone permission

Unlike macOS, Linux does not gate microphone access with a system prompt.
Voxxa just opens the device through PulseAudio / PipeWire / ALSA. If
nothing's coming through the level meter, check `pavucontrol` or
`pw-cli ls Node` to verify the device is unmuted and not redirected.

## Packaging

The release workflow produces both `.AppImage` and `.deb` artifacts.
AppImages are GPG-signed when the release was tagged; the public key is
published on the release page. The `.deb` is unsigned at the apt-repo
level — install with `sudo dpkg -i voxxa_*.deb` and accept the warning.
