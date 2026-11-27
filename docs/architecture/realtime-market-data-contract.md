# Realtime market-data lifecycle contract

## Status

Architecture proposal, version 0.1. This document records the target responsibilities and invariants for realtime market updates. It does not claim that the current mock-driven implementation satisfies them and does not select provider-specific sequence, correction or delivery semantics.

## Scope

This contract covers realtime updates after provider decoding, validation and normalization; candle lifecycle; reconciliation with historical data; and publication into application state.

It does not define transport protocols, provider authentication, historical pagination, financial numeric representation, market calendars, order execution or chart rendering.

## Current behavior observed

- `RealtimeMarketDataProvider::next_update` returns a `MarketCandle` and an `is_closed` boolean.
- The active desktop composition creates mock historical and mock realtime providers directly.
- Realtime updates are converted to `ChartDataPoint` and applied through `ChartState::add_data_point`.
- `is_closed` currently affects tracing only; it does not change application state or prevent later replacement.
- `ChartState::add_data_point` replaces a point with the same open timestamp.
- The realtime mock emits several updates for one open timestamp and marks its final tick closed. Its next update starts the following candle at the previous candle's close timestamp.
- The current chart point does not retain provider identity, update sequence, source version, provenance or lifecycle state.
- Realtime initialization reconstructs a market candle from the final chart point. A partial historical load can therefore seed realtime from the last accepted point without a continuity guarantee.

These are implementation observations, not desired universal provider semantics.

## Responsibility boundaries

### Provider adapter

The adapter owns transport, subscriptions, reconnects, heartbeat handling, provider sequence numbers, provider-native event identifiers and provider-specific lifecycle/correction semantics.

It must preserve enough source metadata to detect duplicates, gaps, stale events, out-of-order delivery and authoritative corrections whenever the provider exposes such metadata. It must not manufacture closure or correction authority from local arrival time.

### Realtime normalization

Normalization maps native events to an internal update with explicit series identity, logical candle key, lifecycle transition, source timestamp, receive timestamp, provider provenance and source version/sequence when available.

It validates identity and candle data before the application sees an update. It classifies exact retransmission, conflicting payload, stale event, out-of-order event, gap and correction without silently choosing a winner.

When provider semantics do not expose enough evidence to classify an event, the normalized result must retain uncertainty rather than invent a sequence or definitive state.

### Application lifecycle policy

The application owns the state machine and acceptance policy for provisional candles, closed candles and authorized corrections. Provider adapters report evidence; they do not decide application publication policy.

The application validates a complete update before mutating published state. It applies lifecycle transitions atomically with quality/provenance metadata and exposes rejected or uncertain updates to observability.

### Chart projection

The chart receives an already accepted projection. It may update visual geometry and viewport state but must not decide source authority, resolve provider conflicts or promote provisional data to closed.

## Series identity and continuity

Every realtime update must identify the same logical series as the historical dataset it extends. Symbol and open timestamp alone are insufficient.

The series identity must eventually include the dimensions that affect the data: canonical instrument, venue/market, timeframe or aggregation definition, price basis and provider/source mapping. Alias resolution belongs to the instrument/provider boundary.

A realtime stream may be attached only when its series identity matches the accepted historical result. A reconstructed candle made from chart display values is not sufficient evidence of continuity.

The application must define whether realtime may start after a partial historical result. Until that policy is explicit, a partial or degraded historical result must not silently authorize a stream that presents the combined series as continuous or complete.

## Candle lifecycle

The target lifecycle distinguishes at least:

- Provisional: mutable candle state while the interval is active.
- Closed: provider/domain evidence indicates that ordinary updates must no longer mutate the candle.
- Corrected: a later authoritative revision explicitly supersedes previously accepted content.
- Unknown: the provider does not provide enough evidence to establish closure.

A provider's boolean `is_closed` may be an input to normalization, but it is not by itself a complete lifecycle protocol. The contract must eventually clarify whether closure is final, whether corrections can follow, how revisions are identified and what happens when providers disagree.

The application must not infer closure solely from local wall-clock time or nominal timeframe. Market sessions, holidays, delayed feeds and provider-specific finalization can make that inference incorrect.

## Update identity, ordering and idempotency

The logical candle key is series identity plus candle period/open time. It is not sufficient to identify individual updates to that candle.

Where available, update identity should use provider event IDs, sequence numbers or revision IDs. If none exist, the adapter must document the weaker guarantees and the normalization layer must avoid fabricating a globally reliable sequence.

Required behavior:

- An exact retransmission is idempotent.
- A stale update cannot regress accepted state.
- An out-of-order update is buffered, rejected or reconciled according to an explicit policy; it is never silently treated as newest.
- A duplicate key with different authoritative content is a conflict unless an explicit revision relationship proves correction.
- A closed candle is immutable to ordinary updates.
- A correction requires identifiable authority/version and an auditable replacement decision.
- A gap is observable and is not filled with synthetic market data.

Arrival order alone is not authority. “Last received wins” is not an acceptable default.

## Historical/realtime reconciliation

Historical and realtime paths must share canonical series identity, candle identity, timestamp precision and provenance rules.

The reconciliation policy must cover:

- Realtime update arriving before historical backfill reaches the same period.
- Historical page containing a candle already observed in realtime.
- Realtime retransmission after a successful historical refresh.
- Historical correction after realtime closure.
- Provider reconnect replaying earlier events.
- Sequence gaps and missing events.
- Conflicting closed values from different source versions.
- Switching provider or venue while retaining an existing chart.

The system must not combine values from different series identities. If reconciliation cannot establish authority, it must surface a conflict and preserve enough evidence for diagnosis rather than silently overwriting one value.

## Failure, disconnect and recovery

A disconnect or heartbeat timeout changes stream health; it does not itself invalidate already accepted candles. The application must expose stream health and last successful event/heartbeat separately from candle lifecycle.

Reconnect behavior must specify replay windows, duplicate handling, sequence resumption and backfill requirements. Reconnection must not imply continuity unless missing events have been reconciled or the affected range is explicitly marked incomplete.

A provider error must not trigger a synthetic fallback. Simulation is a separately selected mode with visible provenance and isolated state.

## Publication and quality

An update is validated before state mutation. The accepted candle, lifecycle state, source version and quality metadata must become visible atomically.

The application must distinguish stream health from dataset quality. A healthy socket does not prove that a series has no gaps; a disconnected socket does not mean that previously accepted closed candles are corrupt.

The chart may display provisional values, but downstream consumers must be able to distinguish provisional, closed, corrected, stale and uncertain data. A visual chart projection alone must not be treated as financial authority.

## Required test matrix

Normalization and application tests must cover:

- Multiple provisional updates to one candle.
- Valid closure transition.
- Ordinary update after closure is rejected.
- Explicit correction with a newer authoritative revision.
- Duplicate identical update and duplicate conflicting payload.
- Out-of-order and stale sequence.
- Sequence gap, reconnect replay and duplicate replay.
- Historical/realtime overlap with identical values.
- Historical/realtime overlap with conflicting values.
- Correction arriving after historical backfill.
- Realtime identity mismatch across instrument, venue, timeframe, price basis or provider.
- Partial historical result and realtime-start policy.
- Disconnect, heartbeat timeout and reconnect.
- Unknown closure semantics.
- No synthetic candle creation on any gap or failure.
- Atomic update of candle, lifecycle, provenance and quality.
- Deterministic results under repeated event sequences.

Property tests should generate event permutations and verify lifecycle invariants, idempotency and the absence of silent regressions. They should exercise the actual normalizer/state machine rather than only the mock generator.

## Decisions intentionally left open

The following require provider and market evidence:

1. Whether provider closure is final or can be revised.
2. Event identity and ordering guarantees per provider.
3. Replay and recovery semantics after disconnect.
4. Correction authority and version precedence.
5. Buffering versus rejection policy for out-of-order events.
6. Whether partial historical data may start realtime and how continuity is represented.
7. How to reconcile multiple providers or venues.
8. Market calendar/session authority and delayed-feed semantics.

Resolve these decisions before adding a production realtime adapter. Keep provider-specific rules inside adapters and stable cross-provider policy in the application/domain boundary.

## Implementation sequence

1. Define canonical series and candle identities with the instrument model.
2. Inventory provider event metadata and lifecycle guarantees for the initial supported provider.
3. Define normalized realtime event and quality semantics.
4. Implement lifecycle transitions and conflict handling independently of the UI.
5. Define historical/realtime reconciliation and reconnect/backfill policy.
6. Publish accepted state and quality atomically.
7. Add the test matrix before enabling a connected realtime mode.
8. Profile event throughput, buffering and memory only after correctness is established.

## Exit criteria

Realtime is production-ready only when series continuity is explicit, lifecycle transitions are enforced, duplicates and corrections are deterministic, reconnect gaps are observable/reconciled, quality and provenance survive publication, and no ordinary update can silently rewrite a closed candle.
