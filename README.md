# watznue

> **watznue** (*"What's new?"*) — Modular, lean update digest & changelog inspector.

`watznue` inspects pending package updates on your Linux system (Fedora DNF5 / DNF4), aggregates advisory metadata from Fedora Bodhi and local package databases, and renders a clean, prioritized digest before you apply updates.

Written in pure, native Rust for sub-10ms startup, single-binary distribution, and zero runtime dependencies.

---

## Features

- **Blazing Fast**: Sub-5ms cached execution time; local disk caching in `~/.cache/watznue/`.
- **Signal over Noise**: Automatically classifies updates into intuitive tiers:
  - **Security & Critical** (CVEs, security advisories)
  - **Core & System** (Kernel, systemd, PipeWire, Mesa)
  - **Desktop & UX** (GNOME, Mutter, Wayland compositors)
  - **User Applications** (Firefox, productivity apps)
  - **Libraries & Development**
- **Resilient Bullet Extraction**: Cleans messy RPM specfiles and Bodhi notes into concise, readable bullet points while filtering out packaging boilerplate.
- **Multiple Output Formats**: High-contrast terminal output, GitHub Flavored Markdown (for Obsidian/notes), and structured JSON.
- **Standalone & Unprivileged**: Runs without `sudo` or forced package manager hooks.
- **License**: GNU General Public License v3.0 (GPL-3.0-or-later).

---

## Installation & Build

### Requirements
Rust 1.75+ and Cargo.

```bash
# Clone and build optimized release binary
git clone https://github.com/gman/watznue.git
cd watznue
cargo build --release

# Install to ~/.local/bin
cargo install --path crates/watznue-cli
```

---

## Usage Examples

```bash
# 1. Check system pending updates live (via DNF5 / DNF4)
watznue

# 2. Preview digest using demo packages (even if system is up to date)
watznue --demo

# 3. Inspect specific packages
watznue kernel mesa mutter firefox

# 4. Filter by category or security only
watznue --demo --category desktop
watznue --demo --security-only

# 5. Export to Markdown (ideal for saving to Obsidian notes)
watznue --demo --format markdown

# 6. Export to machine-readable JSON
watznue --demo --format json
```

---

## Workspace Structure

```
watznue/
├── Cargo.toml               # Cargo workspace manifest
├── LICENSE                  # GNU General Public License v3.0
├── README.md
└── crates/
    ├── watznue-core/        # Headless domain engine (library)
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── models.rs    # Data models & schema
    │       ├── detector.rs  # DNF5 / DNF4 package discovery
    │       ├── sources.rs   # Bodhi REST API client & local changelog fallback
    │       ├── normalizer.rs# Heuristic bullet extractor & classifier
    │       └── cache.rs     # Positive/negative disk caching
    └── watznue-cli/         # Standalone command-line binary
        ├── Cargo.toml
        └── src/
            ├── main.rs      # CLI entrypoint (clap derive)
            └── formatters.rs# Terminal, Markdown, and JSON renderers
```

---

## Roadmap

- [x] **Milestone 1**: Problem validation & heuristic extraction (POC).
- [x] **Milestone 2**: Native Rust core (`watznue-core`) & standalone CLI (`watznue-cli`).
- [ ] **Milestone 3**: Configuration parser (`~/.config/watznue/config.toml`) & RPM packaging.
- [ ] **Milestone 4**: Interactive terminal browser (`watznue-tui` with `ratatui`).
- [ ] **Milestone 5**: Native GTK4/Libadwaita desktop companion (`watznue-gui` with `relm4`).
