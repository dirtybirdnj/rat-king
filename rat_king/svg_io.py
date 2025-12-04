"""SVG input/output utilities for rat-king."""

import sys
import re
from typing import List, Tuple, Optional
from xml.etree import ElementTree as ET

from .geometry import Point, HatchLine, PolygonWithHoles


def parse_path_d(d: str) -> List[Point]:
    """Parse SVG path d attribute into points.

    Simplified parser that handles M, L, H, V, Z commands and their lowercase variants.
    Curves are approximated as straight lines to their endpoints.
    """
    if not d or not d.strip():
        return []

    # Match commands and their arguments
    command_regex = r'[MLHVCSQTAZmlhvcsqtaz][^MLHVCSQTAZmlhvcsqtaz]*'
    commands = re.findall(command_regex, d, re.IGNORECASE)

    points: List[Point] = []
    current_x, current_y = 0.0, 0.0
    start_x, start_y = 0.0, 0.0

    for cmd in commands:
        cmd_type = cmd[0]
        args_str = cmd[1:].strip()
        args = [float(x) for x in re.findall(r'-?[\d.]+(?:[eE][+-]?\d+)?', args_str)]

        if cmd_type == 'M':
            if len(args) >= 2:
                current_x, current_y = args[0], args[1]
                start_x, start_y = current_x, current_y
                points.append(Point(current_x, current_y))
                # Implicit lineto for remaining args
                for i in range(2, len(args) - 1, 2):
                    current_x, current_y = args[i], args[i + 1]
                    points.append(Point(current_x, current_y))

        elif cmd_type == 'm':
            if len(args) >= 2:
                current_x += args[0]
                current_y += args[1]
                start_x, start_y = current_x, current_y
                points.append(Point(current_x, current_y))
                for i in range(2, len(args) - 1, 2):
                    current_x += args[i]
                    current_y += args[i + 1]
                    points.append(Point(current_x, current_y))

        elif cmd_type == 'L':
            for i in range(0, len(args) - 1, 2):
                current_x, current_y = args[i], args[i + 1]
                points.append(Point(current_x, current_y))

        elif cmd_type == 'l':
            for i in range(0, len(args) - 1, 2):
                current_x += args[i]
                current_y += args[i + 1]
                points.append(Point(current_x, current_y))

        elif cmd_type == 'H':
            for x in args:
                current_x = x
                points.append(Point(current_x, current_y))

        elif cmd_type == 'h':
            for dx in args:
                current_x += dx
                points.append(Point(current_x, current_y))

        elif cmd_type == 'V':
            for y in args:
                current_y = y
                points.append(Point(current_x, current_y))

        elif cmd_type == 'v':
            for dy in args:
                current_y += dy
                points.append(Point(current_x, current_y))

        elif cmd_type in ('Z', 'z'):
            # Close path - add point back to start if not already there
            dist = ((current_x - start_x) ** 2 + (current_y - start_y) ** 2) ** 0.5
            if dist > 0.1:
                points.append(Point(start_x, start_y))
            current_x, current_y = start_x, start_y

        elif cmd_type == 'C':
            # Cubic bezier - just take endpoint
            for i in range(0, len(args) - 5, 6):
                current_x, current_y = args[i + 4], args[i + 5]
                points.append(Point(current_x, current_y))

        elif cmd_type == 'c':
            for i in range(0, len(args) - 5, 6):
                current_x += args[i + 4]
                current_y += args[i + 5]
                points.append(Point(current_x, current_y))

        elif cmd_type == 'Q':
            for i in range(0, len(args) - 3, 4):
                current_x, current_y = args[i + 2], args[i + 3]
                points.append(Point(current_x, current_y))

        elif cmd_type == 'q':
            for i in range(0, len(args) - 3, 4):
                current_x += args[i + 2]
                current_y += args[i + 3]
                points.append(Point(current_x, current_y))

        elif cmd_type == 'A':
            for i in range(0, len(args) - 6, 7):
                current_x, current_y = args[i + 5], args[i + 6]
                points.append(Point(current_x, current_y))

        elif cmd_type == 'a':
            for i in range(0, len(args) - 6, 7):
                current_x += args[i + 5]
                current_y += args[i + 6]
                points.append(Point(current_x, current_y))

        # S, s, T, t are smooth curves - also just take endpoints
        elif cmd_type in ('S', 's'):
            for i in range(0, len(args) - 3, 4):
                if cmd_type == 'S':
                    current_x, current_y = args[i + 2], args[i + 3]
                else:
                    current_x += args[i + 2]
                    current_y += args[i + 3]
                points.append(Point(current_x, current_y))

        elif cmd_type in ('T', 't'):
            for i in range(0, len(args) - 1, 2):
                if cmd_type == 'T':
                    current_x, current_y = args[i], args[i + 1]
                else:
                    current_x += args[i]
                    current_y += args[i + 1]
                points.append(Point(current_x, current_y))

    return points


def element_to_polygon(element: ET.Element, ns: dict) -> Optional[PolygonWithHoles]:
    """Convert an SVG element to a PolygonWithHoles."""
    tag = element.tag.split('}')[-1].lower()  # Remove namespace

    points: List[Point] = []

    if tag == 'path':
        d = element.get('d', '')
        points = parse_path_d(d)

    elif tag == 'polygon':
        points_attr = element.get('points', '')
        coords = re.findall(r'-?[\d.]+', points_attr)
        for i in range(0, len(coords) - 1, 2):
            points.append(Point(float(coords[i]), float(coords[i + 1])))

    elif tag == 'polyline':
        points_attr = element.get('points', '')
        coords = re.findall(r'-?[\d.]+', points_attr)
        for i in range(0, len(coords) - 1, 2):
            points.append(Point(float(coords[i]), float(coords[i + 1])))
        # Close the polyline
        if len(points) >= 2:
            first, last = points[0], points[-1]
            dist = ((last.x - first.x) ** 2 + (last.y - first.y) ** 2) ** 0.5
            if dist > 1:
                points.append(Point(first.x, first.y))

    elif tag == 'rect':
        x = float(element.get('x', 0))
        y = float(element.get('y', 0))
        w = float(element.get('width', 0))
        h = float(element.get('height', 0))
        points = [
            Point(x, y),
            Point(x + w, y),
            Point(x + w, y + h),
            Point(x, y + h),
            Point(x, y)
        ]

    elif tag == 'circle':
        import math
        cx = float(element.get('cx', 0))
        cy = float(element.get('cy', 0))
        r = float(element.get('r', 0))
        segments = 32
        for i in range(segments + 1):
            angle = (i / segments) * math.pi * 2
            points.append(Point(cx + r * math.cos(angle), cy + r * math.sin(angle)))

    elif tag == 'ellipse':
        import math
        cx = float(element.get('cx', 0))
        cy = float(element.get('cy', 0))
        rx = float(element.get('rx', 0))
        ry = float(element.get('ry', 0))
        segments = 32
        for i in range(segments + 1):
            angle = (i / segments) * math.pi * 2
            points.append(Point(cx + rx * math.cos(angle), cy + ry * math.sin(angle)))

    if len(points) < 3:
        return None

    return PolygonWithHoles(outer=points, holes=[])


def extract_polygons_from_svg(svg_content: str) -> Tuple[List[PolygonWithHoles], dict]:
    """Extract all fillable polygons from SVG content.

    Returns:
        Tuple of (list of polygons, SVG metadata dict with viewBox, width, height)
    """
    # Parse SVG
    root = ET.fromstring(svg_content)

    # Extract SVG namespace
    ns = {}
    if root.tag.startswith('{'):
        ns_end = root.tag.find('}')
        ns['svg'] = root.tag[1:ns_end]

    # Extract viewBox and dimensions
    metadata = {
        'viewBox': root.get('viewBox', ''),
        'width': root.get('width', ''),
        'height': root.get('height', ''),
    }

    polygons: List[PolygonWithHoles] = []

    # Find all shape elements
    shape_tags = ['path', 'polygon', 'polyline', 'rect', 'circle', 'ellipse']

    def process_element(elem: ET.Element):
        tag = elem.tag.split('}')[-1].lower()
        if tag in shape_tags:
            poly = element_to_polygon(elem, ns)
            if poly:
                polygons.append(poly)

        # Recurse into children
        for child in elem:
            process_element(child)

    process_element(root)

    return polygons, metadata


def lines_to_svg_path(lines: List[HatchLine], precision: int = 2) -> str:
    """Convert a list of HatchLines to an SVG path d attribute."""
    if not lines:
        return ''

    commands = []
    for line in lines:
        commands.append(
            f"M{line.x1:.{precision}f},{line.y1:.{precision}f} "
            f"L{line.x2:.{precision}f},{line.y2:.{precision}f}"
        )

    return ' '.join(commands)


def create_svg_from_lines(
    lines: List[HatchLine],
    viewbox: str = '',
    width: str = '',
    height: str = '',
    stroke: str = 'black',
    stroke_width: str = '1'
) -> str:
    """Create a complete SVG document from fill lines.

    Args:
        lines: List of line segments
        viewbox: SVG viewBox attribute
        width: SVG width attribute
        height: SVG height attribute
        stroke: Stroke color
        stroke_width: Stroke width

    Returns:
        Complete SVG document as string
    """
    path_d = lines_to_svg_path(lines)

    # Build attributes
    attrs = ['xmlns="http://www.w3.org/2000/svg"']
    if viewbox:
        attrs.append(f'viewBox="{viewbox}"')
    if width:
        attrs.append(f'width="{width}"')
    if height:
        attrs.append(f'height="{height}"')

    svg = f'''<?xml version="1.0" encoding="UTF-8"?>
<svg {' '.join(attrs)}>
  <path d="{path_d}" fill="none" stroke="{stroke}" stroke-width="{stroke_width}"/>
</svg>'''

    return svg


def read_svg(path: Optional[str] = None) -> str:
    """Read SVG content from file or stdin.

    Args:
        path: File path, or None to read from stdin

    Returns:
        SVG content as string
    """
    if path is None or path == '-':
        return sys.stdin.read()
    else:
        with open(path, 'r') as f:
            return f.read()


def write_svg(content: str, path: Optional[str] = None):
    """Write SVG content to file or stdout.

    Args:
        content: SVG content
        path: File path, or None to write to stdout
    """
    if path is None or path == '-':
        sys.stdout.write(content)
    else:
        with open(path, 'w') as f:
            f.write(content)
