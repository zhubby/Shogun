# 规则 AI

`RuleBasedAiProvider` 是当前唯一的 AI 实现，基于规则的决策逻辑。

## 决策原则

规则 AI 遵循简单的优先级策略：

1. **防御优先** — 兵力不足时优先征兵
2. **经济开发** — 有空闲武将时开发城池
3. **扩张** — 兵力充裕时出征弱敌
4. **外交响应** — v1 不主动发起外交，但会作为玩家提案目标按领域规则接受或拒绝

## 科技研究

AI 势力在每月结算开始时自动选择研究目标（`choose_ai_research`）：

- 检查可用科技（前置条件已满足）
- 根据当前局势选择军事或内政科技
- 自动开始研究

## 与玩家的一致性

规则 AI 产出的命令通过与玩家相同的验证管道：

```rust
// AI 命令也要过 validate_command
match validate_command(state, &command, &mut reservations) {
    Ok(()) => apply_command(state, &command, &mut report),
    Err(error) => report.warning(format!("命令被拒绝: ...")),
}
```

这保证了 AI 不能作弊——不能执行玩家不能执行的命令。

## 限制

当前规则 AI 的已知限制：

- 不考虑战争迷雾（能看到所有城池状态）
- 不做长期战略规划
- 不主动提交合纵外交提案
- 不利用外交协同（不会配合其他 AI 势力）
- 不根据玩家行为调整策略

<!-- TODO: 补充 AI 决策树详细逻辑、难度等级设计 -->
