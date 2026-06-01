# 命令系统

命令系统是游戏玩法的核心机制。玩家和 AI 通过提交 `Command` 来控制势力行动，系统在每个回合结束时统一验证和执行。

## Command 结构

```rust
pub struct Command {
    pub issuer_faction_id: FactionId,    // 发起势力
    pub city_id: CityId,                  // 执行城池
    pub officer_id: Option<OfficerId>,    // 执行武将（可选）
    pub kind: CommandKind,                // 命令类型
}
```

## 命令类型（CommandKind）

| 类型 | 说明 | 消耗 |
|------|------|------|
| `Develop { focus }` | 开发城池（农业/商业/防御/治安） | 武将行动 |
| `UpgradeCityCore` | 升级城镇核心（最高 10 级） | 武将行动 + 资源 |
| `BuildFacility { kind }` | 建设设施（12 种，最高 5 级） | 武将行动 + 资源 |
| `Recruit { kind, amount }` | 征兵（步/骑/弓） | 武将行动 + 金钱 |
| `Train` | 训练部队 | 武将行动 |
| `AppointGovernor { target }` | 任命太守 | 城池命令 |
| `Transfer { target, troops, officers }` | 调动到友城 | 城池命令 |
| `Expedition { target, assignments }` | 出征敌城 | 城池命令 |
| `Diplomacy { target, proposal }` | 外交行动 | 城池命令 |

## 命令不变量

系统维护两个关键约束：

1. **每城一令** — 一个城池每月只能提交一条命令
2. **每武将一行动** — 一个武将每月只能参与一条命令

这些约束通过 `CommandReservations` 在验证阶段检查：

```rust
struct CommandReservations {
    city_ids: BTreeSet<CityId>,       // 已预订的城池
    officer_ids: BTreeSet<OfficerId>, // 已预订的武将
}
```

## 命令生命周期

```
玩家/AI 提交
    │
    ▼
queue_player_command()          ◀── 玩家路径
  ├─ 检查游戏未结束
  ├─ 检查是玩家势力
  ├─ 验证已有预订
  ├─ validate_command()
  └─ push 到 pending_commands

    ──── 或 ────

resolve_command_batch()         ◀── 回合结算
  ├─ begin_ai_research()
  ├─ 遍历命令：
  │   ├─ validate_command() → Ok → apply_command()
  │   └─ validate_command() → Err → 记录警告
  ├─ resolve_due_army_movements()  行军到达
  ├─ apply_monthly_income()        月度收入
  ├─ advance_research_and_report() 科技推进
  ├─ clear pending_commands
  ├─ refresh_status()              胜败检查
  ├─ advance_month()               时间推进
  ├─ apply_due_life_events()       生命事件
  └─ append_turn_summary()
```

## 玩家 vs AI 的命令路径

- **玩家**通过 `queue_player_command()` 逐条提交，立即验证约束。如果违反（如城池已有命令），返回 `CommandError`。
- **AI**通过 `AiProvider::decide()` 一次性生成一批命令，在 `finish_turn()` 中与玩家命令一起交给 `resolve_command_batch()`。AI 命令也要通过 `validate_command()`，无效的会被拒绝而不是静默执行。

## 验证规则示例

- 开发：城池必须有可用武将
- 征兵：城池必须有武将、足够金钱、不超人口上限
- 调动/出征：目标必须相邻、目标不是己方（出征）或必须是己方（调动）
- 外交：目标势力必须存在、不能对自己外交、不能违反停战
