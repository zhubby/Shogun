APP_NAME := Shogun
APP_ID := io.github.zhubby.Shogun
BIN_NAME := shogun
VERSION := $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
DIST_DIR := dist
HOST_TARGET := $(shell rustc -vV | sed -n 's/^host: //p')
MACOS_TARGET ?= $(HOST_TARGET)
LINUX_TARGET ?= x86_64-unknown-linux-gnu
WINDOWS_TARGET ?= x86_64-pc-windows-msvc
MACOS_APP := $(DIST_DIR)/macos/$(APP_NAME).app
MACOS_ICONSET := $(DIST_DIR)/iconsets/$(APP_NAME).iconset
WINDOWS_ICONSET := $(DIST_DIR)/iconsets/$(APP_NAME)-windows
LINUX_STAGE := $(DIST_DIR)/linux/$(BIN_NAME)-$(VERSION)
WINDOWS_STAGE := $(DIST_DIR)/windows/$(BIN_NAME)-$(VERSION)
LINUX_ICON_SIZES := 16 32 48 64 128 256 512 1024
WINDOWS_ICON_SIZES := 16 32 48 256

.PHONY: icons icons-macos icons-linux icons-windows package package-macos package-linux package-windows clean-package

icons: icons-macos icons-linux icons-windows

icons-macos:
	mkdir -p "$(MACOS_ICONSET)" packaging/macos
	sips -z 16 16 assets/icons/macos.png --out "$(MACOS_ICONSET)/icon_16x16.png"
	sips -z 32 32 assets/icons/macos.png --out "$(MACOS_ICONSET)/icon_16x16@2x.png"
	sips -z 32 32 assets/icons/macos.png --out "$(MACOS_ICONSET)/icon_32x32.png"
	sips -z 64 64 assets/icons/macos.png --out "$(MACOS_ICONSET)/icon_32x32@2x.png"
	sips -z 128 128 assets/icons/macos.png --out "$(MACOS_ICONSET)/icon_128x128.png"
	sips -z 256 256 assets/icons/macos.png --out "$(MACOS_ICONSET)/icon_128x128@2x.png"
	sips -z 256 256 assets/icons/macos.png --out "$(MACOS_ICONSET)/icon_256x256.png"
	sips -z 512 512 assets/icons/macos.png --out "$(MACOS_ICONSET)/icon_256x256@2x.png"
	sips -z 512 512 assets/icons/macos.png --out "$(MACOS_ICONSET)/icon_512x512.png"
	sips -z 1023 1023 assets/icons/macos.png --out "$(DIST_DIR)/iconsets/macos-1023.png"
	sips -z 1024 1024 "$(DIST_DIR)/iconsets/macos-1023.png" --out "$(MACOS_ICONSET)/icon_512x512@2x.png"
	python3 -c "from pathlib import Path; base=Path('$(MACOS_ICONSET)'); items=[('icp4','icon_16x16.png'),('icp5','icon_32x32.png'),('icp6','icon_32x32@2x.png'),('ic07','icon_128x128.png'),('ic08','icon_256x256.png'),('ic09','icon_512x512.png'),('ic10','icon_512x512@2x.png')]; chunks=[kind.encode('ascii')+(len((base/name).read_bytes())+8).to_bytes(4,'big')+(base/name).read_bytes() for kind,name in items]; body=b''.join(chunks); Path('packaging/macos/$(APP_NAME).icns').write_bytes(b'icns'+(len(body)+8).to_bytes(4,'big')+body)"

icons-linux:
	$(foreach size,$(LINUX_ICON_SIZES),mkdir -p packaging/linux/hicolor/$(size)x$(size)/apps; sips -z $(size) $(size) assets/icons/icon.png --out packaging/linux/hicolor/$(size)x$(size)/apps/$(APP_ID).png;)

icons-windows:
	mkdir -p "$(WINDOWS_ICONSET)" packaging/windows
	$(foreach size,$(WINDOWS_ICON_SIZES),sips -z $(size) $(size) assets/icons/icon.png --out "$(WINDOWS_ICONSET)/icon_$(size)x$(size).png";)
	python3 -c "from pathlib import Path; import struct; base=Path('$(WINDOWS_ICONSET)'); sizes=[int(size) for size in '$(WINDOWS_ICON_SIZES)'.split()]; images=[(size,(base/f'icon_{size}x{size}.png').read_bytes()) for size in sizes]; offset=6+16*len(images); entries=[]; blobs=[]; [entries.append(struct.pack('<BBBBHHII', 0 if size == 256 else size, 0 if size == 256 else size, 0, 0, 1, 32, len(data), offset)) or blobs.append(data) or (globals().__setitem__('offset', offset + len(data))) for size,data in images]; Path('packaging/windows/$(APP_NAME).ico').write_bytes(struct.pack('<HHH', 0, 1, len(images)) + b''.join(entries) + b''.join(blobs))"

package: package-macos package-linux package-windows

package-macos: icons-macos
	cargo build --release --target "$(MACOS_TARGET)"
	rm -rf "$(MACOS_APP)" "$(DIST_DIR)/$(APP_NAME)-$(VERSION)-macos.zip"
	mkdir -p "$(MACOS_APP)/Contents/MacOS" "$(MACOS_APP)/Contents/Resources"
	cp "target/$(MACOS_TARGET)/release/$(BIN_NAME)" "$(MACOS_APP)/Contents/MacOS/$(BIN_NAME)-bin"
	printf '#!/bin/sh\nscript_dir="$$(CDPATH= cd -- "$$(dirname -- "$$0")" && pwd)"\ncd "$$script_dir/../Resources" || exit 1\nexec "$$script_dir/$(BIN_NAME)-bin" "$$@"\n' > "$(MACOS_APP)/Contents/MacOS/$(BIN_NAME)"
	chmod +x "$(MACOS_APP)/Contents/MacOS/$(BIN_NAME)"
	cp packaging/macos/Info.plist "$(MACOS_APP)/Contents/Info.plist"
	cp packaging/macos/$(APP_NAME).icns "$(MACOS_APP)/Contents/Resources/$(APP_NAME).icns"
	cp -R assets "$(MACOS_APP)/Contents/Resources/assets"
	cd "$(DIST_DIR)/macos" && zip -qry "../$(APP_NAME)-$(VERSION)-macos.zip" "$(APP_NAME).app"

package-linux: icons-linux
	cargo build --release --target "$(LINUX_TARGET)"
	rm -rf "$(LINUX_STAGE)" "$(DIST_DIR)/$(BIN_NAME)-$(VERSION)-linux.tar.gz"
	mkdir -p "$(LINUX_STAGE)/bin" "$(LINUX_STAGE)/libexec/$(BIN_NAME)" "$(LINUX_STAGE)/share/applications" "$(LINUX_STAGE)/share/icons/hicolor" "$(LINUX_STAGE)/share/$(BIN_NAME)"
	cp "target/$(LINUX_TARGET)/release/$(BIN_NAME)" "$(LINUX_STAGE)/libexec/$(BIN_NAME)/$(BIN_NAME)-bin"
	printf '#!/bin/sh\nscript_dir="$$(CDPATH= cd -- "$$(dirname -- "$$0")" && pwd)"\nprefix="$$(CDPATH= cd -- "$$script_dir/.." && pwd)"\ncd "$$prefix/share/$(BIN_NAME)" || exit 1\nexec "$$prefix/libexec/$(BIN_NAME)/$(BIN_NAME)-bin" "$$@"\n' > "$(LINUX_STAGE)/bin/$(BIN_NAME)"
	chmod +x "$(LINUX_STAGE)/bin/$(BIN_NAME)"
	cp -R assets "$(LINUX_STAGE)/share/$(BIN_NAME)/assets"
	cp packaging/linux/$(APP_ID).desktop "$(LINUX_STAGE)/share/applications/$(APP_ID).desktop"
	cp -R packaging/linux/hicolor/* "$(LINUX_STAGE)/share/icons/hicolor/"
	tar -C "$(DIST_DIR)/linux" -czf "$(DIST_DIR)/$(BIN_NAME)-$(VERSION)-linux.tar.gz" "$(BIN_NAME)-$(VERSION)"

package-windows: icons-windows
	cargo build --release --target "$(WINDOWS_TARGET)"
	rm -rf "$(WINDOWS_STAGE)" "$(DIST_DIR)/$(BIN_NAME)-$(VERSION)-windows.zip"
	mkdir -p "$(WINDOWS_STAGE)"
	cp "target/$(WINDOWS_TARGET)/release/$(BIN_NAME).exe" "$(WINDOWS_STAGE)/$(BIN_NAME).exe"
	cp -R assets "$(WINDOWS_STAGE)/assets"
	cp packaging/windows/$(APP_NAME).ico "$(WINDOWS_STAGE)/$(APP_NAME).ico"
	cd "$(DIST_DIR)/windows" && zip -qry "../$(BIN_NAME)-$(VERSION)-windows.zip" "$(BIN_NAME)-$(VERSION)"

clean-package:
	rm -rf "$(DIST_DIR)"
