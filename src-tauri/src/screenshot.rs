/// screenshot.rs — Capture the Warframe window reward region.
///
/// Uses `xdotool` to locate the Warframe X11 window, then pipes
/// `xwd` → `convert` (ImageMagick) to capture and crop to the bottom 40%
/// of the window in a single shell pipeline.  Returns raw PNG bytes.
///
/// Both `xdotool`, `xwd` and `convert` must be present in $PATH.
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{anyhow, Context};
use log::info;

// ── Window geometry ──────────────────────────────────────────────────────────

/// Window ID returned by xdotool, and the pixel dimensions of that window.
#[derive(Debug, Clone)]
struct WindowInfo {
    id: String,
    width: u32,
    height: u32,
}

/// Run `xdotool search --name "Warframe"` and return the first window ID found.
fn find_warframe_window_id() -> anyhow::Result<String> {
    let output = Command::new("xdotool")
        .args(["search", "--name", "Warframe"])
        .output()
        .context("failed to run xdotool — is it installed?")?;

    if !output.status.success() {
        return Err(anyhow!(
            "xdotool search failed (exit {}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let wid = stdout
        .lines()
        .find(|l| !l.trim().is_empty())
        .map(|l| l.trim().to_owned())
        .ok_or_else(|| anyhow!("xdotool found no window named 'Warframe' — is the game running?"))?;

    info!("found Warframe window id: {wid}");
    Ok(wid)
}

/// Run `xdotool getwindowgeometry <wid>` and parse width × height.
///
/// Example output:
/// ```text
/// Window 12345678
///   Position: 2560,0 (screen: 0)
///   Geometry: 2560x1440
/// ```
fn get_window_geometry(wid: &str) -> anyhow::Result<(u32, u32)> {
    let output = Command::new("xdotool")
        .args(["getwindowgeometry", wid])
        .output()
        .context("failed to run xdotool getwindowgeometry")?;

    if !output.status.success() {
        return Err(anyhow!(
            "xdotool getwindowgeometry failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Find the "Geometry: WxH" line.
    let geometry_line = stdout
        .lines()
        .find(|l| l.trim_start().starts_with("Geometry:"))
        .ok_or_else(|| anyhow!("no Geometry line in xdotool output:\n{stdout}"))?;

    // Extract "WxH" token.
    let dims = geometry_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| anyhow!("could not parse Geometry line: {geometry_line}"))?;

    let (w_str, h_str) = dims
        .split_once('x')
        .ok_or_else(|| anyhow!("unexpected geometry format '{dims}'"))?;

    let width: u32 = w_str
        .parse()
        .with_context(|| format!("cannot parse width '{w_str}'"))?;
    let height: u32 = h_str
        .parse()
        .with_context(|| format!("cannot parse height '{h_str}'"))?;

    info!("Warframe window geometry: {width}x{height}");
    Ok((width, height))
}

fn warframe_window_info() -> anyhow::Result<WindowInfo> {
    let id = find_warframe_window_id()?;
    let (width, height) = get_window_geometry(&id)?;
    Ok(WindowInfo { id, width, height })
}

// ── Crop geometry ─────────────────────────────────────────────────────────────

/// Build an ImageMagick geometry string that captures the reward card name area.
///
/// Format: `<width>x<crop_height>+0+<y_start>`
///
/// Warframe's relic reward UI sits in the upper portion of the screen.
/// Item names appear at the bottom of each card at roughly y=35–45%.
/// Player names appear just below at y=47–52% — we stop before them.
/// We capture y=25–46% (21% of height) to get item names only.
fn reward_crop_geometry(width: u32, height: u32) -> String {
    let y_start = (height as f32 * 0.25).round() as u32;
    let crop_height = (height as f32 * 0.21).round() as u32;
    format!("{width}x{crop_height}+0+{y_start}")
}

// ── Capture pipeline ──────────────────────────────────────────────────────────

/// Capture the reward region of the Warframe window.
///
/// Returns raw PNG bytes, or an error if capture fails.
///
/// Pipeline:
/// ```text
/// magick import -window <wid> -crop <geometry> +repage png:-
/// ```
/// Uses ImageMagick's `import` (via `magick import` for IM7) which captures
/// X11 windows directly — no separate `xwd` tool required.
pub fn capture_reward_region() -> anyhow::Result<Vec<u8>> {
    let info = warframe_window_info()?;
    let geometry = reward_crop_geometry(info.width, info.height);
    info!("capturing reward region with geometry: {geometry}");

    // `magick import` (ImageMagick 7) captures the window and crops in one step.
    // Fall back to bare `import` (ImageMagick 6) if `magick` is unavailable.
    let output = run_import(&info.id, &geometry)
        .context("ImageMagick import failed — is ImageMagick installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("magick import exited {}: {stderr}", output.status));
    }

    if output.stdout.is_empty() {
        return Err(anyhow!("magick import produced no output"));
    }

    info!("captured {} bytes of PNG for reward region", output.stdout.len());
    Ok(output.stdout)
}

/// Try `magick import` (IM7), then fall back to `import` (IM6).
fn run_import(wid: &str, geometry: &str) -> std::io::Result<std::process::Output> {
    let args = [
        "import",
        "-window", wid,
        "-crop", geometry,
        "+repage",
        "png:-",
    ];

    // Try ImageMagick 7 first.
    let result = Command::new("magick")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match result {
        Ok(out) => Ok(out),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // Fall back to ImageMagick 6 bare `import`.
            Command::new("import")
                .args(&args[1..]) // skip the "import" subcommand word
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
        }
        Err(e) => Err(e),
    }
}

/// Save the captured reward region PNG to `path` for debugging OCR input.
pub fn save_debug_screenshot(path: &Path) -> anyhow::Result<()> {
    let png_bytes = capture_reward_region()?;
    std::fs::write(path, &png_bytes)
        .with_context(|| format!("failed to write debug screenshot to {}", path.display()))?;
    info!("debug screenshot saved to {}", path.display());
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crop_geometry_1440p() {
        let geom = reward_crop_geometry(2560, 1440);
        // y_start = 25% of 1440 = 360; height = 21% of 1440 = 302
        assert_eq!(geom, "2560x302+0+360");
    }

    #[test]
    fn crop_geometry_1080p() {
        let geom = reward_crop_geometry(1920, 1080);
        // y_start = 25% of 1080 = 270; height = 21% of 1080 = 227
        assert_eq!(geom, "1920x227+0+270");
    }

    #[test]
    fn crop_geometry_steam_deck() {
        // Steam Deck native: 1280x800
        let geom = reward_crop_geometry(1280, 800);
        // y_start = 25% of 800 = 200; height = 21% of 800 = 168
        assert_eq!(geom, "1280x168+0+200");
    }

    /// Smoke-test the geometry parser with realistic xdotool output.
    #[test]
    fn parse_geometry_line() {
        let fake_output = "Window 12345678\n  Position: 2560,0 (screen: 0)\n  Geometry: 2560x1440\n";
        let line = fake_output
            .lines()
            .find(|l| l.trim_start().starts_with("Geometry:"))
            .unwrap();
        let dims = line.split_whitespace().nth(1).unwrap();
        let (w, h) = dims.split_once('x').unwrap();
        assert_eq!(w, "2560");
        assert_eq!(h, "1440");
    }
}
