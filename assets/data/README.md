# 历史资料库工作流

`database.sqlite` 是游戏运行时读取的本机历史资料库，默认位于系统应用数据目录，不作为仓库源文件维护。

## 源文件

- `schema.sql`：表结构、外键、检查约束和常用索引。
- `seeds/001_core.sql`：核心城镇、主要势力、固定剧本、道路、主要武将和基础履历事件。
- `seeds/002_three_kingdoms_import.sql`：从 `fthux/Characters_of_the_Three_Kingdoms` 语料导入的来源化武将、外部 ID、履历登场事件和可解析亲属关系。
- `seeds/003_officer_relationships.sql`：人工校订的高价值人物、亲属、夫妻、君臣、结义和仇敌关系 overlay。
- `migrations/001_initial_history.sql`：SQLx 初始迁移，创建 v1 历史资料库结构并导入核心 seed。
- `migrations/002_officer_profiles_relationships.sql`：SQLx v2 迁移，添加性别、生平、关系、外部来源 ID、履历初始忠诚度字段，并导入来源化人物和关系 overlay。
- `migrations/003_restrict_officer_gender.sql`：SQLx v3 迁移，收敛旧库性别约束，只保留男/女两种值。
- `map_boundaries.json`：游戏地图坐标系下的州郡轮廓近似资产，参考 CHGIS 类历史行政区口径手工校订，用于 UI 地图绘制，不参与规则判定。

`seeds/002_supplemental_officers.sql` 占位池已停用。资料库验收口径是来源化人物全量进入 SQLite，不再用 `supplemental_%` 伪造数量。

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

游戏启动和新开局默认读取本机 `database.sqlite`。若文件缺失，会创建空库并执行 SQLx migrations。迁移记录使用 SQLx 的 `_sqlx_migrations` 表；SQLite `PRAGMA user_version` 保留为兼容性版本标记，当前为 `3`。

新增或修正历史资料时，优先添加版本化 SQLx 迁移，而不是对已有本机库重复运行全量 seed。空库从 `migrations/001_initial_history.sql` 开始按顺序迁移；已有未版本化 v1 本机库会先校验结构并回填 SQLx 迁移记录，再执行后续增量迁移。

从 CTK 语料重新生成导入 seed：

```bash
rtk cargo run --bin import_three_kingdoms -- <characters-json-dir> assets/data/seeds/002_three_kingdoms_import.sql
```

导入器负责稳定 ID、去重、性别映射、年代解析、来源标签、可信度和关系映射。导入关系只写入两端都已存在于核心 seed 或本次导入 seed 的记录；需要补充校订的人物与关系放入 `003_officer_relationships.sql`。

## 数据分层

- 历史资料库保存静态资料：城市资料、武将资料、势力、道路、剧本快照和履历事件。
- 武将资料包含性别、详细生平、来源可信度、外部来源 ID，以及君臣、亲子、养亲子、夫妻、兄弟、结义、仇敌关系；性别只保留男/女两种值。第一版关系仅用于资料展示，不参与规则判定。
- 履历事件的 `loyalty` 表示剧本构建或后续登场时的初始忠诚度；存档继续保存当前动态忠诚值。
- 州郡边界资产保存静态地图绘制资料，与 SQLite 历史资料库分离，避免边界美术迭代影响存档或剧本构建。
- 存档保存动态局面：当前城池归属、资源、武将状态、命令、外交、报告和已应用履历事件 ID。
- 更新历史资料库不应重写旧存档；旧存档通过已应用事件 ID 避免重复触发登场、迁移和死亡。

## 来源与授权

- `fthux/Characters_of_the_Three_Kingdoms`：MIT 许可的游戏化结构化人物语料，用于导入人物、简介、性别、年代和可解析亲属关系。seed 中通过 `officer_external_ids.source = 'characters_of_the_three_kingdoms'` 保留来源 ID 和链接。
- 人工校订 overlay 使用 `manual_curated` 来源标记，记录在 `officer_external_ids` 或 `officer_relationships.source`，并通过 `confidence` 和 `notes` 保留不确定性。
- CBDB、Wikidata 等外部校验源可继续用于后续校订；不能确认授权的长文本不直接纳入 seed。游戏内长生平应为适合 UI 阅读的摘要。

## 校验

```bash
rtk cargo test --test history_db
```

测试会覆盖建库、v1 到当前版本迁移、旧性别值收敛、外键、索引、来源化人物数量、无 `supplemental_%` 占位、重点性别/生平/外部 ID/关系、固定剧本构建、太守引用、履历初始忠诚度和履历事件幂等性。
