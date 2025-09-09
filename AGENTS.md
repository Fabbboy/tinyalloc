# AGENTS.md

## Instructions for AI Agents

When working with this project, always:

1. **Check INDEX.md first** - Fast-path lookup for components, functions, and concepts
2. **Reference CLAUDE.md** - Contains essential project instructions, build system details, and architecture guidelines  
3. **Update INDEX.md** when discovering new information for future agent reference
4. **Check all markdown files** in the project root and subdirectories for additional context
5. **Exclude vendor directories** - Do not reference any markdown files under `vendor/` paths as they contain external documentation

### INDEX.md Usage Protocol

- **Before any search**: Check if INDEX.md already contains the answer
- **After finding answers**: Update INDEX.md with new findings  
- **Query examples**: "How does mimalloc handle tcache?" → Check INDEX.md → Search if needed → Update INDEX.md
- **Keep current**: INDEX.md should be your knowledge base for the project

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