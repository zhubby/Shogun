# 开发环境

## 前置条件

- **Rust 1.85+**（edition 2024）
- **SQLite 3**
- **mdbook**（文档构建，可选）

## 快速开始

```bash
# 运行游戏
cargo run

# 构建历史数据库（首次运行需要）
cargo run --bin build_history_db

# 编译检查
cargo check

# 运行所有测试
cargo test

# 格式化
cargo fmt

# Lint
cargo clippy --all-targets -- -D warnings
```

## 开发工具

项目配置了 RTK（Rust Token Killer）代理来优化开发命令的输出。在支持 RTK 的环境中，所有 shell 命令应加上 `rtk` 前缀：

```bash
rtk cargo check
rtk cargo test
rtk cargo run
```

## IDE 配置

- **rust-analyzer** — 推荐使用，确保 `Cargo.toml` 被正确识别
- **Bevy 编译加速** — 建议在 `.cargo/config.toml` 中启用动态链接：
  ```toml
  [target.x86_64-unknown-linux-gnu]
  linker = "clang"
  rustflags = ["-C", "link-arg=-fuse-ld=mold"]
  ```

## 构建信息

`build.rs` 使用 `vergen-gitcl` 在编译时注入 git 信息（commit hash、分支等），通过 `src/build_info.rs` 暴露给运行时。
