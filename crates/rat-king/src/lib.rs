//! # rat-king-core
//!
//! Core geometry and pattern generation library.
//!
//! ## Rust Lesson #7: Modules
//!
//! Rust modules are like ES6 modules but more explicit:
//! - `mod foo;` = load from `foo.rs` or `foo/mod.rs`
//! - `pub mod foo;` = also export it publicly
//! - `pub use foo::Bar;` = re-export Bar at this level
//!
//! Unlike Node.js, you must explicitly declare every module.

pub mod clip;
pub mod geometry;
pub mod hatch;
pub mod order;
pub mod patterns;
pub mod svg;

// Re-export common types at crate root for convenience.
pub use clip::{clip_line_to_polygon, clip_lines_to_polygon, point_in_polygon};
pub use geometry::{Line, Point, Polygon};
pub use hatch::{generate_crosshatch_fill, generate_hatch_lines, generate_lines_fill};
pub use order::{order_polygons, order_nearest_neighbor, calculate_travel_distance, OrderingStrategy};
pub use patterns::Pattern;
pub use svg::{extract_polygons_from_svg, SvgError};
