//! Pattern generators for polygon fills.
//!
//! Each pattern generates lines that are clipped to the polygon boundary.

mod zigzag;
mod wiggle;
mod spiral;
mod concentric;
mod radial;
mod honeycomb;

pub use zigzag::generate_zigzag_fill;
pub use wiggle::generate_wiggle_fill;
pub use spiral::{generate_spiral_fill, generate_fermat_fill};
pub use concentric::generate_concentric_fill;
pub use radial::generate_radial_fill;
pub use honeycomb::generate_honeycomb_fill;

// Re-export from hatch module (already implemented)
pub use crate::hatch::{generate_lines_fill, generate_crosshatch_fill};

/// Available pattern types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pattern {
    Lines,
    Crosshatch,
    Zigzag,
    Wiggle,
    Spiral,
    Fermat,
    Concentric,
    Radial,
    Honeycomb,
}

impl Pattern {
    /// Get all available patterns.
    pub fn all() -> &'static [Pattern] {
        &[
            Pattern::Lines,
            Pattern::Crosshatch,
            Pattern::Zigzag,
            Pattern::Wiggle,
            Pattern::Spiral,
            Pattern::Fermat,
            Pattern::Concentric,
            Pattern::Radial,
            Pattern::Honeycomb,
        ]
    }

    /// Get pattern name as string.
    pub fn name(&self) -> &'static str {
        match self {
            Pattern::Lines => "lines",
            Pattern::Crosshatch => "crosshatch",
            Pattern::Zigzag => "zigzag",
            Pattern::Wiggle => "wiggle",
            Pattern::Spiral => "spiral",
            Pattern::Fermat => "fermat",
            Pattern::Concentric => "concentric",
            Pattern::Radial => "radial",
            Pattern::Honeycomb => "honeycomb",
        }
    }

    /// Parse pattern from string.
    pub fn from_name(name: &str) -> Option<Pattern> {
        match name.to_lowercase().as_str() {
            "lines" => Some(Pattern::Lines),
            "crosshatch" => Some(Pattern::Crosshatch),
            "zigzag" => Some(Pattern::Zigzag),
            "wiggle" | "wave" => Some(Pattern::Wiggle),
            "spiral" => Some(Pattern::Spiral),
            "fermat" => Some(Pattern::Fermat),
            "concentric" => Some(Pattern::Concentric),
            "radial" => Some(Pattern::Radial),
            "honeycomb" => Some(Pattern::Honeycomb),
            _ => None,
        }
    }
}
