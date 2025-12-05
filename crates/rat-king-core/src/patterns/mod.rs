//! Pattern generators for polygon fills.
//!
//! Each pattern generates lines that are clipped to the polygon boundary.

mod zigzag;
mod wiggle;
mod spiral;
mod concentric;
mod radial;
mod honeycomb;
mod scribble;
mod crossspiral;
mod hilbert;
mod gyroid;
mod guilloche;
mod lissajous;
mod rose;
mod phyllotaxis;

pub use zigzag::generate_zigzag_fill;
pub use wiggle::generate_wiggle_fill;
pub use spiral::{generate_spiral_fill, generate_fermat_fill};
pub use concentric::generate_concentric_fill;
pub use radial::generate_radial_fill;
pub use honeycomb::generate_honeycomb_fill;
pub use scribble::generate_scribble_fill;
pub use crossspiral::generate_crossspiral_fill;
pub use hilbert::generate_hilbert_fill;
pub use gyroid::generate_gyroid_fill;
pub use guilloche::generate_guilloche_fill;
pub use lissajous::generate_lissajous_fill;
pub use rose::generate_rose_fill;
pub use phyllotaxis::generate_phyllotaxis_fill;

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
    Crossspiral,
    Hilbert,
    Guilloche,
    Lissajous,
    Rose,
    Phyllotaxis,
    // Stubs (not fully implemented yet)
    Scribble,
    Gyroid,
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
            Pattern::Crossspiral,
            Pattern::Hilbert,
            Pattern::Guilloche,
            Pattern::Lissajous,
            Pattern::Rose,
            Pattern::Phyllotaxis,
            Pattern::Scribble,
            Pattern::Gyroid,
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
            Pattern::Crossspiral => "crossspiral",
            Pattern::Hilbert => "hilbert",
            Pattern::Guilloche => "guilloche",
            Pattern::Lissajous => "lissajous",
            Pattern::Rose => "rose",
            Pattern::Phyllotaxis => "phyllotaxis",
            Pattern::Scribble => "scribble",
            Pattern::Gyroid => "gyroid",
        }
    }

    /// Check if pattern is a stub (not fully implemented).
    pub fn is_stub(&self) -> bool {
        matches!(self, Pattern::Scribble | Pattern::Gyroid)
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
            "crossspiral" => Some(Pattern::Crossspiral),
            "hilbert" => Some(Pattern::Hilbert),
            "guilloche" | "spirograph" => Some(Pattern::Guilloche),
            "lissajous" => Some(Pattern::Lissajous),
            "rose" | "rhodonea" => Some(Pattern::Rose),
            "phyllotaxis" | "sunflower" => Some(Pattern::Phyllotaxis),
            "scribble" => Some(Pattern::Scribble),
            "gyroid" => Some(Pattern::Gyroid),
            _ => None,
        }
    }
}
