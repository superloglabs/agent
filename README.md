# superlog-agent

On-host OpenTelemetry agent for instrumenting macOS AppleScript automation. Installed per-machine; reads handler-level events appended by instrumented scripts and ships them to any OTel-compatible ingest endpoint.

## Components

- `superlog-log/` — small Rust binary that instrumented AppleScript handlers call via `do shell script`. Appends NDJSON events to `~/Library/Logs/Superlog/scripts.ndjson`, enriching each line with a nanosecond timestamp, hostname, and a cached `IOPlatformUUID`.
- `collector/` — OpenTelemetry Collector configuration (filelog receiver → OTLP/HTTP exporter). *(coming soon)*
- `launchd/` — LaunchAgent plist template so the Collector runs as a per-user daemon. *(coming soon)*

## Install

```
brew install superlog/tap/superlog-agent
superlog agent install
```

The `superlog agent install` command ships with the [Superlog CLI](https://superlog.sh) and handles token provisioning + `launchctl bootstrap`. You can also install manually — see `launchd/README.md` once it lands.

## How it's used

An AppleScript handler instrumented by the Superlog install wizard looks like:

```applescript
on sync_inbox()
  set _slRun to do shell script "uuidgen"
  try
    do shell script "/opt/homebrew/bin/superlog-log event " & quoted form of ¬
      ("{\"e\":\"start\",\"s\":\"sync_inbox\",\"r\":\"" & _slRun & "\"}")
  end try
  try
    -- original body
    try
      do shell script "/opt/homebrew/bin/superlog-log event " & quoted form of ¬
        ("{\"e\":\"end\",\"s\":\"sync_inbox\",\"r\":\"" & _slRun & "\",\"status\":\"ok\"}")
    end try
  on error errMsg number errNum
    try
      do shell script "/opt/homebrew/bin/superlog-log event " & quoted form of ¬
        ("{\"e\":\"end\",\"s\":\"sync_inbox\",\"r\":\"" & _slRun & "\",\"status\":\"error\",\"code\":" & errNum & "}")
    end try
    error errMsg number errNum
  end try
end sync_inbox
```

Every helper call is wrapped in an inner `try` so telemetry failure never breaks the host script, and the outer error handler re-raises so the script's exit behaviour is unchanged.

## Development

```
cd superlog-log
cargo build --release
./target/release/superlog-log event '{"e":"test","s":"smoke"}'
cat ~/Library/Logs/Superlog/scripts.ndjson
```

## License

Apache-2.0 — see [LICENSE](./LICENSE).
