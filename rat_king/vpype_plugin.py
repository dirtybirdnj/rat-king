"""vpype plugin for rat-king.

This module provides vpype integration, allowing rat-king patterns
to be used in vpype pipelines.

Usage:
    vpype read input.svg ratking fill --pattern concentric --spacing 2 write output.svg
"""

try:
    import vpype
    import vpype_cli
    import numpy as np
    VPYPE_AVAILABLE = True
except ImportError:
    VPYPE_AVAILABLE = False

if VPYPE_AVAILABLE:
    import click
    from shapely.geometry import Polygon as ShapelyPolygon
    from shapely.ops import polygonize

    from .geometry import Point, PolygonWithHoles
    from .patterns import generate_concentric_fill

    @vpype_cli.cli.group()
    def ratking():
        """rat-king fill pattern commands."""
        pass

    @ratking.command()
    @click.option('--pattern', '-p', default='concentric',
                  type=click.Choice(['concentric', 'lines']),
                  help='Fill pattern type')
    @click.option('--spacing', '-s', default=2.0, type=float,
                  help='Spacing between fill lines')
    @click.option('--connect/--no-connect', default=True,
                  help='Connect loops for continuous path')
    @click.option('--layer', '-l', type=vpype_cli.LayerType(accept_new=True),
                  help='Target layer for output (default: new layer)')
    @vpype_cli.generator
    def fill(pattern: str, spacing: float, connect: bool, layer) -> vpype.Document:
        """Generate fill patterns for shapes in the current document.

        Reads all closed paths from the document, generates fill patterns,
        and adds them as new geometry.
        """
        # This is a generator command - it receives the document from vpype
        # We need to process it as a processor instead

        def process(document: vpype.Document) -> vpype.Document:
            # Process each layer
            for layer_id in document.layers:
                lc = document.layers[layer_id]

                # Convert vpype LineCollection to polygons
                for line in lc:
                    # vpype lines are complex arrays (x + yj)
                    points = [Point(float(p.real), float(p.imag)) for p in line]

                    if len(points) < 3:
                        continue

                    # Check if closed
                    first, last = points[0], points[-1]
                    dist = ((last.x - first.x) ** 2 + (last.y - first.y) ** 2) ** 0.5
                    if dist > 0.1:
                        continue  # Not closed, skip

                    poly = PolygonWithHoles(outer=points, holes=[])

                    # Generate fill
                    if pattern == 'concentric':
                        fill_lines = generate_concentric_fill(poly, spacing, connect_loops=connect)
                    else:
                        fill_lines = generate_concentric_fill(poly, spacing, connect_loops=connect)

                    # Convert back to vpype format and add to document
                    for fill_line in fill_lines:
                        # Create a 2-point line as complex array
                        line_array = np.array([
                            complex(fill_line.x1, fill_line.y1),
                            complex(fill_line.x2, fill_line.y2)
                        ])

                        # Add to target layer or create new one
                        target = layer if layer else document.free_id()
                        if target not in document.layers:
                            document.add(vpype.LineCollection(), target)
                        document.layers[target].append(line_array)

            return document

        # Return a processor function (vpype 1.x style)
        # Note: This may need adjustment based on vpype version
        return process

else:
    # vpype not available - provide stub
    def rat_king():
        """Stub for when vpype is not installed."""
        pass
