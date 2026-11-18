//! Turn installer chatter into real download numbers.
//!
//! Every installer EnvHub drives (mise, brew, scoop, winget, apt, pip, curl)
//! redraws a progress frame with carriage returns instead of printing clean
//! lines, and each of them formats that frame differently:
//!
//! ```text
//! fd@10.2.0  download fd-v10.2.0-aarch64-… 204.9 KB 1s [==>            ]   (mise / clx)
//! 45%|████▌     | 45.6/120.0 MB [00:12<00:20, 3.2MB/s]                     (tqdm / pip)
//! Get:3 http://deb.debian.org/... 45% [Connecting]                         (apt)
//! ```
//!
//! Parsing those frames is what lets the install modal show a real percentage,
//! byte counts and a remaining-time estimate instead of an invented creep.

use serde::Serialize;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

/// Install phases, ordered from the earliest to the last one. A later phase
/// never regresses to an earlier one: `mise` prints `install` before
/// `download`, and a stray "installed" line must not rewind the bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    Prepare,
    Download,
    Verify,
    Extract,
    Configure,
    Done,
}

impl Stage {
    pub fn key(self) -> &'static str {
        match self {
            Stage::Prepare => "prepare",
            Stage::Download => "download",
            Stage::Verify => "verify",
            Stage::Extract => "extract",
            Stage::Configure => "configure",
            Stage::Done => "done",
        }
    }

    /// Percentage the bar jumps to as soon as the phase is known.
    fn floor(self) -> u32 {
        match self {
            Stage::Prepare => 8,
            Stage::Download => 20,
            Stage::Verify => 86,
            Stage::Extract => 90,
            Stage::Configure => 94,
            Stage::Done => 100,
        }
    }

    fn from_text(lower: &str) -> Option<Stage> {
        const MARKERS: &[(&str, Stage)] = &[
            ("download", Stage::Download),
            ("downloading", Stage::Download),
            ("fetch", Stage::Download),
            ("retriev", Stage::Download),
            ("get:", Stage::Download),
            ("pull", Stage::Download),
            ("checksum", Stage::Verify),
            ("verify", Stage::Verify),
            ("verifying", Stage::Verify),
            ("sha256", Stage::Verify),
            ("extract", Stage::Extract),
            ("unpack", Stage::Extract),
            ("untar", Stage::Extract),
            ("decompress", Stage::Extract),
            ("expand", Stage::Extract),
            ("installing", Stage::Configure),
            ("linking", Stage::Configure),
            ("activat", Stage::Configure),
            ("配置", Stage::Configure),
            ("解压", Stage::Extract),
            ("校验", Stage::Verify),
            ("下载", Stage::Download),
            ("installed", Stage::Configure),
            ("complete", Stage::Configure),
            ("success", Stage::Configure),
            ("✓", Stage::Configure),
            ("安装成功", Stage::Configure),
        ];
        // No marker may reach `Stage::Done`: "already installed" and "installed
        // but not activated" appear mid-run, and only the process exit decides
        // that the install really finished.
        MARKERS
            .iter()
            .filter(|(marker, _)| lower.contains(marker))
            .map(|(_, stage)| *stage)
            .max()
    }
}

/// One `install-progress-detail` payload. `None` fields mean "this installer
/// does not tell us", and the UI then falls back to the percentage alone.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallProgress {
    pub percent: u32,
    pub stage: &'static str,
    pub downloaded: Option<u64>,
    pub total: Option<u64>,
    pub eta_secs: Option<u64>,
    pub speed_bps: Option<u64>,
}

/// What a single output record was worth.
#[derive(Debug, Default)]
pub struct Observation {
    /// Line to append to the install log, deduplicated against the previous
    /// frame of the same download.
    pub log: Option<String>,
    /// Latest progress, when it changed enough to be worth an event.
    pub progress: Option<InstallProgress>,
    /// Percentage to publish on the legacy `install-progress` channel.
    pub percent_event: Option<u32>,
    pub stage_changed: bool,
}

pub struct ProgressTracker {
    stage: Stage,
    percent: u32,
    downloaded: Option<u64>,
    total: Option<u64>,
    eta_secs: Option<u64>,
    speed_bps: Option<u64>,
    bar_ratio: Option<f64>,
    sample: Option<(Instant, u64)>,
    last_label: String,
    last_emitted: Option<Instant>,
    last_progress: Option<InstallProgress>,
    last_percent: u32,
    /// Minimum gap between two `install-progress-detail` events, so a 20 fps
    /// frame flood does not become 20 fps IPC.
    throttle: Duration,
}

impl Default for ProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressTracker {
    pub fn new() -> Self {
        Self::with_throttle(Duration::from_millis(150))
    }

    /// Same tracker with the event throttle disabled, so tests can observe
    /// every frame without sleeping.
    #[cfg(test)]
    fn untrottled() -> Self {
        Self::with_throttle(Duration::ZERO)
    }

    fn with_throttle(throttle: Duration) -> Self {
        Self {
            stage: Stage::Prepare,
            percent: Stage::Prepare.floor(),
            downloaded: None,
            total: None,
            eta_secs: None,
            speed_bps: None,
            bar_ratio: None,
            sample: None,
            last_label: String::new(),
            last_emitted: None,
            last_progress: None,
            last_percent: 0,
            throttle,
        }
    }

    pub fn observe(&mut self, raw: &str) -> Observation {
        let text = strip_ansi(raw).trim().to_string();
        if text.is_empty() {
            return Observation::default();
        }
        let lower = text.to_lowercase();

        let mut stage_changed = false;
        if let Some(stage) = Stage::from_text(&lower) {
            if stage > self.stage {
                self.stage = stage;
                stage_changed = true;
            }
        }

        let percent_token = parse_percent(&text);
        let sizes = parse_sizes(&text);
        let pair = parse_progress_pair(&text);
        let bar = parse_bar_ratio(&text);
        let speed = parse_speed(&text);

        // Byte counts are cumulative, so they may only grow. `mise` prints the
        // running total without a denominator, which is still enough to show
        // "12.4 MB" while the total is unknown.
        if let Some((done, total)) = pair {
            self.downloaded = Some(done);
            self.total = Some(total);
        } else if let Some(bytes) = sizes.iter().copied().max() {
            self.downloaded = Some(self.downloaded.map_or(bytes, |prev| prev.max(bytes)));
        }
        if let Some(ratio) = bar {
            self.bar_ratio = Some(ratio);
            // Refine the total on every frame: an 18-cell bar is coarse, so the
            // first estimate (`3.5 MB / [=>  ]` ⇒ "32 MB") can be off by more
            // than a factor of two, and a frozen estimate stalls the bar at 50%.
            if self.stage == Stage::Download && pair.is_none() {
                if let Some(done) = self.downloaded {
                    if ratio >= 0.05 && done > 0 {
                        let estimate = (done as f64 / ratio).round() as u64;
                        // Smoothed, or the denominator would jitter by several
                        // MB between neighbouring cells of the same bar.
                        let smoothed = match self.total {
                            Some(previous) if previous > 0 => (previous * 3 + estimate) / 4,
                            _ => estimate,
                        };
                        self.total = Some(smoothed.max(done));
                    }
                }
            }
        }

        let now = Instant::now();
        self.update_rate(now, speed);

        if let Some(eta) = parse_eta(&text) {
            // `clx` prints `0s` for the first frames of a download it has not
            // measured yet; publishing that would flash "0s remaining".
            if eta > 0 || self.stage >= Stage::Done {
                self.eta_secs = Some(eta);
            }
        } else if let (Some(done), Some(total), Some(bps)) =
            (self.downloaded, self.total, self.speed_bps)
        {
            if bps > 0 && total > done {
                self.eta_secs = Some((total - done) / bps);
            }
        }

        self.advance_percent(percent_token);

        // Keep one line per artifact instead of one per redraw: the label is
        // the frame with all numbers, bar art and spinner glyphs removed, so a
        // download that prints 200 frames contributes a single log line.
        let label = frame_label(&text);
        let log = if label != self.last_label || stage_changed {
            Some(text.clone())
        } else {
            None
        };
        self.last_label = label;

        let progress = InstallProgress {
            percent: self.percent,
            stage: self.stage.key(),
            downloaded: self.downloaded,
            total: self.total,
            eta_secs: if self.stage >= Stage::Verify {
                // Remaining time only means something while bytes are moving.
                None
            } else {
                self.eta_secs
            },
            speed_bps: self.speed_bps,
        };
        let changed = self.last_progress.as_ref() != Some(&progress);
        let throttle_ok = self
            .last_emitted
            .map(|at| now.duration_since(at) >= self.throttle)
            .unwrap_or(true);
        let progress_event = if changed && throttle_ok {
            self.last_emitted = Some(now);
            self.last_progress = Some(progress.clone());
            Some(progress)
        } else {
            None
        };

        let percent_event = if self.percent != self.last_percent {
            self.last_percent = self.percent;
            Some(self.percent)
        } else {
            None
        };

        Observation {
            log,
            progress: progress_event,
            percent_event,
            stage_changed,
        }
    }

    /// Called once the child exited with status 0. Nothing the installers print
    /// can reach `Stage::Done` on its own — they say "installed" while they are
    /// still working — so the exit status is the only finish signal.
    pub fn finish(&mut self) -> InstallProgress {
        self.stage = Stage::Done;
        self.percent = 100;
        self.eta_secs = None;
        let progress = InstallProgress {
            percent: 100,
            stage: Stage::Done.key(),
            downloaded: self.downloaded,
            total: self.total,
            eta_secs: None,
            speed_bps: self.speed_bps,
        };
        self.last_progress = Some(progress.clone());
        self.last_percent = 100;
        progress
    }

    /// Smooth download ratio: byte counts when the installer reports a total,
    /// otherwise the fill level of its own progress bar.
    fn download_ratio(&self) -> Option<f64> {
        if let (Some(done), Some(total)) = (self.downloaded, self.total) {
            if total > 0 && done > 0 {
                return Some((done as f64 / total as f64).clamp(0.0, 1.0));
            }
        }
        self.bar_ratio
    }

    fn advance_percent(&mut self, percent_token: Option<u32>) {
        let mut percent = self.percent.max(self.stage.floor());
        if self.stage == Stage::Download {
            // 20% is "download started", 85% is "download finished"; the rest
            // of the phases live above it.
            if let Some(ratio) = self.download_ratio() {
                percent = percent.max((20.0 + ratio * 65.0) as u32);
            } else if let Some(token) = percent_token {
                percent = percent.max(20 + token.min(100) * 65 / 100);
            }
        } else if self.stage == Stage::Prepare {
            if let Some(token) = percent_token {
                percent = percent.max(token.min(19));
            }
        }
        percent = percent.min(if self.stage >= Stage::Done { 100 } else { 99 });
        self.percent = self.percent.max(percent);
    }

    fn update_rate(&mut self, now: Instant, parsed_speed: Option<u64>) {
        if let Some(speed) = parsed_speed {
            self.speed_bps = Some(speed);
            return;
        }
        let Some(downloaded) = self.downloaded else {
            return;
        };
        let Some((at, previous)) = self.sample else {
            self.sample = Some((now, downloaded));
            return;
        };
        let elapsed = now.duration_since(at).as_secs_f64();
        if elapsed < 0.4 {
            return;
        }
        if downloaded > previous {
            let bps = ((downloaded - previous) as f64 / elapsed).round() as u64;
            // Exponential smoothing keeps the ETA from jittering between frames.
            self.speed_bps = Some(match self.speed_bps {
                Some(previous_bps) if previous_bps > 0 => (previous_bps * 3 + bps) / 4,
                _ => bps,
            });
        }
        self.sample = Some((now, downloaded));
    }
}

// ---------------------------------------------------------------- formatting

/// Remove CSI/OSC escape sequences and other control bytes, so the frames can
/// be parsed, logged and rendered in a plain-text log viewer.
pub fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\x1b' => match chars.peek() {
                // CSI: ESC [ ... final-byte (0x40..=0x7e)
                Some('[') => {
                    chars.next();
                    for next in chars.by_ref() {
                        if ('\x40'..='\x7e').contains(&next) {
                            break;
                        }
                    }
                }
                // OSC: ESC ] ... BEL | ESC \
                Some(']') => {
                    chars.next();
                    while let Some(next) = chars.next() {
                        if next == '\x07' {
                            break;
                        }
                        if next == '\x1b' && chars.peek() == Some(&'\\') {
                            chars.next();
                            break;
                        }
                    }
                }
                _ => {
                    chars.next();
                }
            },
            // Spinner glyphs and block art would only pollute the log.
            '\u{a0}' => out.push(' '),
            // Nerd Font icons, including the one `clx` splices into a label it
            // had to truncate.
            c if is_private_use(c) => {}
            // Spinner frames (`◜ ◠ ◝ ◞`, `⠋ ⠙ ⠹`, `◐ ◓ ◑ ◒`) are redrawn on
            // every tick and would make each frame look like a new log line.
            c if is_spinner(c) => {}
            c if c.is_control() && c != '\t' => {}
            c => out.push(c),
        }
    }
    out
}

fn is_private_use(c: char) -> bool {
    matches!(c as u32, 0xE000..=0xF8FF | 0xF0000..=0xFFFFD | 0x100000..=0x10FFFD)
}

fn is_spinner(c: char) -> bool {
    matches!(c as u32, 0x2800..=0x28FF | 0x25D0..=0x25DF | 0x25E0..=0x25EF | 0x25F4..=0x25F7)
}

fn percent_regex() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r"(\d{1,3})\s*%").expect("valid percent regex"))
}

fn shared_unit_pair_regex() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| {
        regex::Regex::new(
            r"(?i)(\d+(?:[.,]\d+)?)\s*/\s*(\d+(?:[.,]\d+)?)\s*([KMGT]i?B|[KMGT]B)",
        )
        .expect("valid pair regex")
    })
}

fn size_regex() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| {
        regex::Regex::new(r"(?i)(\d+(?:[.,]\d+)?)\s*([KMGT]i?B|B)\b").expect("valid size regex")
    })
}

fn speed_regex() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| {
        regex::Regex::new(r"(?i)(\d+(?:[.,]\d+)?)\s*([KMGT]i?B|B)\s*/\s*s")
            .expect("valid speed regex")
    })
}

fn eta_seconds_regex() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| {
        regex::Regex::new(r"(?:(\d{1,2})h)?(?:(\d{1,3})m)?(\d{1,3})s\b").expect("valid eta regex")
    })
}

fn eta_clock_regex() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| {
        regex::Regex::new(r"(?:(\d{1,2}):)?(\d{1,2}):(\d{2})\b").expect("valid clock regex")
    })
}

pub fn parse_percent(text: &str) -> Option<u32> {
    percent_regex()
        .captures(text)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse::<u32>().ok())
        .map(|value| value.min(100))
}

fn unit_multiplier(unit: &str) -> f64 {
    match unit.to_ascii_uppercase().as_str() {
        "B" => 1.0,
        "KB" | "KIB" => 1024.0,
        "MB" | "MIB" => 1024.0 * 1024.0,
        "GB" | "GIB" => 1024.0 * 1024.0 * 1024.0,
        "TB" | "TIB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => 1.0,
    }
}

fn number(raw: &str) -> Option<f64> {
    raw.replace(',', ".").parse::<f64>().ok()
}

/// Every `12.3 MB`-shaped token, in bytes.
pub fn parse_sizes(text: &str) -> Vec<u64> {
    size_regex()
        .captures_iter(text)
        .filter_map(|caps| {
            let value = number(caps.get(1)?.as_str())?;
            let unit = caps.get(2)?.as_str();
            Some((value * unit_multiplier(unit)).round() as u64)
        })
        .collect()
}

/// `45.6 / 120.0 MB` — the denominator's unit applies to both numbers.
pub fn parse_progress_pair(text: &str) -> Option<(u64, u64)> {
    if let Some(caps) = shared_unit_pair_regex().captures(text) {
        let done = number(caps.get(1)?.as_str())?;
        let total = number(caps.get(2)?.as_str())?;
        let multiplier = unit_multiplier(caps.get(3)?.as_str());
        if total > 0.0 {
            return Some((
                (done * multiplier).round() as u64,
                (total * multiplier).round() as u64,
            ));
        }
    }
    let sizes = parse_sizes(text);
    if sizes.len() >= 2 && sizes[1] > sizes[0] {
        return Some((sizes[0], sizes[1]));
    }
    None
}

pub fn parse_speed(text: &str) -> Option<u64> {
    let caps = speed_regex().captures(text)?;
    let value = number(caps.get(1)?.as_str())?;
    let unit = caps.get(2)?.as_str();
    Some((value * unit_multiplier(unit)).round() as u64)
}

/// Fill level of a bar such as `[====>    ]`, `[####    ]` or `[    ]`.
pub fn parse_bar_ratio(text: &str) -> Option<f64> {
    let open = text.find('[')?;
    let rest = &text[open + 1..];
    let close = rest.find(']')?;
    let inner: Vec<char> = rest[..close].chars().collect();
    if inner.len() < 4 {
        return None;
    }
    let drawable = |c: &char| matches!(c, '=' | '#' | '■' | '█' | '▬' | '>' | '-' | ' ');
    if !inner.iter().all(drawable) {
        return None;
    }
    let filled = inner
        .iter()
        .take_while(|c| matches!(c, '=' | '#' | '■' | '█' | '▬'))
        .count();
    if filled == 0 && !inner.contains(&'>') {
        // `apk`/`zypper` print a dot-progress instead of a bar.
        return None;
    }
    let position = inner
        .iter()
        .position(|c| *c == '>')
        .unwrap_or(filled);
    Some((position as f64 / inner.len() as f64).clamp(0.0, 1.0))
}

/// Remaining time in seconds. The last time-looking token of a frame wins:
/// `clx` prints `1m2s`, `tqdm` prints `[elapsed<remaining, speed]`.
pub fn parse_eta(text: &str) -> Option<u64> {
    let mut best: Option<u64> = None;
    for caps in eta_seconds_regex().captures_iter(text) {
        let hours = caps.get(1).and_then(|m| m.as_str().parse::<u64>().ok());
        let minutes = caps.get(2).and_then(|m| m.as_str().parse::<u64>().ok());
        let seconds = caps
            .get(3)
            .and_then(|m| m.as_str().parse::<u64>().ok())
            .unwrap_or(0);
        best = Some(hours.unwrap_or(0) * 3600 + minutes.unwrap_or(0) * 60 + seconds);
    }
    for caps in eta_clock_regex().captures_iter(text) {
        let first = caps.get(1).map(|m| m.as_str());
        let second = caps
            .get(2)
            .and_then(|m| m.as_str().parse::<u64>().ok())
            .unwrap_or(0);
        let third = caps
            .get(3)
            .and_then(|m| m.as_str().parse::<u64>().ok())
            .unwrap_or(0);
        best = Some(match first {
            Some(_) => {
                let hours = first.and_then(|m| m.parse::<u64>().ok()).unwrap_or(0);
                hours * 3600 + second * 60 + third
            }
            None => second * 60 + third,
        });
    }
    best
}

/// The frame with digits, units and bar art collapsed, so two frames of the
/// same download map onto the same label.
fn frame_label(text: &str) -> String {
    let mut label = String::with_capacity(text.len());
    let mut in_number = false;
    let mut pending_space = false;
    for c in text.chars() {
        if c.is_ascii_digit() {
            if !in_number {
                if pending_space && !label.is_empty() {
                    label.push(' ');
                }
                pending_space = false;
                label.push('#');
                in_number = true;
            }
            continue;
        }
        in_number = false;
        let keep = c.is_alphabetic() || c == ':' || c == '/' || c == '.' || c == '@' || c == '_';
        if keep {
            if pending_space && !label.is_empty() {
                label.push(' ');
            }
            pending_space = false;
            label.push(c.to_ascii_lowercase());
        } else {
            pending_space = true;
        }
    }
    label
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_csi_and_osc_sequences() {
        let frame = "\x1b[?2026h\x1b[?25lfd@10.2.0  download 204.9 KB\x1b[?2026l\x1b[1A\x1b[80D\x1b[0J";
        assert_eq!(strip_ansi(frame), "fd@10.2.0  download 204.9 KB");
        assert_eq!(strip_ansi("\x1b]0;title\x07plain"), "plain");
        assert_eq!(strip_ansi("done ✓\r"), "done ✓");
    }

    #[test]
    fn parses_clx_download_frames() {
        let bar = parse_bar_ratio("download fd-v10.2.0-aarch64-… 508.9 KB 0s [=======>          ]");
        let ratio = bar.expect("bar");
        // 7 of 18 cells filled: the `>` sits at index 7.
        assert!((ratio - 7.0 / 18.0).abs() < 0.01, "ratio was {ratio}");
        assert_eq!(parse_sizes("… 508.9 KB 0s"), vec![(508.9 * 1024.0_f64).round() as u64]);
        assert_eq!(parse_eta("… 60.9 KB 1m2s [>                 ]"), Some(62));
        assert_eq!(parse_eta("… 508.9 KB 0s [=======>          ]"), Some(0));
    }

    #[test]
    fn parses_tqdm_frames() {
        let frame = "45%|████▌     | 45.6/120.0 MB [00:12<00:20, 3.2MB/s]";
        assert_eq!(parse_percent(frame), Some(45));
        let (done, total) = parse_progress_pair(frame).expect("pair");
        assert_eq!(done, (45.6 * 1024.0 * 1024.0_f64).round() as u64);
        assert_eq!(total, (120.0 * 1024.0 * 1024.0_f64) as u64);
        assert_eq!(parse_speed(frame), Some((3.2 * 1024.0 * 1024.0) as u64));
        // The remaining side of `<` wins over the elapsed side.
        assert_eq!(parse_eta(frame), Some(20));
    }

    #[test]
    fn parses_apt_style_percent_lines() {
        assert_eq!(parse_percent("Get:3 http://deb.debian.org/debian 45% [Connecting]"), Some(45));
        assert_eq!(parse_percent("no numbers here"), None);
        assert_eq!(parse_bar_ratio("[####      ]"), Some(0.4));
        assert_eq!(parse_bar_ratio("[          ]"), None);
    }

    #[test]
    fn tracker_reports_real_bytes_and_total() {
        let mut tracker = ProgressTracker::untrottled();
        tracker.observe("mise jq@1.7.1  [1/2] download jq-macos-arm64");
        let first = tracker.observe("jq@1.7.1  download jq-macos-arm64  200.0 KB 1m [>                ]");
        let first = first.progress.expect("first progress");
        assert_eq!(first.stage, "download");
        assert_eq!(first.downloaded, Some(200 * 1024));
        let second = tracker
            .observe("jq@1.7.1  download jq-macos-arm64  800.0 KB 1s [========>        ]")
            .progress
            .expect("second progress");
        // 800 KB at 8/18 of the bar implies ~1.7 MB total.
        let total = second.total.expect("total estimated from the bar");
        assert!(total > 1_600_000 && total < 1_900_000, "total was {total}");
        assert!(second.percent > first.percent);
        assert!(second.percent < 86);
    }

    #[test]
    fn parses_real_mise_clx_frames() {
        // Captured from `MISE_FORCE_PROGRESS=1 MISE_TERM_WIDTH=100 mise install
        // node@20` with stdout piped: one header frame, then download frames.
        let header = "\x1b[?2026h\x1b[?25lnode@20.20.2                                                                   \u{e000}\n\x1b[?2026l";
        assert_eq!(strip_ansi(header).trim(), "node@20.20.2");

        let frame = "\x1b[?2026h\x1b[1A\x1b[80D\r\x1b[0J\x1b[?25lnode@20.20.2    download node-v20.20.2-darwin\u{e000} 22.8 MB 2s [=========>        ] \u{e000}\n\x1b[?2026l";
        let frame = strip_ansi(frame);
        assert!(!frame.contains('\u{e000}'), "private-use glyph survived: {frame:?}");
        assert_eq!(parse_eta(&frame), Some(2));
        let (done, _) = (parse_sizes(&frame)[0], 0);
        assert_eq!(done, (22.8 * 1024.0 * 1024.0_f64).round() as u64);

        let mut tracker = ProgressTracker::untrottled();
        tracker.observe(&frame);
        let last = tracker
            .observe("node@20.20.2    download node-v20.20.2-darwin\u{e000} 38.1 MB 1s [==============>   ]")
            .progress
            .expect("progress");
        assert_eq!(last.stage, "download");
        assert!(last.downloaded.unwrap() > 39_000_000);
        // Bar fill implies a total, so the UI can show "38.1 MB / 48.3 MB".
        let total = last.total.expect("total estimated from the bar");
        assert!(total > 40_000_000 && total < 60_000_000, "total was {total}");
        assert!(last.percent > 20 && last.percent < 86);
        // Byte samples need real elapsed time, so speed only appears once
        // frames arrive ≥400 ms apart — the ETA from `clx` covers the gap.
        assert_eq!(last.eta_secs, Some(1));
    }

    #[test]
    fn spinner_frames_collapse_into_one_log_line() {
        // Real `mise install node@18` frames: only the spinner glyph changes.
        let mut tracker = ProgressTracker::untrottled();
        let mut logs = 0;
        for spinner in ["◜", "◠", "◝", "◞", "◡", "◟", "⠋", "⠙"] {
            let frame = format!("node@18.20.8    extract node-v18.20.8-darwin-arm64.tar.gz                     {spinner}");
            if tracker.observe(&frame).log.is_some() {
                logs += 1;
            }
        }
        assert_eq!(logs, 1, "spinner-only changes must not repeat the line");
    }

    #[test]
    fn zero_eta_is_not_published_as_a_countdown() {
        let mut tracker = ProgressTracker::untrottled();
        // `clx` prints `0s` and an empty bar for the first few frames.
        let early = tracker
            .observe("node@18.20.8    download node-v18.20.8-darwin-arm64.tar.gz 64.0 KB 0s [                  ]")
            .progress
            .expect("progress");
        assert_eq!(early.eta_secs, None);

        let measured = tracker
            .observe("node@18.20.8    download node-v18.20.8-darwin-arm64.tar.gz 22.8 MB 12s [=========>        ]")
            .progress
            .expect("progress");
        assert_eq!(measured.eta_secs, Some(12));

        // Once the bytes are in, the countdown disappears instead of freezing.
        let extracting = tracker
            .observe("node@18.20.8    extract node-v18.20.8-darwin-arm64.tar.gz")
            .progress
            .expect("progress");
        assert_eq!(extracting.stage, "extract");
        assert_eq!(extracting.eta_secs, None);
    }

    #[test]
    fn tracker_never_moves_backwards_and_finishes_at_100() {
        let mut tracker = ProgressTracker::untrottled();
        for frame in [
            "mise jq@1.7.1  [1/2] download jq",
            "jq 500.0 KB 2s [=====>           ]",
            "jq 900.0 KB 1s [=========>       ]",
            "mise jq@1.7.1  [2/2] extract jq.tar.gz",
            "mise jq@1.7.1  [2/2] generate checksum",
            "mise jq@1.7.1  ✓ installed",
        ] {
            let _ = tracker.observe(frame);
        }
        assert_eq!(tracker.percent, 94);
        assert_eq!(tracker.stage, Stage::Configure);
        let finished = tracker.finish();
        assert_eq!(finished.percent, 100);
        assert_eq!(finished.stage, "done");
        assert_eq!(tracker.stage, Stage::Done);
    }

    #[test]
    fn repeated_frames_collapse_into_one_log_line() {
        let mut tracker = ProgressTracker::untrottled();
        let first = tracker.observe("download jq.tar.gz 100.0 KB 2s [=>        ]");
        assert!(first.log.is_some());
        let second = tracker.observe("download jq.tar.gz 200.0 KB 1s [==>       ]");
        assert!(second.log.is_none(), "same frame should not be logged again");
        assert!(second.progress.is_some());
        let plain = tracker.observe("pouring bottle into /opt/homebrew");
        assert_eq!(plain.log.as_deref(), Some("pouring bottle into /opt/homebrew"));
    }
}
