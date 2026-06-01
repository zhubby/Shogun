# 核心状态：GameState

`GameState` 是整个游戏的核心状态，定义在 `src/game/model.rs`。它是一个 `#[derive(Serialize, Deserialize)]` 结构，可以直接序列化为存档 JSON。

## 结构概览

```rust
pub struct GameState {
    pub version: u32,                    // 存档版本（SAVE_VERSION）
    pub scenario_id: String,             // 当前剧本 ID
    pub scenario_name: String,           // 剧本显示名
    pub year: i32,                       // 当前年份
    pub month: u8,                       // 当前月份 (1-12)
    pub turn: u32,                       // 回合计数
    pub player_faction_id: FactionId,    // 玩家势力
    pub factions: BTreeMap<FactionId, Faction>,   // 所有势力
    pub cities: BTreeMap<CityId, City>,           // 所有城池
    pub officers: BTreeMap<OfficerId, Officer>,   // 所有武将
    pub roads: Vec<Road>,                // 道路网络
    pub diplomacy: BTreeMap<String, DiplomaticRelation>,  // 外交关系
    pub pending_commands: Vec<Command>,  // 本月待处理命令
    pub army_movements: Vec<ArmyMovement>,  // 行军中部队
    pub technologies: BTreeMap<FactionId, FactionTechnologyState>,  // 科技进度
    pub applied_event_ids: BTreeSet<String>,  // 已应用的生命事件
    pub reports: Vec<TurnReport>,        // 历史月报
    pub status: GameStatus,              // 游戏状态
}
```

## 设计决策

### 为什么用 BTreeMap 而不是 HashMap

`factions`、`cities`、`officers`、`diplomacy`、`technologies` 全部使用 `BTreeMap`：

- **确定性迭代顺序** — 影响 UI 显示、存档内容和测试结果
- **序列化稳定** — JSON 输出在不同运行间保持一致
- **代价可接受** — 策略游戏的数据规模（数十个势力、数百个城池/武将）下 BTreeMap 性能足够

### GameStatus 三态

```rust
pub enum GameStatus {
    Running,
    Victory { reason: String },
    Defeat { reason: String },
}
```

游戏只有三种终态：进行中、胜利（统一天下）、失败（失去所有城池）。`reason` 字段用于 UI 显示具体原因。

### Faction 与 Controller

```rust
pub struct Faction {
    pub id: FactionId,
    pub name: String,
    pub ruler_id: OfficerId,
    pub color: [f32; 3],
    pub selectable: bool,
    pub controlled_by: Controller,
}

pub enum Controller {
    Player,
    RuleAi,
}
```

每个势力有一个统治者（`ruler_id` 指向一个武将）、一个 RGB 颜色（用于地图着色）和一个控制器。目前只有两种控制器：玩家和规则 AI。

### ID 类型

```rust
// src/game/ids.rs
pub type CityId = String;
pub type FactionId = String;
pub type OfficerId = String;
```

当前使用 `String` 作为 ID 类型，简化了序列化和查询。未来如果需要性能优化，可以考虑替换为 interned ID 或 `u32`。

## 关键查询方法

| 方法 | 用途 |
|------|------|
| `cities_for_faction(id)` | 获取势力名下城池 |
| `officers_in_city(id)` | 获取城中活跃武将 |
| `faction_alive(id)` | 势力是否还有城池 |
| `are_adjacent(a, b)` | 两城是否相邻（道路连接） |
| `road_distance_li(a, b)` | 道路距离（里） |
| `travel_months_between(a, b)` | 行军月数 |
| `relation(a, b)` | 查询外交关系 |
| `pending_city_ids()` | 已下命令的城池集合 |
| `pending_officer_ids()` | 已下命令的武将集合 |
| `advance_month()` | 推进一月 |
| `refresh_status()` | 检查胜败条件 |
