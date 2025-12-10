//! Query implementations for SVG analysis.
//!
//! These functions require a full usvg parse and are used for drilling
//! down into specific elements based on various criteria.

use std::collections::HashSet;

use super::types::{
    BoundingBox, ColorResult, ElementResult, ElementSummary, LayerResult, PathSample,
    RegionResult, SampleResult,
};

/// Query elements within a bounding region.
pub fn query_region(tree: &usvg::Tree, x: f64, y: f64, w: f64, h: f64, limit: usize) -> RegionResult {
    let query_bounds = BoundingBox::new(x, y, x + w, y + h);
    let mut elements = Vec::new();

    walk_tree_for_region(tree.root(), &query_bounds, &mut elements, limit);

    RegionResult {
        query_bounds,
        element_count: elements.len(),
        elements,
    }
}

fn walk_tree_for_region(
    group: &usvg::Group,
    bounds: &BoundingBox,
    elements: &mut Vec<ElementSummary>,
    limit: usize,
) {
    if elements.len() >= limit {
        return;
    }

    for child in group.children() {
        if elements.len() >= limit {
            break;
        }

        match child {
            usvg::Node::Group(g) => {
                walk_tree_for_region(g, bounds, elements, limit);
            }
            usvg::Node::Path(path) => {
                if let Some(bbox) = path_bounding_box(path) {
                    if bounds.intersects(&bbox) {
                        elements.push(ElementSummary {
                            element_type: "path".to_string(),
                            id: if path.id().is_empty() {
                                None
                            } else {
                                Some(path.id().to_string())
                            },
                            fill: extract_fill_color(path),
                        });
                    }
                }
            }
            usvg::Node::Image(img) => {
                if let Some(bbox) = image_bounding_box(img) {
                    if bounds.intersects(&bbox) {
                        elements.push(ElementSummary {
                            element_type: "image".to_string(),
                            id: if img.id().is_empty() {
                                None
                            } else {
                                Some(img.id().to_string())
                            },
                            fill: None,
                        });
                    }
                }
            }
            usvg::Node::Text(text) => {
                // Text elements in the region
                elements.push(ElementSummary {
                    element_type: "text".to_string(),
                    id: if text.id().is_empty() {
                        None
                    } else {
                        Some(text.id().to_string())
                    },
                    fill: None,
                });
            }
        }
    }
}

/// Query elements by fill or stroke color.
pub fn query_color(tree: &usvg::Tree, color: &str, limit: usize) -> ColorResult {
    let normalized = color.to_lowercase();
    let mut elements = Vec::new();

    walk_tree_for_color(tree.root(), &normalized, &mut elements, limit);

    ColorResult {
        query_color: color.to_string(),
        match_count: elements.len(),
        elements,
    }
}

fn walk_tree_for_color(
    group: &usvg::Group,
    color: &str,
    elements: &mut Vec<ElementSummary>,
    limit: usize,
) {
    if elements.len() >= limit {
        return;
    }

    for child in group.children() {
        if elements.len() >= limit {
            break;
        }

        match child {
            usvg::Node::Group(g) => {
                walk_tree_for_color(g, color, elements, limit);
            }
            usvg::Node::Path(path) => {
                let fill = extract_fill_color(path);
                let stroke = extract_stroke_color(path);

                let matches = fill.as_ref().map(|f| f.to_lowercase() == color).unwrap_or(false)
                    || stroke.as_ref().map(|s| s.to_lowercase() == color).unwrap_or(false);

                if matches {
                    elements.push(ElementSummary {
                        element_type: "path".to_string(),
                        id: if path.id().is_empty() {
                            None
                        } else {
                            Some(path.id().to_string())
                        },
                        fill,
                    });
                }
            }
            _ => {}
        }
    }
}

/// Get stats for a specific layer/group by ID.
pub fn query_layer(tree: &usvg::Tree, layer_id: &str) -> Option<LayerResult> {
    let group = find_group_by_id(tree.root(), layer_id)?;

    let mut path_count = 0;
    let mut nested_groups = 0;
    let mut colors = HashSet::new();
    let mut total_count = 0;

    count_layer_stats(group, &mut path_count, &mut nested_groups, &mut colors, &mut total_count);

    Some(LayerResult {
        layer_id: layer_id.to_string(),
        element_count: total_count,
        path_count,
        nested_groups,
        bounding_box: group_bounding_box(group),
        colors_used: colors.into_iter().collect(),
    })
}

fn find_group_by_id<'a>(group: &'a usvg::Group, id: &str) -> Option<&'a usvg::Group> {
    if group.id() == id {
        return Some(group);
    }

    for child in group.children() {
        if let usvg::Node::Group(g) = child {
            if let Some(found) = find_group_by_id(g, id) {
                return Some(found);
            }
        }
    }

    None
}

fn count_layer_stats(
    group: &usvg::Group,
    path_count: &mut usize,
    nested_groups: &mut usize,
    colors: &mut HashSet<String>,
    total_count: &mut usize,
) {
    for child in group.children() {
        *total_count += 1;

        match child {
            usvg::Node::Group(g) => {
                *nested_groups += 1;
                count_layer_stats(g, path_count, nested_groups, colors, total_count);
            }
            usvg::Node::Path(path) => {
                *path_count += 1;
                if let Some(fill) = extract_fill_color(path) {
                    colors.insert(fill);
                }
                if let Some(stroke) = extract_stroke_color(path) {
                    colors.insert(stroke);
                }
            }
            _ => {}
        }
    }
}

fn group_bounding_box(group: &usvg::Group) -> Option<BoundingBox> {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    let mut found = false;

    collect_bounds(group, &mut min_x, &mut min_y, &mut max_x, &mut max_y, &mut found);

    if found {
        Some(BoundingBox::new(min_x, min_y, max_x, max_y))
    } else {
        None
    }
}

fn collect_bounds(
    group: &usvg::Group,
    min_x: &mut f64,
    min_y: &mut f64,
    max_x: &mut f64,
    max_y: &mut f64,
    found: &mut bool,
) {
    for child in group.children() {
        match child {
            usvg::Node::Group(g) => {
                collect_bounds(g, min_x, min_y, max_x, max_y, found);
            }
            usvg::Node::Path(path) => {
                if let Some(bbox) = path_bounding_box(path) {
                    *min_x = min_x.min(bbox.min_x);
                    *min_y = min_y.min(bbox.min_y);
                    *max_x = max_x.max(bbox.max_x);
                    *max_y = max_y.max(bbox.max_y);
                    *found = true;
                }
            }
            _ => {}
        }
    }
}

/// Get random sample of paths.
pub fn query_sample(tree: &usvg::Tree, count: usize) -> SampleResult {
    let paths = collect_all_paths(tree.root());
    let total = paths.len();

    // Use deterministic sampling for reproducibility
    let indices = sample_indices(total, count);

    let samples: Vec<PathSample> = indices
        .iter()
        .map(|&i| path_to_sample(paths[i], i))
        .collect();

    SampleResult {
        total_paths: total,
        sampled_count: samples.len(),
        samples,
    }
}

fn collect_all_paths<'a>(group: &'a usvg::Group) -> Vec<&'a usvg::Path> {
    let mut paths = Vec::new();
    collect_paths_recursive(group, &mut paths);
    paths
}

fn collect_paths_recursive<'a>(group: &'a usvg::Group, paths: &mut Vec<&'a usvg::Path>) {
    for child in group.children() {
        match child {
            usvg::Node::Group(g) => {
                collect_paths_recursive(g, paths);
            }
            usvg::Node::Path(path) => {
                paths.push(path);
            }
            _ => {}
        }
    }
}

fn sample_indices(total: usize, count: usize) -> Vec<usize> {
    if total == 0 || count == 0 {
        return Vec::new();
    }

    let count = count.min(total);

    if count >= total {
        return (0..total).collect();
    }

    // Deterministic sampling: evenly spaced indices
    let step = total as f64 / count as f64;
    (0..count)
        .map(|i| ((i as f64 * step) as usize).min(total - 1))
        .collect()
}

fn path_to_sample(path: &usvg::Path, index: usize) -> PathSample {
    PathSample {
        index,
        id: if path.id().is_empty() {
            None
        } else {
            Some(path.id().to_string())
        },
        fill_color: extract_fill_color(path),
        stroke_color: extract_stroke_color(path),
        point_count: count_path_points(path),
        bounding_box: path_bounding_box(path),
    }
}

fn count_path_points(path: &usvg::Path) -> usize {
    let mut count = 0;
    for segment in path.data().segments() {
        count += match segment {
            usvg::tiny_skia_path::PathSegment::MoveTo(_) => 1,
            usvg::tiny_skia_path::PathSegment::LineTo(_) => 1,
            usvg::tiny_skia_path::PathSegment::QuadTo(_, _) => 2,
            usvg::tiny_skia_path::PathSegment::CubicTo(_, _, _) => 3,
            usvg::tiny_skia_path::PathSegment::Close => 0,
        };
    }
    count
}

/// Get details for a specific element by ID.
pub fn query_element(tree: &usvg::Tree, id: &str) -> Option<ElementResult> {
    find_element_by_id(tree.root(), id)
}

fn find_element_by_id(group: &usvg::Group, id: &str) -> Option<ElementResult> {
    // Check if this group matches
    if group.id() == id {
        return Some(group_to_element_result(group));
    }

    for child in group.children() {
        match child {
            usvg::Node::Group(g) => {
                if let Some(result) = find_element_by_id(g, id) {
                    return Some(result);
                }
            }
            usvg::Node::Path(path) => {
                if path.id() == id {
                    return Some(path_to_element_result(path));
                }
            }
            usvg::Node::Image(img) => {
                if img.id() == id {
                    return Some(image_to_element_result(img));
                }
            }
            usvg::Node::Text(text) => {
                if text.id() == id {
                    return Some(text_to_element_result(text));
                }
            }
        }
    }

    None
}

fn group_to_element_result(group: &usvg::Group) -> ElementResult {
    let children: Vec<ElementSummary> = group
        .children()
        .iter()
        .take(20) // Limit children in output
        .map(|child| match child {
            usvg::Node::Group(g) => ElementSummary {
                element_type: "g".to_string(),
                id: if g.id().is_empty() {
                    None
                } else {
                    Some(g.id().to_string())
                },
                fill: None,
            },
            usvg::Node::Path(p) => ElementSummary {
                element_type: "path".to_string(),
                id: if p.id().is_empty() {
                    None
                } else {
                    Some(p.id().to_string())
                },
                fill: extract_fill_color(p),
            },
            usvg::Node::Image(i) => ElementSummary {
                element_type: "image".to_string(),
                id: if i.id().is_empty() {
                    None
                } else {
                    Some(i.id().to_string())
                },
                fill: None,
            },
            usvg::Node::Text(t) => ElementSummary {
                element_type: "text".to_string(),
                id: if t.id().is_empty() {
                    None
                } else {
                    Some(t.id().to_string())
                },
                fill: None,
            },
        })
        .collect();

    let transform = group.abs_transform();
    let transform_str = if transform.is_identity() {
        None
    } else {
        Some(format!(
            "matrix({:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4})",
            transform.sx, transform.ky, transform.kx, transform.sy, transform.tx, transform.ty
        ))
    };

    ElementResult {
        id: group.id().to_string(),
        element_type: "g".to_string(),
        fill: None,
        stroke: None,
        stroke_width: None,
        opacity: None, // usvg 0.45 doesn't expose opacity directly
        transform: transform_str,
        bounding_box: group_bounding_box(group),
        children: Some(children),
    }
}

fn path_to_element_result(path: &usvg::Path) -> ElementResult {
    let transform = path.abs_transform();
    let transform_str = if transform.is_identity() {
        None
    } else {
        Some(format!(
            "matrix({:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4})",
            transform.sx, transform.ky, transform.kx, transform.sy, transform.tx, transform.ty
        ))
    };

    ElementResult {
        id: path.id().to_string(),
        element_type: "path".to_string(),
        fill: extract_fill_color(path),
        stroke: extract_stroke_color(path),
        stroke_width: path.stroke().map(|s| s.width().get() as f64),
        opacity: None, // usvg 0.45 doesn't expose opacity directly
        transform: transform_str,
        bounding_box: path_bounding_box(path),
        children: None,
    }
}

fn image_to_element_result(img: &usvg::Image) -> ElementResult {
    ElementResult {
        id: img.id().to_string(),
        element_type: "image".to_string(),
        fill: None,
        stroke: None,
        stroke_width: None,
        opacity: None, // usvg 0.45 doesn't expose opacity directly
        transform: None,
        bounding_box: image_bounding_box(img),
        children: None,
    }
}

fn text_to_element_result(text: &usvg::Text) -> ElementResult {
    ElementResult {
        id: text.id().to_string(),
        element_type: "text".to_string(),
        fill: None,
        stroke: None,
        stroke_width: None,
        opacity: None, // usvg 0.45 doesn't expose opacity directly
        transform: None,
        bounding_box: None,
        children: None,
    }
}

// Helper functions for color extraction

fn extract_fill_color(path: &usvg::Path) -> Option<String> {
    path.fill().map(|fill| paint_to_string(fill.paint()))
}

fn extract_stroke_color(path: &usvg::Path) -> Option<String> {
    path.stroke().map(|stroke| paint_to_string(stroke.paint()))
}

fn paint_to_string(paint: &usvg::Paint) -> String {
    match paint {
        usvg::Paint::Color(c) => format!("#{:02x}{:02x}{:02x}", c.red, c.green, c.blue),
        usvg::Paint::LinearGradient(_) => "linear-gradient".to_string(),
        usvg::Paint::RadialGradient(_) => "radial-gradient".to_string(),
        usvg::Paint::Pattern(_) => "pattern".to_string(),
    }
}

fn path_bounding_box(path: &usvg::Path) -> Option<BoundingBox> {
    let bounds = path.data().bounds();
    Some(BoundingBox::new(
        bounds.x() as f64,
        bounds.y() as f64,
        bounds.right() as f64,
        bounds.bottom() as f64,
    ))
}

fn image_bounding_box(img: &usvg::Image) -> Option<BoundingBox> {
    // usvg 0.45 uses size() and abs_transform() for image bounds
    let size = img.size();
    Some(BoundingBox::new(
        0.0,
        0.0,
        size.width() as f64,
        size.height() as f64,
    ))
}
