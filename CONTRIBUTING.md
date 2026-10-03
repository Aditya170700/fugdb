# Contributing to FugDB ⚡

Thank you for your interest in contributing to **FugDB**! Whether you are a developer, a designer, a database administrator, or an end-user testing the app, your contributions are warmly welcomed and greatly appreciated.

---

## 📑 Table of Contents

1. [How Can You Contribute?](#-how-can-you-contribute)
2. [Reporting Bugs & Issues (For Users & Non-Developers)](#-reporting-bugs--issues)
3. [Suggesting Enhancements & New Database Drivers](#-suggesting-enhancements--new-features)
4. [Setting Up Your Local Development Environment](#-local-development-setup)
5. [Architecture & Codebase Overview](#-architecture--codebase-overview)
6. [Development Guidelines & Best Practices](#-development-guidelines)
   - [Frontend Guidelines (Svelte 5 + TypeScript)](#frontend-guidelines-svelte-5--typescript)
   - [Backend Guidelines (Rust + Tauri v2)](#backend-guidelines-rust--tauri-v2)
7. [Submitting a Pull Request (PR)](#-submitting-a-pull-request)
8. [Commit Message Conventions](#-commit-message-conventions)
9. [Community & Code of Conduct](#-code-of-conduct)

---

## 💡 How Can You Contribute?

You don't need to be a Rust or Svelte expert to help improve FugDB. Here are several ways to get involved:

* **🐛 Report Bugs**: Found a glitch or edge-case? Tell us how to reproduce it.
* **💡 Suggest Features**: Share your workflow ideas or request support for new database engines.
* **🎨 UI/UX Feedback**: Help us make FugDB more intuitive, accessible, and beautiful.
* **📖 Improve Documentation**: Fix typos, add translation guides, or improve code comments.
* **💻 Write Code**: Implement new features, optimize queries, or fix reported issues.

---

## 🐛 Reporting Bugs & Issues

Before creating a new issue, please check if a similar problem has already been reported in [GitHub Issues](https://github.com/Aditya170700/fugdb/issues).

When submitting a bug report:
1. Use our structured [Bug Report Template](https://github.com/Aditya170700/fugdb/issues/new?template=bug_report.yml).
2. Specify your Operating System (macOS, Windows, Linux) and version.
3. Mention the Database engine and version (e.g. PostgreSQL 16, MySQL 8.0, Redis 7).
4. Provide clear **step-by-step reproduction instructions**.
5. Attach screenshots, error stack traces, or console logs if available (be sure to redact sensitive passwords or credentials!).

---

## 🚀 Suggesting Enhancements & New Features

We love ideas that make database management faster, safer, and more enjoyable.

To submit a suggestion:
1. Use the [Feature Request Template](https://github.com/Aditya170700/fugdb/issues/new?template=feature_request.yml) or [Database Driver Request Template](https://github.com/Aditya170700/fugdb/issues/new?template=database_driver_request.yml).
2. Clearly explain the **problem or workflow friction** you are experiencing.
3. Describe your **ideal solution or UI behavior**.
4. Mention any alternative tools or workarounds you currently use.

---

## 🛠️ Local Development Setup

### Prerequisites

Ensure you have the following tools installed on your system:
* **Node.js**: v18.0.0 or higher ([Download Node.js](https://nodejs.org/))
* **Rust & Cargo**: v1.75.0 or higher ([Install Rust via rustup](https://rustup.rs/))
* **OS-Specific Tauri v2 Dependencies**:
  * **macOS**: Xcode Command Line Tools (`xcode-select --install`)
  * **Linux (Debian/Ubuntu)**: `sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`
  * **Windows**: Microsoft C++ Build Tools & WebView2 (pre-installed on Windows 10/11)

### Step-by-Step Setup

```bash
# 1. Fork and clone your repository
git clone https://github.com/<your-username>/fugdb.git
cd fugdb

# 2. Install frontend dependencies
npm install

# 3. Start test databases via Docker (Optional but recommended)
docker compose up -d

# 4. Run FugDB in desktop development mode (Hot-reload enabled)
npm run tauri dev
```

> **Tip:** You can also run `npm run dev` to preview UI components directly in your browser with mock API fallbacks.

---

## 🏛️ Architecture & Codebase Overview

FugDB is built as a hybrid high-performance desktop application:

```
fugdb/
├── src-tauri/                         # Rust Backend
│   ├── src/
│   │   ├── drivers/                   # Database Engine Adapters (Postgres, MySQL, SQLite, MSSQL, Redis)
│   │   ├── commands/                  # Tauri IPC commands invoked by the UI
│   │   ├── models/                    # Data Transfer Objects (DTOs) & Serde structures
│   │   ├── schema/                    # Schema introspection, diff engine, & ERD generator
│   │   ├── transfer/                  # Streaming ETL, cross-DB pipeline, & file export
│   │   ├── scheduler/                 # Background Cron automation & backup daemon
│   │   ├── keyring.rs                 # Native OS secure password vault
│   │   ├── ssh.rs                     # Bastion SSH TCP port forwarding
│   │   └── state.rs                   # Shared AppState & Connection Pool registry
│   └── Cargo.toml                     # Rust crates and dependencies
├── src/                               # Svelte 5 Frontend
│   ├── lib/
│   │   ├── components/                # Modular UI components
│   │   │   ├── editor/                # CodeMirror 6 SQL editor, history, explain tree
│   │   │   ├── grid/                  # Virtual windowing data grid & in-cell editing
│   │   │   ├── erd/                   # Interactive ERD (@xyflow/svelte)
│   │   │   ├── diff/                  # Schema & Data diff visualizer
│   │   │   ├── monitor/               # Live server health & active processes
│   │   │   ├── notebook/              # Interactive SQL scratchpads (.fugpad)
│   │   │   ├── redis/                 # Redis key tree, multi-type editor, CLI console
│   │   │   └── scheduler/             # Scheduled automations & backup dashboard
│   │   ├── state/                     # Reactive state stores using Svelte 5 runes (.svelte.ts)
│   │   ├── api/                       # Tauri IPC client (`client.ts`) and TypeScript types (`types.ts`)
│   │   └── utils/                     # Formatting utilities, export helpers, safety guards
│   ├── App.svelte                     # Root window shell and global shortcuts
│   └── app.postcss                    # TailwindCSS & theme design tokens
```

---

## 📐 Development Guidelines

### Frontend Guidelines (Svelte 5 + TypeScript)

1. **Svelte 5 Runes**: Always use Svelte 5 runes syntax (`$state`, `$derived`, `$derived.by`, `$props`, `$effect`). Avoid deprecated Svelte 4 `let:`, `writable()`, or reactive declarations `$:` when writing new components.
2. **Light & Dark Mode Compatibility**: Never hardcode colors. Use semantic Tailwind palette tokens (e.g. `bg-surface-900 dark:bg-zinc-900`, `text-slate-800 dark:text-zinc-100`, `border-slate-200 dark:border-zinc-800`).
3. **Virtual Grid Safety**: When modifying the DataGrid, ensure the DOM spacer windowing technique is preserved to avoid UI lag on datasets exceeding 50k+ rows.
4. **Type Safety**: Strictly define interfaces in `src/lib/api/types.ts` for any data passed across the Tauri IPC boundary.
5. **Lint & Typecheck**: Run `npm run check` before submitting your PR to verify zero TypeScript or Svelte compiler errors.

### Backend Guidelines (Rust + Tauri v2)

1. **Asynchronous Non-blocking I/O**: Use `tokio` async primitives for long-running operations. Never block the main thread.
2. **Memory Efficiency**: For large data export/import, stream rows using Tokio channels (`mpsc::channel`) rather than buffering entire tables in memory.
3. **Tauri Async Runtime**: When spawning background worker loops during app boot, use `tauri::async_runtime::spawn(...)`.
4. **Error Handling**: Use the centralized `AppError` enum in `src-tauri/src/error.rs` and return `Result<T, AppError>` for all Tauri commands.
5. **Compiler Warnings**: Run `cargo check` and `cargo clippy` in `src-tauri/` to ensure clean compilation.

---

## 🚀 Submitting a Pull Request (PR)

1. **Create a Feature Branch**:
   ```bash
   git checkout -b feat/my-awesome-feature
   # or
   git checkout -b fix/grid-scrolling-issue
   ```
2. **Commit Your Changes**: Follow clear commit message conventions (see below).
3. **Validate Code Quality**:
   ```bash
   # Check frontend code
   npm run check

   # Check backend Rust code
   cd src-tauri && cargo check && cd ..
   ```
4. **Push to Your Fork**:
   ```bash
   git push origin feat/my-awesome-feature
   ```
5. **Open a Pull Request**: Fill out the [Pull Request Template](.github/PULL_REQUEST_TEMPLATE.md) with details about your changes and screenshots/screen recordings for UI modifications.

---

## 📝 Commit Message Conventions

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

* `feat(editor)`: Add support for multi-cursor editing
* `fix(grid)`: Resolve horizontal scrollbar glitch on Windows
* `perf(transfer)`: Improve CSV streaming throughput with buffered chunks
* `docs(readme)`: Update database compatibility table
* `refactor(redis)`: Clean up Redis key serialization logic
* `style(theme)`: Improve dark mode contrast on modal dialogs

---

## 📜 Code of Conduct

To maintain an open, welcoming, and inclusive community, all contributors and participants are expected to adhere to our [Code of Conduct](CODE_OF_CONDUCT.md). Please report any unacceptable behavior to `adityaric72@gmail.com`.

---

Thank you for helping make FugDB the fastest, most reliable database tool for developers everywhere! ⚡
