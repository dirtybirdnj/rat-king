//! Line chaining - connect line segments into continuous polylines.
//!
//! This module takes disconnected line segments and chains them together
//! where endpoints match (within tolerance), reducing the number of
//! path elements and pen lifts for plotters.

use crate::geometry::{Line, Point};
use std::collections::HashMap;

/// A chain of connected points forming a polyline.
pub type Chain = Vec<Point>;

/// Configuration for line chaining.
#[derive(Debug, Clone)]
pub struct ChainConfig {
    /// Maximum distance between endpoints to consider them connected.
    /// Default: 0.1 (sub-pixel at typical SVG scales)
    pub tolerance: f64,
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self { tolerance: 0.1 }
    }
}

impl ChainConfig {
    pub fn with_tolerance(tolerance: f64) -> Self {
        Self { tolerance }
    }
}

/// Chain connected lines into polylines.
///
/// Takes a list of disconnected line segments and finds sequences where
/// the end of one line matches the start of another (within tolerance).
/// Returns a list of chains (polylines) that minimize the number of
/// separate paths.
///
/// # Algorithm
///
/// 1. Build a spatial hash of all endpoints
/// 2. For each unvisited line, start a new chain
/// 3. Extend chain forward by finding lines whose start matches our end
/// 4. Extend chain backward by finding lines whose end matches our start
/// 5. Repeat until no more connections found
///
/// # Performance
///
/// O(n) average case with spatial hashing, O(n²) worst case if all
/// endpoints hash to same bucket.
pub fn chain_lines(lines: &[Line], config: &ChainConfig) -> Vec<Chain> {
    if lines.is_empty() {
        return Vec::new();
    }

    let tolerance = config.tolerance;
    let tolerance_sq = tolerance * tolerance;

    // Track which lines have been used
    let mut used = vec![false; lines.len()];

    // Build spatial index: grid cell -> list of (line_index, is_start_point)
    // Using grid cells of size `tolerance` for fast neighbor lookup
    let grid_size = tolerance.max(0.001); // Avoid division by zero
    let mut grid: HashMap<(i64, i64), Vec<(usize, bool)>> = HashMap::new();

    for (i, line) in lines.iter().enumerate() {
        let start_cell = point_to_cell(line.x1, line.y1, grid_size);
        let end_cell = point_to_cell(line.x2, line.y2, grid_size);

        grid.entry(start_cell).or_default().push((i, true)); // true = start point
        grid.entry(end_cell).or_default().push((i, false)); // false = end point
    }

    let mut chains = Vec::new();

    for start_idx in 0..lines.len() {
        if used[start_idx] {
            continue;
        }

        // Start a new chain with this line
        used[start_idx] = true;
        let line = &lines[start_idx];
        let mut chain = vec![
            Point::new(line.x1, line.y1),
            Point::new(line.x2, line.y2),
        ];

        // Extend forward: find lines whose START matches our END
        loop {
            let end = chain.last().unwrap();
            if let Some(next_idx) = find_connecting_line(
                end.x, end.y,
                &grid, &lines, &used,
                grid_size, tolerance_sq,
                true, // look for start points
            ) {
                used[next_idx] = true;
                let next_line = &lines[next_idx];
                // Add the end point of the connected line
                chain.push(Point::new(next_line.x2, next_line.y2));
            } else {
                break;
            }
        }

        // Extend backward: find lines whose END matches our START
        loop {
            let start = chain.first().unwrap();
            if let Some(prev_idx) = find_connecting_line(
                start.x, start.y,
                &grid, &lines, &used,
                grid_size, tolerance_sq,
                false, // look for end points
            ) {
                used[prev_idx] = true;
                let prev_line = &lines[prev_idx];
                // Insert the start point of the connected line at the beginning
                chain.insert(0, Point::new(prev_line.x1, prev_line.y1));
            } else {
                break;
            }
        }

        chains.push(chain);
    }

    chains
}

/// Convert a point to a grid cell coordinate.
#[inline]
fn point_to_cell(x: f64, y: f64, grid_size: f64) -> (i64, i64) {
    ((x / grid_size).floor() as i64, (y / grid_size).floor() as i64)
}

/// Find an unused line that connects to the given point.
///
/// If `match_start` is true, look for lines whose START point matches.
/// If false, look for lines whose END point matches.
fn find_connecting_line(
    x: f64, y: f64,
    grid: &HashMap<(i64, i64), Vec<(usize, bool)>>,
    lines: &[Line],
    used: &[bool],
    grid_size: f64,
    tolerance_sq: f64,
    match_start: bool,
) -> Option<usize> {
    let cell = point_to_cell(x, y, grid_size);

    // Check this cell and all 8 neighbors (endpoints might be in adjacent cells)
    for dx in -1..=1 {
        for dy in -1..=1 {
            let check_cell = (cell.0 + dx, cell.1 + dy);
            if let Some(candidates) = grid.get(&check_cell) {
                for &(line_idx, is_start) in candidates {
                    // Skip if already used or wrong endpoint type
                    if used[line_idx] || is_start != match_start {
                        continue;
                    }

                    let line = &lines[line_idx];
                    let (px, py) = if match_start {
                        (line.x1, line.y1)
                    } else {
                        (line.x2, line.y2)
                    };

                    let dist_sq = (px - x).powi(2) + (py - y).powi(2);
                    if dist_sq <= tolerance_sq {
                        return Some(line_idx);
                    }
                }
            }
        }
    }

    None
}

/// Convert chains back to lines (for compatibility or debugging).
pub fn chains_to_lines(chains: &[Chain]) -> Vec<Line> {
    let mut lines = Vec::new();
    for chain in chains {
        for window in chain.windows(2) {
            lines.push(Line::new(
                window[0].x, window[0].y,
                window[1].x, window[1].y,
            ));
        }
    }
    lines
}

/// Calculate statistics about chaining results.
#[derive(Debug, Clone)]
pub struct ChainStats {
    /// Number of input line segments
    pub input_lines: usize,
    /// Number of output chains
    pub output_chains: usize,
    /// Average chain length (points per chain)
    pub avg_chain_length: f64,
    /// Longest chain (points)
    pub max_chain_length: usize,
    /// Reduction ratio (1.0 - chains/lines)
    pub reduction_ratio: f64,
}

impl ChainStats {
    pub fn from_chains(input_count: usize, chains: &[Chain]) -> Self {
        let output_chains = chains.len();
        let total_points: usize = chains.iter().map(|c| c.len()).sum();
        let max_chain_length = chains.iter().map(|c| c.len()).max().unwrap_or(0);

        Self {
            input_lines: input_count,
            output_chains,
            avg_chain_length: if output_chains > 0 {
                total_points as f64 / output_chains as f64
            } else {
                0.0
            },
            max_chain_length,
            reduction_ratio: if input_count > 0 {
                1.0 - (output_chains as f64 / input_count as f64)
            } else {
                0.0
            },
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_empty() {
        let chains = chain_lines(&[], &ChainConfig::default());
        assert!(chains.is_empty());
    }

    #[test]
    fn chain_single_line() {
        let lines = vec![Line::new(0.0, 0.0, 10.0, 10.0)];
        let chains = chain_lines(&lines, &ChainConfig::default());

        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].len(), 2);
    }

    #[test]
    fn chain_two_connected_lines() {
        let lines = vec![
            Line::new(0.0, 0.0, 10.0, 10.0),
            Line::new(10.0, 10.0, 20.0, 10.0), // starts where first ends
        ];
        let chains = chain_lines(&lines, &ChainConfig::default());

        assert_eq!(chains.len(), 1, "Should chain into 1 polyline");
        assert_eq!(chains[0].len(), 3, "Chain should have 3 points");
    }

    #[test]
    fn chain_two_disconnected_lines() {
        let lines = vec![
            Line::new(0.0, 0.0, 10.0, 10.0),
            Line::new(100.0, 100.0, 110.0, 110.0), // far away
        ];
        let chains = chain_lines(&lines, &ChainConfig::default());

        assert_eq!(chains.len(), 2, "Should remain as 2 separate chains");
    }

    #[test]
    fn chain_respects_tolerance() {
        let lines = vec![
            Line::new(0.0, 0.0, 10.0, 10.0),
            Line::new(10.05, 10.05, 20.0, 10.0), // slightly off
        ];

        // With default tolerance (0.1), should chain
        let chains = chain_lines(&lines, &ChainConfig::with_tolerance(0.1));
        assert_eq!(chains.len(), 1, "Should chain with tolerance 0.1");

        // With tight tolerance, should not chain
        let chains = chain_lines(&lines, &ChainConfig::with_tolerance(0.01));
        assert_eq!(chains.len(), 2, "Should not chain with tolerance 0.01");
    }

    #[test]
    fn chain_multiple_segments() {
        // A zigzag pattern: 5 connected lines
        let lines = vec![
            Line::new(0.0, 0.0, 10.0, 10.0),
            Line::new(10.0, 10.0, 20.0, 0.0),
            Line::new(20.0, 0.0, 30.0, 10.0),
            Line::new(30.0, 10.0, 40.0, 0.0),
            Line::new(40.0, 0.0, 50.0, 10.0),
        ];
        let chains = chain_lines(&lines, &ChainConfig::default());

        assert_eq!(chains.len(), 1, "All 5 lines should form 1 chain");
        assert_eq!(chains[0].len(), 6, "Chain should have 6 points");
    }

    #[test]
    fn chain_out_of_order() {
        // Lines given out of order, but still connected
        let lines = vec![
            Line::new(20.0, 0.0, 30.0, 10.0),  // middle
            Line::new(0.0, 0.0, 10.0, 10.0),   // start
            Line::new(10.0, 10.0, 20.0, 0.0),  // connects start to middle
        ];
        let chains = chain_lines(&lines, &ChainConfig::default());

        assert_eq!(chains.len(), 1, "Should chain despite order");
        assert_eq!(chains[0].len(), 4, "Chain should have 4 points");
    }

    #[test]
    fn chain_stats() {
        let lines = vec![
            Line::new(0.0, 0.0, 10.0, 10.0),
            Line::new(10.0, 10.0, 20.0, 0.0),
            Line::new(100.0, 100.0, 110.0, 110.0), // separate
        ];
        let chains = chain_lines(&lines, &ChainConfig::default());
        let stats = ChainStats::from_chains(lines.len(), &chains);

        assert_eq!(stats.input_lines, 3);
        assert_eq!(stats.output_chains, 2);
        assert!(stats.reduction_ratio > 0.3); // At least 33% reduction
    }
}
