//! Tree structure building and rendering for SVG hierarchy.
//!
//! Provides a hierarchical view of the SVG structure with element counts.

use super::types::TreeNode;

/// Build tree structure from usvg tree.
pub fn build_tree(root: &usvg::Group, max_depth: Option<usize>) -> TreeNode {
    build_tree_node(root, "svg", 0, max_depth.unwrap_or(usize::MAX))
}

fn build_tree_node(
    group: &usvg::Group,
    name: &str,
    current_depth: usize,
    max_depth: usize,
) -> TreeNode {
    let id = if group.id().is_empty() {
        None
    } else {
        Some(group.id().to_string())
    };

    let mut children = Vec::new();
    let mut element_count = 0;

    if current_depth < max_depth {
        for child in group.children() {
            match child {
                usvg::Node::Group(g) => {
                    let child_node = build_tree_node(g, "g", current_depth + 1, max_depth);
                    element_count += child_node.element_count + 1;
                    children.push(child_node);
                }
                usvg::Node::Path(p) => {
                    element_count += 1;
                    // Only include paths with IDs in tree view to reduce noise
                    if !p.id().is_empty() {
                        children.push(TreeNode {
                            name: "path".to_string(),
                            id: Some(p.id().to_string()),
                            element_count: 0,
                            children: vec![],
                        });
                    }
                }
                usvg::Node::Image(i) => {
                    element_count += 1;
                    if !i.id().is_empty() {
                        children.push(TreeNode {
                            name: "image".to_string(),
                            id: Some(i.id().to_string()),
                            element_count: 0,
                            children: vec![],
                        });
                    }
                }
                usvg::Node::Text(t) => {
                    element_count += 1;
                    if !t.id().is_empty() {
                        children.push(TreeNode {
                            name: "text".to_string(),
                            id: Some(t.id().to_string()),
                            element_count: 0,
                            children: vec![],
                        });
                    }
                }
            }
        }

        // If there are many paths without IDs, add a summary node
        let paths_without_id = group
            .children()
            .iter()
            .filter(|c| matches!(c, usvg::Node::Path(p) if p.id().is_empty()))
            .count();

        if paths_without_id > 0 {
            children.push(TreeNode {
                name: format!("({} paths)", paths_without_id),
                id: None,
                element_count: paths_without_id,
                children: vec![],
            });
        }
    } else {
        // At max depth, just count children
        element_count = count_all_children(group);
        if element_count > 0 {
            children.push(TreeNode {
                name: format!("({} elements)", element_count),
                id: None,
                element_count,
                children: vec![],
            });
        }
    }

    TreeNode {
        name: name.to_string(),
        id,
        element_count,
        children,
    }
}

fn count_all_children(group: &usvg::Group) -> usize {
    let mut count = 0;
    for child in group.children() {
        count += 1;
        if let usvg::Node::Group(g) = child {
            count += count_all_children(g);
        }
    }
    count
}

/// Render tree in human-readable text format.
pub fn render_tree_text(node: &TreeNode, indent: usize, is_last: bool, prefix: &str) -> String {
    let mut output = String::new();

    // Build the tree branch characters
    let connector = if indent == 0 {
        ""
    } else if is_last {
        "└── "
    } else {
        "├── "
    };

    // Build the display string
    let id_str = node
        .id
        .as_ref()
        .map(|s| format!(" #{}", s))
        .unwrap_or_default();

    let count_str = if node.element_count > 0 && !node.name.starts_with('(') {
        format!(" ({})", node.element_count)
    } else {
        String::new()
    };

    output.push_str(&format!(
        "{}{}{}{}{}\n",
        prefix, connector, node.name, id_str, count_str
    ));

    // Build prefix for children
    let child_prefix = if indent == 0 {
        String::new()
    } else {
        format!("{}{}   ", prefix, if is_last { " " } else { "│" })
    };

    // Render children
    let child_count = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        let is_last_child = i == child_count - 1;
        output.push_str(&render_tree_text(
            child,
            indent + 1,
            is_last_child,
            &child_prefix,
        ));
    }

    output
}

/// Simplified tree rendering for compact output.
pub fn render_tree_compact(node: &TreeNode, indent: usize) -> String {
    let mut output = String::new();
    let spaces = "  ".repeat(indent);

    let id_str = node
        .id
        .as_ref()
        .map(|s| format!(" #{}", s))
        .unwrap_or_default();

    let count_str = if node.element_count > 0 && !node.name.starts_with('(') {
        format!(" ({})", node.element_count)
    } else {
        String::new()
    };

    output.push_str(&format!("{}{}{}{}\n", spaces, node.name, id_str, count_str));

    for child in &node.children {
        output.push_str(&render_tree_compact(child, indent + 1));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_tree_compact() {
        let tree = TreeNode {
            name: "svg".to_string(),
            id: None,
            element_count: 10,
            children: vec![
                TreeNode {
                    name: "g".to_string(),
                    id: Some("Layer1".to_string()),
                    element_count: 5,
                    children: vec![],
                },
                TreeNode {
                    name: "g".to_string(),
                    id: Some("Layer2".to_string()),
                    element_count: 5,
                    children: vec![],
                },
            ],
        };

        let output = render_tree_compact(&tree, 0);
        assert!(output.contains("svg (10)"));
        assert!(output.contains("g #Layer1 (5)"));
        assert!(output.contains("g #Layer2 (5)"));
    }
}
