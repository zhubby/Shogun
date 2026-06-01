# 存档系统

存档系统（`src/game/save.rs`）负责将 `GameState` 序列化到磁盘，支持多槽位和版本管理。

## SaveManager

```rust
pub struct SaveManager {
    base_dir: PathBuf,
}
```

存档目录默认位于 `ProjectDirs::from("", "", "Shogun").data_local_dir()/saves`，也可以通过 `new()` 指定自定义路径。

## 存档流程

```
save_slot(slot_id, display_name, &game_state)
  ├─ validate_slot_id()         检查 ID 合法性
  ├─ 创建 slots 目录
  ├─ 序列化 GameState 为 JSON
  ├─ 写入 slots/{slot_id}.json
  └─ 更新 manifest.json
```

## 元数据

每个存档槽位有一个 `SaveSlotMeta`：

```rust
pub struct SaveSlotMeta {
    pub slot_id: String,
    pub display_name: String,
    pub scenario_id: String,
    pub scenario_name: String,
    pub player_faction_id: String,
    pub turn: u32,
    pub year: i32,
    pub month: u8,
    pub saved_at_unix: u64,
}
```

元数据独立于完整存档存储，使得 `list_slots()` 可以快速列出所有存档而不需要反序列化完整的 `GameState`。

## 版本管理

```rust
pub const SAVE_VERSION: u32 = 5;
```

`SAVE_VERSION` 标记存档格式版本。每次不兼容的修改都应该递增此值。

### 向后兼容策略

- **优先使用 additive 字段** — 新增字段使用 `#[serde(default)]` 提供默认值
- **避免删除字段** — 保留旧字段，标记为废弃
- **不兼容变更** — 递增 `SAVE_VERSION`，`SaveManager` 在加载时检查版本
- **manifest 兼容** — `list_slots()` 应该能读取旧版 manifest

## 设计决策

| 决策 | 理由 |
|------|------|
| JSON 格式 | 人类可读、调试友好、serde 直接支持 |
| 多槽位 | 策略游戏通常需要并行多局 |
| manifest 分离 | 快速列表，不需要加载完整存档 |
| Unix 时间戳 | 排序简单，无时区问题 |
