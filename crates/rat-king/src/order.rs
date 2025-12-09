//! Polygon ordering optimization for minimizing plotter travel.
//!
//! When filling multiple polygons, the order in which they are processed
//! affects the total pen travel distance. This module provides algorithms
//! to optimize this ordering.
//!
//! ## Algorithms
//!
//! - **Nearest Neighbor**: Simple greedy approach - O(n²) but usually good enough
//! - **2-opt**: Local optimization improvement (future)

use crate::geometry::{Point, Polygon};
use std::collections::HashSet;

/// Calculate the centroid of a polygon.
pub fn polygon_centroid(polygon: &Polygon) -> Point {
    if polygon.outer.is_empty() {
        return Point::new(0.0, 0.0);
    }

    let sum_x: f64 = polygon.outer.iter().map(|p| p.x).sum();
    let sum_y: f64 = polygon.outer.iter().map(|p| p.y).sum();
    let n = polygon.outer.len() as f64;

    Point::new(sum_x / n, sum_y / n)
}

/// Order polygons using nearest-neighbor heuristic.
///
/// Starting from the first polygon (or nearest to origin), greedily select
/// the nearest unvisited polygon at each step. This typically reduces
/// travel distance by 30-50% compared to document order.
///
/// Returns indices into the original polygon slice in optimized order.
pub fn order_nearest_neighbor(polygons: &[Polygon]) -> Vec<usize> {
    let n = polygons.len();
    if n <= 1 {
        return (0..n).collect();
    }

    // Precompute centroids
    let centroids: Vec<Point> = polygons.iter().map(polygon_centroid).collect();

    let mut order = Vec::with_capacity(n);
    let mut remaining: HashSet<usize> = (0..n).collect();

    // Start from polygon nearest to origin
    let first = remaining.iter()
        .min_by(|&&a, &&b| {
            let dist_a = centroids[a].x.powi(2) + centroids[a].y.powi(2);
            let dist_b = centroids[b].x.powi(2) + centroids[b].y.powi(2);
            dist_a.partial_cmp(&dist_b).unwrap()
        })
        .copied()
        .unwrap();

    order.push(first);
    remaining.remove(&first);

    // Greedy nearest neighbor
    while !remaining.is_empty() {
        let current = *order.last().unwrap();
        let current_centroid = &centroids[current];

        let nearest = remaining.iter()
            .min_by(|&&a, &&b| {
                let dist_a = current_centroid.distance(centroids[a]);
                let dist_b = current_centroid.distance(centroids[b]);
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .copied()
            .unwrap();

        order.push(nearest);
        remaining.remove(&nearest);
    }

    order
}

/// Calculate total travel distance for a given polygon order.
///
/// Assumes the pen travels from one polygon centroid to the next.
/// Returns the total distance in the same units as the polygon coordinates.
pub fn calculate_travel_distance(polygons: &[Polygon], order: &[usize]) -> f64 {
    if order.len() <= 1 {
        return 0.0;
    }

    let centroids: Vec<Point> = polygons.iter().map(polygon_centroid).collect();

    let mut total = 0.0;
    for i in 1..order.len() {
        let from = &centroids[order[i - 1]];
        let to = &centroids[order[i]];
        total += from.distance(*to);
    }

    total
}

/// Ordering strategy for polygons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OrderingStrategy {
    /// Keep original document order
    #[default]
    Document,
    /// Nearest neighbor greedy optimization
    NearestNeighbor,
}

impl OrderingStrategy {
    /// Get strategy name as string.
    pub fn name(&self) -> &'static str {
        match self {
            OrderingStrategy::Document => "document",
            OrderingStrategy::NearestNeighbor => "nearest",
        }
    }

    /// Parse strategy from string.
    pub fn from_name(name: &str) -> Option<OrderingStrategy> {
        match name.to_lowercase().as_str() {
            "document" | "doc" | "original" => Some(OrderingStrategy::Document),
            "nearest" | "nn" | "nearest-neighbor" => Some(OrderingStrategy::NearestNeighbor),
            _ => None,
        }
    }

    /// All available strategies.
    pub fn all() -> &'static [OrderingStrategy] {
        &[OrderingStrategy::Document, OrderingStrategy::NearestNeighbor]
    }
}

/// Apply ordering strategy to get polygon indices.
pub fn order_polygons(polygons: &[Polygon], strategy: OrderingStrategy) -> Vec<usize> {
    match strategy {
        OrderingStrategy::Document => (0..polygons.len()).collect(),
        OrderingStrategy::NearestNeighbor => order_nearest_neighbor(polygons),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_polygon_at(x: f64, y: f64, size: f64) -> Polygon {
        Polygon::new(vec![
            Point::new(x, y),
            Point::new(x + size, y),
            Point::new(x + size, y + size),
            Point::new(x, y + size),
        ])
    }

    #[test]
    fn centroid_calculation() {
        let poly = make_polygon_at(0.0, 0.0, 10.0);
        let centroid = polygon_centroid(&poly);
        assert!((centroid.x - 5.0).abs() < 0.001);
        assert!((centroid.y - 5.0).abs() < 0.001);
    }

    #[test]
    fn nearest_neighbor_reduces_travel() {
        // Create polygons in a zigzag pattern
        let polygons = vec![
            make_polygon_at(0.0, 0.0, 10.0),    // bottom-left
            make_polygon_at(100.0, 0.0, 10.0),  // bottom-right
            make_polygon_at(10.0, 10.0, 10.0),  // near first
            make_polygon_at(90.0, 10.0, 10.0),  // near second
        ];

        let doc_order: Vec<usize> = (0..polygons.len()).collect();
        let nn_order = order_nearest_neighbor(&polygons);

        let doc_travel = calculate_travel_distance(&polygons, &doc_order);
        let nn_travel = calculate_travel_distance(&polygons, &nn_order);

        // Nearest neighbor should find a shorter path
        assert!(nn_travel <= doc_travel,
            "NN travel {} should be <= doc travel {}",
            nn_travel, doc_travel);
    }

    #[test]
    fn order_preserves_all_polygons() {
        let polygons: Vec<Polygon> = (0..10)
            .map(|i| make_polygon_at(i as f64 * 20.0, 0.0, 10.0))
            .collect();

        let order = order_nearest_neighbor(&polygons);

        assert_eq!(order.len(), polygons.len());

        // All indices should be present exactly once
        let mut sorted = order.clone();
        sorted.sort();
        assert_eq!(sorted, (0..10).collect::<Vec<_>>());
    }

    #[test]
    fn empty_and_single_polygon() {
        let empty: Vec<Polygon> = vec![];
        assert_eq!(order_nearest_neighbor(&empty), Vec::<usize>::new());

        let single = vec![make_polygon_at(0.0, 0.0, 10.0)];
        assert_eq!(order_nearest_neighbor(&single), vec![0]);
    }

    #[test]
    fn strategy_parsing() {
        assert_eq!(OrderingStrategy::from_name("nearest"), Some(OrderingStrategy::NearestNeighbor));
        assert_eq!(OrderingStrategy::from_name("document"), Some(OrderingStrategy::Document));
        assert_eq!(OrderingStrategy::from_name("invalid"), None);
    }
}
