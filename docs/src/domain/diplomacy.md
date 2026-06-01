# 外交系统

外交系统管理势力间的关系，包括好感度、停战和宣战。

## DiplomaticRelation

```rust
pub struct DiplomaticRelation {
    pub faction_a: FactionId,
    pub faction_b: FactionId,
    pub score: i16,              // 好感度分数
    pub truce_until_turn: Option<u32>,  // 停战截止回合
}
```

外交关系存储在 `BTreeMap<String, DiplomaticRelation>` 中，key 由 `diplomacy_key(a, b)` 生成，保证 `A→B` 和 `B→A` 指向同一条记录（按字典序排列）。

## 外交行动

| 提案 | 说明 |
|------|------|
| `ImproveRelations` | 改善关系（消耗金钱，增加好感度） |
| `Truce` | 缔结停战（双方约定一段时间内不攻击） |
| `DeclareWar` | 宣战（可能降低好感度） |

## 约束

- 不能对自己外交
- 目标势力必须存在且存活
- 停战期间不能宣战

<!-- TODO: 补充好感度计算公式、停战持续月数、AI 外交决策逻辑 -->
