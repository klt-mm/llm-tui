# LLM-TUI

<div align="center">

**A local-first, keyboard-driven terminal UI client for llama.cpp and OpenAI-compatible LLM endpoints**

[![Version](https://img.shields.io/badge/version-1.0.2-blue.svg)](https://github.com/klt-mm/llm-tui/releases)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-69+-brightgreen.svg)](https://github.com/klt-mm/llm-tui/actions)
[![Rust](https://img.shields.io/badge/Rust-2024-orange.svg)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/status-Production%20Ready-success.svg)](https://github.com/klt-mm/llm-tui)

[Installation](#installation) • [Usage](#usage) • [Documentation](https://klt-mm.github.io/llm-tui) • [Contributing](#contributing)

</div>

---

## ✨ Features

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

### 🎯 Advanced Features
- Prompt library with variables and tags
- Generation settings (temperature, top_p, max_tokens)
- Model and provider switching
- Markdown rendering with syntax highlighting
- Generation metrics and diagnostics

## 📦 Installation

### Quick Install (Recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/klt-mm/llm-tui/main/install.sh | bash
```

### Build from Source

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

### Platform-Specific Instructions

#### Linux (Ubuntu/Debian)
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

#### macOS
```bash
# Install dependencies with Homebrew
brew install rust sqlite

# Build and install
git clone https://github.com/klt-mm/llm-tui.git
cd llm-tui
cargo build --release
sudo cp ./target/release/llm-tui /usr/local/bin/
```

#### Android (Termux)
```bash
# Install dependencies
pkg install rust git sqlite

# Build and install
git clone https://github.com/klt-mm/llm-tui.git
cd llm-tui
cargo build --release
cp ./target/release/llm-tui $PREFIX/bin/
```

### Uninstallation

```bash
# If installed via script
curl -fsSL https://raw.githubusercontent.com/klt-mm/llm-tui/main/uninstall.sh | bash

# Or manually
sudo rm /usr/local/bin/llm-tui
rm -rf ~/.config/llm-tui
rm -f ~/llm-tui.db
```

## 🚀 Quick Start

1. **Launch LLM-TUI:**
   ```bash
   llm-tui
   ```

2. **Configure your provider** (create `~/.config/llm-tui/config.toml`):
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

3. **Start chatting:**
   - Press `Ctrl+N` to create a new conversation
   - Type your message and press `Enter` to send
   - The response will stream in real-time

## ⌨️ Keyboard Shortcuts

### Global Shortcuts
| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | New conversation |
| `Ctrl+K` | Command palette |
| `Ctrl+P` | Prompt picker |
| `Ctrl+L` | Prompt list |
| `Ctrl+F` | Search |
| `Ctrl+G` | Generation settings |
| `Ctrl+B` | Branch history |
| `Ctrl+T` | Test connection |
| `Ctrl+M` | Cycle model |
| `Ctrl+R` | Retry generation |
| `Alt+C` | Cancel generation |
| `?` | Show help |
| `Esc` | Quit / Close modal |

### Sidebar Navigation
| Shortcut | Action |
|----------|--------|
| `j` / `↓` | Navigate down |
| `k` / `↑` | Navigate up |
| `Enter` | Open conversation |
| `r` | Rename conversation |
| `d` | Delete conversation |
| `n` | Prompts screen |
| `1-9` | Select model |

### Chat Input
| Shortcut | Action |
|----------|--------|
| `Enter` | Send message |
| `Tab` | Toggle focus to sidebar |
| `/` | Open search (when input empty) |

## 📖 Usage

### Basic Workflow

1. **Start a Conversation:** Press `Ctrl+N` or select from sidebar
2. **Send a Message:** Type and press `Enter`
3. **Cancel Generation:** Press `Alt+C`
4. **Retry:** Press `Ctrl+R`

### Advanced Features

#### Command Palette
Press `Ctrl+K` to open the command palette. Type to filter commands, then press `Enter` to execute.

#### Prompt Management
- `Ctrl+P` - Open prompt picker
- `Ctrl+L` - Open prompt list
- Create prompts with variables: `Write a {{language}} function that {{action}}`
- Organize with tags for easy filtering

#### Search
Press `Ctrl+F` or `/` (when input is empty) to search across messages and prompts.

#### Branching
- `Ctrl+B` - Open branch history
- Select a message and press `Enter` to edit as branch
- Explore different conversation paths

#### Tool Calling
LLM-TUI includes built-in tools:
- `shell` - Execute shell commands
- `read_file` - Read file contents
- `write_file` - Write to files
- `list_directory` - List directory contents

#### Vision Support
Attach images to your messages:
```
/image /path/to/image.png
```

#### Generation Settings
Press `Ctrl+G` to adjust:
- Temperature (0.0-2.0)
- Top P (0.0-1.0)
- Max Tokens

## 🏗️ Architecture

```
TUI (ratatui)
  ↓
Application / Services (app.rs, events.rs)
  ↓
Domain (src/domain/)          ← must not import ratatui, sqlx, reqwest, or terminal APIs
  ├── LLM Provider (src/llm/)
  └── Repositories (src/persistence/)
        ↓
      SQLite + FTS5
```

### Key Modules
- `src/domain/` — Pure domain types
- `src/llm/` — Provider adapters (OpenAI-compatible)
- `src/persistence/` — SQLite repositories and migrations
- `src/events.rs` — Event-driven state transitions
- `src/tui.rs` — Terminal UI rendering
- `src/app.rs` — Application state and event dispatch
- `src/context.rs` — Context engineering and token budgeting
- `src/tools/` — Tool calling system
- `src/image.rs` — Image loading and encoding

## 🔧 Configuration

Configuration file location: `~/.config/llm-tui/config.toml`

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

### Environment Variables
- `LLM_TUI_BASE_URL` - Provider base URL
- `LLM_TUI_API_KEY` - API key
- `LLM_TUI_DATABASE_URL` - Database path (default: ./llm-tui.db)
- `RUST_LOG` - Log level (default: llm_tui=info)

## 🛠️ Development

### Prerequisites
- Rust 2024 edition or later
- Git
- SQLite development libraries

### Setup
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

### Development Workflow
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

### Testing
```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test phase6

# Run with output
cargo test -- --nocapture
```

## 📊 Project Status

**Current Status:** Production Ready ✅

All core phases (0-6) are complete:
- ✅ Phase 0: Architecture foundation
- ✅ Phase 1: llama.cpp chat
- ✅ Phase 2: Core UX
- ✅ Phase 3: Prompt and search
- ✅ Phase 4: Context engineering
- ✅ Phase 5: Branching and diagnostics
- ✅ Phase 6: Advanced provider capabilities

**Test Coverage:** 69+ tests passing
**Quality Gates:** All passing (cargo test, clippy, fmt)

## 📚 Documentation

- [Website](https://klt-mm.github.io/llm-tui) - Full documentation
- [Installation Guide](https://klt-mm.github.io/llm-tui/installation.html)
- [Usage Guide](https://klt-mm.github.io/llm-tui/usage.html)
- [Contributing Guide](https://klt-mm.github.io/llm-tui/contributing.html)
- [Resources](https://klt-mm.github.io/llm-tui/resources.html)
- [Design Docs](docs/) - Architecture and design documents

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](https://klt-mm.github.io/llm-tui/contributing.html) for details.

### Quick Start for Contributors
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Ensure all tests pass
5. Submit a pull request

## 🔒 Security

See [SECURITY.md](SECURITY.md) for security policy and reporting guidelines.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [llama.cpp](https://github.com/ggerganov/llama.cpp) - Efficient LLM inference
- [ratatui](https://github.com/ratatui/ratatui) - Terminal UI framework
- [sqlx](https://github.com/launchbadge/sqlx) - Rust SQL toolkit
- [OpenAI](https://openai.com/) - API compatibility

## 📞 Support

- [GitHub Issues](https://github.com/klt-mm/llm-tui/issues) - Report bugs and request features
- [GitHub Discussions](https://github.com/klt-mm/llm-tui/discussions) - Ask questions and share ideas
- [Documentation](https://klt-mm.github.io/llm-tui) - Full documentation

## 🌟 Star History

If you find LLM-TUI useful, consider giving it a star on GitHub!

---

<div align="center">

**Built with ❤️ using Rust**

[Website](https://klt-mm.github.io/llm-tui) • [GitHub](https://github.com/klt-mm/llm-tui) • [Documentation](https://klt-mm.github.io/llm-tui)

</div>
