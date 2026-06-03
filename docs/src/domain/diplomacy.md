# 外交系统

外交系统管理势力间的关系，包括好感度、停战、过路权、宣战和资源交换。外交不再是城池命令，而是势力级月度提案。

## DiplomaticRelation

```rust
pub struct DiplomaticRelation {
    pub faction_a: FactionId,
    pub faction_b: FactionId,
    pub score: i16,              // 好感度分数
    pub truce_until_turn: Option<u32>,  // 停战截止回合
    pub passage_rights: BTreeMap<FactionId, u32>,  // 可过路势力及截止回合
}
```

外交关系存储在 `BTreeMap<String, DiplomaticRelation>` 中，key 由 `diplomacy_key(a, b)` 生成，保证 `A→B` 和 `B→A` 指向同一条记录（按字典序排列）。

## 外交提案

玩家通过 `queue_player_diplomacy()` 提交 `DiplomacyOrder`，进入 `GameState.pending_diplomacy`。外交提案按月结算，但不占用城池命令和武将行动。

| 提案 | 说明 |
|------|------|
| `ImproveRelations` | 修好，只允许我方给出资源，必定成功并提升关系 |
| `RequestPeace` | 求和，成功后停战 12 个月，关系至少回到 0 |
| `Truce` | 停战，成功后 6 个月内不能互相出征 |
| `DeclareWar` | 宣战，立即清除停战和过路权，关系变为 -80 |
| `PassageRight` | 借道，目标势力授予发起方 12 个月单向过路权 |
| `ResourceExchange` | 互市，按条款交换金、粮、建材 |

资源条款使用 `ResourceBundle { gold, food, materials }`。价值公式为 `gold + food / 2 + materials * 2`，接受分为当前关系加条款净值除以 100。

敌对关系沿用宣战语义表示：关系值为 -80，且没有停战和双方过路权。宣战会立即进入敌对；出征敌城在出发时也会将进攻方和目标方标记为敌对。

## 结算顺序

外交提案在城市命令前结算。这样当月达成停战会阻止同月出征，而当月宣战会先解除停战，再允许同月出征通过城市命令验证。

过路权只在出发时验证。调动目标仍必须是己方城市；出征目标仍必须是非己方城市且没有有效停战。路线可以经过己方城市和向我方开放过路权的第三方城市，行军距离按整条路线累计。出征或持续围城会保持双方敌对；如果有效停战成立，出征队会撤回而不是继续围城。

## 约束

- 不能对自己外交
- 目标势力必须存在且存活
- 同一发起方每月对同一目标最多一个待处理外交提案
- 玩家必须选择己方交付城和接收城
- 给出资源必须由交付城支付，要求资源成功后进入接收城
- AI v1 不主动发起外交，但会作为目标按规则接受或拒绝玩家提案
