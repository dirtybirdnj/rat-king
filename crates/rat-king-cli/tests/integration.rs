//! Integration tests for rat-king CLI commands.
//!
//! These tests run the actual binary and verify end-to-end behavior.

use std::process::Command;
use std::path::PathBuf;

/// Get the path to the rat-king binary from the workspace root.
fn binary_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // Go up from rat-king-cli to crates

    // Try release first, then debug
    let release = path.join("target/release/rat-king");
    if release.exists() {
        return release;
    }
    path.join("target/debug/rat-king")
}

/// Get the path to a test SVG file.
fn test_svg_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // Go up from rat-king-cli to crates
    path.pop(); // Go up from crates to repo root
    path.push("test_assets/essex.svg");
    path
}

#[test]
fn patterns_command_lists_all_patterns() {
    let output = Command::new(binary_path())
        .arg("patterns")
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check for some known patterns
    assert!(stdout.contains("lines"), "Should list 'lines' pattern");
    assert!(stdout.contains("crosshatch"), "Should list 'crosshatch' pattern");
    assert!(stdout.contains("spiral"), "Should list 'spiral' pattern");
    assert!(stdout.contains("honeycomb"), "Should list 'honeycomb' pattern");
    assert!(stdout.contains("harmonograph"), "Should list 'harmonograph' pattern");

    // Should have at least 30 patterns (one per line after header)
    let line_count = stdout.lines().count();
    assert!(line_count >= 30, "Should list at least 30 patterns, got {}", line_count);
}

#[test]
fn fill_command_produces_svg() {
    let svg_path = test_svg_path();
    if !svg_path.exists() {
        eprintln!("Skipping test - test SVG not found at {:?}", svg_path);
        return;
    }

    let output = Command::new(binary_path())
        .args(["fill", svg_path.to_str().unwrap(), "-p", "lines"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Output should be valid SVG
    assert!(stdout.contains("<?xml"), "Should have XML declaration");
    assert!(stdout.contains("<svg"), "Should have SVG element");
    assert!(stdout.contains("<line"), "Should have line elements");
    assert!(stdout.contains("</svg>"), "Should close SVG element");
}

#[test]
fn fill_command_produces_json() {
    let svg_path = test_svg_path();
    if !svg_path.exists() {
        eprintln!("Skipping test - test SVG not found at {:?}", svg_path);
        return;
    }

    let output = Command::new(binary_path())
        .args(["fill", svg_path.to_str().unwrap(), "-p", "lines", "--json"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Output should be valid JSON with lines array
    assert!(stdout.contains("\"lines\""), "Should have lines key");
    assert!(stdout.contains("\"x1\""), "Should have x1 coordinate");
    assert!(stdout.contains("\"y1\""), "Should have y1 coordinate");
    assert!(stdout.contains("\"x2\""), "Should have x2 coordinate");
    assert!(stdout.contains("\"y2\""), "Should have y2 coordinate");
}

#[test]
fn fill_command_different_patterns_produce_output() {
    let svg_path = test_svg_path();
    if !svg_path.exists() {
        eprintln!("Skipping test - test SVG not found at {:?}", svg_path);
        return;
    }

    // Test a selection of patterns to ensure they all work
    let patterns = ["lines", "crosshatch", "spiral", "honeycomb", "grid", "diagonal"];

    for pattern in patterns {
        let output = Command::new(binary_path())
            .args(["fill", svg_path.to_str().unwrap(), "-p", pattern])
            .output()
            .unwrap_or_else(|_| panic!("Failed to execute command for pattern {}", pattern));

        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            stdout.contains("<line"),
            "Pattern '{}' should produce line elements",
            pattern
        );
    }
}

#[test]
fn benchmark_command_runs() {
    let svg_path = test_svg_path();
    if !svg_path.exists() {
        eprintln!("Skipping test - test SVG not found at {:?}", svg_path);
        return;
    }

    let output = Command::new(binary_path())
        .args(["benchmark", svg_path.to_str().unwrap(), "-p", "lines"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    // Benchmark output may go to stdout or stderr
    assert!(combined.contains("BENCHMARK"), "Should show benchmark header");
    assert!(combined.to_lowercase().contains("lines"), "Should show pattern name");
    assert!(combined.contains("Time"), "Should show timing info");
}

#[test]
fn help_command_shows_usage() {
    let output = Command::new(binary_path())
        .arg("help")
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    // Help should show usage info
    assert!(combined.contains("fill"), "Should mention fill command");
    assert!(combined.contains("benchmark"), "Should mention benchmark command");
    assert!(combined.contains("patterns"), "Should mention patterns command");
}

#[test]
fn fill_with_spacing_option() {
    let svg_path = test_svg_path();
    if !svg_path.exists() {
        eprintln!("Skipping test - test SVG not found at {:?}", svg_path);
        return;
    }

    // Smaller spacing should produce more lines
    let dense_output = Command::new(binary_path())
        .args(["fill", svg_path.to_str().unwrap(), "-p", "lines", "-s", "2", "--json"])
        .output()
        .expect("Failed to execute command");

    let sparse_output = Command::new(binary_path())
        .args(["fill", svg_path.to_str().unwrap(), "-p", "lines", "-s", "10", "--json"])
        .output()
        .expect("Failed to execute command");

    let dense_stdout = String::from_utf8_lossy(&dense_output.stdout);
    let sparse_stdout = String::from_utf8_lossy(&sparse_output.stdout);

    // Count occurrences of "x1" to estimate line count
    let dense_lines = dense_stdout.matches("\"x1\"").count();
    let sparse_lines = sparse_stdout.matches("\"x1\"").count();

    assert!(
        dense_lines > sparse_lines,
        "Smaller spacing ({} lines) should produce more lines than larger spacing ({} lines)",
        dense_lines, sparse_lines
    );
}

#[test]
fn fill_with_angle_option() {
    let svg_path = test_svg_path();
    if !svg_path.exists() {
        eprintln!("Skipping test - test SVG not found at {:?}", svg_path);
        return;
    }

    // Different angles should produce different output
    let output_0 = Command::new(binary_path())
        .args(["fill", svg_path.to_str().unwrap(), "-p", "lines", "-a", "0"])
        .output()
        .expect("Failed to execute command");

    let output_45 = Command::new(binary_path())
        .args(["fill", svg_path.to_str().unwrap(), "-p", "lines", "-a", "45"])
        .output()
        .expect("Failed to execute command");

    let stdout_0 = String::from_utf8_lossy(&output_0.stdout);
    let stdout_45 = String::from_utf8_lossy(&output_45.stdout);

    // Both should produce valid SVG
    assert!(stdout_0.contains("<line"), "Angle 0 should produce lines");
    assert!(stdout_45.contains("<line"), "Angle 45 should produce lines");

    // They should be different (the actual coordinates will differ)
    assert_ne!(stdout_0, stdout_45, "Different angles should produce different output");
}
