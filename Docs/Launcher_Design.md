# Verbium Launcher Design Specification

The Launcher is a crucial component of the Verbium ecosystem, responsible for managing the compilation configuration of static plugins.

## 1. Core Responsibilities

To balance performance and flexibility, Verbium adopts a **Self-Bootstrapping** architecture. Traditional standalone launchers have been deprecated in favor of the built-in `manager` plugin (also known as the integrated launcher).

1.  **Scanning**: Iterates through the `src/plugins/` directory to discover available plugins by identifying `plugin.toml`.
2.  **Code Generation**: The main program's `build.rs` automatically reads `plugin.toml` and generates `src/plugins/generated.rs`.
    - **Module Definitions**: Automatically generates `pub mod <name>;`.
    - **Constant Binding**: Generates `PLUGIN_NAME_<ID>` constants for compile-time identification alignment.
    - **Static Registration**: Generates the `get_extra_plugins` function for automatic plugin instantiation.
3.  **Dependency & Feature Injection**: The `manager` plugin directly modifies the root `Cargo.toml`:
    - **External Dependencies**: Injects `[external_dependencies]` into the `# --- BEGIN PLUGIN DEPENDENCIES ---` marker block.
    - **Feature Synchronization**: Automatically generates `plugin_<name>` features and updates the `default` feature list based on user selections.
4.  **Build Environment Management**: Uses `cargo` commands to implement compilation, cleanup, execution, and exporting of specific versions.

## 2. Plugin Discovery Protocol

Plugins must be defined in subdirectories of `src/plugins/` and include a standard `plugin.toml`.

### plugin.toml Format Definition
```toml
[plugin]
name = "my_plugin"           # Unique identifier (must be a valid Rust module name)
display_name = "My Plugin"   # Friendly name shown in the Launcher list
version = "0.1.0"            # Plugin version
author = "Your Name"         # Author info
description = "Description"  # Short description of the plugin
dependencies = ["core"]      # Internal plugin dependency order (for topological sorting)

[external_dependencies]
# Will be automatically injected into the root Cargo.toml [dependencies]
serde = { version = "1.0", features = ["derive"] }
rand = "0.8"
```

## 3. Configuration Synchronization Logic

To avoid conflicts caused by manual modification of `Cargo.toml`, the `manager` plugin manages specific regions:

- **Dependency Injection Point**:
  ```toml
  # --- BEGIN PLUGIN DEPENDENCIES ---
  # ... Automatically generated: From <plugin_a> & <plugin_b> ...
  # ... Duplicates will be automatically merged ...
  # --- END PLUGIN DEPENDENCIES ---
  ```
- **Feature Synchronization**: The `manager` automatically maintains the `plugin_*` list under the `[features]` section and rewrites `default = [...]` based on the enabled state.

## 4. Metadata Sharing & Validation

- **Compile-Time Constants**: When implementing the `name()` method, plugins **must** reference `crate::plugins::PLUGIN_NAME_...`.
- **Consistency Check**: During instantiation, `generated.rs` enforces validation via `assert_eq!(p.name(), CONST_NAME)` to ensure the name in the Rust implementation matches the `plugin.toml` configuration, preventing configuration drift.

## 5. UI Interaction Flow (Integrated Launcher)

1.  **Environment Check**: Detects `launcher_config.toml` at startup, automatically loading the project path and the last enabled plugin state.
2.  **Plugin List**: The central panel displays all scanned plugins; clicking a checkbox updates the features to be compiled in real-time.
3.  **Configuration Panel**: The bottom section supports selecting the build mode (Debug/Release) and toggling the "Compile & Start" linked switch.
4.  **Console Interaction**: All `cargo` output (stdout/stderr) is redirected to the Console panel on the right, supporting scroll tracking.
5.  **One-Click Sync & Run**: Clicking "▶ Build & Run" triggers the following sequence: Synchronize `Cargo.toml` -> Invoke `cargo run` -> Current process exits (or Cargo takes over the new window).
