#![deny(unsafe_code)]

//! X11 and native window screenshot capture utilities for Phonon UI DevTools.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Metadata describing a captured UI screenshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenshotInfo {
    pub file_path: PathBuf,
    pub label: String,
    pub width: u32,
    pub height: u32,
    pub file_size_bytes: u64,
}

/// Locates the native Phonon window ID on the active X11 display.
pub fn find_phonon_window_id() -> Option<String> {
    // Attempt 1: xdotool search
    if let Ok(output) = Command::new("xdotool")
        .args(["search", "--name", "Phonon"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(first_line) = stdout.lines().next() {
                let trimmed = first_line.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }

    // Attempt 2: xwininfo root tree search
    if let Ok(output) = Command::new("xwininfo")
        .args(["-root", "-tree"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("Phonon") {
                    let mut parts = line.split_whitespace();
                    if let Some(id_part) = parts.next() {
                        if id_part.starts_with("0x") {
                            return Some(id_part.to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

/// Captures a screenshot of the specified window ID or the full screen to the given path.
pub fn capture_window_screenshot(
    window_id: Option<&str>,
    output_path: &Path,
    label: &str,
) -> Result<ScreenshotInfo, String> {
    if let Some(parent) = output_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let target_win = window_id.unwrap_or("root");

    let status = Command::new("import")
        .args(["-window", target_win, output_path.to_str().unwrap_or("")])
        .status()
        .map_err(|e| format!("Failed to execute import command: {e}"))?;

    if !status.success() {
        return Err(format!("import command failed with exit code {status:?}"));
    }

    let metadata = std::fs::metadata(output_path)
        .map_err(|e| format!("Failed to read screenshot metadata: {e}"))?;

    // Read image dimensions via file or identify command if available
    let (width, height) = read_image_dimensions(output_path).unwrap_or((1280, 850));

    Ok(ScreenshotInfo {
        file_path: output_path.to_path_buf(),
        label: label.to_string(),
        width,
        height,
        file_size_bytes: metadata.len(),
    })
}

/// Reads image dimensions using `identify` CLI tool if present.
fn read_image_dimensions(path: &Path) -> Option<(u32, u32)> {
    let output = Command::new("identify")
        .args(["-format", "%w %h", path.to_str()?])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut parts = text.split_whitespace();
    let w: u32 = parts.next()?.parse().ok()?;
    let h: u32 = parts.next()?.parse().ok()?;
    Some((w, h))
}
