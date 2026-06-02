pub fn menu_build_label() -> String {
    let release = env!("CARGO_PKG_VERSION");

    format!("游戏版本: v{release}")
}
