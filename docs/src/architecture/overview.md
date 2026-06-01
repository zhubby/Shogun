# 架构概览

Shogun 采用**两层架构**：纯净的领域逻辑层（`game`）和引擎/UI 层（`core`）。

## 分层图

```
┌─────────────────────────────────────────────┐
│                   core                      │
│  Bevy App · egui UI · Audio · i18n · Input  │
│                                             │
│  state.rs  hud.rs  menu.rs  map.rs  ...     │
├─────────────────────────────────────────────┤
│               公开 API 调用                  │
│         game::queue_player_command()         │
│         game::resolve_command_batch()        │
│         game::ScenarioData::build_game()     │
│         game::SaveManager::save_slot()       │
│         ...                                  │
├─────────────────────────────────────────────┤
│                   game                      │
│  model · commands · city · officer · ai     │
│  technology · diplomacy · combat · save     │
│  history_db · scenario · map_boundary       │
│                                             │
│  零 Bevy 依赖 · 零 egui 依赖 · 纯 Rust      │
└─────────────────────────────────────────────┘
```

## 核心原则

1. **game 层不依赖任何引擎** — `src/game/` 中没有 Bevy、egui 或 winit 的 import。这意味着领域逻辑可以独立编译、测试和复用。

2. **core 通过公开 API 调用 game** — UI 层不直接操纵 game 内部状态，而是通过 `queue_player_command()`、`finish_turn()` 等导出函数。

3. **单一 crate，逻辑分层** — 项目没有使用 Cargo workspace，而是通过模块可见性（`pub` vs `pub(super)`）来维护边界。这降低了依赖管理的复杂度。

4. **确定性优先** — 在影响 UI、存档和测试的场合使用 `BTreeMap`/`BTreeSet` 而非 `HashMap`，保证迭代顺序一致。

## 数据流

```
玩家操作 ──▶ core/actions.rs ──▶ game::queue_player_command()
                                        │
                                        ▼
                                   GameState.pending_commands
                                        │
        结束回合 ──▶ game::resolve_command_batch()
                                        │
                                        ▼
                              validate → apply → income → advance
                                        │
                                        ▼
                                   TurnReport ──▶ UI 显示
```

## 入口点

- **`src/main.rs`** — 极简入口，仅调用 `shogun::core::run()`
- **`src/lib.rs`** — 导出 `build_info`、`core`、`game` 三个模块
- **`src/bin/build_history_db.rs`** — 辅助二进制，重建本地 SQLite 数据库
- **`src/bin/import_three_kingdoms.rs`** — 从 CTK 语料导入历史人物
