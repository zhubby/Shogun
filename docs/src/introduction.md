# 简介

**Shogun（三国争霸）** 是一款基于 [Bevy](https://bevyengine.org/) 引擎和 [egui](https://github.com/emilk/egui) 即时模式 GUI 构建的回合制策略游戏原型，使用 Rust 编写。

玩家在古代中国的历史地图上管理势力、武将、城池和外交——每月发布军令，开发城池、招募武将、调动部队、谈判结盟，见证历史演进。

## 核心功能

- **回合制策略** — 月度命令系统，每城一令、每武将一行动
- **历史数据** — SQLite 驱动的势力、武将、城池、剧本快照和履历事件
- **AI 对手** — 规则 AI，遵守与玩家相同的命令规则
- **存档/读档** — 版本化存档槽位，序列化向后兼容
- **国际化** — 支持英文（en-US）和中文（zh-CN），基于 [i18n-embed](https://crates.io/crates/i18n-embed)（Fluent）
- **音频** — 主菜单 BGM，支持分场景音量控制（rodio）
- **跨平台打包** — macOS `.app`、Linux tarball、Windows zip

## 技术栈

| 层面 | 技术 | 说明 |
|------|------|------|
| 游戏引擎 | Bevy 0.18 | ECS 架构、资产加载、窗口管理 |
| UI | bevy_egui + egui | 即时模式 GUI，适合策略游戏密集面板 |
| 数据库 | SQLite (sqlx) | 历史资料库，静态武将/城池/剧本数据 |
| 序列化 | serde + serde_json | 存档、AI 请求/响应 |
| 国际化 | i18n-embed (Fluent) | en-US / zh-CN 双语 |
| 音频 | rodio | MP3 播放，设备枚举 |
| 打包 | Makefile | macOS / Linux / Windows 三平台 |

## 文档结构

本文档按照项目的分层结构组织：

1. **设计总览** — 架构概览、分层原则、模块职责
2. **领域模型** — `src/game/` 下的核心类型和规则
3. **数据层** — 历史数据库、剧本、存档、地图边界
4. **UI 与引擎层** — `src/core/` 下的 Bevy 集成和界面设计
5. **AI 系统** — AI 接口抽象和规则 AI 实现
6. **构建与发布** — 开发环境、测试策略、打包流程
