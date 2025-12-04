"""Concentric fill pattern generator.

Ported from svg-grouper's src/utils/fillPatterns.ts
"""

import math
from typing import List
from ..geometry import Point, HatchLine, PolygonWithHoles
from ..geometry.polygon import polygon_signed_area, robust_inset_polygon


def generate_concentric_fill(
    polygon: PolygonWithHoles,
    spacing: float,
    connect_loops: bool = True
) -> List[HatchLine]:
    """Generate concentric fill lines (snake pattern from outside in).

    Args:
        polygon: The polygon to fill (outer boundary with optional holes)
        spacing: Distance between concentric rings
        connect_loops: If True, add connecting lines between loops for continuous path

    Returns:
        List of line segments representing the fill pattern
    """
    lines: List[HatchLine] = []
    outer = polygon.outer

    if len(outer) < 3:
        return lines

    min_area = spacing * spacing * 0.5  # Reduced threshold for small shapes

    # Calculate bounds and max iterations
    min_x = min(p.x for p in outer)
    max_x = max(p.x for p in outer)
    min_y = min(p.y for p in outer)
    max_y = max(p.y for p in outer)

    max_dimension = max(max_x - min_x, max_y - min_y)
    max_loops = min(100, int(math.ceil(max_dimension / spacing)) + 2)

    loops: List[List[Point]] = []
    current_polygon = list(outer)
    last_area = abs(polygon_signed_area(current_polygon))

    for _ in range(max_loops):
        if len(current_polygon) < 3 or last_area < min_area:
            break

        loops.append(list(current_polygon))

        # Use robust inset that handles complex shapes
        current_polygon = robust_inset_polygon(current_polygon, spacing)

        if len(current_polygon) < 3:
            break

        new_area = abs(polygon_signed_area(current_polygon))
        if new_area >= last_area or new_area < min_area:
            break
        last_area = new_area

    # If no loops were generated, at least draw the original polygon outline
    if len(loops) == 0 and len(outer) >= 3:
        loops.append(list(outer))

    # Convert loops to line segments
    for loop_idx, loop in enumerate(loops):
        # Draw the loop as connected line segments
        for i in range(len(loop)):
            j = (i + 1) % len(loop)
            lines.append(HatchLine(
                x1=loop[i].x,
                y1=loop[i].y,
                x2=loop[j].x,
                y2=loop[j].y
            ))

        # Connect to next loop if requested
        if connect_loops and loop_idx < len(loops) - 1:
            next_loop = loops[loop_idx + 1]
            last_point = loop[-1]

            # Find closest point on next loop
            closest_idx = 0
            closest_dist = float('inf')

            for i, p in enumerate(next_loop):
                d = math.sqrt(
                    (p.x - last_point.x) ** 2 +
                    (p.y - last_point.y) ** 2
                )
                if d < closest_dist:
                    closest_dist = d
                    closest_idx = i

            # Add connecting line
            lines.append(HatchLine(
                x1=last_point.x,
                y1=last_point.y,
                x2=next_loop[closest_idx].x,
                y2=next_loop[closest_idx].y
            ))

    return lines
