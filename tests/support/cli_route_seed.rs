//! Seed a stored route credential with the same CLI binary that later reads it.
//! macOS Keychain items trust only the creating application, so an item written by the
//! test process would make the `coda` child wait for an interactive access prompt.
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn seed_route(data_dir: &Path, route_id: &str, base_url: &str, model: &str, secret: &str) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_coda"))
        .args([
            "--json",
            "init",
            "--provider",
            "openai",
            "--route",
            route_id,
        ])
        .args(["--base-url", base_url, "--model", model])
        .args([
            "--max-tokens",
            "4096",
            "--parallel-limit",
            "1",
            "--api-key-stdin",
        ])
        .arg("--data-dir")
        .arg(data_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn coda init");
    child
        .stdin
        .take()
        .expect("coda init stdin")
        .write_all(format!("{secret}\n").as_bytes())
        .expect("failed to send route secret");
    let out = child
        .wait_with_output()
        .expect("failed to wait on coda init");
    assert!(
        out.status.success(),
        "route seed failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
