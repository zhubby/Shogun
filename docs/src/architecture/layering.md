# 分层设计

## 为什么 game 不依赖 Bevy

策略游戏的领域逻辑（规则、状态、序列化）与渲染引擎有本质区别：

- **领域逻辑**变化频繁（平衡性调整、新规则），需要快速编译和独立测试
- **引擎集成**变化较少但引入重依赖（Bevy、wgpu、winit），编译慢
- 混合在一起会导致每次规则修改都要等待引擎重编译

因此，`src/game/` 被设计为**纯 Rust 库**：

```rust
// game 模块的 Cargo 依赖面（理想情况）
serde, serde_json     // 序列化
sqlx (sqlite)         // 历史数据库查询
directories           // 存档路径
// 没有 bevy, egui, winit
```

## 边界规则

| 规则 | 说明 |
|------|------|
| game 不 import bevy | `src/game/**/*.rs` 中不得出现 `use bevy` |
| game 不 import egui | 同上，UI 类型不出现在领域层 |
| core 调用 game 的 pub API | UI 通过导出函数操作状态 |
| core 不重复 game 的规则 | 验证、计算等逻辑只在 game 中实现 |

## game 内部的模块可见性

game 模块使用 `pub use` 将所有子模块的公开类型统一导出：

```rust
// src/game/mod.rs
pub mod model;
pub mod commands;
pub mod city;
pub mod officer;
// ...

pub use model::*;
pub use commands::*;
pub use city::*;
pub use officer::*;
```

这使得 core 层可以直接 `use shogun::game::*` 引入所有需要的类型，而不必记住每个类型属于哪个子模块。

## core 层的内部组织

core 层通过 `pub(super)` 限制模块间的可见性：

```
core/
  mod.rs          # pub fn run() — 唯一对外入口
  state.rs        # GameUiState — UI 状态（非 pub）
  actions.rs      # 玩家动作处理（pub(super)）
  hud/            # 游戏内 HUD 编排与面板模块
  menu.rs         # 主菜单
  map.rs          # 地图渲染与交互
  city_panel.rs   # 城池详情面板
  style.rs        # egui 主题、字体、面板样式
  i18n.rs         # 翻译器封装
  audio.rs        # BGM 管理
  settings.rs     # 设置应用
  display_settings.rs  # 显示配置
```

`state.rs` 中的 `GameUiState` 是 core 层的核心资源，持有 Bevy `#[derive(Resource)]`。它包含：
- 当前屏幕（主菜单/游戏内）
- UI 选中状态（城池、武将、命令类别）
- 游戏状态引用（`Option<GameState>`）
- 设置和消息

## 依赖方向

```
main.rs ──▶ core ──▶ game
               │
               ▼
          Bevy / egui / rodio / i18n-embed
```

```
bin/build_history_db ──▶ game
bin/import_three_kingdoms ──▶ game
```

辅助二进制直接依赖 game 层，不经过 core，因为它们不需要 UI。
