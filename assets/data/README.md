# 历史资料库工作流

`database.sqlite` 是游戏运行时读取的本机历史资料库，默认位于系统应用数据目录，不作为仓库源文件维护。

## 源文件

- `../../migrations/001_initial_history.sql`：当前历史资料库基线，包含表结构、索引、核心剧本资料、来源化 CTK 人物导入和人工关系 overlay。
- `../../migrations/003_expand_officer_catalog.sql`：武将资料扩充迁移，删除匿名低质量补充人物，补齐 420 名命名武将，并把男女比例固定为 350:70。
- `map_boundaries.json`：游戏地图坐标系下的州郡轮廓近似资产，参考 CHGIS 类历史行政区口径手工校订，用于 UI 地图绘制，不参与规则判定。

旧的 schema、seed 和历史增量迁移不再保留。当前迁移基线就是未来资料库的起点，不兼容旧本机库。

## 生成与迁移

```bash
rtk cargo run --bin build_history_db
```

默认生成物：

- `ProjectDirs::data_local_dir()/database.sqlite`

也可以传入显式路径生成测试或开发用数据库：

```bash
rtk cargo run --bin build_history_db -- /tmp/database.sqlite
```

游戏启动和新开局只读取本机 `database.sqlite`。若文件缺失，会从仓库根目录 `./migrations` 执行 SQLx migrations。迁移记录只使用 SQLx 的 `_sqlx_migrations` 表；仓库不再维护 JSON 后备剧本。

新增或修正历史资料时，新增 `./migrations/*.sql`，不要改写已经发布的迁移文件。旧本机资料库、旧 seed 和旧 schema snapshot 不再作为兼容来源。

从 CTK 语料重新生成导入片段：

```bash
rtk cargo run --bin import_three_kingdoms -- <characters-json-dir> /tmp/ctk_import_fragment.sql
```

导入器负责稳定 ID、去重、性别映射、年代解析、来源标签、可信度和关系映射。生成结果应作为未来 SQLx 迁移草稿人工校订后纳入 `./migrations`。

## 数据分层

- 历史资料库保存静态资料：城市资料、武将资料、势力、道路、剧本快照和履历事件。
- 剧本快照将短剧本名 `scenarios.name` 与年号 `scenarios.era_name` 分开保存；界面需要完整标题时再组合，避免把年号硬编码进剧本名。
- 武将资料包含性别、详细生平、来源可信度、外部来源 ID，以及君臣、亲子、养亲子、夫妻、兄弟、结义、仇敌关系；性别只保留男/女两种值。当前历史库固定为 420 名武将，其中男 350、女 70。关系仅用于资料展示，不参与规则判定。
- 女性人物允许使用稳定通称，如夫人、皇后、太后、大乔、小乔等；不收录“某从兄”“某妻”“无名氏”这类无法作为武将名展示的占位描述。
- 履历事件的 `loyalty` 表示剧本构建或后续登场时的初始忠诚度；存档继续保存当前动态忠诚值。
- 州郡边界资产保存静态地图绘制资料，与 SQLite 历史资料库分离，避免边界美术迭代影响存档或剧本构建。
- 存档保存动态局面：当前城池归属、资源、武将状态、命令、外交、报告和已应用履历事件 ID。

## 来源与授权

- `fthux/Characters_of_the_Three_Kingdoms`：MIT 许可的游戏化结构化人物语料，用于导入人物、简介、性别、年代和可解析亲属关系。资料库通过 `officer_external_ids.source = 'characters_of_the_three_kingdoms'` 保留来源 ID 和链接。
- 人工校订 overlay 使用 `manual_curated` 来源标记，记录在 `officer_external_ids` 或 `officer_relationships.source`，并通过 `confidence` 和 `notes` 保留不确定性。`expansion_003` 人物只使用高/中可信度，生平摘要为原创短句，不复制外部百科正文。
- CBDB、Wikidata 等外部校验源可继续用于后续校订；不能确认授权的长文本不直接纳入迁移。游戏内长生平应为适合 UI 阅读的摘要。

## 校验

```bash
rtk cargo test --test history_db
```

测试会覆盖基线建库、外键、索引、420 名武将总量、350:70 性别比例、来源化人物数量、无 `supplemental_%` 或匿名占位、重点性别/生平/外部 ID/关系、关系对称性、固定剧本构建、太守引用、履历初始忠诚度和履历事件幂等性。
