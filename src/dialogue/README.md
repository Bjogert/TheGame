# Dialogue Module

The dialogue module exposes the broker abstraction, queued request runner, and plugin wiring for NPC conversations.

- `DialogueBroker` trait + provider enum wrap the active backend. `OpenAiDialogueBroker` now calls the real OpenAI Chat Completions API when `OPENAI_API_KEY` is present, automatically falling back to the legacy stub when the key is missing so tests keep working offline. The broker reports its live/fallback state through `DialogueBrokerStatus`, so UI layers can surface the active mode.
- `DialogueRequestQueue` tracks pending requests, global/per-NPC cooldowns, and retry backoff. Systems emit `DialogueResponseEvent` and `DialogueRequestFailedEvent` so UI/telemetry layers can react.
- `DialogueTelemetry` retains the latest responses/failures in a ring buffer for UI surfaces that want to show recent NPC chatter without re-subscribing to events, and `DialogueTelemetryLog` mirrors that data to `logs/dialogue_history.jsonl` as JSON lines for offline tooling. The log now includes broker status snapshots so you can confirm whether the OpenAI path is live or using fallback responses.
- `DialogueContext` carries structured events (trades, schedule updates, etc.) to keep LLM prompts grounded in live simulation data.
- `DialoguePlugin` registers the queue, rate-limit resources, telemetry collector, and logs the active provider on startup. Override the `ActiveDialogueBroker` resource if another provider is desired. Press `F7` in-game to enqueue a “dialogue probe” request that exercises the broker and writes obvious success/failure entries to the telemetry log.

The module intentionally keeps cooldown values conservative; tune them once real APIs clarify their throttling requirements.

## Module Layout
- `broker/mod.rs` exposes the `DialogueBroker` trait, provider enum, and helper types for queue integration.
- `broker/config.rs` parses environment variables and holds the shared OpenAI defaults (`DEFAULT_MODEL`, `DEFAULT_TIMEOUT_SECS`, etc.).
- `broker/openai.rs` implements the primary provider, relying on config defaults while falling back to local fabrication when credentials are absent.
- Constants for prompts, retry timing, and trade context strings are grouped at the top of `broker/openai.rs` to avoid scatter across call sites.

## Configuration
- Set `OPENAI_API_KEY` (and optionally `OPENAI_MODEL`, `OPENAI_BASE_URL`, `OPENAI_TEMPERATURE`, `OPENAI_MAX_OUTPUT_TOKENS`, `OPENAI_TIMEOUT_SECS`) via environment variables. During development the game automatically loads `secrets.env` from the repository root if it exists (the file is already git-ignored), so you can keep credentials local without exporting them manually. Dialogue telemetry persists to `logs/dialogue_history.jsonl`; delete the file if you want to reset history between runs.
- Without an API key the broker returns fallback responses so the simulation continues to run during offline work or test execution. The startup log and telemetry history will call this out explicitly so you know real OpenAI traffic is not flowing.
