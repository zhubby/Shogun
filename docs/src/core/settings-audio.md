# 设置与音频

## 设置系统

设置按主菜单设置面板的标签页组织：

| 标签页 | 配置项 |
|--------|--------|
| **显示** | 窗口分辨率、窗口模式（窗口/全屏/无边框）、垂直同步 |
| **音频** | 主音量、输出设备 |
| **语言** | UI 语言（en-US / zh-CN） |
| **游戏性** | 自动存档开关 |
| **快捷键** | 全局快捷键绑定 |
| **AI** | 推理模型和多模态模型配置 |

### 设置持久化

`GameSettingsStore` 使用 `directories::ProjectDirs` 存储设置文件：

```rust
pub struct GameSettingsStore {
    path: PathBuf,
}
```

设置在游戏启动时加载，通过 `GameUiState` 中的 `applied_settings` 和 `pending_settings` 分离"已应用"和"待确认"状态。

### 游戏性设置

`GameplaySettings` 保存影响战局流程的偏好。自动存档默认关闭；开启后，新开游戏和每回合结束会覆盖写入存档目录下的最新自动存档。自动存档不写入普通槽位 manifest，也不占用玩家手动存档槽。

### 显示设置

`DisplaySettings` 管理窗口配置：

- **窗口分辨率** — 默认 1280x820，支持多种预设
- **窗口模式** — Windowed / Fullscreen / BorderlessFullscreen
- **垂直同步** — 控制 `PresentMode`

## 音频系统

`MainMenuAudio` 管理主菜单背景音乐：

- **格式** — MP3（Bevy 的 `mp3` feature + rodio）
- **播放控制** — 仅在主菜单播放，进入游戏后停止
- **设备枚举** — 支持选择音频输出设备
- **音量控制** — 主音量和 BGM 音量独立调节

### BGM 同步

`sync_main_menu_bgm` 系统在每帧检查：
1. 当前屏幕是否为主菜单
2. BGM 是否应该播放
3. 音频设备是否可用

如果设备不可用，会显示警告消息并禁用 BGM。

<!-- TODO: 补充游戏内音效设计（目前只有菜单 BGM） -->
