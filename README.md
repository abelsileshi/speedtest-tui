# ⚡ speedtest-tui

[![Rust](https://img.shields.io/badge/rust-2021_edition-orange?logo=rust)](https://www.rust-lang.org/)
[![ratatui](https://img.shields.io/badge/ratatui-0.29-blueviolet)](https://ratatui.rs)
[![tokio](https://img.shields.io/badge/tokio-async-blue)](https://tokio.rs)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

A professional-grade, fully async **internet speed test** that runs entirely in your terminal. No browser, no Electron, no tracking — just fast, accurate measurements rendered with a beautiful TUI.

```
⚡ speedtest-tui — download · upload · latency · jitter · packet loss
```

---

## Table of Contents

- [Features](#features)
- [Requirements](#requirements)
- [For Non-Developer Users](#for-non-developer-users-pre-built-binaries)
- [Installation (from source)](#installation-from-source)
- [Usage](#usage)
- [Keyboard Shortcuts](#keyboard-shortcuts)
- [UI Layout](#ui-layout)
- [Configuration](#configuration)
- [History & Export](#history--export)
- [Quality Score](#quality-score)
- [Architecture](#architecture)
- [Dependencies](#dependencies)
- [Contributing](#contributing)
- [Platform Notes](#platform-notes)
- [FAQ](#faq)
- [License](#license)

---

## Features

- **Multi-stream download & upload** — 8 parallel TCP workers saturate the link for accurate real-world results
- **Live throughput sparkline** — Braille-dot chart updates in real time, split into DL and UL rows
- **Latency suite** — 100-probe ping, jitter, and packet-loss measurement before speed tests
- **Animated status bar** — Braille spinner + scrolling pulse wave while active; static checkmark when done; stage progress pills
- **Quality score** — letter grade (A–F) and ★ star rating computed from ping, download, upload, and packet loss; displayed live in the Server Details panel after the test
- **History mini-graph** — Inline line chart showing DL, UL and Ping across all past runs
- **History table** — Full scrollable table of every saved result, opened with `H`
- **Dark / Light theme** — Toggle with `T` at runtime; also configurable in `config.toml`
- **Export** — JSON and CSV export on demand (`E` key) or automatically on test completion
- **CI / quiet mode** — `--quiet` flag prints a single JSON object and exits; pipe-friendly
- **Zero external accounts** — Uses Cloudflare's public speed-test endpoints; no API key required
- **Clean error messages** — IP/ISP lookup failures show human-readable messages instead of raw exceptions

---

## Requirements

| Tool | Minimum version |
|---|---|
| Rust toolchain | 1.75 *(only needed if building from source)* |
| Cargo | bundled with Rust |
| Terminal | 256-colour + Unicode support |
| Network | Any IPv4/IPv6 internet connection |

---

## For Non-Developer Users (pre-built binaries)

You do **not** need to install Rust or compile anything. Pre-built binaries are provided on the [Releases page](https://github.com/abelsileshi/speedtest-tui/releases) for every major platform.

### Step 1 — Download your binary

Go to **[Releases](https://github.com/abelsileshi/speedtest-tui/releases)** and download the file for your system:

| Operating system | File to download |
|---|---|
| **Linux** (x86_64) | `speedtest-tui-linux-x86_64.tar.gz` |
| **Linux** (ARM64 — Raspberry Pi, etc.) | `speedtest-tui-linux-aarch64.tar.gz` |
| **macOS** (Apple Silicon M1/M2/M3) | `speedtest-tui-macos-aarch64.tar.gz` |
| **macOS** (Intel) | `speedtest-tui-macos-x86_64.tar.gz` |
| **Windows** (64-bit) | `speedtest-tui-windows-x86_64.zip` |

### Step 2 — Install

**Linux / macOS:**

```bash
# Unpack the archive
tar -xzf speedtest-tui-linux-x86_64.tar.gz    # adjust filename for your platform

# Move to a folder that is on your PATH so you can run it from anywhere
sudo mv speedtest-tui /usr/local/bin/

# Run it
speedtest-tui
```

**macOS Gatekeeper note:** macOS may block the binary the first time because it was downloaded from the internet. Run this once to allow it:

```bash
xattr -dr com.apple.quarantine /usr/local/bin/speedtest-tui
```

**Windows:**

1. Extract the `.zip` file.
2. Move `speedtest-tui.exe` to a folder of your choice (e.g., `C:\Tools\`).
3. Open **Windows Terminal** or **PowerShell**, navigate to that folder, and run:
   ```powershell
   .\speedtest-tui.exe
   ```
   > For best results use **Windows Terminal** — it renders the Unicode charts and colours correctly. The old `cmd.exe` prompt does not.

### Step 3 — Run

```bash
speedtest-tui
```

The TUI opens, runs the full test pipeline, and displays your results. Press `Q` to quit.

---

## Installation (from source)

If you are a developer or want to build the latest unreleased code:

```bash
git clone https://github.com/abelsileshi/speedtest-tui.git
cd speedtest-tui
cargo build --release
```

The binary is placed at `./target/release/speedtest-tui`.

### Install to `~/.cargo/bin`

```bash
cargo install --path .
# Binary is now globally available:
speedtest-tui
```

### Optimised release build

The `[profile.release]` in `Cargo.toml` already enables `lto = true`, `codegen-units = 1`, and `strip = true`. `cargo build --release` always produces a small, fully optimised binary.

### Cross-compile

```bash
# ARM64 Linux (e.g. Raspberry Pi 4)
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu

# macOS universal binary (runs natively on both Intel and Apple Silicon)
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
lipo -create -output speedtest-tui \
  target/x86_64-apple-darwin/release/speedtest-tui \
  target/aarch64-apple-darwin/release/speedtest-tui
```

---

## Usage

### Basic run

```bash
speedtest-tui
```

Launches the full TUI, runs the complete test pipeline, and saves the result to your local history.

### CLI flags

| Flag | Short | Description |
|---|---|---|
| `--no-upload` | `-n` | Skip the upload phase |
| `--quiet` | `-q` | Print a single JSON result to stdout and exit (no TUI) |
| `--export <FORMAT>` | `-e` | Export result on completion: `json` or `csv` |
| `--theme <THEME>` | `-t` | Force theme at launch: `dark` or `light` |
| `--server <ID>` | `-s` | Use a specific server ID instead of auto-selection |
| `--help` | `-h` | Show help |
| `--version` | `-V` | Print version |

### Examples

```bash
# Download only, dark theme
speedtest-tui --no-upload --theme dark

# Quiet mode — pipe result to jq
speedtest-tui --quiet | jq '.download_mbps'

# Auto-export CSV after every run
speedtest-tui --export csv

# Pin to a specific test server
speedtest-tui --server 2
```

---

## Keyboard Shortcuts

All shortcuts work while the TUI is open.

| Key | Action |
|---|---|
| `R` | Restart test |
| `H` | Open history table (full scrollable list of all past results) |
| `T` | Toggle dark / light theme |
| `E` | Export current result to JSON |
| `?` | Show help overlay |
| `↑` / `↓` | Scroll history table |
| `Q` / `Esc` | Quit app / close overlay and return to main screen |
| `Ctrl-C` | Force quit |

---

## UI Layout

```
┌─────────────────────────────────────────────────────────────────┐
│  ⟳ RETEST  ⊟ HISTORY  ◑ THEME  ↑ EXPORT  ? HELP               │  ← nav bar
│  ✓ Test complete   ✓CONNECT  ✓PING  ✓DOWNLOAD  ✓UPLOAD  ✓DONE │  ← status / stage pills
│ ┌── THROUGHPUT SPARKLINE ─────────────────────┬───────────────┐ │
│ │ ▁▂▄▆█████████████ (download)                │  Peak 32.3    │ │  ← braille sparkline
│ │ ···▁▂▄████ (upload)                         │  14.52 Mbps   │ │
│ └─────────────────────────────────────────────┴───────────────┘ │
│ ┌── DOWNLOAD ──────────────┐ ┌── UPLOAD ───────────────────────┐│
│ │ ████████████████████░░░░ │ │ ████████░░░░░░░░░░░░░░░░░░░░░░  ││  ← speed gauges
│ │     14.52 Mbps           │ │      5.30 Mbps                  ││
│ └──────────────────────────┘ └─────────────────────────────────┘│
│ ┌── TEST HISTORY ──────────┐ ┌── SERVER DETAILS ───────────────┐│
│ │  Line chart: DL/UL/Ping  │ │  ISP / IP / Server / Location   ││  ← bottom section
│ │  across all past runs    │ │  Latency / Jitter / PKT LOSS    ││
│ │                          │ │  ─────────────────────          ││
│ │                          │ │  QUALITY  ★★★★☆ (B)            ││  ← quality score
│ └──────────────────────────┘ └─────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

The **QUALITY** row appears at the bottom of the Server Details panel once the test is complete and shows the ★ star rating alongside the letter grade.

---

## Configuration

The config file is loaded automatically on startup. If it does not exist, all defaults are used.

### File locations

| Platform | Config file | History file |
|---|---|---|
| **Linux** | `~/.config/speedtest-tui/config.toml` | `~/.local/share/speedtest-tui/history.json` |
| **macOS** | `~/Library/Application Support/speedtest-tui/config.toml` | `~/Library/Application Support/speedtest-tui/history.json` |
| **Windows** | `%APPDATA%\speedtest-tui\config\config.toml` | `%APPDATA%\speedtest-tui\data\history.json` |

### All options

```toml
# ~/.config/speedtest-tui/config.toml

# UI theme: "dark" | "light" | "auto" (follows system preference)
theme = "auto"

# Number of parallel TCP workers for download and upload tests
parallel_workers = 8

# Number of HTTP probes for latency measurement
ping_count = 100

# Duration (seconds) for each of the download and upload phases
test_duration_secs = 10

# Pin to a specific server ID; leave empty for automatic best-server selection
preferred_server = ""

# Automatically export after every completed test: "json" | "csv" | ""
export_on_complete = ""
```

---

## History & Export

### History

All completed test results are appended to `history.json` automatically. The file stores up to **1 000** entries (oldest are pruned). View results two ways:

- **In-app mini graph** — The bottom-left panel always shows a line chart of DL, UL, and Ping across all runs.
- **In-app table** — Press `H` to open the full scrollable table with timestamps and all metrics.

### JSON schema

Each result object in `history.json`:

```jsonc
{
  "timestamp": "2026-06-09T10:27:00Z",
  "isp": "Ethio Telecom",
  "ip": "196.188.x.x",
  "location": "Addis Ababa, Ethiopia",
  "server_name": "Cloudflare",
  "server_host": "speed.cloudflare.com",
  "ping_ms": 25.4,
  "jitter_ms": 6.8,
  "packet_loss_pct": 0.0,
  "download_mbps": 14.52,
  "upload_mbps": 5.30,
  "quality_score": 3.25,
  "quality_grade": "C"
}
```

### CSV columns

`timestamp, isp, ip, location, server, ping_ms, jitter_ms, packet_loss_pct, download_mbps, upload_mbps, grade`

### Quiet / CI mode

```bash
# Get download speed as a number
speedtest-tui --quiet | jq -r '.download_mbps'

# Append to CSV from a cron job (no TUI)
speedtest-tui --quiet --export csv
```

---

## Quality Score

After each test, a **quality score** (1.0–5.0) and **letter grade** (A–F) is computed from four independent dimensions and displayed in the Server Details panel as `★★★★☆ (B)`.

### Scoring dimensions

Each dimension is scored 1–5 independently, then averaged:

**Ping (latency)**

| Latency | Score |
|---|---|
| < 20 ms | 5 |
| 20 – 49 ms | 4 |
| 50 – 99 ms | 3 |
| 100 – 149 ms | 2 |
| ≥ 150 ms | 1 |

**Download speed**

| Speed | Score |
|---|---|
| > 200 Mbps | 5 |
| 50 – 200 Mbps | 4 |
| 10 – 50 Mbps | 3 |
| 2 – 10 Mbps | 2 |
| < 2 Mbps | 1 |

**Upload speed**

| Speed | Score |
|---|---|
| > 50 Mbps | 5 |
| 10 – 50 Mbps | 4 |
| 3 – 10 Mbps | 3 |
| 1 – 3 Mbps | 2 |
| < 1 Mbps | 1 |

**Packet loss**

| Loss | Score |
|---|---|
| 0% | 5 |
| < 1% | 4 |
| 1 – 3% | 3 |
| 3 – 5% | 2 |
| ≥ 5% | 1 |

### Letter grades

The average of the four dimension scores is truncated to an integer for the grade:

| Integer score | Grade | Star display | Grade colour |
|---|---|---|---|
| 5 | **A** | ★★★★★ | Green |
| 4 | **B** | ★★★★☆ | Yellow-green |
| 3 | **C** | ★★★☆☆ | Yellow |
| 2 | **D** | ★★☆☆☆ | Orange |
| 1 | **F** | ★☆☆☆☆ | Red |

> **Note:** The four dimensions have equal weight (25% each). The `quality_score` field in JSON is the raw floating-point average (e.g., `3.25`). The grade is derived by truncating to the nearest integer.

---

## Architecture

### Source tree

```
src/
├── main.rs                  # Entry point: terminal setup, event loop, networking orchestration
├── cli.rs                   # clap CLI definitions (flags, enums)
├── config.rs                # Config struct, TOML load/save, platform path helpers
│
├── app/
│   ├── mod.rs               # Re-exports
│   ├── state.rs             # AppState, Phase enum, SpeedStats, TestResult, IpInfo
│   │                        #   → compute_quality_score() lives here
│   ├── events.rs            # NetworkMsg enum — async channel messages from workers
│   └── metrics.rs           # Rolling average helpers, latency stats updater
│
├── network/
│   ├── mod.rs
│   ├── server.rs            # IP/ISP lookup (ip-api.com), server list, best-server selection
│   ├── ping.rs              # HTTP-based latency probes (100 samples), jitter, packet loss
│   ├── download.rs          # 8-worker parallel download via Cloudflare /__down
│   └── upload.rs            # 8-worker parallel upload via Cloudflare /__up
│
├── storage/
│   ├── mod.rs
│   ├── history.rs           # Load/append history.json (max 1000 entries)
│   └── export.rs            # JSON and CSV export
│
└── ui/
    ├── mod.rs
    ├── dashboard.rs         # Root render fn; all panel renderers including quality score display
    ├── theme.rs             # ThemeColors struct for dark/light mode
    ├── history.rs           # Full-screen history table renderer
    ├── graphs.rs            # Shared throughput Chart widget
    ├── latency.rs           # Latency stats bar
    ├── streams.rs           # Per-worker gauge bars
    └── widgets.rs           # Sparkline, arc gauge, quality_stars() helper
```

### Async design

```
main thread (tokio)
    │
    ├─ Terminal event loop  ──── crossterm events (keyboard, resize)
    │        │
    │   mpsc::Sender<NetworkMsg>          ← non-blocking try_recv() every 33 ms
    │        │
    └─ Network task (tokio::spawn)
             │
             ├─ fetch_ip_info()        → IpInfoReceived { ip, isp, location }
             ├─ select_best_server()   → ServersReceived(Vec<ServerInfo>)
             ├─ run_ping_test()        → PingSample(ms) × ping_count, then PingComplete
             ├─ run_download_test()    → DownloadSample { worker_id, bytes, mbps, aggregate_mbps }
             │                             × (8 workers × many iterations)
             │                           DownloadComplete(avg_mbps)
             └─ run_upload_test()      → UploadSample { … } × (8 workers)
                                          UploadComplete(avg_mbps)
```

The event loop calls `rx.try_recv()` on every 33 ms tick (~30 fps), draining all pending messages before calling `terminal.draw()`.

### Network endpoints

| Purpose | URL |
|---|---|
| IP / ISP lookup | `http://ip-api.com/json/?fields=status,country,regionName,city,isp,query` |
| Server ping probe | `https://speed.cloudflare.com/__down?bytes=1` |
| Download test | `https://speed.cloudflare.com/__down?bytes=25000000` |
| Upload test | `https://speed.cloudflare.com/__up` (POST, 4 MB body per request) |

All network calls use **rustls** (no OpenSSL dependency) via `reqwest`.

---

## Dependencies

| Crate | Version | Purpose |
|---|---|---|
| `tokio` | 1 | Async runtime (full feature set) |
| `reqwest` | 0.12 | HTTP client with rustls TLS, streaming, gzip |
| `ratatui` | 0.29 | Terminal UI framework |
| `crossterm` | 0.28 | Cross-platform terminal I/O and event handling |
| `serde` / `serde_json` | 1 | Serialisation for history JSON and quiet-mode output |
| `clap` | 4 | CLI argument parsing with derive macros |
| `directories` | 5 | Platform-aware config/data directory paths |
| `chrono` | 0.4 | Timestamps with serde support |
| `anyhow` | 1 | Ergonomic error handling |
| `csv` | 1 | CSV export |
| `toml` | 0.8 | Config file parsing |
| `tracing` / `tracing-subscriber` | 0.1 / 0.3 | Structured logging (enabled via `RUST_LOG`) |
| `rand` | 0.8 | Jitter/noise simulation helpers |
| `unicode-width` | 0.2 | Correct character width for terminal rendering |

---

## Contributing

Contributions are welcome! Here is how to get started.

### Set up

```bash
git clone https://github.com/abelsileshi/speedtest-tui.git
cd speedtest-tui
cargo build          # debug build — fast compile
cargo run            # run in debug mode
```

### Enable debug logging

```bash
RUST_LOG=debug cargo run 2>debug.log
# In another terminal:
tail -f debug.log
```

### Code style

```bash
cargo fmt            # format all source files (required before every PR)
cargo clippy         # lint — all warnings must be fixed
cargo test           # run the test suite
```

### PR checklist

- [ ] `cargo fmt` — no formatting diffs
- [ ] `cargo clippy -- -D warnings` — zero warnings
- [ ] `cargo test` passes
- [ ] New behaviour is covered by at least one unit test
- [ ] `README.md` updated if public-facing behaviour changed
- [ ] Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `docs:`, `refactor:`)

### Roadmap / open issues

- [ ] **GitHub Actions CI** — automated builds producing the pre-built binaries for all platforms on every release tag
- [ ] PNG export (terminal screenshot via ratatui backend capture)
- [ ] IPv6 detection and dual-stack reporting
- [ ] Custom server list from user-provided URLs
- [ ] `--server list` subcommand to print all available servers
- [ ] Configurable chart time window (default 30 s)
- [ ] Homebrew formula / AUR package
- [ ] Packet-loss simulation mode for UI testing

---

## Platform Notes

| Platform | Status | Notes |
|---|---|---|
| Linux (x86_64) | ✅ Fully tested | Primary development platform |
| macOS (Apple Silicon / Intel) | ✅ Works | Universal binary build supported |
| Windows 10/11 | ✅ Works | Use Windows Terminal for correct Unicode rendering; `cmd.exe` is not supported |
| Raspberry Pi (ARM64) | ✅ Works | Cross-compile with `aarch64-unknown-linux-gnu` target |
| WSL2 | ✅ Works | Use Windows Terminal as the host; measure WSL2 host networking, not native |

---

## FAQ

**The pre-built binary says "permission denied" on Linux/macOS.**  
The file needs to be made executable after downloading:
```bash
chmod +x speedtest-tui
```

**macOS says the app "cannot be opened because the developer cannot be verified".**  
This is Gatekeeper blocking an unsigned binary. Run once:
```bash
xattr -dr com.apple.quarantine speedtest-tui
```

**Why is the left side of the sparkline empty at the start?**  
The sparkline pre-allocates a fixed number of columns (equal to chart width) and fills them from the right as new samples arrive. During the first few seconds there are fewer samples than columns — this is expected and fills in as the test progresses.

**The ISP shows "ISP lookup unavailable".**  
The app uses `http://ip-api.com` for ISP detection. If that endpoint is unreachable (firewall, HTTP blocked, or free-tier rate limit hit) the app shows this placeholder and continues the speed test normally.

**The quality grade seems low even though my internet is fast.**  
The grade averages four dimensions equally. A single bad dimension (e.g., high packet loss or high latency) pulls the overall score down. Check the Server Details panel for the individual metrics.

**Can I run this without a terminal that supports Unicode?**  
The Braille-dot sparkline and block-character bars require Unicode support. On terminals that lack it, the charts will show broken characters. Use a modern terminal emulator (Windows Terminal, iTerm2, Alacritty, Kitty, GNOME Terminal).

---

## License

MIT — see [LICENSE](LICENSE).

---

## Acknowledgements

- [Cloudflare Speed Test](https://speed.cloudflare.com) — public, no-key speed test endpoints
- [ratatui](https://ratatui.rs) — the Rust TUI framework powering the UI
- [ip-api.com](https://ip-api.com) — free IP geolocation and ISP lookup