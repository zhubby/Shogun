# 科技树

科技系统为势力提供长期成长路径，分为军事和内政两个分支。

## 科技结构

```rust
pub struct TechnologySpec {
    pub id: TechnologyId,
    pub branch: TechnologyBranch,    // Military 或 Domestic
    pub name: &'static str,
    pub turns: u8,                    // 研究所需月数
    pub gold_cost: i32,               // 金钱消耗
    pub prerequisites: &'static [TechnologyId],  // 前置科技
    pub effect: &'static str,         // 效果描述
}
```

## 科技列表

共 27 项科技，分布在两个分支：

### 军事分支 (Military)

民兵操练、军械后勤、斥候道路、铁制兵器、严明军纪、坚固驻防、粮草护送、协同作战、城门火计、轮防制度、军粮仓储、攻城器械

### 内政分支 (Domestic)

武将考绩、大郡制度、户籍登记、水利勘察、市场登记、仓储制度、平准法、工匠登记、运河修复、通关文牒、行政记录、常平仓、工坊行会、郡国评议、运河税制、度支尚书

## 研究流程

```
start_research(faction_id, technology_id)
  ├─ 检查前置科技
  ├─ 检查金钱
  └─ 设置 active research

advance_active_research()  (每月结算)
  ├─ 扣除月度研究费用
  ├─ 增加进度
  └─ 完成 → 加入 completed 集合
```

## FactionTechnologyState

```rust
pub struct FactionTechnologyState {
    pub active: Option<TechnologyId>,    // 当前研究
    pub progress: BTreeMap<TechnologyId, u8>,  // 各科技进度
    pub funded: BTreeSet<TechnologyId>,  // 已拨款
    pub completed: BTreeSet<TechnologyId>,  // 已完成
}
```

## 科技加成

完成的科技通过 `TechnologyBonuses` 影响全局：

- 经济加成（农业/商业/开发效率）
- 军事加成（攻击/防御/攻城百分比）
- 后勤加成（行军时间减少）
- 征兵折扣
- 训练加成

AI 势力在每月结算开始时自动选择研究目标（`begin_ai_research`）。

<!-- TODO: 补充每项科技的具体数值效果 -->
