# CorreoMQTT Rust Port Agent Guide

This repository is being ported from Java/JavaFX to Rust. Follow `architecture.md` for the technical direction.

## Required Decisions

- Use `egui` through `eframe` for the desktop app.
- Keep each Rust source file under 500 lines.
- Migrate old CorreoMQTT configuration files.
- Old Java plugins may be dropped and reinitialized. Do not promise PF4J jar compatibility.
- Preserve user secrets carefully. Never log passwords, key material, decrypted password maps, or export passwords.

## Agent behavior

- Do not run tests if not requested.

## Expected Workspace Shape

The target Rust workspace should use small crates:

- `correo-app`: binary, eframe bootstrap, service assembly.
- `correo-ui`: egui screens, panels, dialogs, view models.
- `correo-core`: commands, events, app state, domain orchestration.
- `correo-mqtt`: MQTT trait and client implementations.
- `correo-storage`: config, secrets, histories, import/export, migration.
- `correo-scripting`: JavaScript runtime and host bindings.
- `correo-plugins`: WASM plugin runtime, manifests, built-ins.
- `correo-diagnostics`: tracing, log capture, user-visible diagnostics.
- `xtask`: repo automation.

Do not create a single large application crate unless the task explicitly asks for a temporary prototype.

## Rust Coding Rules

- Use stable Rust and commit the chosen toolchain in `rust-toolchain.toml` when the workspace is created.
- Prefer explicit domain types and `thiserror` errors over stringly typed error handling.
- Use `serde` compatibility structs for legacy JSON. Convert them into current domain models with `TryFrom`.
- Keep async boundaries explicit. UI code sends commands; services emit events.
- Avoid global mutable state. Use owned services, channels, and testable state structs.
- Keep UI files small by splitting screens into panels, rows, dialogs, and adapters before hitting 500 lines.
- Avoid blocking the egui frame loop. Long work belongs in Tokio tasks or dedicated workers.
- Run `cargo fmt` before handing off code changes.
- Run the smallest useful tests for the touched crate. Use wider checks when shared contracts change.

## Migration Rules

- Detect old data in the existing Java paths:
  - Windows: `%APPDATA%/CorreoMqtt`
  - macOS: `~/Library/Application Support/CorreoMqtt`
  - Linux/Unix: `~/.correomqtt`
- Create a timestamped backup before modifying or replacing user data.
- Preserve known fields from old `config.json`, histories, scripts, and `.cqc` imports.
- Ignore unknown JSON fields, but record migration warnings for fields that cannot be mapped.
- Support old password import:
  - `AES/GCM/NoPadding`
  - legacy `AES/CBC/PKCS5Padding`
- Store new secrets through the OS keyring abstraction.
- Reinitialize plugin state from Rust plugin manifests and bundled replacements. Old jars, old `plugins/jars`, old `plugins/config`, old `protocol.xml`, and old PF4J metadata are not compatibility targets.

## Plugin Rules

- Prefer WASM plugins hosted by `wasmtime`.
- Do not use Rust dynamic libraries for public plugin ABI.
- Plugin DTOs must be JSON-serializable and versioned.
- Plugins must declare capabilities in a manifest.
- Host APIs must deny filesystem, network, secrets, and MQTT access unless the plugin surface explicitly grants a capability.
- Start with non-UI extension points: incoming transforms, outgoing transforms, validators, detail transforms, and detail formatters.
- Treat arbitrary UI injection as future design work.

## Scripting Rules

- Use an embedded JavaScript runtime with explicit host bindings.
- Preserve compatibility aliases for the current script shape where practical:
  - `clientFactory.getBlockingClient()`
  - `clientFactory.getAsyncClient()`
  - `clientFactory.getPromiseClient()`
  - `sleep(ms)`
  - `logger`
  - `queue.process()`
  - `queue.jumpOut()`
  - `join()`
- Do not expose arbitrary host class lookup, filesystem, process, or network access.
- Script cancellation must interrupt JS execution and cancel owned MQTT operations.
- Persist execution metadata and logs incrementally.

## UI Rules

- This is a dense desktop tool, not a marketing surface.
- Prioritize predictable panes, lists, tabs, dialogs, keyboard-friendly controls, and clear status.
- Use native-feeling egui widgets and stable layout dimensions.
- Icon-only buttons must be square and use the shared square icon button sizing.
- Keyboard focus must be visible with a dotted outline: white in dark mode, black in light mode. Tab order should start with the main menu, continue through the active use-case view from top-left to bottom-right including toolbars and tables, and end with right-aligned header controls such as language and theme. Tables must be reachable by Tab, and focused tables or rows must support arrow-key row focus movement for keyboard selection.
- Menu popups opened from left-aligned buttons must align their left edge to the button and grow right. Menu popups opened from right-aligned buttons must align their right edge to the button and grow left.
- Keep connection state, errors, and script/plugin failures visible without blocking normal work.
- Do not mix service logic into egui widgets.

## Verification Expectations

For user-facing behavior, include a verification path:

- Config migration: golden JSON fixture tests.
- Secrets migration: encrypted fixture tests with non-production sample passwords.
- MQTT: local broker integration tests.
- UI: focused manual QA notes or screenshot-driven validation when the UI exists.
- Scripting: runtime tests for blocking, async, promise, queue, cancellation, and logs.
- Plugins: WASM fixture tests for every supported hook.

## Handoff Expectations

When delegating work, include:

- Owner.
- Acceptance criteria.
- Risks and assumptions.
- Smallest verification that proves success.

## Security Notes

- Never paste secrets into issues, logs, tests, fixtures, screenshots, or docs.
- Use synthetic credentials in fixtures.
- Redact connection usernames only when they are sensitive in context; always redact passwords and key material.
- Treat plugin and script host APIs as untrusted-code boundaries.
- Escalate signing, notarization, paid services, or external release commitments to company leadership.
