# .aether/ - Aether Memory Directory

This directory contains Aether's project-specific memory and configuration.

## Structure

- `memory.toml` - Validation memories (decisions, patterns)
- `config.toml` - Project-specific validation config
- `graph.json` - Code dependency graph
- `drift/` - Architectural drift snapshots
- `decisions/` - Individual decision entries

## Git Integration

This directory is designed to be versioned with git. 
Team members will share the same memory and learned patterns.

## Files to commit

- All `.toml` and `.json` files (except in cache/)
- `drift/` snapshots
- `decisions/` entries

## Files to ignore (see .gitignore)

- `cache/` - Temporary cache files
- `*.tmp`, `*.lock` - Runtime files

## How it works

Aether learns from your validation decisions and stores them here.
Over time, it builds project-specific knowledge that improves validation accuracy.

## Benefits of versioning

- **Team sync**: Everyone works with the same learned patterns
- **History**: See how validation decisions evolved
- **Rollback**: Return to previous memory states if needed
- **Code review**: Memory changes can be reviewed like code
