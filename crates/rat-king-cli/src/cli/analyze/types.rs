//! Data structures for SVG analysis results.
//!
//! These types are designed to help AI agents inspect large SVG files
//! without token overload by providing structured summaries.

use serde::Serialize;

/// Complete analysis result containing summary and optional query/tree results.
#[derive(Debug, Serialize)]
pub struct AnalyzeResult {
    pub summary: SvgSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_result: Option<QueryResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tree: Option<TreeNode>,
}

/// Summary statistics from streaming pass - always computed.
#[derive(Debug, Serialize)]
pub struct SvgSummary {
    /// File size in bytes
    pub file_size_bytes: u64,
    /// Human-readable file size (e.g., "15.2 MB")
    pub file_size_human: String,

    /// SVG viewBox if present
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_box: Option<ViewBox>,
    /// SVG width attribute
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,
    /// SVG height attribute
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<String>,

    /// Element counts by type
    pub element_counts: ElementCounts,

    /// Top-level groups (layers)
    pub top_level_groups: Vec<GroupInfo>,
    /// Number of elements with transform attributes
    pub transform_count: usize,

    /// Unique fill colors found
    pub fill_colors: Vec<ColorInfo>,
    /// Unique stroke colors found
    pub stroke_colors: Vec<ColorInfo>,

    /// Computed bounding box (from full parse if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounding_box: Option<BoundingBox>,

    /// Warnings about potential issues
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

/// SVG viewBox parsed values.
#[derive(Debug, Serialize, Clone)]
pub struct ViewBox {
    pub min_x: f64,
    pub min_y: f64,
    pub width: f64,
    pub height: f64,
}

/// Element counts by type.
#[derive(Debug, Serialize, Default, Clone)]
pub struct ElementCounts {
    pub paths: usize,
    pub groups: usize,
    pub circles: usize,
    pub rects: usize,
    pub ellipses: usize,
    pub lines: usize,
    pub polylines: usize,
    pub polygons: usize,
    pub text: usize,
    pub images: usize,
    pub use_elements: usize,
    pub defs: usize,
    pub clip_paths: usize,
    pub masks: usize,
    pub gradients: usize,
    pub patterns: usize,
    pub total: usize,
}

/// Information about a top-level group (layer).
#[derive(Debug, Serialize, Clone)]
pub struct GroupInfo {
    /// Group ID if present
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Number of direct children
    pub child_count: usize,
    /// Whether this group has a transform
    pub has_transform: bool,
    /// Element counts within this group
    pub element_counts: GroupElementCounts,
    /// Colors used in this group (fill and stroke combined), sorted by count
    pub colors: Vec<ColorInfo>,
}

/// Element counts for a group (subset of full ElementCounts).
#[derive(Debug, Serialize, Clone, Default)]
pub struct GroupElementCounts {
    pub paths: usize,
    pub groups: usize,
    pub rects: usize,
    pub circles: usize,
    pub ellipses: usize,
    pub lines: usize,
    pub polylines: usize,
    pub polygons: usize,
    pub text: usize,
    pub images: usize,
    pub use_elements: usize,
}

/// Color usage information.
#[derive(Debug, Serialize, Clone)]
pub struct ColorInfo {
    /// Color value (e.g., "#FF0000", "red", "linear-gradient")
    pub color: String,
    /// Number of elements using this color
    pub count: usize,
}

/// Bounding box coordinates.
#[derive(Debug, Serialize, Clone)]
pub struct BoundingBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub width: f64,
    pub height: f64,
}

/// Query result variants.
#[derive(Debug, Serialize)]
#[serde(tag = "query_type")]
pub enum QueryResult {
    Region(RegionResult),
    Color(ColorResult),
    Layer(LayerResult),
    Sample(SampleResult),
    Element(ElementResult),
}

/// Result of a region query.
#[derive(Debug, Serialize)]
pub struct RegionResult {
    /// The queried bounding box
    pub query_bounds: BoundingBox,
    /// Number of elements found
    pub element_count: usize,
    /// Elements in the region (limited)
    pub elements: Vec<ElementSummary>,
}

/// Result of a color query.
#[derive(Debug, Serialize)]
pub struct ColorResult {
    /// The queried color
    pub query_color: String,
    /// Number of elements with this color
    pub match_count: usize,
    /// Matching elements (limited)
    pub elements: Vec<ElementSummary>,
}

/// Result of a layer query.
#[derive(Debug, Serialize)]
pub struct LayerResult {
    /// The queried layer ID
    pub layer_id: String,
    /// Total elements in this layer
    pub element_count: usize,
    /// Number of paths
    pub path_count: usize,
    /// Number of nested groups
    pub nested_groups: usize,
    /// Bounding box of the layer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounding_box: Option<BoundingBox>,
    /// Colors used in this layer
    pub colors_used: Vec<String>,
}

/// Result of a sample query.
#[derive(Debug, Serialize)]
pub struct SampleResult {
    /// Total paths in the SVG
    pub total_paths: usize,
    /// Number of samples returned
    pub sampled_count: usize,
    /// Sampled paths
    pub samples: Vec<PathSample>,
}

/// A sampled path with metadata.
#[derive(Debug, Serialize)]
pub struct PathSample {
    /// Index of this path in document order
    pub index: usize,
    /// Element ID if present
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Fill color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_color: Option<String>,
    /// Stroke color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_color: Option<String>,
    /// Number of points in the path
    pub point_count: usize,
    /// Bounding box
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounding_box: Option<BoundingBox>,
}

/// Result of an element query.
#[derive(Debug, Serialize)]
pub struct ElementResult {
    /// Element ID
    pub id: String,
    /// Element type (path, g, rect, etc.)
    pub element_type: String,
    /// Fill color/paint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,
    /// Stroke color/paint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<String>,
    /// Stroke width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<f64>,
    /// Opacity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,
    /// Transform matrix as string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<String>,
    /// Bounding box
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounding_box: Option<BoundingBox>,
    /// Child elements (for groups)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<ElementSummary>>,
}

/// Brief summary of an element (used in query results).
#[derive(Debug, Serialize, Clone)]
pub struct ElementSummary {
    /// Element type
    pub element_type: String,
    /// Element ID if present
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Fill color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,
}

/// Tree node for hierarchical view.
#[derive(Debug, Serialize, Clone)]
pub struct TreeNode {
    /// Element type or "svg"
    pub name: String,
    /// Element ID if present
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Total descendant count
    pub element_count: usize,
    /// Child nodes
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<TreeNode>,
}

// Helper implementations

impl SvgSummary {
    pub fn new(file_size_bytes: u64) -> Self {
        Self {
            file_size_bytes,
            file_size_human: format_file_size(file_size_bytes),
            view_box: None,
            width: None,
            height: None,
            element_counts: ElementCounts::default(),
            top_level_groups: Vec::new(),
            transform_count: 0,
            fill_colors: Vec::new(),
            stroke_colors: Vec::new(),
            bounding_box: None,
            warnings: Vec::new(),
        }
    }
}

impl BoundingBox {
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
            width: max_x - min_x,
            height: max_y - min_y,
        }
    }

    /// Check if a point is inside this bounding box.
    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y
    }

    /// Check if another bounding box intersects this one.
    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.min_x <= other.max_x
            && self.max_x >= other.min_x
            && self.min_y <= other.max_y
            && self.max_y >= other.min_y
    }
}

/// Format bytes as human-readable size.
fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}
