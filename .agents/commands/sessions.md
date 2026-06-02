---
description: Manage Claude Code session history, aliases, and session metadata.
---

# Sessions Command

Manage Claude Code session history - list, load, alias, and edit sessions stored in `~/.claude/session-data/` with legacy reads from `~/.claude/sessions/`.

## Usage

`/sessions [list|load|alias|info|help] [options]`

## Actions

### List Sessions

Display all sessions with metadata, filtering, and pagination.

Use `/sessions info` when you need operator-surface context for a swarm: branch, worktree path, and session recency.

```bash
/sessions                              # List all sessions (default)
/sessions list                         # Same as above
/sessions list --limit 10              # Show 10 sessions
/sessions list --date 2026-02-01       # Filter by date
/sessions list --search abc            # Search by session ID
```

### Load Session

Load and display a session's content (by ID or alias).

```bash
/sessions load <id|alias>             # Load session
/sessions load 2026-02-01             # By date (for no-id sessions)
/sessions load a1b2c3d4               # By short ID
/sessions load my-alias               # By alias name
```

### Create Alias

```bash
/sessions alias <id> <name>           # Create alias
/sessions alias 2026-02-01 today-work # Create alias named "today-work"
```

### Remove Alias

```bash
/sessions alias --remove <name>        # Remove alias
/sessions unalias <name>              # Same as above
```

### Session Info

```bash
/sessions info <id|alias>              # Show session details
```

### List Aliases

```bash
/sessions aliases                      # List all aliases
```

## Notes

- Sessions are stored as markdown files in `~/.claude/session-data/` with legacy reads from `~/.claude/sessions/`
- Aliases are stored in `~/.claude/session-aliases.json`
- Session IDs can be shortened (first 4-8 characters usually unique enough)
- Use aliases for frequently referenced sessions
