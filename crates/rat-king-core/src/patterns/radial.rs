//! Radial fill pattern - lines radiating from center.

use std::f64::consts::PI;
use crate::geometry::{Line, Point, Polygon};
use crate::clip::{point_in_polygon, line_polygon_intersections};

/// Generate radial fill for a polygon - lines radiating from center.
///
/// `spacing` is angular spacing in degrees between rays.
pub fn generate_radial_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
) -> Vec<Line> {
    let outer = &polygon.outer;
    if outer.len() < 3 {
        return Vec::new();
    }

    let Some((min_x, min_y, max_x, max_y)) = polygon.bounding_box() else {
        return Vec::new();
    };

    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;

    // Max radius to ensure rays extend beyond polygon
    let diagonal = ((max_x - min_x).powi(2) + (max_y - min_y).powi(2)).sqrt();
    let max_radius = diagonal;

    let start_angle = angle_degrees * PI / 180.0;
    let angle_step = spacing * PI / 180.0;
    let num_rays = if spacing > 0.0 { (360.0 / spacing) as i32 } else { 36 };

    let mut lines = Vec::new();

    for i in 0..num_rays {
        let ray_angle = start_angle + i as f64 * angle_step;

        // Ray from center outward
        let end_x = center_x + max_radius * ray_angle.cos();
        let end_y = center_y + max_radius * ray_angle.sin();

        // Find intersections with polygon boundary
        let intersections = line_polygon_intersections(
            center_x, center_y, end_x, end_y, outer
        );

        if intersections.is_empty() {
            continue;
        }

        let center_inside = point_in_polygon(center_x, center_y, outer);

        if center_inside {
            // Ray goes from center to nearest intersection
            let (ix, iy, _) = intersections
                .iter()
                .min_by(|a, b| {
                    let da = (a.0 - center_x).powi(2) + (a.1 - center_y).powi(2);
                    let db = (b.0 - center_x).powi(2) + (b.1 - center_y).powi(2);
                    da.partial_cmp(&db).unwrap()
                })
                .copied()
                .unwrap_or((end_x, end_y, 0.0));

            // Check if segment passes through holes
            let mid_x = (center_x + ix) / 2.0;
            let mid_y = (center_y + iy) / 2.0;
            let in_hole = polygon.holes.iter().any(|hole| {
                point_in_polygon(mid_x, mid_y, hole)
            });

            if !in_hole {
                lines.push(Line::new(center_x, center_y, ix, iy));
            }
        } else if intersections.len() >= 2 {
            // Center is outside - use intersection pairs
            let mut sorted: Vec<_> = intersections.clone();
            sorted.sort_by(|a, b| {
                let da = (a.0 - center_x).powi(2) + (a.1 - center_y).powi(2);
                let db = (b.0 - center_x).powi(2) + (b.1 - center_y).powi(2);
                da.partial_cmp(&db).unwrap()
            });

            // Take pairs of intersections
            for j in (0..sorted.len() - 1).step_by(2) {
                let (x1, y1, _) = sorted[j];
                let (x2, y2, _) = sorted[j + 1];

                let mid_x = (x1 + x2) / 2.0;
                let mid_y = (y1 + y2) / 2.0;

                if point_in_polygon(mid_x, mid_y, outer) {
                    let in_hole = polygon.holes.iter().any(|hole| {
                        point_in_polygon(mid_x, mid_y, hole)
                    });
                    if !in_hole {
                        lines.push(Line::new(x1, y1, x2, y2));
                    }
                }
            }
        }
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_radial_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_radial_fill(&poly, 15.0, 0.0);
        assert!(!lines.is_empty());
        // Should have ~24 rays (360/15)
        assert!(lines.len() >= 20);
    }
}
