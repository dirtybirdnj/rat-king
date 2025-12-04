"""Tests for concentric fill pattern."""

import pytest
from rat_king.geometry import Point, PolygonWithHoles
from rat_king.patterns import generate_concentric_fill


def test_simple_square():
    """Test concentric fill on a simple square."""
    # 10x10 square
    square = PolygonWithHoles(
        outer=[
            Point(0, 0),
            Point(10, 0),
            Point(10, 10),
            Point(0, 10),
            Point(0, 0),
        ],
        holes=[]
    )

    lines = generate_concentric_fill(square, spacing=2.0)

    # Should generate multiple loops
    assert len(lines) > 4  # At least outer loop (4 sides)

    # All lines should be within bounds
    for line in lines:
        assert 0 <= line.x1 <= 10
        assert 0 <= line.y1 <= 10
        assert 0 <= line.x2 <= 10
        assert 0 <= line.y2 <= 10


def test_empty_polygon():
    """Test with empty polygon."""
    empty = PolygonWithHoles(outer=[], holes=[])
    lines = generate_concentric_fill(empty, spacing=2.0)
    assert len(lines) == 0


def test_triangle():
    """Test concentric fill on a triangle."""
    triangle = PolygonWithHoles(
        outer=[
            Point(5, 0),
            Point(10, 10),
            Point(0, 10),
            Point(5, 0),
        ],
        holes=[]
    )

    lines = generate_concentric_fill(triangle, spacing=1.0)
    assert len(lines) > 3  # At least outer loop


def test_no_connect():
    """Test without loop connection."""
    square = PolygonWithHoles(
        outer=[
            Point(0, 0),
            Point(10, 0),
            Point(10, 10),
            Point(0, 10),
            Point(0, 0),
        ],
        holes=[]
    )

    lines_connected = generate_concentric_fill(square, spacing=2.0, connect_loops=True)
    lines_disconnected = generate_concentric_fill(square, spacing=2.0, connect_loops=False)

    # Connected should have more lines (the connecting segments)
    assert len(lines_connected) >= len(lines_disconnected)


def test_small_spacing():
    """Test with small spacing (many loops)."""
    square = PolygonWithHoles(
        outer=[
            Point(0, 0),
            Point(100, 0),
            Point(100, 100),
            Point(0, 100),
            Point(0, 0),
        ],
        holes=[]
    )

    lines = generate_concentric_fill(square, spacing=5.0)

    # Should generate many loops for 100x100 square with spacing 5
    # At least several loops worth of lines
    assert len(lines) > 20
