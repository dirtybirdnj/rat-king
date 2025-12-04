"""Polygon operations for rat-king.

Ported from svg-grouper's src/utils/geometry.ts
"""

import math
from typing import List
from .types import Point


def polygon_signed_area(polygon: List[Point]) -> float:
    """Calculate the signed area of a polygon.

    Positive = counter-clockwise, negative = clockwise.
    """
    if len(polygon) < 3:
        return 0.0

    area = 0.0
    n = len(polygon)
    for i in range(n):
        j = (i + 1) % n
        area += polygon[i].x * polygon[j].y
        area -= polygon[j].x * polygon[i].y

    return area / 2.0


def point_in_polygon(point: Point, polygon: List[Point]) -> bool:
    """Check if a point is inside a polygon using ray casting."""
    if len(polygon) < 3:
        return False

    inside = False
    n = len(polygon)

    j = n - 1
    for i in range(n):
        xi, yi = polygon[i].x, polygon[i].y
        xj, yj = polygon[j].x, polygon[j].y

        if ((yi > point.y) != (yj > point.y)) and \
           (point.x < (xj - xi) * (point.y - yi) / (yj - yi) + xi):
            inside = not inside

        j = i

    return inside


def offset_polygon_inward(polygon: List[Point], offset_distance: float) -> List[Point]:
    """Offset a polygon inward by a given distance.

    Uses miter joints with limited extension to prevent spikes.
    """
    if len(polygon) < 3:
        return []

    signed_area = polygon_signed_area(polygon)
    winding_sign = 1 if signed_area > 0 else -1

    result = []
    n = len(polygon)

    for i in range(n):
        prev = polygon[(i - 1 + n) % n]
        curr = polygon[i]
        next_pt = polygon[(i + 1) % n]

        # Edge vectors
        e1x = curr.x - prev.x
        e1y = curr.y - prev.y
        e2x = next_pt.x - curr.x
        e2y = next_pt.y - curr.y

        len1 = math.sqrt(e1x * e1x + e1y * e1y)
        len2 = math.sqrt(e2x * e2x + e2y * e2y)

        if len1 < 0.0001 or len2 < 0.0001:
            continue

        # Normals (perpendicular to edges, pointing inward)
        n1x = -e1y / len1 * winding_sign
        n1y = e1x / len1 * winding_sign
        n2x = -e2y / len2 * winding_sign
        n2y = e2x / len2 * winding_sign

        # Average normal (bisector direction)
        nx = n1x + n2x
        ny = n1y + n2y
        nlen = math.sqrt(nx * nx + ny * ny)

        if nlen < 0.0001:
            nx, ny = n1x, n1y
        else:
            nx /= nlen
            ny /= nlen

            # Miter scaling
            dot = n1x * nx + n1y * ny
            if abs(dot) > 0.1:
                miter_scale = 1 / abs(dot)
                # Limit miter to prevent spikes
                limited_scale = min(miter_scale, 2.5)
                nx *= limited_scale
                ny *= limited_scale

        result.append(Point(
            x=curr.x + nx * offset_distance,
            y=curr.y + ny * offset_distance
        ))

    return result


def segments_intersect(a1: Point, a2: Point, b1: Point, b2: Point) -> bool:
    """Check if two line segments intersect (excluding endpoints)."""
    def direction(p1: Point, p2: Point, p3: Point) -> float:
        return (p3.x - p1.x) * (p2.y - p1.y) - (p2.x - p1.x) * (p3.y - p1.y)

    d1 = direction(b1, b2, a1)
    d2 = direction(b1, b2, a2)
    d3 = direction(a1, a2, b1)
    d4 = direction(a1, a2, b2)

    if ((d1 > 0 and d2 < 0) or (d1 < 0 and d2 > 0)) and \
       ((d3 > 0 and d4 < 0) or (d3 < 0 and d4 > 0)):
        return True

    return False


def is_polygon_self_intersecting(polygon: List[Point]) -> bool:
    """Check if a polygon is self-intersecting."""
    n = len(polygon)
    if n < 4:
        return False

    for i in range(n):
        a1 = polygon[i]
        a2 = polygon[(i + 1) % n]

        for j in range(i + 2, n):
            # Skip adjacent edges
            if j == (i + n - 1) % n:
                continue

            b1 = polygon[j]
            b2 = polygon[(j + 1) % n]

            if segments_intersect(a1, a2, b1, b2):
                return True

    return False


def robust_inset_polygon(polygon: List[Point], inset_distance: float) -> List[Point]:
    """Robust polygon inset that handles complex shapes.

    First tries standard offset, then falls back to centroid scaling if needed.
    """
    if len(polygon) < 3:
        return []

    # First try standard offset
    offset_result = offset_polygon_inward(polygon, inset_distance)

    # Validate the result
    if len(offset_result) >= 3 and not is_polygon_self_intersecting(offset_result):
        original_area = abs(polygon_signed_area(polygon))
        new_area = abs(polygon_signed_area(offset_result))
        # Make sure area decreased (valid inset)
        if new_area < original_area and new_area > 0:
            return offset_result

    # Fallback: use centroid-based scaling
    centroid_x = sum(p.x for p in polygon) / len(polygon)
    centroid_y = sum(p.y for p in polygon) / len(polygon)

    # Calculate average distance to centroid
    avg_dist = sum(
        math.sqrt((p.x - centroid_x) ** 2 + (p.y - centroid_y) ** 2)
        for p in polygon
    ) / len(polygon)

    if avg_dist <= inset_distance:
        return []  # Would collapse to point

    scale = (avg_dist - inset_distance) / avg_dist

    return [
        Point(
            x=centroid_x + (p.x - centroid_x) * scale,
            y=centroid_y + (p.y - centroid_y) * scale
        )
        for p in polygon
    ]
