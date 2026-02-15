# TaskVim

A modal, keyboard-first, high-performance terminal task manager written in Rust, inspired by Vim and Neovim.

## Features

- Modal editing (Normal, Insert, Visual, Command modes)
- SQLite backend for reliable storage
- Scriptable configuration via Lua
- Filtering DSL for complex task queries
- High performance, designed to handle thousands of tasks

## Installation

### Prerequisites

- Rust (stable)
- SQLite3

### Build from source

```bash
git clone https://github.com/maskedsyntax/taskvim
cd taskvim
cargo build --release
```

The binary will be available at `target/release/taskvim`.

## Usage

Run the application:

```bash
taskvim
```

### Modes

- **Normal Mode**: Navigate and manage tasks.
- **Insert Mode**: Add or edit tasks.
- **Command Mode**: Execute commands using `:`.

### Keybindings (Default)

- `j` / `k`: Navigate up/down
- `i`: Enter Insert Mode (Add new task)
- `d`: Delete selected task
- `:`: Enter Command Mode
- `Esc`: Return to Normal Mode
- `q`: Quit

## Configuration

TaskVim can be configured using Lua. Create a configuration file at:
`~/.config/taskvim/init.lua` (Linux/macOS)

Example configuration:

```lua
set.theme("gruvbox")

map("n", "dd", "delete_task")
```

## License

MIT
