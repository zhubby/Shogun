# 跨平台打包

项目通过 Makefile 支持三个平台的打包。

## 打包命令

```bash
make package-macos    # dist/Shogun-0.1.0-macos.zip
make package-linux    # dist/shogun-0.1.0-linux.tar.gz
make package-windows  # dist/shogun-0.1.0-windows.zip
make package          # 全平台
```

## macOS

- 构建 `.app` bundle（`dist/macos/Shogun.app`）
- 包含 `Contents/Resources/assets` 目录
- 生成 `.icns` 图标文件
- 打包为 `.zip`

## Linux

- 构建 FHS 兼容目录结构
- 包含 `hicolor` 图标主题
- 打包为 `.tar.gz`

## Windows

- 构建可执行文件 + assets 目录
- 生成 `.ico` 图标文件
- 打包为 `.zip`

## 图标生成

```bash
make icons            # 生成所有平台图标
make icons-macos      # 仅 macOS (.icns)
make icons-linux      # 仅 Linux (hicolor PNG)
make icons-windows    # 仅 Windows (.ico)
```

图标源文件：
- `assets/icons/icon.png` — 通用图标
- `assets/icons/macos.png` — macOS 专用图标
- `assets/icons/banner_logo.png` — 主菜单横幅

## 资产路径

打包后的应用需要在不同目录结构中找到资产：

```
macOS .app:   Shogun.app/Contents/Resources/assets/
Linux FHS:    /usr/share/shogun/assets/
Windows:      shogun/assets/
开发环境:     ./assets/
```

`runtime_assets_dir()` 自动探测正确的路径。

<!-- TODO: 补充 CI/CD 流水线、自动发布流程 -->
