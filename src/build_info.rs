use std::sync::LazyLock;

static MENU_BUILD_LABEL: LazyLock<String> = LazyLock::new(|| {
    let release = env!("CARGO_PKG_VERSION");
    let branch = option_env!("VERGEN_GIT_BRANCH").unwrap_or("unknown");
    let revision = option_env!("VERGEN_GIT_DESCRIBE")
        .filter(|describe| !describe.is_empty())
        .or(option_env!("VERGEN_GIT_SHA"))
        .unwrap_or("unknown");
    let status = match option_env!("VERGEN_GIT_DIRTY") {
        Some("true") => "dirty",
        Some("false") => "clean",
        _ => "unknown",
    };

    format!("release {release} | git {branch} {revision} {status}")
});

pub fn menu_build_label() -> String {
    MENU_BUILD_LABEL.clone()
}
