# Shogun (三国争霸)

A Three Kingdoms strategy game built with [Bevy](https://bevyengine.org/) and [egui](https://github.com/emilk/egui), written in Rust.

Manage factions, officers, cities, and diplomacy across a historical map of ancient China. Issue monthly commands — develop cities, recruit officers, move troops, negotiate alliances — and watch history unfold.

## Features

- **Turn-based strategy** — monthly command system with one action per city and one action per officer
- **Historical data** — SQLite-backed catalog of factions, officers, cities, scenarios, and life events
- **AI opponents** — rule-based AI that plays by the same command rules as the player
- **Save/Load** — versioned save slots with backward-compatible serialization
- **Internationalization** — UI in English (en-US) and Chinese (zh-CN) via [i18n-embed](https://crates.io/crates/i18n-embed) (Fluent)
- **Audio** — main menu BGM with per-context volume controls (rodio)
- **Cross-platform packaging** — macOS `.app`, Linux tarball, and Windows zip via Makefile

## Requirements

- Rust 1.85+ (edition 2024)
- SQLite 3

## Quick Start

```sh
# Run the game
cargo run

# Build the local historical database
cargo run --bin build_history_db

# Check compilation
cargo check

# Run all tests
cargo test

# Run specific test suites
cargo test --test gameplay
cargo test --test history_db
cargo test --test map_boundaries

# Lint
cargo clippy --all-targets -- -D warnings

# Format
cargo fmt
```

## Project Structure

```
src/
  main.rs          # Binary entrypoint
  lib.rs           # Library surface
  core/            # Bevy app, egui UI, audio, i18n, settings, input
  game/            # Domain model, rules, AI, commands, saves, historical data
  bin/             # Helper binaries (build_history_db, import_three_kingdoms)
assets/
  data/            # SQLite schema, seeds, migrations
  fonts/           # Bundled fonts (CJK support)
  audio/           # Sound assets
  icons/           # App icons
tests/             # Integration tests (gameplay, history_db, map boundaries)
locales/           # i18n Fluent translation files (en-US, zh-CN)
migrations/        # SQLite migration sources
packaging/         # Platform-specific packaging (macOS, Linux, Windows)
```

## Architecture

The crate is split into two layers:

- **`game`** — pure domain logic with no UI or engine dependencies. Contains the `GameState`, command validation/resolution, combat, diplomacy, officer lifecycle, technology, AI, save/load, and scenario loading.
- **`core`** — Bevy and egui integration. Handles rendering, user input, screen flow (main menu → in-game), settings persistence, localization, and audio.

`core` calls into `game` via its public API. `game` has no knowledge of Bevy or egui.

## Packaging

Build distributable packages with the Makefile:

```sh
make package-macos    # dist/Shogun-0.1.0-macos.zip
make package-linux    # dist/shogun-0.1.0-linux.tar.gz
make package-windows  # dist/shogun-0.1.0-windows.zip
make package          # All platforms
```

## License

This project is a prototype and does not currently have a formal license.
