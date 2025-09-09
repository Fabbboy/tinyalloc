# AGENTS.md

## Instructions for AI Agents

When working with this project, always:

1. **Reference CLAUDE.md first** - Contains essential project instructions, build system details, and architecture guidelines
2. **Check all markdown files** in the project root and subdirectories for additional context
3. **Exclude vendor directories** - Do not reference any markdown files under `vendor/` paths as they contain external documentation

Follow the guidelines in CLAUDE.md exactly as written - they override default behaviors.

## Prerequisites

Before working with this project:

1. **Install Just**: If `just` command is not available, install it using:
   ```bash
   cargo install just
   ```

2. **Update Git Submodules**: If CMake configuration files are missing or build fails, update submodules:
   ```bash
   git submodule update --init --recursive
   ```

These steps ensure all dependencies and build tools are properly configured.