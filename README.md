# TaskIt

A sleek and modern GUI Task Tracker built with Rust and GTK3.

## Features

- **Local-First**: All data is stored locally in a SQLite database.
- **Task Management**: Create, edit, delete, and toggle tasks.
- **Project Management**: Organize tasks into projects with full CRUD support.
- **Date Support**: Set due dates for tasks using an integrated calendar picker.
- **Dynamic Views**:
  - **Inbox**: Tasks without a project.
  - **Today**: Tasks due today.
  - **Upcoming**: Future tasks.
  - **Project View**: Filter tasks by project.
- **Search**: Quickly find tasks with the real-time search bar.
- **Theme Support**: Seamlessly toggle between Light and Dark modes.
- **Modern UI**: Clean design using standard system symbolic icons.

## Architecture

- **Language**: Rust
- **Framework**: GTK3
- **Database**: SQLite (via `rusqlite`)
- **State Management**: Manual state refresh pattern for robust UI updates.

## Installation

### Prerequisites

Ensure you have the following system dependencies installed:

```bash
# Ubuntu/Debian
sudo apt install libgtk-3-dev
```

### Build and Run

```bash
# Clone the repository
git clone https://github.com/maskedsyntax/taskit.git
cd taskit

# Run the application
cargo run
```

## Usage

- **Add Task**: Type a title in the input box, select an optional date, and press "Enter" or click "Add".
- **Edit Task/Project**: Click the pencil icon to open a rename popover.
- **Delete**: Click the trash icon to remove an item.
- **Search**: Use the search bar in the header to filter tasks by title.
- **Theme**: Click the sun/moon icon in the header to toggle themes.

## License

GPL-3.0
