# 剧本与场景

剧本（Scenario）定义一局游戏的初始状态：哪些势力存在、城池归属、武将分布和外交关系。

## 单一数据源

游戏只从 SQLite 历史资料库构建新局。`SqliteHistoricalCatalog::build_game()` 读取：

- `scenarios`：剧本名称与年月。
- `scenario_faction_states`：剧本内存在的势力、君主和可选状态。
- `scenario_city_states`：城池归属、资源、兵力、发展值和太守。
- `scenario_diplomacy`：初始外交关系。
- `officer_life_events`：截至剧本年月应出现或退场的武将状态。

仓库不再维护 JSON 后备剧本，也不再把小剧本内嵌进二进制。若本机 `database.sqlite` 缺失，运行时会从 `migrations/` 执行 SQLx migrations 创建资料库；若资料库不可用，新游戏入口会提示错误而不是切换到备用数据。

## 构建流程

```rust
let catalog = SqliteHistoricalCatalog::open_default()?;
let game = catalog.build_game("ad200", "liu_bei")?;
```

构建出的 `GameState` 包含完整地图、势力、武将、道路、外交、科技初始状态和已应用履历事件 ID。后续存档只保存动态局面；静态历史资料仍由 SQLite 资料库提供。
