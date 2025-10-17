# Dialogue Module

The dialogue module exposes the broker abstraction, queued request runner, and plugin wiring for NPC conversations.

- `DialogueBroker` trait + provider enum wrap the active backend. `OpenAiDialogueBroker` now calls the real OpenAI Chat Completions API when `OPENAI_API_KEY` is present, automatically falling back to the legacy stub when the key is missing so tests keep working offline.
- `DialogueRequestQueue` tracks pending requests, global/per-NPC cooldowns, and retry backoff. Systems emit `DialogueResponseEvent` and `DialogueRequestFailedEvent` so UI/telemetry layers can react.
- `DialogueTelemetry` retains the latest responses/failures in a ring buffer for UI surfaces that want to show recent NPC chatter without re-subscribing to events, and `DialogueTelemetryLog` mirrors that data to `logs/dialogue_history.jsonl` as JSON lines for offline tooling.
- `DialogueContext` carries structured events (trades, schedule updates, etc.) to keep LLM prompts grounded in live simulation data.
- `DialoguePlugin` registers the queue, rate-limit resources, telemetry collector, and logs the active provider on startup. Override the `ActiveDialogueBroker` resource if another provider is desired.

The module intentionally keeps cooldown values conservative; tune them once real APIs clarify their throttling requirements.

## Configuration
- Set `OPENAI_API_KEY` (and optionally `OPENAI_MODEL`, `OPENAI_BASE_URL`, `OPENAI_TEMPERATURE`, `OPENAI_MAX_OUTPUT_TOKENS`, `OPENAI_TIMEOUT_SECS`) via environment variables. A sample `secrets.env` file lives at the repository root for local development. Dialogue telemetry persists to `logs/dialogue_history.jsonl`; delete the file if you want to reset history between runs.
- Without an API key the broker returns fallback responses so the simulation continues to run during offline work or test execution.
