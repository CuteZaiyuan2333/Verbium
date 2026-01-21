# Agent Tab Plugin Design Document

This document records the design philosophy, architectural planning, and development progress of the Verbium Agent plugin.

## 1. Core Vision
To build an intelligent agent runtime with "separation of UI and logic." Rust handles high-performance UI rendering and exposes low-level system interfaces, while specific Agent behaviors, mode switching, and tool calling logic are dynamically implemented via **Rhai** scripts.

## 2. Design Principles
- **Script-Driven**: The Agent's "brain" resides in external Rhai scripts, allowing users to modify logic without recompilation.
- **Flexible Configuration**: Users can customize the script root directory, supporting multiple working modes (Chat/Dev/Solo, etc.).
- **Plugin Decoupling**: The Agent plugin serves only as an egui chat shell and does not hard-code any specific AI logic.

## 3. Architecture Planning

### 3.1 Rust Host Layer (The Shell)
- **Responsibilities**:
    - Managing the Rhai engine lifecycle.
    - Persisting user settings (script directory, API configurations).
    - Rendering the chat interface.
    - Establishing a Bridge between Rust and Rhai, exposing `AppCommand` to scripts.
- **Location**: `src/plugins/agent/`

### 3.2 Rhai Script Layer (The Logic)
- **Responsibilities**:
    - Defining Prompt strategies.
    - Calling LLM APIs.
    - Handling context awareness (reading code, analyzing errors).
    - Deciding when to invoke editor tools.
- **Location**: `.rhai` files under the user-defined directory.

### 3.3 Interaction Flow
1. User enters a message.
2. Rust collects the current environment context.
3. The selected Rhai script is loaded, and the `main` function is executed.
4. The script executes logic and returns a response.
5. The UI displays the results.

## 4. Roadmap

### Phase 1: Infrastructure (Current Phase)
- [ ] Create the plugin shell.
- [ ] Implement the settings interface, allowing users to specify the script directory.
- [ ] Implement placeholder Tab UI and register it to the menu bar.

### Phase 2: Rhai Integration
- [ ] Introduce the `rhai` dependency.
- [ ] Implement automatic script directory scanning and a mode-switching menu.
- [ ] Establish basic Bridge functions (e.g., `print`, `get_active_file`).

### Phase 3: Conversational Capability
- [ ] Implement the chat interface UI (message bubbles, scroll areas).
- [ ] Integrate `reqwest` for scripts to call AI APIs.
- [ ] Support asynchronous processing to avoid blocking the UI during requests.

### Phase 4: Deep Integration
- [ ] Expose `AppCommand` to Rhai.
- [ ] Enable the Agent to automatically modify code, open files, etc.
- [ ] Refine error handling and the log viewer.
