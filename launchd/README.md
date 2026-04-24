# launchd/

LaunchAgent plist template for running the Collector as a per-user daemon on macOS.

`sh.superlog.agent.plist` is a template. The `superlog agent install` CLI substitutes `@PLACEHOLDER@` tokens and writes the result to `~/Library/LaunchAgents/sh.superlog.agent.plist` with mode 0600.

## Placeholders

| Placeholder | Substituted with |
|---|---|
| `@COLLECTOR_BIN@` | Absolute path to the `otelcol-contrib` binary (typically `$(brew --prefix)/bin/otelcol-contrib`) |
| `@CONFIG_PATH@` | Absolute path to the rendered Collector config (typically `~/Library/Application Support/Superlog/collector.yaml`) |
| `@LOG_PATH@` | NDJSON input file — `$HOME/Library/Logs/Superlog/scripts.ndjson` |
| `@STORAGE_DIR@` | Collector cursor/state directory — `$HOME/Library/Application Support/Superlog/storage` |
| `@INGEST_ENDPOINT@` | e.g. `https://ingest.superlog.sh` |
| `@INGEST_TOKEN@` | User's Superlog ingest token (also why the plist is 0600) |
| `@PROJECT_ID@` | Superlog project id — sent as `x-superlog-project-id` for server-side attribution |
| `@SERVICE_NAME@` | `service.name` resource attribute — typically `<client>-mac-automation` |
| `@STDOUT_PATH@` / `@STDERR_PATH@` | e.g. `$HOME/Library/Logs/Superlog/collector.stdout.log` |

## Install manually

If you're not using `superlog agent install`:

```
# After substituting placeholders into a rendered plist:
launchctl bootstrap gui/$UID ~/Library/LaunchAgents/sh.superlog.agent.plist
launchctl kickstart -k gui/$UID/sh.superlog.agent
```

And to stop / remove:

```
launchctl bootout gui/$UID/sh.superlog.agent
rm ~/Library/LaunchAgents/sh.superlog.agent.plist
```

## Why plist env vars, not a keychain-sourced token

Putting `SUPERLOG_INGEST_TOKEN` directly in the plist keeps the agent to a single process with no wrapper script. The plist is installed 0600 and lives under the user's home, so exposure is limited to someone with shell as that user — who can already read any dotfile or the token file at `~/Library/Application Support/Superlog/token`. Moving to keychain-backed secrets is a future enhancement; contributions welcome.
