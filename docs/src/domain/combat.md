# 战斗与行军

## 兵种系统

游戏有三种兵种，形成石头剪刀布的克制关系：

```
步兵 (Infantry) → 克制 → 骑兵 (Cavalry)
骑兵 (Cavalry)  → 克制 → 弓兵 (Archers)
弓兵 (Archers)  → 克制 → 步兵 (Infantry)
```

## TroopPool

```rust
pub struct TroopPool {
    pub infantry: u32,
    pub cavalry: u32,
    pub archers: u32,
}
```

兵力按种类管理，支持按比例损耗（`loss_pool`）、按比率增援（`add_total_preserving_ratio`）等操作。

## 行军系统

### 距离计算

```rust
pub const MAP_COORDINATE_LI: f32 = 4.0;      // 坐标单位 → 里
pub const MARCH_LI_PER_MONTH: u32 = 500;     // 每月行军速度
pub const MAX_TRAVEL_MONTHS: u32 = 3;        // 最大行军月数
```

两城间的行军月数 = `distance_li / 500`，上限 3 个月。

### ArmyMovement

行军中的部队用 `ArmyMovement` 表示：

```rust
pub struct ArmyMovement {
    pub kind: ArmyMovementKind,    // Transfer（调动）或 Expedition（出征）
    pub source_city_id: CityId,
    pub target_city_id: CityId,
    pub commander_id: OfficerId,
    pub troops: TroopPool,
    pub distance_li: u32,
    pub departure_turn: u32,
    pub arrival_turn: u32,
}
```

- **调动（Transfer）** — 在友城间移动武将和兵力，无战斗；可用于向己方被围城市驰援
- **出征（Expedition）** — 进攻敌方城池，到达后触发战斗；被围城市不能主动出征

`Expedition.siege_started_turn` 表示部队已经进入围城阶段。同一座城同一时间只能由一个势力围城；同一围城方可以继续派出征队加入围城，其他势力到达时会撤回友城。

### 远征编制

出征支持三将编制：

```rust
pub struct ExpeditionAssignment {
    pub officer_id: OfficerId,
    pub role: ExpeditionRole,    // Commander（主将）或 Deputy（副将）
    pub troop_kind: TroopKind,   // 统率兵种
    pub troops: u32,
}
```

一名主将 + 最多两名副将，各自统率一个兵种的部队。

## 战斗结算

战斗在行军到达时触发（`resolve_due_army_movements`）。攻防双方的攻击力受以下因素影响：

- 武将统率/武力
- 兵种克制
- 科技加成（攻击/防御百分比）
- 城防加成（守方）
- 训练度

<!-- TODO: 补充战斗伤害公式、攻城机制、战后处置 -->
