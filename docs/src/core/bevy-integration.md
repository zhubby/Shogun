# Bevy 集成

`src/core/` 使用 Bevy 0.18 作为游戏引擎框架，主要利用其窗口管理、资产加载和 ECS 系统。

## App 构建

```rust
App::new()
    .add_plugins(DefaultPlugins
        .set(AssetPlugin { ... })      // 资产目录配置
        .set(WindowPlugin { ... })      // 窗口分辨率/模式
    )
    .add_plugins(EguiPlugin::default())  // egui 集成
    .add_plugins(AppIconPlugin)          // 窗口图标
    .init_collection::<MainMenuAssets>() // 主菜单资产
    .insert_resource(GameUiState::new()) // UI 状态
    .add_systems(Startup, setup_camera)
    .add_systems(Update, sync_main_menu_bgm)
    .add_systems(EguiPrimaryContextPass, (prepare_assets, game_ui_system).chain())
    .run();
```

## 系统调度

| 系统 | 调度时机 | 职责 |
|------|----------|------|
| `setup_camera` | `Startup` | 创建 2D 摄像机 |
| `sync_main_menu_bgm` | `Update` | 同步主菜单 BGM 状态 |
| `prepare_main_menu_assets_for_egui` | `EguiPrimaryContextPass` | 准备 egui 纹理资产 |
| `game_ui_system` | `EguiPrimaryContextPass` | 主 UI 渲染入口 |

## 资产路径解析

游戏需要支持多种运行环境（开发、macOS .app、Linux 安装、Windows 便携）：

```
候选路径（按优先级）：
1. 可执行文件目录/../Resources/assets   (macOS .app)
2. 可执行文件目录/assets               (Windows/Linux 便携)
3. 可执行文件目录/../../share/shogun/assets  (Linux FHS 安装)
4. 当前工作目录/assets                 (开发环境)
5. CARGO_MANIFEST_DIR/assets           (cargo run)
```

`runtime_assets_dir()` 依次检查候选路径，返回第一个存在的目录。

## 最小 ECS 使用

项目有意最小化 ECS 的使用——大部分状态存在 `GameUiState` 资源中，而不是分散到多个组件/实体。这是因为：

- 策略游戏的状态高度耦合，拆分到 ECS 组件会增加复杂度
- egui 即时模式渲染需要每帧访问完整状态
- 存档/读档操作需要完整的状态快照

<!-- TODO: 补充 Bevy 版本升级注意事项 -->
