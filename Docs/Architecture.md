# Verbium System Architecture Design

This document describes the core architectural principles, module divisions, and communication mechanisms of the Verbium editor framework.

## 1. Core Design Principles

### 1.1 Static Plugin Architecture
Unlike common Dynamic Link Library (DLL/so) or WASM plugin systems, Verbium chooses to **statically compile** plugins into the final executable.
- **Advantages**:
    - **Performance**: Zero runtime overhead, enjoying Link-Time Optimization (LTO).
    - **Security**: Leverages Rust's compile-time type checking to avoid ABI compatibility issues.
    - **Simplicity**: No need for complex dynamic loaders or sandboxing environments.
- **Disadvantages**: Adding/removing plugins requires recompilation (automated by the Launcher).

### 1.2 Reactive Injection
The Host program does not hold references to specific plugins. Instead, it defines a series of **Hooks**. During the rendering of each frame, the Host iterates through all registered plugins and calls their corresponding methods, allowing plugins to "paint" their UI into designated locations.

## 2. System Modules

### 2.1 Core Layer (Kernel)
Located in `src/lib.rs` and `src/app.rs`.
- **Responsibilities**:
    - Managing window lifecycle (`eframe`).
    - Managing docking layouts (`egui_dock`).
    - Maintaining the plugin list and loading order (topological sorting in `src/plugins/mod.rs`).
    - Message distribution (Command Dispatch).
- **Characteristics**: Agnostic of specific business logic, responsible only for scheduling.

### 2.2 Plugin Layer
Located in `src/plugins/`. Each subdirectory is an independent plugin.
- **Core Plugin**: Provides basic functions (Exit, Layout Reset, About page).
- **User Plugins**: Implement specific business logic (e.g., Code Editor, File Manager).

### 2.3 Launcher (External & Integrated)
- **External Launcher**: Users first download the pre-compiled binary from Releases. It is the entry point for the project, guiding users to configure project paths, select plugins, and perform the initial compilation.
- **Integrated Manager**: After compilation, the full-featured editor still contains the `manager` plugin. Its logic is consistent with the external launcher, allowing for continuous iterative builds during development.
- **Self-Bootstrapping Principle**: The external launcher "activates" the source code repository, synchronizes `Cargo.toml` dependencies and features based on configuration, and finally generates a customized editor executable.

## 3. Communication & Interaction Mechanism

To maintain plugin decoupling, Verbium adopts a Mediator pattern based on a message queue.

### 3.1 AppCommand Protocol
All state changes must be sent to the command queue via the `AppCommand` enum, processed uniformly by the Host at the end of each frame.

```rust
pub enum AppCommand {
    OpenTab(Tab),            // Directly open a Tab instance
    CloseTab(String),        // Close a Tab by title
    TileAll,                 // Tile layout
    ResetLayout,             // Reset layout
    OpenFile(PathBuf),       // Request to open a file
    RevealInShell(PathBuf),  // Locate in the system file manager
    CopyToClipboard(String), // Write to clipboard
    Notify { message: String, level: NotificationLevel }, // Global notification
    ToggleSettings,          // Open settings panel
}
```

### 3.2 Async I/O & Feedback Pattern
To ensure UI smoothness, plugins handling time-consuming operations (e.g., reading large files) should follow these specifications:
1. **Asynchronous Execution**: Perform I/O via `std::thread::spawn` or an async runtime.
2. **State Transition**: The UI should implement a `Loading` placeholder state and display a spinner.
3. **Global Notification**: Operation results (save successful, delete failed, etc.) must be reported via `AppCommand::Notify`.

## 4. Plugin Specifications
- **Metadata Binding**: The plugin `name()` must reference constants automatically generated in `generated.rs`, avoiding hard-coding.
- **System Isolation**: Direct calls to platform-specific code (e.g., `explorer`) within plugins are strictly prohibited; they must be delegated to the Host via `AppCommand`.
