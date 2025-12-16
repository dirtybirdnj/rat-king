//! CLI command implementations.
//!
//! This module contains the implementations for the various CLI subcommands:
//! - `fill` - Generate pattern fills for SVG polygons
//! - `benchmark` - Benchmark pattern generation performance
//! - `harness` - Run test harness with visual analysis
//! - `patterns` - List available patterns
//! - `analyze` - Analyze SVG structure for AI agents
//! - `swatches` - Generate pattern swatch sheets
//! - `banner` - Generate randomized pattern banners
//! - `showcase` - Generate pattern detail progression pages

pub mod common;
pub mod fill;
pub mod benchmark;
pub mod harness;
pub mod analyze;
pub mod swatches;
pub mod banner;
pub mod hershey;
pub mod showcase;

pub use common::generate_pattern;
pub use fill::cmd_fill;
pub use benchmark::cmd_benchmark;
pub use harness::{
    AnalysisResult, HarnessResult, VisualHarnessReport,
    analyze_pattern_vs_solid, generate_diff_image,
};
pub use analyze::cmd_analyze;
pub use swatches::cmd_swatches;
pub use banner::cmd_banner;
pub use showcase::cmd_showcase;
