# TaskVim

A modal, keyboard-first, high-performance terminal task manager written in Rust, inspired by Vim and Neovim.

## Features

- Modal editing (Normal, Insert, Visual, Command, and Stats modes)
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
- **Insert Mode**: Edit task titles or add new tasks.
- **Visual Mode**: Select multiple tasks for bulk operations.
- **Command Mode**: Execute commands using `:`.
- **Stats Mode**: View productivity statistics.

### Keybindings (Default)

#### Navigation
- `j` / `k`: Move selection down/up
- `gg`: Move to top
- `G`: Move to bottom
- `Ctrl+d`: Page down
- `Ctrl+u`: Page up

#### Task Actions
- `i`: Edit selected task title
- `a`: Add new task at the end
- `o`: Add new task below selection
- `O`: Add new task above selection
- `d`: Delete selected task (or visual selection)
- `Enter`: Cycle task status (Todo -> Doing -> Done -> Archived)
- `+` / `>`: Increase priority
- `-` / `<`: Decrease priority

#### Visual Mode
- `v`: Toggle Visual Mode
- `j` / `k`: Expand selection
- `d`: Bulk delete selected tasks

#### General
- `:`: Enter Command Mode
- `Esc`: Return to Normal Mode
- `q`: Quit

### Commands

- `:w`: Save changes
- `:q`: Quit
- `:wq`: Save and quit
- `:stats`: Open statistics view
- `:sort [priority|created|position]`: Sort tasks
- `:filter [expression]`: Filter tasks using DSL (e.g., `:filter status=todo priority>=3`)
- `:filter`: Clear current filter

### Filtering DSL

Syntax: `field[operator]value`

Supported fields: `status`, `priority`, `project`, `due`, `created`
Supported operators: `=`, `!=`, `>`, `<`, `>=`, `<=`, `contains`

Example:
`:filter project=work priority>=4`

## Configuration

TaskVim can be configured using Lua. Create a configuration file at:
`~/.config/taskvim/init.lua` (Linux/macOS)

### Settings

- `set.theme("theme_name")`: Set the UI theme.
- `set.default_priority(number)`: Set the default priority for new tasks (1-5).
- `set.sidebar(boolean)`: Show or hide the sidebar.

### Example configuration

```lua
set.theme("gruvbox")
set.default_priority(3)
set.sidebar(true)

-- Keymaps (WIP)
map("n", "dd", "delete_task")
```

## License

MIT
