# 地图边界

地图边界（`src/game/map_boundary.rs`）是纯美术资产，用于在 UI 地图上绘制州郡轮廓。

## 数据格式

边界数据存储在 `assets/data/map_boundaries.json`：

```rust
pub struct MapBoundaryCatalog {
    pub version: u32,
    pub notes: String,
    pub boundaries: Vec<MapBoundary>,
}
```

每个 `MapBoundary` 包含：
- 名称（州或郡）
- 级别（`MapBoundaryLevel`：Province / Commandery）
- 有效年份范围
- 多边形顶点坐标

## 设计决策

- **与 SQLite 分离** — 边界是美术资产，不参与游戏规则，避免美术迭代影响存档
- **年代感知** — `boundaries_for_year()` 返回特定年份有效的边界（历史行政区随时间变化）
- **不参与判定** — 游戏的地域判定由道路网络（`Road`）决定，边界只用于绘制

## 数据来源

参考 CHGIS 类历史行政区口径手工校订，坐标对齐游戏地图坐标系。

<!-- TODO: 补充地图坐标系说明、边界编辑工具 -->
