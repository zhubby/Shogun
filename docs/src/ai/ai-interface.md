# AI 接口设计

AI 系统通过 trait 抽象，支持规则 AI、mock 测试和未来可能的 LLM AI 实现。

## AiProvider trait

```rust
pub trait AiProvider {
    fn decide(&self, request: AiDecisionRequest) -> AiDecisionResponse;
}
```

## 请求/响应

```rust
pub struct AiDecisionRequest {
    pub turn: u32,
    pub year: i32,
    pub month: u8,
    pub faction_id: FactionId,
    pub cities: Vec<City>,
    pub officers: Vec<Officer>,
    pub roads: Vec<Road>,
    pub army_movements: Vec<ArmyMovement>,
    pub diplomacy: Vec<DiplomaticRelation>,
}

pub struct AiDecisionResponse {
    pub commands: Vec<Command>,
    pub diagnostics: Vec<String>,
}
```

## 设计决策

| 决策 | 理由 |
|------|------|
| 请求/响应模式 | AI 不直接操纵 GameState，只返回命令列表 |
| 完整状态快照 | AI 收到所有城池/武将数据，不受"战争迷雾"限制 |
| 诊断字段 | AI 可以返回决策理由，便于调试 |
| 命令仍需验证 | AI 返回的命令要通过与玩家相同的 `validate_command()` |
| `AiDecisionRequest::from_state()` | 从 GameState 提取 AI 需要的数据切片 |

## MockAiProvider

测试用的 mock 实现，接受预定义的 JSON 响应：

```rust
pub struct MockAiProvider {
    pub scripted: BTreeMap<FactionId, String>,  // faction → JSON
}
```

## 调用时机

1. 每月结算开始 → `begin_ai_research()` — AI 选择研究目标
2. `finish_turn()` → 为每个 AI 势力调用 `decide()` → 收集命令
3. AI 命令与玩家命令一起交给 `resolve_command_batch()`

<!-- TODO: 补充 AI 可见性设计（是否应该模拟信息不对称） -->
