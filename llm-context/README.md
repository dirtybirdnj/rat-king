# LLM Context Files for rat-king

This folder contains documentation artifacts designed to help local LLM models (Qwen, Ollama, DeepSeek, etc.) understand the rat-king codebase without needing to scan all source files.

## For Claude Code Users

**Claude should ignore this folder unless explicitly asked.** These files are optimized for local LLMs with limited context windows. Claude Code can read the actual source files directly for more accurate responses.

## Files in This Folder

| File | Purpose | When to Include |
|------|---------|-----------------|
| `CODEBASE.md` | Project overview, directory structure, tech stack | Always include first |
| `API.md` | Function signatures, types, public interfaces | When modifying or extending code |
| `ARCHITECTURE.md` | System design, data flow, module relationships | When adding new patterns or features |

## Token Estimates

- `CODEBASE.md`: ~600 tokens
- `API.md`: ~1,200 tokens
- `ARCHITECTURE.md`: ~800 tokens
- **Total**: ~2,600 tokens

## How to Use with Local LLMs

### Quick Context (Just CODEBASE.md)
```
cat llm-context/CODEBASE.md
```

### Full Context (All Files)
```
cat llm-context/*.md
```

### Task-Specific Inclusion

**"Add a new fill pattern"**
```
cat llm-context/CODEBASE.md llm-context/ARCHITECTURE.md llm-context/API.md
```

**"How does clipping work?"**
```
cat llm-context/ARCHITECTURE.md
```

**"What patterns are available?"**
```
cat llm-context/CODEBASE.md
```

## Example Prompts for Local LLMs

### Adding a New Pattern
```
<context>
[paste CODEBASE.md, API.md, ARCHITECTURE.md]
</context>

I want to add a new "checkerboard" pattern that creates alternating filled/empty squares.
Show me how to implement it following the existing pattern conventions.
```

### Understanding the Codebase
```
<context>
[paste CODEBASE.md]
</context>

Explain the relationship between the rat-king library crate and the CLI crate.
```

### Debugging
```
<context>
[paste ARCHITECTURE.md]
</context>

My pattern lines are appearing outside the polygon boundary. Where in the code
does line clipping happen and how does it work?
```

## Keeping These Files Updated

When making significant changes to rat-king:

1. **New patterns**: Update `CODEBASE.md` (pattern list) and `API.md` (function signature)
2. **New modules**: Update `ARCHITECTURE.md` (module diagram) and `CODEBASE.md` (directory)
3. **API changes**: Update `API.md` with new/changed signatures
4. **New CLI commands**: Update `CODEBASE.md` (usage section)
