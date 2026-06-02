# 武将与官职

武将是势力的核心资源，担任太守、将领和朝官等角色。

## Officer 结构

每个武将有：
- **基础属性** — 统率、武力、智力、政治、魅力
- **状态** — `OfficerStatus`（Active/Unavailable/Dead 等）
- **位置** — 当前所在城池
- **忠诚度** — 0-100，影响叛变概率
- **官职** — 当前担任的官职
- **关系** — 与其他武将的关系（君臣、亲子、夫妻、兄弟、结义、仇敌）
- **标签** — 来自历史资料库的规范标签，用于资料展示和武将浏览器筛选

## 官职系统

游戏包含 32 个官职，分为多个品秩等级：

| 品秩 | 代表官职 | 效果方向 |
|------|----------|----------|
| 万石 (WanShi) | 太傅、大将军 | 全局加成 |
| 中二千石 (ZhongErQianShi) | 太尉、司徒、司空、九卿 | 专项加成 |
| 比二千石 (BiErQianShi) | 骠骑/车骑/卫将军 | 军事加成 |
| 千石及以下 | 太守、郡守 | 城池加成 |

每个官职有一个 `OfficialPostEffect`，描述对经济、军事、治安等方面的加成。

### 品秩与忠诚

官职品秩影响武将忠诚度加成——品秩越高，忠诚加成越大。同时，高品秩官职有俸禄加成。

## 武将关系

```rust
pub enum OfficerRelationshipKind {
    Lord,        // 君臣
    Parent,      // 亲子
    Adoptive,    // 养亲子
    Spouse,      // 夫妻
    Sibling,     // 兄弟
    Sworn,       // 结义
    Enemy,       // 仇敌
}
```

关系数据来自历史数据库，第一版仅用于资料展示，不参与规则判定。

## 武将标签

标签由历史资料库的 `officer_tag_definitions`、`officer_tag_aliases` 和 `officer_tags` 提供。`OfficerProfile.tags` 保存规范标签 ID，例如 `role:ruler`、`role:general`、`affiliation:shu_han`、`basis:history`。

武将浏览器支持按标签分面筛选：同一分类内为 OR，不同分类之间为 AND。例如同时选择 `role:general`、`role:administrator` 和 `affiliation:shu_han`，会返回蜀汉阵营中属于武将或文官的人员。

## 性别

武将性别保留男/女两种值，来自历史数据来源。

<!-- TODO: 补充武将能力值对命令效果的具体影响公式 -->
<!-- TODO: 补充忠诚度变化和叛变机制 -->
