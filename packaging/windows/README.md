# Windows Packaging

`packaging/windows/Shogun.ico` is generated from `assets/icons/icon.png`.
Regenerate it with `make icons-windows`.
The Windows package target creates a portable zip containing `shogun.exe`, assets, and the icon file.

The executable uses the runtime winit window icon path. Embedding `Shogun.ico` directly into `shogun.exe` is intentionally left to a future installer or resource-compilation step.
