# Packaging Icons

Run `make icons` after changing icon source files.
This regenerates macOS, Linux, and Windows packaging icons.

`assets/icons/macos.png` is the canonical source for macOS app icons.
`make icons-macos` generates `packaging/macos/Shogun.icns` from that file.
Copy it into the app bundle's `Contents/Resources` directory alongside an `Info.plist` that references `Shogun.icns`.

`assets/icons/icon.png` is the canonical source for Linux and other runtime icons.
`make icons-linux` regenerates the Linux hicolor PNGs.
The Linux hicolor PNGs under `packaging/linux/hicolor/` are intended for installation under `share/icons/hicolor/`, and `packaging/linux/io.github.zhubby.Shogun.desktop` should be installed under `share/applications/`.

`make icons-windows` generates `packaging/windows/Shogun.ico` from `assets/icons/icon.png`.
The Windows package is a portable zip that includes the icon as an asset; the runtime window icon is set by winit.

The runtime Bevy/winit icon path only affects platforms where winit supports window icons, such as Linux on X11.
macOS Dock/Finder icons require the bundle `.icns`, and Linux Wayland launchers require the `.desktop` file plus installed hicolor icons.

Package commands:

```text
make package-macos
make package-linux
make package-windows
make package
```
