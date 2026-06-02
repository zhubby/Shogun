# HUD 与面板

游戏内界面由 `src/core/hud/` 目录模块组合而成。`hud/mod.rs` 负责入口编排，各子模块按面板职责维护具体 UI。

## 模块划分

- `chrome.rs` — 顶部状态栏、底部入口栏、返回主菜单确认框
- `map_overlays.rs` — 地图控制、选中城池摘要
- `city_list.rs` — 城池列表弹窗、城池详情抽屉
- `save_report.rs` — 存档面板、月报查看器
- `events.rs` — 事件中心、事件弹窗
- `technology.rs` — 科技面板和科技树
- `officer_browser.rs` — 武将浏览器、家臣面板、武将筛选表格
- `officer_detail.rs` — 武将详情模态框、关系图、头像生成入口
- `shrine.rs` — 宗庙、亲族图、婚姻、继承、禅让
- `officer_common.rs` — 武将 HUD 共享显示 helper

## HUD 组成

```rust
pub fn in_game_hud(ctx, ui_state, t) {
    top_status_hud(...)        // 顶部：势力名、年月、资源、结束回合
    map_controls_hud(...)      // 右侧：缩放、地图开关
    left_city_summary_hud(...) // 左侧：选中城池摘要
    city_list_hud(...)         // 浮动：所有城池列表
    save_hud(...)              // 浮动：存档/读档面板
    city_drawer_hud(...)       // 浮动：城池详情抽屉
    report_hud(...)            // 浮动：月报查看器
    bottom_map_actions_hud(..) // 底部：命令分类按钮
    officer_browser_hud(...)   // 浮动：武将浏览器
    retainer_hud(...)          // 浮动：家臣面板
    shrine_hud(...)            // 浮动：宗庙面板
    technology_hud(...)        // 浮动：科技面板
    event_center_hud(...)      // 浮动：事件中心
    event_popup_hud(...)       // 浮动：事件提醒
}
```

## 布局常量

```rust
const MAP_MIN_ZOOM: f32 = 0.65;
const MAP_MAX_ZOOM: f32 = 5.0;
const MAP_ZOOM_STEP: f32 = 1.2;
const HUD_MARGIN: f32 = 16.0;
const HUD_TOP_OFFSET: f32 = 14.0;
const HUD_TOP_HEIGHT: f32 = 68.0;
```

## 面板样式

`style.rs` 定义了一组战争主题的 UI 样式：

- `war_panel_frame` — 主面板边框
- `war_sub_panel_frame` — 子面板
- `war_bar_frame` — 状态栏
- `war_danger` / `war_warning` / `war_success` — 语义颜色
- `war_text_muted` — 次要文字颜色
- `modal_title_bar` — 对话框标题栏

## 命令面板

议政厅按类别组织城市命令：

- **内政 (Domestic)** — 开发、升级、建设
- **军事 (Military)** — 征兵、训练、调动、出征
- **任命登用 (Posts and Recruitment)** — 任命太守、登用在野武将

外交不再放在议政厅。右下角 HUD 的“合纵”按钮打开独立外交弹窗，采用三栏布局：

- 左栏：选择目标势力，显示势力色、城数、兵力、关系、停战和过路权状态
- 中栏：选择修好、求和、停战、宣战、借道、互市等提案，并预览接受分和效果
- 右栏：选择交付城、接收城，编辑我方给出和要求对方的金、粮、建材条款

弹窗底部显示本月待处理外交提案，可在月结前撤回。

<!-- TODO: 补充 UI 交互规范、面板打开/关闭动画 -->
