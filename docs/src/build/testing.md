# 测试策略

项目使用集成测试为主，放在 `tests/` 目录下。

## 测试套件

| 测试文件 | 覆盖范围 | 命令 |
|----------|----------|------|
| `gameplay.rs` | 游戏规则、AI、存档、命令系统 | `cargo test --test gameplay` |
| `history_db.rs` | SQLite 迁移、数据完整性、外键 | `cargo test --test history_db` |
| `map_boundaries.rs` | 地图边界资产验证 | `cargo test --test map_boundaries` |

## 测试原则

- **行为命名** — 测试名描述行为而非实现，如 `cannot_attack_non_adjacent_city`
- **回归覆盖** — bug 修复必须有回归测试
- **规则变更** — 规则修改必须更新相关测试
- **数据完整性** — 数据库测试检查外键、索引、数量、来源

## gameplay 测试覆盖

- 命令验证（每城一令、每武将一行动）
- 战斗结算（兵种克制、城防加成）
- 外交（停战、宣战约束）
- 科技研究（前置条件、完成触发）
- 存档/读档（版本兼容、多槽位）
- AI 决策（规则 AI 的基本行为）
- 生命事件（登场、死亡、效忠）

## history_db 测试覆盖

- 基线建库（迁移执行）
- 外键完整性
- 索引存在性
- 来源化人物数量
- 无占位符数据
- 性别/生平/外部 ID/关系检查
- 剧本构建
- 太守引用
- 生命事件幂等性

## CI 检查清单

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```
