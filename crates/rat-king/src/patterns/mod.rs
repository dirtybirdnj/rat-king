//! Pattern generators for polygon fills.
//!
//! Each pattern generates lines that are clipped to the polygon boundary.

pub mod util;

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
mod pentagon15;
mod pentagon14;
mod grid;
mod brick;
mod truchet;
mod stipple;
mod peano;
mod sierpinski;
mod diagonal;
mod herringbone;
mod stripe;
mod tessellation;
mod harmonograph;

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
pub use pentagon15::generate_pentagon15_fill;
pub use pentagon14::generate_pentagon14_fill;
pub use grid::generate_grid_fill;
pub use brick::generate_brick_fill;
pub use truchet::generate_truchet_fill;
pub use stipple::generate_stipple_fill;
pub use peano::generate_peano_fill;
pub use sierpinski::generate_sierpinski_fill;
pub use diagonal::generate_diagonal_fill;
pub use herringbone::generate_herringbone_fill;
pub use stripe::{generate_stripe_fill, generate_stripe_fill_configured, StripeConfig};
pub use tessellation::generate_tessellation_fill;
pub use harmonograph::generate_harmonograph_fill;

// Re-export from hatch module (already implemented)
pub use crate::hatch::{generate_lines_fill, generate_crosshatch_fill};

/// Metadata describing a pattern for UI display.
#[derive(Debug, Clone, Copy)]
pub struct PatternMetadata {
    /// Label for the spacing parameter
    pub spacing_label: &'static str,
    /// Label for the angle parameter
    pub angle_label: &'static str,
    /// Brief description of the pattern
    pub description: &'static str,
}

impl PatternMetadata {
    /// Create new pattern metadata.
    pub const fn new(spacing_label: &'static str, angle_label: &'static str, description: &'static str) -> Self {
        Self { spacing_label, angle_label, description }
    }
}

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
    Scribble,
    Gyroid,
    Pentagon15,
    Pentagon14,
    Grid,
    Brick,
    Truchet,
    Stipple,
    Peano,
    Sierpinski,
    Diagonal,
    Herringbone,
    Stripe,
    Tessellation,
    Harmonograph,
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
            Pattern::Pentagon15,
            Pattern::Pentagon14,
            Pattern::Grid,
            Pattern::Brick,
            Pattern::Truchet,
            Pattern::Stipple,
            Pattern::Peano,
            Pattern::Sierpinski,
            Pattern::Diagonal,
            Pattern::Herringbone,
            Pattern::Stripe,
            Pattern::Tessellation,
            Pattern::Harmonograph,
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
            Pattern::Pentagon15 => "pentagon15",
            Pattern::Pentagon14 => "pentagon14",
            Pattern::Grid => "grid",
            Pattern::Brick => "brick",
            Pattern::Truchet => "truchet",
            Pattern::Stipple => "stipple",
            Pattern::Peano => "peano",
            Pattern::Sierpinski => "sierpinski",
            Pattern::Diagonal => "diagonal",
            Pattern::Herringbone => "herringbone",
            Pattern::Stripe => "stripe",
            Pattern::Tessellation => "tessellation",
            Pattern::Harmonograph => "harmonograph",
        }
    }

    /// Check if pattern is a stub (not fully implemented).
    /// All patterns are now fully implemented.
    pub fn is_stub(&self) -> bool {
        false
    }

    /// Get UI metadata for this pattern.
    ///
    /// Returns (spacing_label, angle_label, description) for UI display.
    pub fn metadata(&self) -> PatternMetadata {
        match self {
            Pattern::Lines | Pattern::Crosshatch | Pattern::Diagonal =>
                PatternMetadata::new("Line Spacing", "Angle", "Parallel lines at angle"),
            Pattern::Zigzag =>
                PatternMetadata::new("Amplitude", "Angle", "Zigzag waves with amplitude"),
            Pattern::Wiggle =>
                PatternMetadata::new("Wavelength", "Angle", "Smooth sine waves"),
            Pattern::Spiral =>
                PatternMetadata::new("Turn Spacing", "Start Angle", "Archimedean spiral"),
            Pattern::Fermat =>
                PatternMetadata::new("Turn Spacing", "Rotation", "Fermat (parabolic) spiral"),
            Pattern::Concentric =>
                PatternMetadata::new("Ring Spacing", "N/A", "Concentric offset rings"),
            Pattern::Radial =>
                PatternMetadata::new("Ray Count", "Offset", "Radial rays from center"),
            Pattern::Honeycomb =>
                PatternMetadata::new("Cell Size", "Angle", "Hexagonal honeycomb grid"),
            Pattern::Crossspiral =>
                PatternMetadata::new("Arm Spacing", "Arms", "Crossed spiral arms"),
            Pattern::Hilbert =>
                PatternMetadata::new("Detail", "Rotation", "Hilbert space-filling curve"),
            Pattern::Guilloche =>
                PatternMetadata::new("Complexity", "Phase", "Spirograph-like curves"),
            Pattern::Lissajous =>
                PatternMetadata::new("Frequency", "Phase", "Lissajous figure curves"),
            Pattern::Rose =>
                PatternMetadata::new("Petals", "Rotation", "Rose/rhodonea curves"),
            Pattern::Phyllotaxis =>
                PatternMetadata::new("Dot Spacing", "Golden Angle", "Sunflower seed pattern"),
            Pattern::Scribble =>
                PatternMetadata::new("Density", "Chaos", "Random scribble fill"),
            Pattern::Gyroid =>
                PatternMetadata::new("Cell Size", "Rotation", "3D gyroid projection"),
            Pattern::Pentagon15 =>
                PatternMetadata::new("Tile Size", "Rotation", "Penrose P3 tiling"),
            Pattern::Pentagon14 =>
                PatternMetadata::new("Tile Size", "Rotation", "Cairo pentagonal tiling"),
            Pattern::Grid =>
                PatternMetadata::new("Cell Size", "Angle", "Square grid pattern"),
            Pattern::Brick =>
                PatternMetadata::new("Brick Width", "Angle", "Running bond brick"),
            Pattern::Truchet =>
                PatternMetadata::new("Tile Size", "Rotation", "Random Truchet tiles"),
            Pattern::Stipple =>
                PatternMetadata::new("Dot Spacing", "Randomness", "Stippled dot pattern"),
            Pattern::Peano =>
                PatternMetadata::new("Detail", "Rotation", "Peano space-filling curve"),
            Pattern::Sierpinski =>
                PatternMetadata::new("Detail", "Rotation", "Sierpinski arrowhead"),
            Pattern::Herringbone =>
                PatternMetadata::new("Segment Size", "Angle", "Herringbone chevrons"),
            Pattern::Stripe =>
                PatternMetadata::new("Band Width", "Angle", "Grouped stripe bands"),
            Pattern::Tessellation =>
                PatternMetadata::new("N/A", "N/A", "Triangulate polygon"),
            Pattern::Harmonograph =>
                PatternMetadata::new("Curve Count", "Phase", "Decaying pendulum curves"),
        }
    }

    /// Get the spacing multiplier for this pattern.
    ///
    /// Some patterns need the spacing parameter scaled for better default behavior.
    pub fn spacing_multiplier(&self) -> f64 {
        match self {
            Pattern::Zigzag | Pattern::Wiggle | Pattern::Spiral | Pattern::Fermat
            | Pattern::Honeycomb | Pattern::Crossspiral | Pattern::Grid
            | Pattern::Brick | Pattern::Truchet | Pattern::Herringbone | Pattern::Stripe => 2.0,
            _ => 1.0,
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
            "crossspiral" => Some(Pattern::Crossspiral),
            "hilbert" => Some(Pattern::Hilbert),
            "guilloche" | "spirograph" => Some(Pattern::Guilloche),
            "lissajous" => Some(Pattern::Lissajous),
            "rose" | "rhodonea" => Some(Pattern::Rose),
            "phyllotaxis" | "sunflower" => Some(Pattern::Phyllotaxis),
            "scribble" => Some(Pattern::Scribble),
            "gyroid" => Some(Pattern::Gyroid),
            "pentagon15" | "pent15" => Some(Pattern::Pentagon15),
            "pentagon14" | "pent14" => Some(Pattern::Pentagon14),
            "grid" => Some(Pattern::Grid),
            "brick" | "running-bond" => Some(Pattern::Brick),
            "truchet" => Some(Pattern::Truchet),
            "stipple" | "dots" => Some(Pattern::Stipple),
            "peano" => Some(Pattern::Peano),
            "sierpinski" | "arrowhead" => Some(Pattern::Sierpinski),
            "diagonal" => Some(Pattern::Diagonal),
            "herringbone" | "chevron" => Some(Pattern::Herringbone),
            "stripe" | "stripes" | "bands" => Some(Pattern::Stripe),
            "tessellation" | "triangulate" | "triangles" => Some(Pattern::Tessellation),
            "harmonograph" | "pendulum" => Some(Pattern::Harmonograph),
            _ => None,
        }
    }
}
