//! CLI command implementations.
//!
//! This module contains the implementations for the various CLI subcommands:
//! - `fill` - Generate pattern fills for SVG polygons
//! - `benchmark` - Benchmark pattern generation performance
//! - `harness` - Run test harness with visual analysis
//! - `patterns` - List available patterns

pub mod common;
pub mod fill;
pub mod benchmark;
pub mod harness;

pub use common::generate_pattern;
pub use fill::cmd_fill;
pub use benchmark::cmd_benchmark;
pub use harness::{
    AnalysisResult, HarnessResult, VisualHarnessReport,
    analyze_pattern_vs_solid, generate_diff_image,
};
