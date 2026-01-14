# ccperm

A TUI (Terminal User Interface) viewer and editor for [Claude Code](https://claude.ai/code) permission settings.

[![Crates.io](https://img.shields.io/crates/v/ccperm.svg)](https://crates.io/crates/ccperm)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

## Overview

`ccperm` provides an interactive terminal interface to view and manage Claude Code's permission settings stored in `~/.claude/settings.json`. It helps you understand which tools and commands are allowed, denied, or require confirmation.

## Features

- **Tree View**: Permissions are automatically categorized and displayed in a collapsible tree structure
- **Categories**: Git, NPM, GCloud, GitHub, FileSystem, Web, Python, Cargo, Docker, and Other
- **Edit Support**: Add, edit, and delete permission rules
- **Dual Settings**: Switch between global (`settings.json`) and local (`settings.local.json`) settings
- **Tab Navigation**: Easily switch between Allow, Deny, and Ask permission types

## Installation

### From crates.io

```bash
cargo install ccperm
```

### From source

```bash
git clone https://github.com/nyanko3141592/ccperm.git
cd ccperm
cargo install --path .
```

## Usage

Simply run:

```bash
ccperm
```

### Key Bindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `h` / `←` | Collapse category |
| `l` / `→` / `Enter` | Expand category |
| `Tab` | Switch tab (Allow/Deny/Ask) |
| `a` | Add new permission |
| `e` | Edit selected permission |
| `d` | Delete selected permission |
| `s` | Save changes |
| `r` | Reload from file |
| `g` | Switch to Global settings |
| `L` | Switch to Local settings |
| `?` | Show help |
| `q` / `Esc` | Quit |

## Screenshots

```
┌─────────────────────────────────────────────────────────────────┐
│ ccperm - Claude Code Permission Manager           [?] Help [q] │
├─────────────────────────────────────────────────────────────────┤
│ [Allow] [Deny] [Ask]                              Source: [G]   │
├─────────────────────────────────────────────────────────────────┤
│ ▼ Git (5)                                                       │
│   ├─ git commit:*                                               │
│   ├─ git push                                                   │
│   └─ git add:*                                                  │
│ ▼ NPM (4)                                                       │
│   ├─ npm install:*                                              │
│   └─ npm run build:*                                            │
│ ▶ GCloud (12)                                                   │
│ ▶ Web (2)                                                       │
├─────────────────────────────────────────────────────────────────┤
│ [a]dd [e]dit [d]elete [s]ave [r]eload             Total: 36     │
└─────────────────────────────────────────────────────────────────┘
```

## Configuration Files

ccperm reads and writes to the following files:

- `~/.claude/settings.json` - Global Claude Code settings
- `~/.claude/settings.local.json` - Local overrides

### Permission Format

Permissions follow the pattern:
```
Tool(command:pattern)
```

Examples:
- `Bash(npm install:*)` - Allow npm install with any arguments
- `Bash(git commit)` - Allow git commit (exact match)
- `WebFetch(domain:github.com)` - Allow fetching from github.com

## Requirements

- Rust 1.70 or later
- Claude Code installed (`~/.claude/` directory must exist)

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
