# Modern TUI User Guide

The modern TUI (Terminal User Interface) provides a rich interactive experience for ckb-cli on Unix platforms (Linux/macOS).

## Quick Start

```bash
# Launch modern TUI (default on Unix)
ckb-cli

# Use classic rustyline REPL instead
ckb-cli --classic
```

## Interface Layout

```
┌─────────────────────────────────────────────────────────────────────┐
│ CKB CLI v2.0.0 | Chain: mainnet | Height: 12,345,678 | Peers: 42   │ <- Status Bar
├──────────────────────────────────┬──────────────────────────────────┤
│ [Command] [Logs]                 │                                  │ <- Tab Bar
├──────────────────────────────────┤  📊 Chain Status                 │
│                                  │  ━━━━━━━━━━━━━━━━                │
│  📄 Output Panel                 │                                  │
│                                  │  Height: 12345678                │
│  > rpc get_tip_header            │  Epoch:  123                     │
│  {                               │  Sync:   ✓                       │
│    "number": "12345678",         │  Peers:  42                      │
│    "hash": "0x123..."            │  Last:   2s ago                  │
│  }                               │                                  │
│                                  │  ━━━━━━━━━━━━━━━━                │
│                                  │  📝 Recent Commands              │
│                                  │  ━━━━━━━━━━━━━━━━                │
│                                  │  rpc get_tip_header              │
│                                  │  wallet get-capacity             │
├──────────────────────────────────┤                                  │
│ CKB> rpc get_blockchain_info█    │                                  │ <- Input
├──────────────────────────────────┴──────────────────────────────────┤
│ ^P Palette | ^R History | ^E Export | ^C Exit | ? Help              │ <- Help Bar
└─────────────────────────────────────────────────────────────────────┘
```

### Panes

| Pane | Description |
|------|-------------|
| **Status Bar** | Shows version, chain name, block height, and peer count |
| **Tab Bar** | Switch between Command and Logs views |
| **Output Panel** | Scrollable command output with syntax highlighting |
| **Sidebar** | Live chain status and recent command history |
| **Input Area** | Command entry with completion support |
| **Help Bar** | Quick reference for keyboard shortcuts |

## Keyboard Shortcuts

### Global

| Key | Action |
|-----|--------|
| `Ctrl+C` | Exit TUI |
| `Ctrl+L` | Clear output |
| `Ctrl+P` | Open command palette |
| `Ctrl+E` | Export output to file |
| `Ctrl+R` | Search command history |
| `?` | Toggle help overlay |
| `F1` | Focus sidebar |
| `F2` | Focus output |
| `F3` | Focus input |

### Tab Navigation

| Key | Action |
|-----|--------|
| `Ctrl+Tab` | Switch to next tab |
| `Alt+1` | Switch to Command tab |
| `Alt+2` | Switch to Logs tab |

### Input Area

| Key | Action |
|-----|--------|
| `Enter` | Execute command |
| `Tab` | Show/cycle completions |
| `Shift+Tab` | Previous completion |
| `↑` / `↓` | Navigate command history |
| `Esc` | Cancel completion/search |

### Output Area (when focused)

| Key | Action |
|-----|--------|
| `/` | Search in output |
| `Esc` | Exit search mode |

## Features

### Command Palette (Ctrl+P)

Quick fuzzy search for all available commands and subcommands:

1. Press `Ctrl+P` to open
2. Type to filter commands
3. Use `↑`/`↓` to navigate
4. Press `Enter` to select
5. Press `Esc` to cancel

### Command Completion (Tab)

Context-aware completion for commands, subcommands, and flags:

- Type partial command and press `Tab`
- Completions show with color coding:
  - Green: Commands/subcommands
  - Cyan: Flags
  - Red: Required flags (marked with `*`)
- Fuzzy matching supported (e.g., `wlt` matches `wallet`)

### History Search (Ctrl+R)

Incremental search through command history:

1. Press `Ctrl+R` to open search
2. Type to filter history
3. Use `↑`/`↓` to navigate matches
4. Press `Enter` to select
5. Press `Esc` to cancel

### Output Search (/)

Search within command output:

1. Focus the output pane (`F2`)
2. Press `/` to enter search mode
3. Type search query
4. Matches are highlighted in yellow
5. Press `Esc` to exit search mode (highlights remain)

### Syntax Highlighting

Output is automatically syntax-highlighted:

| Element | Color |
|---------|-------|
| JSON keys | Blue |
| Strings | Orange |
| Numbers | Green |
| Booleans | Blue |
| Null | Gray |
| Errors | Red |

### Live Chain Status

The sidebar displays real-time chain information (updates every 2 seconds):

- **Height**: Current block number
- **Epoch**: Current epoch number
- **Sync**: Sync status indicator
- **Peers**: Connected peer count
- **Last**: Time since last update

### Tabs

Switch between different views:

- **Command**: Main command output (default)
- **Logs**: Timestamped log entries for debugging

### Export (Ctrl+E)

Export all output to a file:

- Files saved to `~/.ckb-cli/exports/output_<timestamp>.txt`
- Includes all commands and their output from the session

### Mouse Support

- Click to focus panes
- Scroll wheel to navigate output

## Configuration

The TUI respects existing ckb-cli configuration in `~/.ckb-cli/config`:

```json
{
  "url": "http://127.0.0.1:8114",
  "color": true,
  "output_format": "yaml"
}
```

### History File

Command history is shared with classic mode:

- Location: `~/.ckb-cli/history`
- Format: Plain text, one command per line
- Max entries: 1000

## Troubleshooting

### TUI doesn't start

- Ensure you're on a Unix platform (Linux/macOS)
- Check terminal supports 256 colors
- Try `ckb-cli --classic` for fallback mode

### Display issues

- Resize terminal window
- Ensure terminal font supports Unicode
- Try a different terminal emulator

### Connection errors

- Verify CKB node is running
- Check `API_URL` environment variable
- Use `ckb-cli --url <URL>` to specify node

## Classic Mode

For users who prefer the traditional readline interface:

```bash
ckb-cli --classic
```

Classic mode provides:
- Simple readline-based input
- Basic command completion
- Command history (shared with TUI)

## Platform Support

| Platform | TUI Support |
|----------|-------------|
| Linux | ✅ Full support |
| macOS | ✅ Full support |
| Windows | ❌ Uses classic mode |
