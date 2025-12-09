#!/usr/bin/env python3
"""
Shapely-based hatch fill benchmark.
Generates parallel lines and clips to polygon boundaries.

This is a straightforward implementation representing typical
"reach for Shapely" code. Not heavily optimized.

Usage:
    python benchmark_shapely.py [svg_file]
    python benchmark_shapely.py test_assets/essex.svg

Compare with rat-king:
    ./crates/target/release/rat-king benchmark test_assets/essex.svg -p lines
"""

import time
import math
import sys
from pathlib import Path

try:
    from shapely.geometry import Polygon, LineString, MultiPolygon
    from shapely.ops import unary_union
    from svgpathtools import svg2paths2
    import numpy as np
except ImportError as e:
    print(f"Missing dependency: {e}")
    print("Install with: pip install shapely svgpathtools numpy")
    sys.exit(1)


def svg_to_polygons(svg_path: str) -> list[Polygon]:
    """Extract polygons from SVG file using svgpathtools."""
    paths, attributes, svg_attributes = svg2paths2(svg_path)
    polygons = []

    for path in paths:
        if len(path) == 0:
            continue

        # Sample points along the path
        points = []
        num_samples = max(10, int(path.length() / 2))

        for i in range(num_samples):
            t = i / num_samples
            try:
                pt = path.point(t)
                points.append((pt.real, pt.imag))
            except:
                continue

        if len(points) >= 3:
            try:
                poly = Polygon(points)
                if poly.is_valid and poly.area > 0:
                    polygons.append(poly)
            except:
                continue

    return polygons


def generate_hatch_lines(
    polygon: Polygon,
    spacing: float = 2.5,
    angle_deg: float = 45.0
) -> list[tuple]:
    """
    Generate hatch lines for a polygon using Shapely.
    Returns list of ((x1,y1), (x2,y2)) tuples.
    """
    if not polygon.is_valid or polygon.is_empty:
        return []

    bounds = polygon.bounds  # (minx, miny, maxx, maxy)
    minx, miny, maxx, maxy = bounds

    # Expand bounds to account for rotation
    cx, cy = (minx + maxx) / 2, (miny + maxy) / 2
    diagonal = math.sqrt((maxx - minx)**2 + (maxy - miny)**2)

    # Generate parallel lines
    angle_rad = math.radians(angle_deg)
    cos_a, sin_a = math.cos(angle_rad), math.sin(angle_rad)

    lines = []

    # Line direction perpendicular to angle
    dx, dy = cos_a, sin_a
    # Step direction (perpendicular to line direction)
    step_x, step_y = -sin_a * spacing, cos_a * spacing

    # Number of lines needed
    num_lines = int(diagonal / spacing) + 2

    # Starting point (offset from center)
    start_offset = -num_lines * spacing / 2

    for i in range(num_lines):
        offset = start_offset + i * spacing

        # Line endpoints (extend beyond bounds)
        p1_x = cx + offset * (-sin_a) - diagonal * cos_a
        p1_y = cy + offset * cos_a - diagonal * sin_a
        p2_x = cx + offset * (-sin_a) + diagonal * cos_a
        p2_y = cy + offset * cos_a + diagonal * sin_a

        line = LineString([(p1_x, p1_y), (p2_x, p2_y)])

        # Clip to polygon
        try:
            clipped = line.intersection(polygon)

            if clipped.is_empty:
                continue

            # Handle different geometry types
            if clipped.geom_type == 'LineString':
                coords = list(clipped.coords)
                if len(coords) >= 2:
                    lines.append((coords[0], coords[-1]))
            elif clipped.geom_type == 'MultiLineString':
                for geom in clipped.geoms:
                    coords = list(geom.coords)
                    if len(coords) >= 2:
                        lines.append((coords[0], coords[-1]))
        except Exception:
            continue

    return lines


def benchmark_shapely(svg_path: str, spacing: float = 2.5, angle: float = 45.0):
    """Run the full benchmark."""
    print(f"Loading: {svg_path}")

    # Load polygons
    load_start = time.perf_counter()
    polygons = svg_to_polygons(svg_path)
    load_time = time.perf_counter() - load_start
    print(f"Loaded {len(polygons)} polygons in {load_time*1000:.1f}ms")

    # Generate hatch fills
    print(f"Generating lines fill (spacing={spacing}, angle={angle})...")

    gen_start = time.perf_counter()
    total_lines = 0

    for i, poly in enumerate(polygons):
        lines = generate_hatch_lines(poly, spacing, angle)
        total_lines += len(lines)

        # Progress indicator for long runs
        if (i + 1) % 50 == 0:
            elapsed = time.perf_counter() - gen_start
            print(f"  Progress: {i+1}/{len(polygons)} polygons, {elapsed:.1f}s elapsed")

    gen_time = time.perf_counter() - gen_start

    print()
    print("=" * 50)
    print("RESULTS (Python + Shapely)")
    print("=" * 50)
    print(f"Polygons:        {len(polygons)}")
    print(f"Lines generated: {total_lines}")
    print(f"Load time:       {load_time*1000:.1f}ms")
    print(f"Generation time: {gen_time*1000:.1f}ms")
    print(f"TOTAL TIME:      {(load_time + gen_time)*1000:.1f}ms ({load_time + gen_time:.2f}s)")
    print("=" * 50)

    return load_time + gen_time


if __name__ == "__main__":
    svg_path = sys.argv[1] if len(sys.argv) > 1 else "test_assets/essex.svg"

    if not Path(svg_path).exists():
        print(f"Error: {svg_path} not found")
        print("Usage: python benchmark_shapely.py [svg_file]")
        sys.exit(1)

    benchmark_shapely(svg_path)
