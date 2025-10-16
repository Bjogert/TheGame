# Dialogue Scaffolding Research (S1.2)

## Candidate LLM Providers
| Provider | Pros | Cons | Notes |
|----------|------|------|-------|
| OpenAI (GPT-4o mini / Turbo) | High quality, robust tooling, streaming support | Paid API, strict rate limits | Good default; budget needs monitoring |
| Local Models (Llama 3, Mistral) | Full control, no per-token cost | Requires on-prem GPU/quant hosting, ops overhead | Option for later hybrid deployments |

## Proposed API Strategy
- Start with managed providers (OpenAI) for rapid iteration.
- Wrap calls behind a DialogueBroker abstraction so swapping vendors is trivial.
- Store credentials/rate limits in environment config; avoid hardcoding.

## Rate Limiting & Budget
- Global bucket: 60 requests/minute (configurable).
- Per-NPC cooldown: minimum 30 seconds between dialogue turns.
- Daily budget cap (tokens) to prevent runaway costs.
- Backpressure: queue requests; surface UI indicators when NPCs are "thinking".

## Prompt Scaffolding
`
<system>
You are {npc_name}, a villager in {settlement}. Remain grounded in world facts.
Time: {world_time}
Recent activity: {activity}
Known relationships: {relationships}
Goals today: {goals}
</system>
<conversation_history>
{recent_messages}
</conversation_history>
<player_message>{player_input}</player_message>
`
- Requires Identity + Schedule data (already available).
- Future inputs: inventory, mood, relationship graph, location.

## Simulation Data Needed
- identity.display_name (existing)
- schedule_state.current_activity (existing)
- relationships: placeholder for future social graph
- world_time: from WorldClock
- goals/mood: upcoming systems (needs & intent planner)

## Next Steps
1. Prototype DialogueBroker trait + enums for provider selection.
2. Define RUST struct for prompt context.
3. Explore caching/summary strategy to stay inside context limits.
4. Document error handling (timeouts, retries, throttling).
