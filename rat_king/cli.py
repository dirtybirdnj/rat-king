"""Command-line interface for rat-king."""

import sys
import time
import click

from .svg_io import (
    read_svg,
    write_svg,
    extract_polygons_from_svg,
    create_svg_from_lines,
)
from .patterns import generate_concentric_fill
from .geometry import HatchLine


@click.group()
@click.version_option()
def main():
    """rat-king: Fill pattern generator for pen plotters.

    Generate line fill patterns for SVG shapes, designed for pen plotter workflows.
    Can be used standalone or as a vpype plugin.

    Examples:

        rat-king fill input.svg -o output.svg

        cat input.svg | rat-king fill --pattern concentric --spacing 2 > output.svg
    """
    pass


@main.command()
@click.argument('input', default='-', required=False)
@click.option('-o', '--output', default='-', help='Output file (default: stdout)')
@click.option('--pattern', '-p', default='concentric',
              type=click.Choice(['concentric', 'lines']),
              help='Fill pattern type')
@click.option('--spacing', '-s', default=2.0, type=float,
              help='Spacing between fill lines (default: 2.0)')
@click.option('--connect/--no-connect', default=True,
              help='Connect loops for continuous path (default: connect)')
@click.option('--stroke', default='black', help='Stroke color (default: black)')
@click.option('--stroke-width', default='1', help='Stroke width (default: 1)')
@click.option('--verbose', '-v', is_flag=True, help='Print timing and statistics')
def fill(input, output, pattern, spacing, connect, stroke, stroke_width, verbose):
    """Generate fill patterns for shapes in an SVG file.

    INPUT: SVG file path, or - for stdin (default)

    Reads an SVG, extracts all closed shapes (paths, polygons, rects, circles),
    and generates fill line patterns for each shape.
    """
    start_time = time.time()

    # Read input SVG
    try:
        svg_content = read_svg(input if input != '-' else None)
    except Exception as e:
        click.echo(f"Error reading input: {e}", err=True)
        sys.exit(1)

    if verbose:
        click.echo(f"Read {len(svg_content)} bytes", err=True)

    # Extract polygons
    polygons, metadata = extract_polygons_from_svg(svg_content)

    if verbose:
        click.echo(f"Found {len(polygons)} shapes", err=True)

    if not polygons:
        click.echo("No fillable shapes found in input", err=True)
        sys.exit(1)

    # Generate fill patterns
    all_lines: list[HatchLine] = []

    for i, poly in enumerate(polygons):
        if verbose:
            click.echo(f"Processing shape {i + 1}/{len(polygons)}...", err=True)

        if pattern == 'concentric':
            lines = generate_concentric_fill(poly, spacing, connect_loops=connect)
        else:
            # lines pattern - not yet implemented, use concentric as fallback
            click.echo(f"Pattern '{pattern}' not yet implemented, using concentric", err=True)
            lines = generate_concentric_fill(poly, spacing, connect_loops=connect)

        all_lines.extend(lines)

    if verbose:
        click.echo(f"Generated {len(all_lines)} line segments", err=True)

    # Create output SVG
    output_svg = create_svg_from_lines(
        all_lines,
        viewbox=metadata.get('viewBox', ''),
        width=metadata.get('width', ''),
        height=metadata.get('height', ''),
        stroke=stroke,
        stroke_width=stroke_width,
    )

    # Write output
    try:
        write_svg(output_svg, output if output != '-' else None)
    except Exception as e:
        click.echo(f"Error writing output: {e}", err=True)
        sys.exit(1)

    elapsed = time.time() - start_time
    if verbose:
        click.echo(f"Completed in {elapsed:.3f}s", err=True)


@main.command()
def patterns():
    """List available fill patterns."""
    click.echo("Available patterns:")
    click.echo()
    click.echo("  concentric  - Nested polygon outlines, shrinking inward")
    click.echo("  lines       - Parallel line hatching (coming soon)")
    click.echo()
    click.echo("Use: rat-king fill --pattern <name> input.svg")


if __name__ == '__main__':
    main()
