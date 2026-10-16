use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Files(PathBuf);
impl Files {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "adventure-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self(directory)
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

// An unusable SDL driver makes accidental device creation fail in headless workflows.
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_adventure"))
        .env("SDL_VIDEODRIVER", "adventure-no-video")
        .env("SDL_AUDIODRIVER", "adventure-no-audio")
        .args(args)
        .output()
        .unwrap()
}
fn succeeds(output: Output) -> Output {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[test]
fn headless_trace_record_verify_and_live_validation_need_no_devices() {
    let files = Files::new();
    let trace = files.0.join("session.trace");
    let trace = trace.to_str().unwrap();
    succeeds(run(&["--headless", "--frames", "7", "--trace", trace]));
    let output = succeeds(run(&["--verify", trace]));
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Verified 7 frames against main and ROM")
    );
    succeeds(run(&["--validate", "--headless", "--frames", "7"]));

    // Explicit display overrides verification's headless default before execution.
    assert!(!run(&["--verify", trace, "--display"]).status.success());
    let text = fs::read_to_string(trace).unwrap();
    fs::write(trace, text.replace("init 1", "init 2")).unwrap();
    let output = run(&["--verify", trace]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("divergence"));
}

#[test]
fn input_recording_replays_headlessly_and_defaults_to_display() {
    let files = Files::new();
    let input = files.0.join("session.input");
    let input = input.to_str().unwrap();
    succeeds(run(&["--headless", "--frames", "7", "--record", input]));
    succeeds(run(&["--replay", input, "--headless"]));
    assert!(!run(&["--replay", input]).status.success());
    assert!(!run(&["--frames", "1"]).status.success());
}

#[test]
fn invalid_mode_combinations_fail_before_opening_files() {
    for args in [
        vec!["--replay", "missing", "--validate"],
        vec!["--verify", "missing", "--record", "unused"],
        vec!["--verify", "missing", "--replay", "missing"],
        vec!["--headless", "--display"],
        vec!["--headless"],
        vec!["--frames", "0"],
        vec!["--headless", "--snapshot", "unused"],
        vec!["--record", "same", "--trace", "same"],
    ] {
        let output = run(&args);
        assert!(!output.status.success(), "{args:?}");
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(!error.contains("No such file"), "{error}");
        assert!(!error.contains("panicked"), "{error}");
    }
}
