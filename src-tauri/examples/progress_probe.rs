//! Probe: replay a real `mise install` through the progress tracker.
//!
//! ```sh
//! cd src-tauri && cargo run --example progress_probe -- node@18
//! ```
use envhub_lib::commands::progress::ProgressTracker;
use envhub_lib::env_helper;

#[tokio::main]
async fn main() {
    let target = std::env::args().nth(1).unwrap_or_else(|| "node@18".into());
    let mut command = tokio::process::Command::new("mise");
    command
        .args(["install", &target])
        .env("MISE_FORCE_PROGRESS", "1")
        .env("MISE_TERM_WIDTH", "100")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child = command.spawn().expect("spawn mise");
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let tracker = std::sync::Arc::new(std::sync::Mutex::new(ProgressTracker::new()));
    async fn run<R>(pipe: R, tracker: std::sync::Arc<std::sync::Mutex<ProgressTracker>>)
    where
        R: tokio::io::AsyncRead + Unpin,
    {
        env_helper::forward_records(pipe, move |record| {
            let mut tracker = tracker.lock().unwrap();
            let observation = tracker.observe(record);
            if let Some(line) = observation.log {
                println!("LOG   {line}");
            }
            if let Some(progress) = observation.progress {
                println!(
                    "PERCENT {:>3}% stage={} bytes={:?}/{:?} speed={:?} eta={:?}",
                    progress.percent,
                    progress.stage,
                    progress.downloaded,
                    progress.total,
                    progress.speed_bps,
                    progress.eta_secs
                );
            }
        })
        .await;
    }
    let out = tokio::spawn(run(stdout, tracker.clone()));
    let err = tokio::spawn(run(stderr, tracker.clone()));
    let status = child.wait().await.unwrap();
    let _ = tokio::join!(out, err);
    println!("EXIT {status}");
    if status.success() {
        let finished = tracker.lock().unwrap().finish();
        println!("PERCENT {:>3}% stage={}", finished.percent, finished.stage);
    }
}
