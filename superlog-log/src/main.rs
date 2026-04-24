use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, ExitCode};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value};

const USAGE: &str = "usage: superlog-log event '<json>'";

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let subcmd = match args.next() {
        Some(s) => s,
        None => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    if subcmd != "event" {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    }
    let payload = match args.next() {
        Some(s) => s,
        None => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };

    // Parse and enrich. A malformed payload is a genuine caller bug (the
    // codemod produces these) so we surface it as a nonzero exit — but the
    // calling AppleScript wraps the whole invocation in `try`, so the host
    // script is unaffected either way.
    let mut obj: Map<String, Value> = match serde_json::from_str::<Value>(&payload) {
        Ok(Value::Object(m)) => m,
        Ok(_) => {
            eprintln!("superlog-log: payload must be a JSON object");
            return ExitCode::from(2);
        }
        Err(e) => {
            eprintln!("superlog-log: invalid JSON: {e}");
            return ExitCode::from(2);
        }
    };

    let ts_ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    obj.entry("ts".to_string())
        .or_insert_with(|| Value::from(ts_ns));
    obj.entry("host".to_string())
        .or_insert_with(|| Value::from(hostname()));
    obj.entry("machine_id".to_string())
        .or_insert_with(|| Value::from(machine_id()));

    let mut line = serde_json::to_string(&Value::Object(obj)).unwrap_or_else(|_| payload.clone());
    line.push('\n');

    // Append is best-effort. If the log dir can't be created or the file can't
    // be opened, we print to stderr and exit 0 — the host script wraps us in
    // `try` and we'd rather drop a telemetry line than surface a wrapped error
    // that someone might eventually un-wrap.
    match append(&line) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("superlog-log: append failed: {e}");
            ExitCode::SUCCESS
        }
    }
}

fn append(line: &str) -> std::io::Result<()> {
    let path = log_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = OpenOptions::new().create(true).append(true).open(&path)?;
    f.write_all(line.as_bytes())?;
    Ok(())
}

fn log_path() -> PathBuf {
    let home = env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/tmp"));
    home.join("Library/Logs/Superlog/scripts.ndjson")
}

fn hostname() -> String {
    // `hostname` crate would be cleaner but we're keeping deps minimal; the
    // binary is in the hot path of every instrumented handler.
    Command::new("/bin/hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn machine_id() -> String {
    // IOPlatformUUID — stable per-machine identifier on macOS. Reading it
    // forks `ioreg`, which is slower than we'd like; cache the result in a
    // dotfile so we only pay this once per install.
    let cache = env::var_os("HOME")
        .map(PathBuf::from)
        .map(|h| h.join("Library/Application Support/Superlog/machine_id"));
    if let Some(path) = &cache {
        if let Ok(s) = fs::read_to_string(path) {
            let t = s.trim();
            if !t.is_empty() {
                return t.to_string();
            }
        }
    }
    let id = ioreg_platform_uuid().unwrap_or_else(|| "unknown".to_string());
    if let Some(path) = &cache {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(path, &id);
    }
    id
}

fn ioreg_platform_uuid() -> Option<String> {
    // Line looks like:   |   "IOPlatformUUID" = "ABCDEF01-2345-..."
    // Grab the value inside the last pair of quotes on a line that mentions the key.
    let out = Command::new("/usr/sbin/ioreg")
        .args(["-d2", "-c", "IOPlatformExpertDevice"])
        .output()
        .ok()?;
    let text = String::from_utf8(out.stdout).ok()?;
    for line in text.lines() {
        if !line.contains("IOPlatformUUID") {
            continue;
        }
        let mut parts = line.rsplit('"');
        let _trailing = parts.next()?;
        let value = parts.next()?;
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}
