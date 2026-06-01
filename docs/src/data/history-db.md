# 历史资料库

历史资料库（`src/game/history_db.rs`）是游戏的静态数据来源，使用 SQLite 存储历史武将、城池、势力、剧本快照和生命事件。

## 设计思路

### 静态数据 vs 动态状态

| 层面 | 存储位置 | 内容 |
|------|----------|------|
| 静态资料 | `database.sqlite` | 城市资料、武将资料、势力、道路、剧本快照、生命事件 |
| 动态局面 | 存档 JSON | 当前城池归属、资源、武将状态、命令、外交、报告 |
| 地图边界 | `map_boundaries.json` | 州郡轮廓（美术资产，不参与规则） |

这种分离使得：
- 历史数据修正不影响已有存档
- 地图美术迭代不触及数据库
- 存档体积小（只保存动态变化）

### 数据库文件位置

运行时数据库默认位于 `ProjectDirs::data_local_dir()/database.sqlite`，不在仓库中追踪。通过辅助二进制重建：

```bash
cargo run --bin build_history_db
```

### 迁移策略

使用 SQLx 迁移（`./migrations/` 目录）：

- 当前基线为 `001_initial_history.sql`，包含表结构、索引和种子数据
- 新增或修正历史资料时，添加新的迁移文件，不修改已发布的迁移
- 游戏启动时自动执行未应用的迁移
- 迁移记录使用 SQLx 的 `_sqlx_migrations` 表

当前武将目录固定为 420 名命名人物，男 350、女 70。扩充数据通过 `003_expand_officer_catalog.sql` 增加 `expansion_003` 标记人物，并删除匿名低质量补充人物。女性人物可以使用稳定通称（夫人、皇后、太后、大乔、小乔等），但不收录无法作为展示名的“某从兄”“某妻”“无名氏”占位描述。

## HistoricalCatalog trait

```rust
pub trait HistoricalCatalog:
    CityCatalog<Error = HistoryDbError>
    + OfficerCatalog<Error = HistoryDbError>
{
    fn scenarios(&self) -> Result<Vec<HistoricalScenario>, HistoryDbError>;
    fn selectable_factions(&self, scenario_id: &str) -> Result<Vec<Faction>, HistoryDbError>;
    fn build_game(&self, scenario_id: &str, player_faction_id: &str) -> Result<GameState, HistoryDbError>;
    fn life_events_until(&self, year: i32, month: u8) -> Result<Vec<LifeEvent>, HistoryDbError>;
}
```

这个 trait 是 game 层对历史数据的唯一接口。`SqliteHistoricalCatalog` 是 SQLite 实现，而测试可以使用 mock 实现。

## 生命事件（LifeEvent）

生命事件驱动武将在时间线上的登场、效忠、移动和死亡：

```rust
pub enum LifeEventKind {
    Appear,            // 武将登场（可被招募）
    ServeFaction,      // 加入势力
    MoveToCity,        // 移动到城池
    BecomeUnavailable, // 变得不可用
    Die,               // 死亡
}
```

生命事件在每月结算末尾应用（`apply_due_life_events`），已应用的事件 ID 记入 `GameState.applied_event_ids` 防止重复。

## 数据来源与授权

| 来源 | 标记 | 说明 |
|------|------|------|
| CTK 语料 | `characters_of_the_three_kingdoms` | MIT 许可的人物数据 |
| 人工校订 | `manual_curated` | 手工验证和补充；`expansion_003` 使用原创短句摘要，不复制外部百科正文 |

不确定的信息通过 `confidence`（High/Medium/Low）和 `notes` 字段标注。

## 必要表

```
cities, factions, officers, officer_external_ids,
officer_relationships, roads, scenarios,
scenario_faction_states, scenario_city_states,
officer_life_events, scenario_diplomacy
```
