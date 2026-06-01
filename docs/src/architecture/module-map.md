# 模块地图

下表列出 `src/game/` 下每个模块的职责：

| 模块 | 文件 | 职责 |
|------|------|------|
| **model** | `model.rs` | 核心状态 `GameState`、`Faction`、`Command`、`CommandKind`、`TroopPool`、`DiplomaticRelation`、`TurnReport`、`SAVE_VERSION` |
| **commands** | `commands.rs` | 命令验证、队列、月度结算（`queue_player_command`、`resolve_command_batch`）、战斗、收入、生命事件 |
| **city** | `city.rs` | `City`、`CityProfile`、`FacilityKind`（12 种设施）、城池等级、经济效果计算 |
| **officer** | `officer.rs` | `Officer`、`OfficerStats`、官职系统（32 个官职）、忠诚度、武将关系、生命阶段 |
| **technology** | `technology.rs` | `TechnologyId`（27 项科技）、军事/内政双分支、研究进度、科技加成 |
| **ai** | `ai.rs` | `AiProvider` trait、`AiDecisionRequest/Response`、`RuleBasedAiProvider`、`MockAiProvider` |
| **save** | `save.rs` | `SaveManager`、存档槽位、版本化序列化、`ProjectDirs` 路径 |
| **history_db** | `history_db.rs` | `HistoricalCatalog` trait、`SqliteHistoricalCatalog`、`LifeEvent`、迁移管理、剧本构建 |
| **ids** | `ids.rs` | 类型别名：`CityId`、`FactionId`、`OfficerId` 等 |
| **map_boundary** | `map_boundary.rs` | `MapBoundaryCatalog` JSON 资产、州郡轮廓、`TerritoryCell` |

`src/core/` 模块：

| 模块 | 文件 | 职责 |
|------|------|------|
| **mod** | `mod.rs` | `run()` 入口、Bevy App 构建、系统注册、资产路径解析 |
| **state** | `state.rs` | `GameUiState` 资源、`Screen` 枚举、UI 选中状态、面板开关 |
| **actions** | `actions.rs` | 玩家动作封装：开新局、结束回合、清除命令、存档读档 |
| **hud** | `hud.rs` | 游戏内 HUD 组装：顶部状态栏、地图控制、城池列表、报告面板 |
| **menu** | `menu.rs` | 主菜单渲染、新游戏/读档/设置对话框 |
| **map** | `map.rs` | 地图绘制、缩放/平移交互、城池标记、势力着色 |
| **city_panel** | `city_panel.rs` | 城池详情面板（总览/设施/武将标签页） |
| **city_intel** | `city_intel.rs` | 城池情报摘要（用于列表和地图提示） |
| **style** | `style.rs` | egui 主题配置、字体加载、面板样式常量 |
| **labels** | `labels.rs` | 枚举到中文/英文显示名的映射 |
| **i18n** | `i18n.rs` | `Translator` 封装、Fluent 翻译加载 |
| **audio** | `audio.rs` | `MainMenuAudio`、BGM 播放/停止、设备管理 |
| **settings** | `settings.rs` | 设置应用逻辑（显示、音频、语言切换） |
| **display_settings** | `display_settings.rs` | `GameSettings`、窗口分辨率/模式/刷新率 |
| **app_icon** | `app_icon.rs` | 窗口图标设置插件 |
