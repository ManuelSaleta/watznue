# wazznue

> **wazznue** (*"What's new?"*) — Modular, lean update digest & changelog inspector.

`wazznue` inspects pending package updates on your Linux system (Fedora DNF5 / DNF4), aggregates advisory metadata from Fedora Bodhi and local updateinfo, and renders a clean, prioritized digest before you apply updates.

---

## Features

- **Blazing Fast**: Caches advisory metadata locally in `~/.cache/wazznue/` for near-instant repeat execution.
- **Signal over Noise**: Automatically classifies updates into intuitive tiers:
  - **Security & Critical** (CVEs, security advisories)
  - **Core & System** (Kernel, systemd, PipeWire, Mesa)
  - **Desktop & UX** (GNOME, Mutter, Wayland compositors)
  - **User Applications** (Firefox, productivity apps)
  - **Libraries & Development**
- **Resilient Bullet Extraction**: Cleans messy RPM specfiles and Bodhi notes into concise, readable bullet points.
- **Multiple Output Formats**: Rich colorized terminal output, GitHub Flavored Markdown (for Obsidian/notes), and machine-readable JSON.
- **Standalone**: Never requires `sudo` or forced DNF hooks.
- **License**: GNU General Public License v3.0 (GPL-3.0-or-later). Any extensions or modifications must be contributed back under the same copyleft terms.

---

## Quick Start (Python POC)

### 1. Requirements & Setup
Python 3.10+ on Fedora Workstation.

```bash
cd /home/gman/Projects/wazznue
python3 -m venv .venv
source .venv/bin/activate
pip install rich httpx
```

### 2. Usage Examples

```bash
# Check system pending updates (using dnf5 / dnf)
./bin/wazznue

# Preview output with realistic demo packages (even if up to date)
./bin/wazznue --demo

# Inspect specific packages
./bin/wazznue kernel mesa mutter firefox

# Filter by category or security
./bin/wazznue --demo --category desktop
./bin/wazznue --demo --security-only

# Export to Markdown (ideal for Obsidian notes)
./bin/wazznue --demo --format markdown

# Export to JSON
./bin/wazznue --demo --format json
```

---

## Project Structure

```
wazznue/
├── bin/
│   └── wazznue             # Executable launcher script
├── wazznue/
│   ├── __init__.py
│   ├── cli.py              # CLI argument parser & orchestration
│   ├── detectors.py        # DNF5 / DNF4 package discovery
│   ├── sources.py          # Bodhi REST API client & local cache
│   ├── normalizer.py       # Resilient changelog cleaner & classifier
│   ├── formatters.py       # Rich CLI, Markdown, and JSON formatters
│   └── models.py           # Dataclasses & schema models
├── pyproject.toml
└── README.md
```

---

## Next Evolutionary Steps

1. **Rust Core (`wazznue-core`)**: Migrate engine to Rust for sub-10ms execution and zero runtime dependencies.
2. **Interactive TUI**: Add `ratatui`-based interactive package and advisory browser.
3. **Desktop Companion**: Optional GTK4/Libadwaita interface (`relm4`).
