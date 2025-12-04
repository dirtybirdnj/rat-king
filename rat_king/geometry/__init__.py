"""Geometry utilities for rat-king."""

from .types import Point, HatchLine, PolygonWithHoles
from .polygon import (
    polygon_signed_area,
    point_in_polygon,
    offset_polygon_inward,
)

__all__ = [
    "Point",
    "HatchLine",
    "PolygonWithHoles",
    "polygon_signed_area",
    "point_in_polygon",
    "offset_polygon_inward",
]
