//! Concentric fill pattern - polygon offset rings.

use crate::geometry::{Line, Point, Polygon};

/// Generate concentric fill lines (rings from outside in).
///
/// Creates nested polygon outlines with optional connecting lines.
pub fn generate_concentric_fill(
    polygon: &Polygon,
    spacing: f64,
    connect_loops: bool,
) -> Vec<Line> {
    let outer = &polygon.outer;
    if outer.len() < 3 {
        return Vec::new();
    }

    let Some((min_x, min_y, max_x, max_y)) = polygon.bounding_box() else {
        return Vec::new();
    };

    let max_dimension = (max_x - min_x).max(max_y - min_y);
    let max_loops = ((max_dimension / spacing).ceil() as usize + 2).min(100);
    let min_area = spacing * spacing * 0.5;

    let mut loops: Vec<Vec<Point>> = Vec::new();
    let mut current_polygon = outer.clone();

    for _ in 0..max_loops {
        if current_polygon.len() < 3 {
            break;
        }

        let area = polygon_area(&current_polygon).abs();
        if area < min_area {
            break;
        }

        loops.push(current_polygon.clone());

        // Inset the polygon
        current_polygon = inset_polygon(&current_polygon, spacing);

        if current_polygon.len() < 3 {
            break;
        }
    }

    // If no loops generated, at least draw the outline
    if loops.is_empty() && outer.len() >= 3 {
        loops.push(outer.clone());
    }

    let mut lines = Vec::new();

    for (loop_idx, loop_points) in loops.iter().enumerate() {
        // Draw the loop as connected line segments
        for i in 0..loop_points.len() {
            let j = (i + 1) % loop_points.len();
            lines.push(Line::new(
                loop_points[i].x, loop_points[i].y,
                loop_points[j].x, loop_points[j].y,
            ));
        }

        // Connect to next loop
        if connect_loops && loop_idx < loops.len() - 1 {
            let next_loop = &loops[loop_idx + 1];
            if let Some(last_point) = loop_points.last() {
                // Find closest point on next loop
                let closest = next_loop
                    .iter()
                    .min_by(|a, b| {
                        let da = (a.x - last_point.x).powi(2) + (a.y - last_point.y).powi(2);
                        let db = (b.x - last_point.x).powi(2) + (b.y - last_point.y).powi(2);
                        da.partial_cmp(&db).unwrap()
                    });

                if let Some(closest_point) = closest {
                    lines.push(Line::new(
                        last_point.x, last_point.y,
                        closest_point.x, closest_point.y,
                    ));
                }
            }
        }
    }

    lines
}

/// Calculate signed area of a polygon (shoelace formula).
fn polygon_area(points: &[Point]) -> f64 {
    let n = points.len();
    if n < 3 {
        return 0.0;
    }

    let mut area = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        area += points[i].x * points[j].y;
        area -= points[j].x * points[i].y;
    }
    area / 2.0
}

/// Simple centroid-based polygon inset.
///
/// Moves each vertex toward the centroid by the inset distance.
fn inset_polygon(points: &[Point], inset: f64) -> Vec<Point> {
    if points.len() < 3 {
        return Vec::new();
    }

    // Calculate centroid
    let centroid_x = points.iter().map(|p| p.x).sum::<f64>() / points.len() as f64;
    let centroid_y = points.iter().map(|p| p.y).sum::<f64>() / points.len() as f64;

    let mut result = Vec::with_capacity(points.len());

    for p in points {
        let dx = p.x - centroid_x;
        let dy = p.y - centroid_y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < inset {
            // Point collapses to centroid
            result.push(Point::new(centroid_x, centroid_y));
        } else {
            let scale = (dist - inset) / dist;
            result.push(Point::new(
                centroid_x + dx * scale,
                centroid_y + dy * scale,
            ));
        }
    }

    // Remove duplicate points that collapsed
    let mut deduped = Vec::new();
    for p in result {
        if deduped.is_empty() || deduped.last().map_or(true, |last: &Point| {
            (last.x - p.x).abs() > 0.001 || (last.y - p.y).abs() > 0.001
        }) {
            deduped.push(p);
        }
    }

    deduped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_concentric_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_concentric_fill(&poly, 10.0, true);
        assert!(!lines.is_empty());
    }
}
