/// ocr.rs — Tesseract OCR pipeline for Warframe reward screen text extraction.
///
/// Strategy: split the captured region into 4 equal vertical strips (one per
/// reward card) and OCR each strip independently with PSM 6 (single text block).
/// This guarantees left-to-right ordering and gives tesseract a simpler image.
use std::process::Command;

use anyhow::{anyhow, Context};
use log::info;

// ── Temp file helpers ─────────────────────────────────────────────────────────

fn tmp_path(tag: &str) -> std::path::PathBuf {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("tennohelios_{tag}_{ts}.png"))
}

fn write_tmp(bytes: &[u8], tag: &str) -> anyhow::Result<std::path::PathBuf> {
    let path = tmp_path(tag);
    std::fs::write(&path, bytes)
        .with_context(|| format!("failed to write temp file: {}", path.display()))?;
    Ok(path)
}

// ── ImageMagick helpers ───────────────────────────────────────────────────────

/// Returns (width, height) of a PNG file using `magick identify`.
fn image_dimensions(path: &std::path::Path) -> anyhow::Result<(u32, u32)> {
    let out = Command::new("magick")
        .args(["identify", "-format", "%wx%h", path.to_str().unwrap_or("")])
        .output()
        .context("magick identify failed")?;

    let s = String::from_utf8_lossy(&out.stdout);
    // Output: "2560x302"
    let (w, h) = s.trim().split_once('x')
        .ok_or_else(|| anyhow!("unexpected magick identify output: {s}"))?;
    Ok((w.parse().context("bad width")?, h.parse().context("bad height")?))
}

/// Crop a PNG file to `geometry` and return PNG bytes.
fn crop_to_bytes(src: &std::path::Path, geometry: &str) -> anyhow::Result<Vec<u8>> {
    let out = Command::new("magick")
        .args([
            src.to_str().unwrap_or(""),
            "-crop", geometry,
            "+repage",
            "png:-",
        ])
        .output()
        .context("magick crop failed")?;

    if !out.status.success() {
        return Err(anyhow!("magick crop error: {}", String::from_utf8_lossy(&out.stderr)));
    }
    Ok(out.stdout)
}

// ── Tesseract ─────────────────────────────────────────────────────────────────

/// Run tesseract on `png_bytes` with PSM 6 (single block). Returns raw text.
fn run_tesseract(png_bytes: &[u8]) -> anyhow::Result<String> {
    let path = write_tmp(png_bytes, "strip")?;

    let result = Command::new("tesseract")
        .args([
            path.to_str().unwrap_or(""),
            "stdout",
            "--psm", "6",
            "-l", "eng",
        ])
        .output()
        .context("failed to run tesseract — is it installed?");

    let _ = std::fs::remove_file(&path);
    let out = result?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(anyhow!("tesseract exited {}: {stderr}", out.status));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

// ── Item name extraction ───────────────────────────────────────────────────────

/// Substrings that must appear (case-insensitive) for a line to be a Warframe item name.
const ITEM_KEYWORDS: &[&str] = &[
    "blueprint", "prime", "forma", "relic",
    "receiver", "stock", "barrel", "blade", "hilt", "grip", "handle",
    "neuroptics", "chassis", "systems", "carapace", "cerebrum",
    "gauntlet", "plate", "pouch", "band", "buckle", "wings", "ornament",
    "string", "link", "guard",
];

fn looks_like_item(line: &str) -> bool {
    let lower = line.to_lowercase();
    ITEM_KEYWORDS.iter().any(|kw| lower.contains(kw))
}

fn clean_line(raw: &str) -> String {
    raw.trim_matches(|c: char| !c.is_alphanumeric())
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Extract the item name from raw tesseract output for one card strip.
///
/// Item names sometimes wrap across two lines (e.g. "Caliban Prime Chassis" /
/// "Blueprint"). We collect all keyword-matching lines and join them so the
/// full name is reconstructed correctly.
fn best_name_from_text(text: &str) -> Option<String> {
    let parts: Vec<String> = text
        .lines()
        .map(str::trim)
        .filter(|l| l.len() >= 3 && looks_like_item(l))
        .map(|l| clean_line(l))
        .filter(|l| !l.is_empty())
        .collect();

    if parts.is_empty() {
        return None;
    }
    Some(parts.join(" "))
}

// ── Public API ─────────────────────────────────────────────────────────────────

/// Extract Warframe item names from a reward-region PNG.
///
/// Splits the image into 4 equal vertical strips (one per reward card) and
/// OCRs each independently, guaranteeing left-to-right order.
pub fn extract_item_names(png_bytes: &[u8]) -> anyhow::Result<Vec<String>> {
    // Write full image once.
    let full_path = write_tmp(png_bytes, "full")?;

    let result = extract_from_strips(&full_path);
    let _ = std::fs::remove_file(&full_path);
    let names = result?;

    info!("OCR found {} item candidates: {names:?}", names.len());
    Ok(names)
}

fn extract_from_strips(path: &std::path::Path) -> anyhow::Result<Vec<String>> {
    let (width, height) = image_dimensions(path)?;

    // Reward cards are centred in the middle 50% of the screen width.
    let center_x = width / 4;
    let center_w = width / 2;
    let strip_w  = center_w / 4;

    // First crop to the card area only.
    let center_bytes = crop_to_bytes(path, &format!("{center_w}x{height}+{center_x}+0"))
        .context("crop to card area failed")?;
    let center_path = write_tmp(&center_bytes, "center")?;

    // Always produce exactly 4 results — empty string for undetected cards.
    let mut names: Vec<String> = Vec::with_capacity(4);

    for i in 0..4u32 {
        let x_offset = i * strip_w;
        let geometry = format!("{strip_w}x{height}+{x_offset}+0");

        let strip_bytes = crop_to_bytes(&center_path, &geometry)
            .with_context(|| format!("crop card strip {i} failed"))?;

        let text = run_tesseract(&strip_bytes).unwrap_or_default();
        info!("card {i} OCR:\n{text}");

        names.push(best_name_from_text(&text).unwrap_or_default());
    }

    let _ = std::fs::remove_file(&center_path);
    Ok(names)
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blueprint_matches() {
        assert!(looks_like_item("Ash Prime Neuroptics Blueprint"));
    }

    #[test]
    fn forma_matches() {
        assert!(looks_like_item("Forma Blueprint"));
    }

    #[test]
    fn string_matches() {
        assert!(looks_like_item("Paris Prime String"));
    }

    #[test]
    fn noise_rejected() {
        assert!(!looks_like_item("---"));
        assert!(!looks_like_item("Owned"));
        assert!(!looks_like_item("Crafted"));
    }

    #[test]
    fn clean_strips_junk() {
        assert_eq!(clean_line("|Ash Prime Neuroptics Blueprint|"), "Ash Prime Neuroptics Blueprint");
    }

    #[test]
    fn best_name_picks_item_line() {
        let text = "4 Crafted\nAsh Prime Neuroptics Blueprint\nsome noise";
        assert_eq!(best_name_from_text(text).as_deref(), Some("Ash Prime Neuroptics Blueprint"));
    }
}
