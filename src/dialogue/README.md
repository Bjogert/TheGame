# Dialogue Module

The dialogue module exposes the broker abstraction, queued request runner, and plugin wiring for NPC conversations.

- `DialogueBroker` trait + provider enum wrap the active backend. The default `OpenAiDialogueBroker` currently fabricates placeholder responses while exercising the queue and context plumbing.
- `DialogueRequestQueue` tracks pending requests, global/per-NPC cooldowns, and retry backoff. Systems emit `DialogueResponseEvent` and `DialogueRequestFailedEvent` so UI/telemetry layers can react.
- `DialogueContext` carries structured events (trades, schedule updates, etc.) to keep LLM prompts grounded in live simulation data.
- `DialoguePlugin` registers the queue, events, rate-limit resources, and logs the active provider on startup. Swap the broker resource when a real provider is ready.

The module intentionally keeps cooldown values conservative; tune them once real APIs clarify their throttling requirements.
