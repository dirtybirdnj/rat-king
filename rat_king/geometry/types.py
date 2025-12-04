"""Type definitions for rat-king geometry."""

from dataclasses import dataclass
from typing import List


@dataclass
class Point:
    """2D point."""
    x: float
    y: float

    def __iter__(self):
        yield self.x
        yield self.y


@dataclass
class HatchLine:
    """A line segment defined by two endpoints."""
    x1: float
    y1: float
    x2: float
    y2: float

    @property
    def start(self) -> Point:
        return Point(self.x1, self.y1)

    @property
    def end(self) -> Point:
        return Point(self.x2, self.y2)


@dataclass
class PolygonWithHoles:
    """A polygon with optional holes."""
    outer: List[Point]
    holes: List[List[Point]]

    def __init__(self, outer: List[Point], holes: List[List[Point]] = None):
        self.outer = outer
        self.holes = holes if holes is not None else []
