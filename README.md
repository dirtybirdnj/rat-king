# rat-king

A vpype plugin for pen plotter workflows: line fills, pattern generation, and path optimization.

## Installation

```bash
pip install rat-king

# With vpype integration
pip install rat-king[vpype]
```

## Standalone CLI Usage

```bash
# Fill shapes with concentric pattern
rat-king fill input.svg --pattern concentric --spacing 2 -o output.svg

# Read from stdin, write to stdout
cat input.svg | rat-king fill --pattern concentric --spacing 2 > output.svg
```

## vpype Plugin Usage

```bash
# Basic fill
vpype read input.svg ratking fill --pattern concentric --spacing 2 write output.svg

# Chain with other commands
vpype read input.svg ratking fill --pattern lines --spacing 3 linesort write output.svg
```

## Available Patterns

| Pattern | Description |
|---------|-------------|
| `concentric` | Nested polygon outlines, shrinking inward |
| `lines` | Parallel line hatching (coming soon) |
| `crosshatch` | Two sets of lines at angles (coming soon) |
| `spiral` | Archimedean spiral from center (coming soon) |

## Development

```bash
# Clone and install in development mode
git clone https://github.com/dirtybirdnj/rat-king.git
cd rat-king
pip install -e ".[dev,vpype]"

# Run tests
pytest

# Lint
ruff check .
```

## Related Projects

- [vpype](https://github.com/abey79/vpype) - Swiss-Army-knife CLI for pen plotter workflows
- [svg-grouper](https://github.com/dirtybirdnj/svg-grouper) - GUI for pen plotter SVG preparation
