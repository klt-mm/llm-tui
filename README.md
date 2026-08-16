# 🦀 LLM-TUI

<div align="center">

**A local-first, keyboard-driven terminal UI client for llama.cpp and OpenAI-compatible LLM endpoints**

[![Version](https://img.shields.io/badge/version-1.0.2-f97316.svg?style=flat-square)](https://github.com/klt-mm/llm-tui/releases)
[![License](https://img.shields.io/badge/license-MIT-10b981.svg?style=flat-square)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-91+-10b981.svg?style=flat-square)](https://github.com/klt-mm/llm-tui/actions)
[![Rust](https://img.shields.io/badge/Rust-2024-f97316.svg?style=flat-square)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/status-Production%20Ready-10b981.svg?style=flat-square)](https://github.com/klt-mm/llm-tui)
[![Termux](https://img.shields.io/badge/Termux-Supported-f97316.svg?style=flat-square)](#android-termux)

### 🚀 [Installation](#-installation) • 📖 [Usage](#-usage) • 📚 [Documentation](https://klt-mm.github.io/llm-tui) • 🤝 [Contributing](#-contributing)

</div>

---

## ✨ Features

<table>
<tr>
<td width="50%">

### 🚀 Local-First
- Works seamlessly with llama.cpp and any OpenAI-compatible endpoint
- Your data stays local with SQLite persistence
- No cloud dependencies or external services required

### ⌨️ Keyboard-Driven
- Full keyboard navigation with vim-style bindings
- Fast and efficient workflow
- Command palette for quick access to all features

### 💾 Persistent Storage
- SQLite database with FTS5 full-text search
- Never lose your conversations
- Efficient storage and retrieval

### 🔧 Tool Calling
- Built-in tools for shell commands, file operations, and more
- Extensible tool system with custom tool support
- Automatic tool execution and result handling

</td>
<td width="50%">

### 🖼️ Vision Support
- Attach images to your messages
- Works with vision-capable models (GPT-4V, etc.)
- Base64 encoding for seamless integration

### 📊 Context Engineering
- Smart token budgeting and context management
- Automatic context optimization
- Token usage display in status bar

### 🔍 Full-Text Search
- Search across conversations and prompts with FTS5
- Find anything instantly
- Powerful search capabilities

### 🌿 Branching
- Branch conversations and explore different paths
- Edit as branch functionality
- Visual branch indicators

</td>
</tr>
</table>

### 🎯 Advanced Features
- Prompt library with variables and tags
- Generation settings (temperature, top_p, max_tokens)
- Model and provider switching
- Markdown rendering with syntax highlighting
- Generation metrics and diagnostics

## 📦 Installation

<div align="center">

### 🚀 Quick Install (Recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/klt-mm/llm-tui/main/install.sh | bash
```

*The install script auto-detects your platform (Linux, macOS, Termux) and handles everything!*

</div>

### 🛠️ Build from Source

<details>
<summary><b>Click to expand build instructions</b></summary>

```bash
# Clone the repository
git clone https://github.com/klt-mm/llm-tui.git
cd llm-tui

# Build in release mode
cargo build --release

# The binary will be at ./target/release/llm-tui
# Optionally, copy it to your PATH
sudo cp ./target/release/llm-tui /usr/local/bin/
```

</details>

### 💻 Platform-Specific Instructions

<details>
<summary><b>🐧 Linux (Ubuntu/Debian)</b></summary>

```bash
# Install dependencies
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev git sqlite3 libsqlite3-dev

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Build and install
git clone https://github.com/klt-mm/llm-tui.git
cd llm-tui
cargo build --release
sudo cp ./target/release/llm-tui /usr/local/bin/
```

</details>

<details>
<summary><b>🍎 macOS</b></summary>

```bash
# Install dependencies with Homebrew
brew install rust sqlite

# Build and install
git clone https://github.com/klt-mm/llm-tui.git
cd llm-tui
cargo build --release
sudo cp ./target/release/llm-tui /usr/local/bin/
```

</details>

<details>
<summary><b>📱 Android (Termux)</b></summary>

Termux is fully supported! The installation script automatically detects Termux and handles the installation appropriately.

**Quick Install (Recommended):**
```bash
# The install script auto-detects Termux
curl -fsSL https://raw.githubusercontent.com/klt-mm/llm-tui/main/install.sh | bash
```

**Manual Installation:**
```bash
# Install dependencies
pkg update
pkg install rust git sqlite openssl

# Clone and build
git clone https://github.com/klt-mm/llm-tui.git
cd llm-tui
cargo build --release

# Install to Termux bin directory
cp ./target/release/llm-tui $PREFIX/bin/
```

**Termux-Specific Notes:**
- No `sudo` required - Termux runs in user space
- Binary installs to `$PREFIX/bin` (automatically in PATH)
- Configuration stored in `~/.config/llm-tui/`
- Database stored in `~/llm-tui.db` by default
- Full feature support including tool calling and vision

**Uninstall on Termux:**
```bash
# Using uninstall script (auto-detects Termux)
curl -fsSL https://raw.githubusercontent.com/klt-mm/llm-tui/main/uninstall.sh | bash

# Or manually
rm $PREFIX/bin/llm-tui
rm -rf ~/.config/llm-tui
rm -f ~/llm-tui.db
```

</details>

### 🗑️ Uninstallation

<details>
<summary><b>Click to expand uninstall instructions</b></summary>

```bash
# If installed via script
curl -fsSL https://raw.githubusercontent.com/klt-mm/llm-tui/main/uninstall.sh | bash

# Or manually
sudo rm /usr/local/bin/llm-tui
rm -rf ~/.config/llm-tui
rm -f ~/llm-tui.db
```

</details>

## 🚀 Quick Start

<div align="center">

### 1️⃣ Launch LLM-TUI

```bash
llm-tui
```

### 2️⃣ Configure your provider

Create `~/.config/llm-tui/config.toml`:

```toml
[provider]
base_url = "http://localhost:8080/v1"
api_key = "your-api-key-here"

[generation]
temperature = 0.7
top_p = 0.9
max_tokens = 2048

[context]
max_tokens = 4096
reserve_for_response = 1024
```

### 3️⃣ Start chatting!

- Press `Ctrl+N` to create a new conversation
- Type your message and press `Enter` to send
- The response will stream in real-time

</div>

## ⌨️ Keyboard Shortcuts

<div align="center">

### 🌐 Global Shortcuts

| Shortcut | Action | | Shortcut | Action |
|----------|--------|-|----------|--------|
| `Ctrl+N` | New conversation | | `Ctrl+K` | Command palette |
| `Ctrl+P` | Prompt picker | | `Ctrl+L` | Prompt list |
| `Ctrl+F` | Search | | `Ctrl+G` | Generation settings |
| `Ctrl+B` | Branch history | | `Ctrl+T` | Test connection |
| `Ctrl+M` | Cycle model | | `Ctrl+R` | Retry generation |
| `Alt+C` | Cancel generation | | `?` | Show help |
| `Esc` | Quit / Close modal | | | |

### 📂 Sidebar Navigation

| Shortcut | Action | | Shortcut | Action |
|----------|--------|-|----------|--------|
| `j` / `↓` | Navigate down | | `k` / `↑` | Navigate up |
| `Enter` | Open conversation | | `r` | Rename conversation |
| `d` | Delete conversation | | `n` | Prompts screen |
| `1-9` | Select model | | | |

### 💬 Chat Input

| Shortcut | Action |
|----------|--------|
| `Enter` | Send message |
| `Tab` | Toggle focus to sidebar |
| `/` | Open search (when input empty) |

</div>

## 📖 Usage

### Basic Workflow

<table>
<tr>
<td width="25%" align="center">

**1️⃣ Start**

`Ctrl+N` or select from sidebar

</td>
<td width="25%" align="center">

**2️⃣ Send**

Type and press `Enter`

</td>
<td width="25%" align="center">

**3️⃣ Cancel**

Press `Alt+C`

</td>
<td width="25%" align="center">

**4️⃣ Retry**

Press `Ctrl+R`

</td>
</tr>
</table>

### 🎯 Advanced Features

<details>
<summary><b>🎨 Command Palette</b></summary>

Press `Ctrl+K` to open the command palette. Type to filter commands, then press `Enter` to execute.

Available commands:
- New Conversation
- Search
- Prompt Picker
- Prompt List
- Select Model
- Select Provider
- Generation Settings
- Branch History
- Test Connection
- Keyboard Shortcuts
- Quit

</details>

<details>
<summary><b>📝 Prompt Management</b></summary>

- `Ctrl+P` - Open prompt picker
- `Ctrl+L` - Open prompt list
- Create prompts with variables: `Write a {{language}} function that {{action}}`
- Organize with tags for easy filtering

</details>

<details>
<summary><b>🔍 Search</b></summary>

Press `Ctrl+F` or `/` (when input is empty) to search across messages and prompts.

</details>

<details>
<summary><b>🌿 Branching</b></summary>

- `Ctrl+B` - Open branch history
- Select a message and press `Enter` to edit as branch
- Explore different conversation paths

</details>

<details>
<summary><b>🔧 Tool Calling</b></summary>

LLM-TUI includes built-in tools:
- `shell` - Execute shell commands
- `read_file` - Read file contents
- `write_file` - Write to files
- `list_directory` - List directory contents

</details>

<details>
<summary><b>🖼️ Vision Support</b></summary>

Attach images to your messages:
```
/image /path/to/image.png
```

</details>

<details>
<summary><b>⚙️ Generation Settings</b></summary>

Press `Ctrl+G` to adjust:
- Temperature (0.0-2.0)
- Top P (0.0-1.0)
- Max Tokens

</details>

## 🏗️ Architecture

<div align="center">

```
TUI (ratatui)
      ↓
Application / Services
   (app.rs, events.rs)
      ↓
   Domain Layer
(must not import infrastructure)
      ↓
┌─────────────┬─────────────┐
│ LLM Provider│ Repositories│
│  (src/llm/) │(src/persist)│
└─────────────┴─────────────┘
      ↓
  SQLite + FTS5
```

</div>

### 📦 Key Modules

<table>
<tr>
<td width="50%">

- `src/domain/` — Pure domain types
- `src/llm/` — Provider adapters (OpenAI-compatible)
- `src/persistence/` — SQLite repositories and migrations
- `src/events.rs` — Event-driven state transitions
- `src/tui.rs` — Terminal UI rendering

</td>
<td width="50%">

- `src/app.rs` — Application state and event dispatch
- `src/context.rs` — Context engineering and token budgeting
- `src/tools/` — Tool calling system
- `src/image.rs` — Image loading and encoding
- `src/config.rs` — Configuration management

</td>
</tr>
</table>

## 🔧 Configuration

<div align="center">

**Configuration file location:** `~/.config/llm-tui/config.toml`

</div>

```toml
[provider]
base_url = "http://localhost:8080/v1"
api_key = "your-api-key-here"

[generation]
temperature = 0.7
top_p = 0.9
max_tokens = 2048

[context]
max_tokens = 4096
reserve_for_response = 1024
```

### 🌍 Environment Variables

<table>
<tr>
<td width="50%">

- `LLM_TUI_BASE_URL` - Provider base URL
- `LLM_TUI_API_KEY` - API key

</td>
<td width="50%">

- `LLM_TUI_DATABASE_URL` - Database path
- `RUST_LOG` - Log level

</td>
</tr>
</table>
- `LLM_TUI_DATABASE_URL` - Database path (default: ./llm-tui.db)
- `RUST_LOG` - Log level (default: llm_tui=info)

## 🛠️ Development

### 📋 Prerequisites

- Rust 2024 edition or later
- Git
- SQLite development libraries

### 🚀 Setup

<details>
<summary><b>Click to expand setup instructions</b></summary>

```bash
# Clone the repository
git clone https://github.com/klt-mm/llm-tui.git
cd llm-tui

# Build
cargo build

# Run tests
cargo test

# Run the application
cargo run
```

</details>

### 💻 Development Workflow

<details>
<summary><b>Click to expand workflow commands</b></summary>

```bash
# Watch for changes and rebuild
cargo watch -x check

# Watch and run tests
cargo watch -x test

# Run quality gates
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

</details>

### 🧪 Testing

<details>
<summary><b>Click to expand testing commands</b></summary>

```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test phase6

# Run with output
cargo test -- --nocapture
```

</details>

## 📊 Project Status

<div align="center">

### ✅ Production Ready

**All core phases (0-6) are complete!**

<table>
<tr>
<td width="33%" align="center">

**Phase 0** ✅  
Architecture foundation

</td>
<td width="33%" align="center">

**Phase 1** ✅  
llama.cpp chat

</td>
<td width="33%" align="center">

**Phase 2** ✅  
Core UX

</td>
</tr>
<tr>
<td width="33%" align="center">

**Phase 3** ✅  
Prompt and search

</td>
<td width="33%" align="center">

**Phase 4** ✅  
Context engineering

</td>
<td width="33%" align="center">

**Phase 5** ✅  
Branching and diagnostics

</td>
</tr>
<tr>
<td colspan="3" align="center">

**Phase 6** ✅  
Advanced provider capabilities

</td>
</tr>
</table>

**Test Coverage:** 91+ tests passing  
**Quality Gates:** All passing (cargo test, clippy, fmt)

</div>

## 📚 Documentation

<div align="center">

| Resource | Link |
|----------|------|
| 🌐 **Website** | [klt-mm.github.io/llm-tui](https://klt-mm.github.io/llm-tui) |
| 📦 **Installation Guide** | [Installation](https://klt-mm.github.io/llm-tui/installation.html) |
| 📖 **Usage Guide** | [Usage](https://klt-mm.github.io/llm-tui/usage.html) |
| 🤝 **Contributing Guide** | [Contributing](https://klt-mm.github.io/llm-tui/contributing.html) |
| 📚 **Resources** | [Resources](https://klt-mm.github.io/llm-tui/resources.html) |
| 🏗️ **Design Docs** | [docs/](docs/) |

</div>

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](https://klt-mm.github.io/llm-tui/contributing.html) for details.

### 🚀 Quick Start for Contributors

<table>
<tr>
<td width="20%" align="center">

**1️⃣ Fork**

Fork the repository

</td>
<td width="20%" align="center">

**2️⃣ Branch**

Create feature branch

</td>
<td width="20%" align="center">

**3️⃣ Code**

Make your changes

</td>
<td width="20%" align="center">

**4️⃣ Test**

Ensure tests pass

</td>
<td width="20%" align="center">

**5️⃣ PR**

Submit pull request

</td>
</tr>
</table>

## 🔒 Security

See [SECURITY.md](SECURITY.md) for security policy and reporting guidelines.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

<table>
<tr>
<td width="50%">

- [llama.cpp](https://github.com/ggerganov/llama.cpp) - Efficient LLM inference
- [ratatui](https://github.com/ratatui/ratatui) - Terminal UI framework

</td>
<td width="50%">

- [sqlx](https://github.com/launchbadge/sqlx) - Rust SQL toolkit
- [OpenAI](https://openai.com/) - API compatibility

</td>
</tr>
</table>

## 📞 Support

<div align="center">

| Channel | Link |
|---------|------|
| 🐛 **Report Bugs** | [GitHub Issues](https://github.com/klt-mm/llm-tui/issues) |
| 💬 **Discussions** | [GitHub Discussions](https://github.com/klt-mm/llm-tui/discussions) |
| 📚 **Documentation** | [Full Documentation](https://klt-mm.github.io/llm-tui) |

</div>

## 🌟 Star History

<div align="center">

If you find LLM-TUI useful, consider giving it a star on GitHub!

⭐ **[Star this repository](https://github.com/klt-mm/llm-tui)** ⭐

</div>

---

<div align="center">

### 🦀 Built with ❤️ using Rust

**[🌐 Website](https://klt-mm.github.io/llm-tui)** • **[💻 GitHub](https://github.com/klt-mm/llm-tui)** • **[📚 Documentation](https://klt-mm.github.io/llm-tui)**

*LLM-TUI v1.0.2 - Production Ready*

</div>
