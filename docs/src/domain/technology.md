# 科技树

科技系统为势力提供长期成长路径，分为军事和内政两个分支。

## 科技目录

科技静态数据来自 SQLite 历史资料库，不再硬编码在 Rust 常量中。`migrations/011_technology_catalog.sql` 定义：

- `technologies`：科技 ID、分支、名称、研究月数、立项金钱、显示说明、图标键、显示顺序、AI 优先级。
- `technology_prerequisites`：科技前置关系。
- `technology_effects`：科技效果类型和数值。

运行时由 `SqliteHistoricalCatalog::technology_catalog` 加载为 `TechnologyCatalog`，再注入 `GameState.technology_catalog`。存档只保存势力的动态研究状态，不保存静态科技目录。

## 科技列表

共 28 项科技，分布在两个分支：

### 军事分支 (Military)

乡勇操练、军械整备、斥候路网、铁制兵器、严整军纪、坚壁戍防、粮道护送、骑步协同、城门火攻、守备轮戍、军府仓储、攻城器械、将校考课、都督府制

### 内政分支 (Domestic)

户籍清丈、水利勘测、市籍整理、仓廪制度、平准市易、工匠名籍、灌渠修复、商旅关津、官署文书、常平仓法、工坊行会、郡县考课、漕运税制、度支尚书

## 研究流程

```
start_research(faction_id, technology_id)
  ├─ 检查前置科技
  ├─ 检查势力总金钱
  ├─ 一次性扣除立项费用
  └─ 设置 active research

advance_active_research()  (每月结算)
  ├─ 增加进度
  └─ 完成 → 加入 completed 集合
```

## FactionTechnologyState

```rust
pub struct FactionTechnologyState {
    pub active: Option<TechnologyId>,          // 当前研究，字符串科技 ID
    pub progress: BTreeMap<TechnologyId, u8>,  // 各科技进度
    pub funded: BTreeSet<TechnologyId>,        // 已付立项费用
    pub completed: BTreeSet<TechnologyId>,     // 已完成
}
```

## 科技加成

完成的科技通过 `TechnologyBonuses` 影响全局：

- 经济加成（农业/商业/开发效率）
- 军事加成（攻击/防御/攻城百分比）
- 后勤加成（行军时间减少）
- 征兵折扣
- 训练加成
- 战损减免与伤兵转化加成

AI 势力在每月结算开始时自动选择研究目标（`begin_ai_research`）。

科技效果由 `technology_effects.effect_kind` 映射到 `TechnologyBonuses` 字段；新增科技或调整数值应通过 SQLx 迁移修改资料库数据。
