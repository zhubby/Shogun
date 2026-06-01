# HUD 与面板

游戏内界面由 `hud.rs` 中的多个 HUD 函数组合而成，每个函数负责一个 UI 区域。

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
    technology_hud(...)        // 浮动：科技面板
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

底部命令栏按类别组织命令：

- **内政 (Domestic)** — 开发、升级、建设
- **军事 (Military)** — 征兵、训练、调动、出征
- **外交 (Diplomacy)** — 改善关系、停战、宣战

<!-- TODO: 补充 UI 交互规范、面板打开/关闭动画 -->
