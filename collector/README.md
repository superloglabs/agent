# collector/

OpenTelemetry Collector configuration for the Superlog on-host agent.

`config.yaml` reads NDJSON events from a local log file (default `~/Library/Logs/Superlog/scripts.ndjson`) and ships them as OTel log records to an OTLP/HTTP endpoint.

## Requirements

The config uses `filelog` (a contrib-only receiver) and the `file_storage` extension, so it requires **otelcol-contrib**, not the core distribution. The Homebrew formula bundles a build of otelcol-contrib; if running manually you can install it separately.

## Environment variables

The plist (`../launchd/sh.superlog.agent.plist`) sets these at launch time. If running manually, export them before invoking the Collector.

| Var | Example |
|---|---|
| `SUPERLOG_LOG_PATH` | `$HOME/Library/Logs/Superlog/scripts.ndjson` |
| `SUPERLOG_STORAGE_DIR` | `$HOME/Library/Application Support/Superlog/storage` |
| `SUPERLOG_INGEST_ENDPOINT` | `https://ingest.superlog.sh` |
| `SUPERLOG_INGEST_TOKEN` | `superlog_live_...` |
| `SUPERLOG_PROJECT_ID` | Superlog project id (sent as `x-superlog-project-id`) |
| `SUPERLOG_SERVICE_NAME` | `<client>-mac-automation` |

## Run locally (without launchd)

```
export SUPERLOG_LOG_PATH=$HOME/Library/Logs/Superlog/scripts.ndjson
export SUPERLOG_STORAGE_DIR=/tmp/sl-storage && mkdir -p $SUPERLOG_STORAGE_DIR
export SUPERLOG_INGEST_ENDPOINT=https://ingest.superlog.sh
export SUPERLOG_INGEST_TOKEN=superlog_live_...
export SUPERLOG_PROJECT_ID=prj_...
export SUPERLOG_SERVICE_NAME=dev-smoke

otelcol-contrib --config=./config.yaml
```

Then invoke `superlog-log event '{"e":"test","s":"smoke"}'` in another shell and watch the Collector pick it up.
