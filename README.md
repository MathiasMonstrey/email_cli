# Mail TUI

A terminal-based email client for Office Exchange with vim-like navigation.

## Features

Current progress:
- [x] Project setup and architecture
- [x] Configuration handling
- [x] Mock Exchange email fetching
- [x] TUI implementation with split view
- [x] Vim-like navigation
- [x] Help menu
- [x] Email refresh functionality
- [x] Status bar with notifications

End goal:
- A terminal UI application that can:
  - Connect to Office Exchange
  - Fetch emails from the current quarter
  - Display emails in a split view (titles/dates on left, content/sender on right)
  - Navigate through emails using vim-like keybindings
  - Show a help menu with '?' key

## Usage

```bash
# Run with default config
mail-tui

# Specify config file
mail-tui --config path/to/config.toml
```

## Development

### Prerequisites

- Rust and Cargo (install via [rustup](https://rustup.rs/))
- [cargo-watch](https://github.com/watchexec/cargo-watch) for hot reloading

### Building

```bash
# Build the project
cargo build

# Build with optimizations
cargo build --release
```

### Running with Hot Reload

Install cargo-watch if you don't have it already:
```bash
cargo install cargo-watch
```

Run with hot reload during development:
```bash
cargo watch -x run
```

This will automatically rebuild and restart the application whenever you save changes to any source file.

### Testing

```bash
# Run tests
cargo test

# Run tests with hot reload
cargo watch -x test
```

## Configuration

Create a `config.toml` file:

```toml
[exchange]
email = "your.email@company.com"
password = "your_password"
server = "outlook.office365.com"
```

## Keyboard Shortcuts

- `j/k` or `↑/↓`: Navigate up/down through email list
- `l` or `→` or `Enter`: View selected email details
- `h` or `←` or `Esc`: Return to email list
- `g`: Go to first email
- `G`: Go to last email
- `r`: Refresh emails
- `/`: Search emails
- `q`: Quit application
- `?`: Show help menu
