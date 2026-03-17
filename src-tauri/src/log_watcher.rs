/// log_watcher.rs — watches EE.log for Warframe relic reward screen events.
///
/// Detection strategy (based on wfinfo-ng research):
/// Warframe writes specific lines to EE.log when game state changes.
/// We tail the file from the end and scan each new line for trigger patterns.
///
/// Known trigger lines (in order of reliability):
///   "Created /Lotus/Interface/ProjectionRewardChoice.swf"  ← relic reward screen opened
///   "Got rewards"                                          ← rewards accepted (too late)
///
/// We fire on ProjectionRewardChoice which gives the most reaction time.
use std::{
    fs::File,
    io::{BufRead, BufReader, Seek, SeekFrom},
    path::PathBuf,
    sync::mpsc,
    time::Duration,
};

use log::{debug, error, info, warn};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

/// Sent through the channel when a reward screen is detected.
#[derive(Debug, Clone)]
pub struct RewardScreenEvent {
    /// The raw log line that triggered detection.
    pub trigger_line: String,
}

/// Patterns that indicate the relic reward screen is visible.
/// Ordered from most specific / earliest to least.
const TRIGGER_PATTERNS: &[&str] = &[
    // Reward choice UI created — fires as screen appears
    "Created /Lotus/Interface/ProjectionRewardChoice.swf",
    // Fallback: older log format seen in some Proton builds
    "Pause countdown done",
];

/// Watch `log_path` for reward screen events and send them on `tx`.
///
/// This function blocks — run it in a dedicated thread or Tokio task.
/// It will return only on unrecoverable error.
pub fn watch(log_path: PathBuf, tx: mpsc::Sender<RewardScreenEvent>) -> anyhow::Result<()> {
    info!("Starting log watcher on: {}", log_path.display());

    // Open file and seek to end so we only read *new* lines written after startup.
    let mut file = File::open(&log_path)?;
    file.seek(SeekFrom::End(0))?;
    let mut position = file.stream_position()?;

    // Channel for notify filesystem events.
    let (fs_tx, fs_rx) = mpsc::channel::<notify::Result<Event>>();

    let mut watcher = RecommendedWatcher::new(
        fs_tx,
        Config::default().with_poll_interval(Duration::from_millis(200)),
    )?;

    // Canonicalise so that symlinks (e.g. macOS /var/folders → /private/var) are resolved.
    let watch_path = log_path.canonicalize().unwrap_or_else(|_| log_path.clone());
    watcher.watch(&watch_path, RecursiveMode::NonRecursive)?;

    info!("Watcher active — waiting for EE.log writes…");

    for event_result in &fs_rx {
        match event_result {
            Ok(event) => {
                // On macOS FSEvents, writes may arrive as Modify, Access(Write), or Any.
                // Skip only Remove events; re-read on everything else.
                let skip = matches!(event.kind, EventKind::Remove(_));
                if !skip {
                    debug!("FS event: {:?}", event.kind);
                    if let Some(trigger) = read_new_lines(&log_path, &mut position) {
                        info!("Reward screen detected: {:?}", trigger);
                        if tx.send(trigger).is_err() {
                            // Receiver dropped — time to exit
                            warn!("Receiver dropped, stopping log watcher");
                            return Ok(());
                        }
                    }
                }
            }
            Err(e) => {
                error!("File watch error: {e}");
            }
        }
    }

    Ok(())
}

/// Read lines written to `log_path` since `position`, update `position`,
/// and return the first `RewardScreenEvent` found (if any).
fn read_new_lines(log_path: &PathBuf, position: &mut u64) -> Option<RewardScreenEvent> {
    let mut file = match File::open(log_path) {
        Ok(f) => f,
        Err(e) => {
            warn!("Could not open EE.log: {e}");
            return None;
        }
    };

    // EE.log can be truncated (e.g. on game restart) — handle gracefully.
    let file_len = file.metadata().ok()?.len();
    if file_len < *position {
        info!("EE.log was truncated, resetting read position");
        *position = 0;
    }

    if file.seek(SeekFrom::Start(*position)).is_err() {
        return None;
    }

    let mut found: Option<RewardScreenEvent> = None;
    let reader = BufReader::new(&file);

    for line in reader.lines() {
        match line {
            Ok(text) => {
                debug!("EE.log line: {text}");
                if found.is_none() {
                    for &pattern in TRIGGER_PATTERNS {
                        if text.contains(pattern) {
                            found = Some(RewardScreenEvent {
                                trigger_line: text.clone(),
                            });
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Error reading EE.log line: {e}");
                break;
            }
        }
    }

    // Update position to current end so next read starts here.
    if let Ok(new_pos) = file.stream_position() {
        *position = new_pos;
    }

    found
}

/// Default EE.log path for Warframe running under Steam + Proton on Linux.
pub fn default_log_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    PathBuf::from(home)
        .join(".local/share/Steam/steamapps/compatdata/230410/pfx/drive_c")
        .join("users/steamuser/AppData/Local/Warframe/EE.log")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::time::Duration;
    use tempfile::NamedTempFile;

    /// Write `content` to a temp file and return it (file stays open / alive).
    fn make_log(content: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().expect("temp file");
        f.write_all(content.as_bytes()).expect("write");
        f.flush().expect("flush");
        f
    }

    // ── Unit tests for read_new_lines ────────────────────────────────────────

    #[test]
    fn detects_projection_reward_choice() {
        let log = make_log(
            "1.0 Script [Info]: something normal\n\
             2.0 Script [Info]: Created /Lotus/Interface/ProjectionRewardChoice.swf\n\
             3.0 Script [Info]: another line\n",
        );
        let path = log.path().to_path_buf();
        let mut pos = 0u64;

        let result = read_new_lines(&path, &mut pos);
        assert!(result.is_some(), "should detect reward screen");
        let event = result.unwrap();
        assert!(
            event.trigger_line.contains("ProjectionRewardChoice"),
            "trigger_line should contain the pattern"
        );
    }

    #[test]
    fn detects_pause_countdown_fallback() {
        let log = make_log("1.0 Sys [Info]: Pause countdown done\n");
        let path = log.path().to_path_buf();
        let mut pos = 0u64;

        let result = read_new_lines(&path, &mut pos);
        assert!(result.is_some(), "should detect via fallback pattern");
    }

    #[test]
    fn no_false_positive_on_normal_lines() {
        let log = make_log(
            "1.0 Script [Info]: Loading mission\n\
             2.0 Net [Info]: Connection established\n\
             3.0 AI [Info]: Enemy spawned\n",
        );
        let path = log.path().to_path_buf();
        let mut pos = 0u64;

        let result = read_new_lines(&path, &mut pos);
        assert!(result.is_none(), "should not fire on unrelated lines");
    }

    #[test]
    fn advances_position_after_read() {
        let content = "line one\nline two\n";
        let log = make_log(content);
        let path = log.path().to_path_buf();
        let mut pos = 0u64;

        read_new_lines(&path, &mut pos);
        assert_eq!(
            pos,
            content.len() as u64,
            "position should advance to end of file"
        );
    }

    #[test]
    fn only_reads_new_lines_after_position() {
        // First part: normal lines
        let mut log = make_log("line one\nline two\n");
        let path = log.path().to_path_buf();
        let mut pos = 0u64;

        // Read first batch — no triggers
        let r1 = read_new_lines(&path, &mut pos);
        assert!(r1.is_none());

        // Append a trigger line
        write!(
            log,
            "Created /Lotus/Interface/ProjectionRewardChoice.swf\n"
        )
        .expect("append");
        log.flush().expect("flush");

        // Second read — should pick up only the new line
        let r2 = read_new_lines(&path, &mut pos);
        assert!(r2.is_some(), "should detect trigger appended after position");
    }

    #[test]
    fn handles_truncated_log_gracefully() {
        let content = "some initial content\n";
        let log = make_log(content);
        let path = log.path().to_path_buf();

        // Set position beyond the file length (simulates game restart / log rotation)
        let mut pos = 9999u64;

        // Should not panic — should reset and read from start
        let _ = read_new_lines(&path, &mut pos);

        // After truncation reset, pos should be at end-of-file, not at 9999
        assert!(
            pos < 9999,
            "position should have been reset from 9999, got {pos}"
        );
        assert_eq!(
            pos,
            content.len() as u64,
            "position should be at end of file after re-read"
        );
    }

    // ── Integration test: detection loop with a live file ────────────────────
    //
    // We test our detection logic (incremental read + pattern match + channel
    // send) with a simple polling loop rather than the notify watcher.  The
    // notify integration is thin and its behaviour is OS-specific; on macOS
    // FSEvents is sandbox-restricted in test contexts and unreliable with temp
    // file symlinks.  On Linux (the target platform) it works as expected.

    #[test]
    fn detection_loop_finds_trigger_in_live_file() {
        use std::thread;

        let mut log = make_log("");
        let path = log.path().to_path_buf();

        let (tx, rx) = mpsc::channel::<RewardScreenEvent>();

        // Simulate the detection loop: poll every 50 ms for up to 5 s.
        let poll_path = path.clone();
        thread::spawn(move || {
            let mut pos = 0u64;
            for _ in 0..100 {
                if let Some(event) = read_new_lines(&poll_path, &mut pos) {
                    let _ = tx.send(event);
                    return;
                }
                thread::sleep(Duration::from_millis(50));
            }
        });

        // Give the polling thread a moment to start, then append the trigger.
        thread::sleep(Duration::from_millis(100));
        writeln!(
            log,
            "Created /Lotus/Interface/ProjectionRewardChoice.swf"
        )
        .expect("write trigger");
        log.flush().expect("flush");

        let event = rx
            .recv_timeout(Duration::from_secs(5))
            .expect("detection loop should find trigger within 5 s");

        assert!(
            event.trigger_line.contains("ProjectionRewardChoice"),
            "unexpected trigger_line: {:?}",
            event.trigger_line
        );
    }
}
