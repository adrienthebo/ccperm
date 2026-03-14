# ccperm

A TUI (Terminal User Interface) viewer and editor for [Claude Code](https://claude.ai/code) permission settings.

[![Crates.io](https://img.shields.io/crates/v/ccperm.svg)](https://crates.io/crates/ccperm)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

## Overview

`ccperm` provides an interactive terminal interface to view and manage Claude Code's permission settings. It supports all three settings sources (User, Project, and Local) and helps you understand which tools and commands are allowed, denied, or require confirmation.

## Features

- **Tree View**: Permissions are automatically categorized and displayed in a collapsible tree structure
- **Categories**: Git, NPM, GCloud, GitHub, FileSystem, Web, Python, Cargo, Docker, Go, MCP, Skill, SlashCommand, and Other
- **Edit Support**: Add, edit, delete, move, and sort permission rules
- **Three Settings Sources**: User (`~/.claude/settings.json`), Project (`<git-root>/.claude/settings.json`), and Local (`<git-root>/.claude/settings.local.json`)
- **Tab Navigation**: Switch between Allow, Deny, and Ask permission types

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

#### Navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `h` / `←` | Collapse category |
| `→` / `Enter` | Expand category |
| `G` | Jump to end |
| `Tab` | Switch tab (Allow/Deny/Ask) |

#### Actions

| Key | Action |
|-----|--------|
| `a` | Add new permission |
| `e` | Edit selected permission |
| `c` | Change permission type (allow/deny/ask) |
| `d` | Delete selected permission |
| `m` | Move permission to another source |
| `o` | Sort permissions |
| `s` | Save changes |
| `r` | Reload from file |

#### Settings Sources

| Key | Action |
|-----|--------|
| `u` | Switch to User settings |
| `p` | Switch to Project settings |
| `l` | Switch to Local settings |

#### Other

| Key | Action |
|-----|--------|
| `?` | Show help |
| `q` / `Esc` | Quit |

## Screenshots

```
┌─────────────────────────────────────────────────────────────────┐
│ ccperm  [U]ser  [P]roject  [L]ocal  [?] Help  [q] Quit         │
├─────────────────────────────────────────────────────────────────┤
│ Allow (36) │ Deny (0) │ Ask (0)                                 │
├─────────────────────────────────────────────────────────────────┤
│ ▼ Git (5)                                                       │
│   ├─ Bash(git commit:*)                                         │
│   ├─ Bash(git push)                                             │
│   └─ Bash(git add:*)                                            │
│ ▼ NPM (4)                                                       │
│   ├─ Bash(npm install:*)                                        │
│   └─ Bash(npm run build:*)                                      │
│ ▶ GCloud (12)                                                   │
│ ▶ Skill (3)                                                     │
│ ▶ SlashCommand (2)                                              │
│ ▶ Other (8)                                                     │
├─────────────────────────────────────────────────────────────────┤
│ [a]dd [e]dit [d]elete [c]hange [m]ove s[o]rt [s]ave [r]eload   │
└─────────────────────────────────────────────────────────────────┘
```

## Configuration Files

ccperm reads and writes to the following files:

- `~/.claude/settings.json` — User settings (always available)
- `<git-root>/.claude/settings.json` — Project settings (in git repos)
- `<git-root>/.claude/settings.local.json` — Local settings (in git repos)

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
