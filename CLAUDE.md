# mail-tui Development Guidelines

## Build Commands
```bash
cargo build              # Build project
cargo build --release    # Build with optimizations
cargo test               # Run all tests
cargo test -- --test-threads=1 <test_name>  # Run single test
cargo watch -x run       # Hot reload during development
cargo watch -x test      # Run tests with hot reload
cargo clippy             # Lint code
cargo fmt                # Format code
```

## Code Style
- **Imports**: Group by std, external crates, then internal modules
- **Error Handling**: Use anyhow::Result for main functions, thiserror for library errors
- **Naming**: 
  - snake_case for variables, functions, modules
  - CamelCase for types, traits, enums
- **Types**: Prefer explicit typing over inference, especially in public interfaces
- **Documentation**: Document public API with /// comments, use //! for module docs
- **Architecture**: Follow the module structure of config, email, ui for new features
- **UI Components**: Use the existing TUI patterns with split views and vim-like navigation
- **Async**: Use tokio for async operations, with proper error propagation