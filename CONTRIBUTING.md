# Contributing

Thanks for your interest in improving `speedtest-tui`.

## Development Setup

```bash
git clone https://github.com/abelsileshi/speedtest-tui.git
cd speedtest-tui
cargo build
cargo run
```

## Before Opening a Pull Request

Run the standard checks locally:

```bash
cargo fmt
cargo clippy -- -D warnings
cargo build
```

## Pull Request Guidelines

- Keep changes focused and easy to review
- Update `README.md` when user-facing behavior changes
- Include screenshots for visible UI changes when possible
- Prefer small, descriptive commit messages

## Reporting Issues

When filing a bug, include:

- your operating system
- terminal emulator
- exact command used
- screenshot or terminal output if relevant
