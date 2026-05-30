# Repository Guidelines

<!-- BEGIN COMPOUND CODEX TOOL MAP -->
## Compound Codex Tool Mapping (Claude Compatibility)

This section maps Claude Code plugin tool references to Codex behavior.
Only this block is managed automatically.

Tool mapping:
- Read: use shell reads (cat/sed) or rg
- Write: create files via shell redirection or apply_patch
- Edit/MultiEdit: use apply_patch
- Bash: use shell_command
- Grep: use rg (fallback: grep)
- Glob: use rg --files or find
- LS: use ls via shell_command
- WebFetch/WebSearch: use curl or Context7 for library docs
- AskUserQuestion/Question: present choices as a numbered list in chat and wait for a reply number. For multi-select (multiSelect: true), accept comma-separated numbers. Never skip or auto-configure -- always wait for the user's response before proceeding.
- Task/Subagent/Parallel: run sequentially in main thread; use multi_tool_use.parallel for tool calls
- TodoWrite/TodoRead: use file-based todos in todos/ with todo-create skill
- Skill: open the referenced SKILL.md and follow it
- ExitPlanMode: ignore
<!-- END COMPOUND CODEX TOOL MAP -->

@/Users/zhubby/.codex/RTK.md

## Project Structure & Module Organization

This repository is a single-crate Rust 2021 Bevy strategy game prototype.

- `src/main.rs`: binary entrypoint; it should stay thin and delegate to `shogun::core::run`.
- `src/lib.rs`: library surface used by integration tests and helper binaries.
- `src/core/`: Bevy setup, `bevy_egui` UI, visual styling, and player interaction flow.
- `src/game/`: domain model and rules. Keep game behavior here, not in the UI layer.
- `src/game/model.rs`: core state, commands, reports, diplomacy, and save version.
- `src/game/commands.rs`: command validation, monthly resolution, combat, diplomacy, and life-event application.
- `src/game/ai.rs`: AI decision request/response types and rule-based AI provider.
- `src/game/history_db.rs`: SQLite-backed historical catalog and database builder.
- `src/game/scenario.rs`: JSON scenario loading and fallback scenario construction.
- `src/game/save.rs`: save-slot persistence and metadata.
- `src/bin/build_history_db.rs`: helper binary that rebuilds a local `database.sqlite`.
- `assets/data/`: SQLite schema, seed SQL, migration sources, and data notes.
- `assets/scenarios/`: JSON fallback scenarios loaded by `ScenarioData`.
- `assets/fonts/`: bundled fonts used by the UI.
- `tests/`: integration tests for gameplay and historical database integrity.

Keep rules, serialization, save compatibility, and data validation in `src/game`. `src/core` should call exported domain functions instead of duplicating rule checks. Do not introduce Bevy or egui dependencies into `src/game` modules.

## Build, Test, and Development Commands

All shell commands should be prefixed with `rtk`.

- `rtk cargo check`: fast compile verification.
- `rtk cargo test`: run all tests.
- `rtk cargo test --test gameplay`: run gameplay rule, AI, and save tests.
- `rtk cargo test --test history_db`: run SQLite seed/schema integrity tests.
- `rtk cargo run`: launch the Bevy app.
- `rtk cargo run --bin build_history_db`: rebuild the local runtime `database.sqlite` from migrations.
- `rtk cargo fmt`: format Rust code.
- `rtk cargo clippy --all-targets -- -D warnings`: lint strictly before larger changes.

When changing schema, seed SQL, or migrations, rebuild a test database and run `rtk cargo test --test history_db`. When changing command rules or save behavior, run `rtk cargo test --test gameplay` at minimum.

## Rust Style and Idioms

- Follow Rust 2021 and `rustfmt` defaults.
- Prefer concrete domain types and enums over stringly typed branching.
- Match on `CommandKind`, `GameStatus`, `Controller`, and other enums; convert to display strings only at UI/report boundaries.
- Use the ID aliases in `src/game/ids.rs` for city, faction, officer, and scenario identifiers.
- Use `BTreeMap`/`BTreeSet` when deterministic ordering affects UI, saves, tests, or reports.
- Return `Result` with domain errors for player input, data loading, saves, and database operations.
- Avoid `unwrap()`/`expect()` in production paths that can be reached by user input, file data, or database content. Use validation, `?`, `ok_or_else`, and explicit error messages.
- Guard invalid states early with clear returns instead of deeply nested conditionals.
- Prefer `let-else`, `is_some_and`, `then_some`, `transpose`, and small helper functions when they keep control flow readable.
- Keep public API surfaces small; expose only what integration tests, binaries, or UI actually need.
- Use crates and Rust APIs over subprocesses for data generation or validation.

## Game Rule Boundaries

- Route player actions through `queue_player_command` so command reservations and validation remain centralized.
- Validate AI output with the same command rules used for the player; invalid AI commands should be rejected or reported, not silently applied.
- Keep the monthly resolution order intentional: validate/apply commands, apply income, clear pending commands, refresh status, advance time, then apply due historical life events when a catalog is present.
- Preserve the one-command-per-city and one-action-per-officer invariants unless a feature explicitly changes them and tests are updated.
- Keep combat, transfer, diplomacy, recruitment, and development resource checks in `commands.rs`.
- User-facing reports and errors should stay understandable from the UI; avoid exposing raw database or parser details directly to players.

## Bevy and egui UI Guidelines

- Keep `GameUiState` as UI state only; avoid storing a second source of truth for game rules outside `GameState`.
- Do not block egui render/update systems with heavy IO, database rebuilds, or long computations. If work becomes noticeable, move it out of the frame path and surface pending/completed state in the UI.
- Reuse existing layout constants, palette helpers, panel frames, and map interaction patterns in `src/core`.
- Keep command controls wired to domain functions and show validation errors through the existing message/report surfaces.
- Preserve bundled font configuration when adding text-heavy UI, especially for Chinese display names.
- For visual changes, run the app when feasible and check the main menu plus in-game screen at the default 1280x820 window size.

## Historical Data and Assets

- Treat `assets/data/schema.sql` and `assets/data/seeds/*.sql` as the source of truth for historical data.
- Runtime `database.sqlite` lives in the game data directory and is not tracked. Update versioned migrations after schema or seed changes.
- Keep SQLite foreign keys valid and maintain indexes needed by `tests/history_db.rs`.
- Use stable ASCII identifiers for `id` fields; display names may use Chinese text.
- Prefer adding historically uncertain information with explicit confidence/notes fields rather than pretending precision.
- Keep `assets/scenarios/early_three_kingdoms.json` compatible with `ScenarioData`; it is the fallback when the historical catalog is unavailable.
- Update `assets/data/README.md` or `assets/fonts/README.md` when the data or bundled font story changes.

## Save and Serialization Safety

- `SAVE_VERSION` in `src/game/model.rs` marks the save format. Bump it for incompatible save changes.
- Prefer additive serde-compatible fields with sensible defaults over breaking existing save files.
- Keep save metadata stable enough for `SaveManager::list_slots` to read older slots gracefully.
- Add regression coverage for save/load changes, especially multi-slot behavior and version handling.

## Dependency Management

Dependencies are declared in the root `Cargo.toml`; there is no workspace-level dependency table today.

- Keep the direct dependency list small and aligned with the app: Bevy UI, egui integration, filesystem directories, SQLite, serde, and test helpers.
- Before adding a dependency, check whether an existing dependency or the standard library covers the need cleanly.
- Avoid introducing a workspace split unless there is a concrete second crate boundary that reduces complexity.

## Testing Guidelines

- Put integration tests under `tests/` for end-to-end gameplay, database, save, and scenario behavior.
- Put small unit tests next to implementation only when they exercise private helper behavior that is awkward through public APIs.
- Name tests by behavior, for example `cannot_attack_non_adjacent_city`.
- Add regression tests for bug fixes and rule changes.
- Data changes should assert counts, foreign-key integrity, indexes, selectable factions, governors, and life events as applicable.
- UI-only changes should still keep `rtk cargo check` passing; run focused tests if UI changes touch command flow or state transitions.

## Commit and Pull Request Guidelines

Use Conventional Commits. Keep each commit to one logical change.

Format:

```text
<type>(<scope>): <subject>

<body>

<footer>
```

- Subject line: imperative mood, lowercase, no trailing period, max 72 characters.
- Body: explain what changed and why, not a line-by-line implementation recap.
- Footer: use `BREAKING CHANGE:`, `Closes #123`, or related metadata when relevant.

Common types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`, `ci`, `build`, `revert`.

PRs should include purpose, impacted modules, test evidence, data/schema regeneration notes when relevant, and screenshots or a short visual note for UI-facing changes.

## Security and Configuration

- Never commit API keys, private tokens, or local machine secrets.
- Treat generated saves and local app data as user data; do not add them to tracked fixtures unless they are intentionally minimal and documented.
- Avoid loading arbitrary paths from UI input without validation.
