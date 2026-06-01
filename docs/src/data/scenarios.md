# 剧本与场景

剧本（Scenario）定义了一局游戏的初始状态——哪些势力存在、城池归属、武将分布、外交关系。

## 双数据源

游戏支持两种剧本来源：

| 来源 | 格式 | 优先级 | 说明 |
|------|------|--------|------|
| 历史数据库 | SQLite | 高 | 完整历史数据，含生命事件 |
| JSON 后备 | `early_three_kingdoms.json` | 低 | 内嵌在二进制中，无需数据库即可运行 |

### JSON 后备剧本

`ScenarioData` 从 JSON 加载，并通过 `include_str!` 内嵌到二进制中：

```rust
const DEFAULT_SCENARIO_JSON: &str =
    include_str!("../../assets/scenarios/early_three_kingdoms.json");
```

这保证了即使没有本地数据库，游戏也能启动和运行。

### 历史数据库剧本

`SqliteHistoricalCatalog::build_game()` 从数据库构建完整的 `GameState`，包含：
- 剧本快照（势力、城池归属、外交）
- 武将初始状态和位置
- 生命事件时间线

## ScenarioData 结构

```rust
pub struct ScenarioData {
    pub id: String,
    pub name: String,
    pub start_year: i32,
    pub start_month: u8,
    pub player_selectable_factions: Vec<FactionId>,
    pub factions: Vec<FactionSeed>,
    pub cities: Vec<CitySeed>,
    pub officers: Vec<OfficerSeed>,
    pub roads: Vec<Road>,
    pub diplomacy: Vec<DiplomaticRelation>,
}
```

`build_game(player_faction_id)` 将种子数据转化为完整的 `GameState`。

<!-- TODO: 补充 Seed 类型的详细字段、新剧本的创建方式 -->
