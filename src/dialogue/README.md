# Dialogue Module

The dialogue module owns the abstractions required to coordinate LLM-backed conversations. For S1.3 the focus is on plumbing and
rate-limiting rather than calling real APIs.

## Key Components
- **DialogueProvider** – enum listing supported backends (`OpenAi`, `Anthropic`, `Local`).
- **DialogueBroker** – trait that provider adapters implement. The default `LocalEchoBroker` immediately acknowledges requests
  while logging dispatch details.
- **DialogueBrokerRegistry** – Bevy resource responsible for storing the active brokers and routing requests.
- **DialogueRequestQueue** – Resource that stores pending requests, enforces global/per-NPC cooldowns, and tracks retry metrics.
- **run_dialogue_queue** – System executed each frame to tick cooldowns, pull ready requests, and hand them off to the
  appropriate broker.

## Rate Limiting
- Global cooldown defaults to 1 request/sec across the whole simulation.
- Each NPC has an independent cooldown (30 seconds by default) to prevent spamming the same agent.
- Requests exceeding provider capacity are rescheduled with backoff; after the retry budget is exhausted the request is dropped
  and logged.

## Next Steps
- Replace `LocalEchoBroker` with real HTTP clients for OpenAI/Anthropic and wire authentication/config loading.
- Surface queue depth/metrics in debug UI.
- Persist dialogue transcripts and integrate with the upcoming UI chat panel.
