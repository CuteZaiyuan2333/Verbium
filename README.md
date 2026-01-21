# Verbium

Verbium is a high-performance, extensible code editor built with a **static plugin architecture**. The project features a "self-bootstrapping" architecture, where the launcher logic is fully integrated into internal plugins.

## 🖼 Preview

![Verbium Screenshot](./Verbium%202026_1_21%2018_39_16.png)

## 🚀 Quick Start

Verbium uses a "launcher-driven" build mode. For the best experience, please follow these steps:

1.  **Prepare Environment**: Ensure your system has the [Rust compilation environment](https://www.rust-lang.org/) installed.
2.  **Get Source Code**: Clone this repository locally.
    ```bash
    git clone https://github.com/CuteZaiyuan2333/Verbium.git
    ```
3.  **Download Launcher**: Download the latest version of `verbium-launcher.exe` from the [Releases](https://github.com/CuteZaiyuan2333/Verbium/releases) page.
4.  **Configure & Build**:
    *   Run the downloaded `verbium-launcher.exe`.
    *   Select the root directory of the project you just cloned.
    *   Check the plugins you need (e.g., Terminal, Code Editor, etc.).
    *   Click **▶ Build & Run**. The launcher will automatically synchronize dependencies and compile the full editor for you.

## 🏗 Self-Bootstrapping Architecture

We use a lightweight, independent launcher to guide the entire lifecycle of the editor:

1.  **External Bootstrapping**: Users interact with the build process through the pre-compiled `verbium-launcher.exe`, without needing to manually modify configuration files.
2.  **Dynamic Evolution**: The launcher rewrites `Cargo.toml` in real-time based on the selected plugins and calls `cargo` to compile a production version optimized for the current environment.
3.  **Development Loop**: Inside the generated editor, the `manager` plugin (integrated launcher) is retained, allowing developers to adjust configurations and rebuild at any time during execution.

## 🛠 Features

*   **Built-in Launcher**: Supports project path management, plugin toggling, and automatic dependency synchronization.
*   **Export Function**: Supports exporting the compiled specific version (e.g., an editor containing only specific tools) to a designated directory.
*   **Cleanup Tools**: Built-in support for `cargo clean`.
*   **Static Compilation**: Enjoy ultimate performance powered by Rust's Link-Time Optimization (LTO).

## 📦 Deployment Suggestions

Download the pre-compiled `verbium-launcher.exe` (a lightweight version containing only launcher functionality) from GitHub Releases as your "Editor Management Center." Alternatively, you can directly use `cargo build --release` to generate the full-featured version.

## 📜 License

This project is licensed under the [MIT License](LICENSE).

---

**[简体中文](./README-Chinese.md)** | **English**

