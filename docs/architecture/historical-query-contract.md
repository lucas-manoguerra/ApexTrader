# Historical market-data query contract

## Status

Architecture proposal, version 0.1. This document defines responsibility boundaries and behavioral requirements for historical queries. It does not introduce provider-specific assumptions or commit ApexTrader to a pagination protocol, timestamp precision, or range-bound convention that has not been validated against target providers.

The current `MarketDataProvider::fetch_candles` API remains in place until contract decisions and a migration plan are approved. This document is not evidence that the current implementation already satisfies the target contract.

## Scope

This contract covers historical candle retrieval, provider-boundary normalization, application validation and publication. It does not define realtime reconciliation, financial numeric representation, instrument catalog ownership, chart rendering or persistence.

## Responsibility boundaries

### Provider adapter

A provider-specific adapter owns transport, authentication, rate limits, retries, provider-native pagination, translation of native request/response formats, preservation of source timestamps and precision, and mapping provider errors into stable categories. It reports completeness or truncation whenever the provider exposes sufficient evidence.

It must not invent missing candles, label an incomplete response complete, or hide conflicting records by choosing an arbitrary winner.

### Historical normalization

Normalization converts provider-native records into the internal historical contract. It is the only layer that interprets provider-specific timestamp units, boundary conventions, symbol aliases, pagination tokens and duplicate semantics.

It validates timestamps without lossy conversion, maps provider identity to an explicit internal series identity, applies documented native boundary semantics, produces deterministic order, distinguishes exact duplicates from conflicts, and preserves provenance and quality information.

It must not infer market calendars, synthesize gaps, or decide correction authority without an explicit domain policy.

### Application use case

The use case owns orchestration and the acceptance policy for publishing a normalized result. It validates the request, verifies series identity, defensively checks normalized ordering and uniqueness, preserves quality/rejections, and publishes only after the complete batch has been evaluated.

It must not implement provider pagination or reinterpret provider-native timestamp semantics.

### Chart projection

Chart projection is a one-way presentation conversion. It is not authoritative for query identity, provenance, quality, precision or completeness.

## Query identity and time semantics

A historical query must eventually identify the requested series, not merely a display symbol. Identity must distinguish instrument, venue/market, timeframe or aggregation, provider mapping and price basis whenever those dimensions affect the series.

The current `Symbol + from_ms + to_ms` signature cannot express all of that identity. Do not expand it with guessed defaults. First establish the instrument/series model and intended provider scope.

The internal contract must explicitly define:

- The timestamp used to select records, currently observed as candle open time.
- Epoch, unit and precision, including whether sub-millisecond source timestamps exist.
- Start/end bound semantics and inclusivity.
- Zero-width range behavior.
- Treatment of source timestamps finer than internal precision.
- Whether bounds select by candle open time or by overlap with its market interval.

The current application filters candle open timestamps with inclusive bounds `[from_ms, to_ms]`. This is observed legacy behavior, not a final provider-independent decision. The mock also selects aligned open timestamps inclusively. Neither establishes the behavior of an external provider.

Do not change range semantics until call sites, pagination and intended provider contracts have been reviewed together. Query selection bounds are distinct from the duration and closure semantics of a candle.

## Ordering and logical uniqueness

The normalized result must be strictly ordered by its logical candle key. For the current single-symbol path, open timestamp is the only available ordering field; it is not sufficient as a global identity when multiple series are supported.

For each logical key:

- One record is accepted.
- An exact repeat may be idempotent only when equality is defined over authoritative fields and provenance permits it.
- Same key with different authoritative content is a conflict, not an ordinary duplicate.
- Conflicts must be surfaced with source/version context.
- No implicit last-write-wins policy is permitted.
- Sorting is allowed only in normalization after identity and conflict handling are defined. Sorting alone does not resolve conflicting duplicates.

The current use case rejects duplicates and descending timestamps while iterating provider output. It cannot distinguish harmless retransmission from conflicting data and reports both with one reason. This is a defensive check, not a normalization strategy.

A second, later boundary currently applies a different policy: `ChartState::add_data_points` sorts points and retains the last point for a repeated open timestamp, while `ChartState::add_data_point` replaces an existing point at that timestamp. The current presentation history path first uses `LoadCandles` and then passes its accepted points to `ChartState`, but these chart APIs remain callable by other ingestion paths. They must not become the authority for provider duplicate resolution. The target flow must resolve identity and conflicts before chart projection and provide the chart with an already validated sequence.

## Pagination and continuity

Pagination belongs to the provider adapter. The adapter must guarantee that concatenating pages for one logical query does not silently omit or duplicate records.

Do not assume the next page starts at `last_timestamp + 1`; this is unsafe unless source precision and provider boundary behavior are guaranteed. Candidate mechanisms include opaque cursors, provider sequence tokens, stable snapshots, or timestamp continuation with documented overlap-and-deduplicate behavior. These are alternatives, not equivalent guarantees.

A paginated result must expose whether a confirmed terminal page was reached. Local limits, timeouts and partial provider responses must be represented as incomplete. Retries must be idempotent and must not silently overwrite conflicts.

## Partial responses, gaps and empty results

The contract must distinguish:

- Complete response with records.
- Complete response with no records.
- Partial response with records.
- Partial response with no records.
- Response containing rejected records.
- Response with unresolved conflicts.
- Provider or transport failure.

An empty successful response is not automatically an error. It only means that the provider returned no records under the semantics it applied. A missing timestamp is not automatically a data gap: gap detection requires timeframe and applicable market calendar. No layer may synthesize a candle to fill a gap.

Partial data must remain visibly partial. A successful `Vec` alone cannot prove completeness.

## Candle lifecycle and source authority

Historical retrieval may include provisional/open candles, closed candles or corrections. The adapter preserves lifecycle and source-version metadata when available. Closure must not be inferred solely from wall-clock time or nominal interval. If the provider exposes no lifecycle evidence, represent uncertainty.

Conflicting historical and realtime values require a separate reconciliation policy. This contract does not choose a winner.

## Error and quality model

The eventual structured result must expose requested identity/range, effective provider query/range, provider provenance, completion state and pagination termination evidence, accepted/rejected/duplicate/conflict counts, record-level reasons, precision conversions and boundary transformations, and whether lifecycle information is known or unknown.

A single error string or successful `Vec` is insufficient. Introduce Rust types only when ownership and required fields are stable.

## Acceptance and publication

The application defines an explicit acceptance policy before publishing. Partial data may be permitted for chart inspection, but must never be labelled complete or silently become eligible for financial decisions.

Validation must finish before published state mutates. If partial acceptance is allowed, records and quality metadata must be committed together. Provider errors must never trigger implicit synthetic fallback. Simulation remains a separately selected and clearly identified mode.

## Required test matrix

Adapter and normalization tests must cover:

- Exact lower/upper bounds and records immediately outside them.
- Zero-width/inverted ranges.
- Source timestamps finer than internal precision, epoch/unit conversion and overflow.
- Ascending, descending and mixed page order.
- Identical and conflicting duplicates within/across pages.
- Empty first/terminal pages and confirmed end-of-data.
- Page retries, repeated continuation tokens and idempotency.
- Pagination truncation, timeout and rate-limit interruption.
- Partial responses with and without records.
- Provider records outside the effective requested range.
- Open/closed lifecycle metadata where available.
- Calendar-expected versus unexplained gaps.
- Deterministic output for identical input and policy.
- No synthetic records on failures or gaps.

Application tests must cover request validation before provider invocation, identity mismatch, defensive ordering/uniqueness checks, quality preservation, rejected-batch non-publication, atomic publication of records and quality, empty success versus provider failure, and absence of implicit mock fallback.

Property tests must assert invariants over records and page boundaries, not merely verify that a mock agrees with its own generator.

## Decisions intentionally left open

These decisions require concrete provider/market evidence:

1. Canonical query-bound convention and whether to preserve current inclusive behavior.
2. Canonical timestamp precision and whether milliseconds suffice.
3. Complete series identity and instrument catalog.
4. Pagination mechanism and snapshot/consistency guarantees.
5. Exact duplicate equality and source-authority rules.
6. Conditions under which partial historical data may be published.
7. Lifecycle and correction semantics.

Resolve these individually, record stable public-contract decisions in ADRs, and validate against provider fixtures before implementation.

## Implementation sequence

1. Inventory every historical-query call site and subsequent state mutation.
2. Select initial provider/market scope and collect authoritative API documentation and fixtures.
3. Resolve query identity, timestamp precision and range semantics.
4. Define normalized request, response-quality and provenance types at the correct boundary.
5. Implement one adapter normalization path with deterministic pagination and conflict handling.
6. Make the use case consume the normalized contract and publish records with quality atomically.
7. Add the test matrix before enabling additional providers.
8. Benchmark large backfills and memory behavior after correctness is established.

## Exit criteria

The boundary is production-ready only when request identity and time semantics are explicit, pagination completion is provable, ordering and duplicate/conflict behavior are deterministic, partial quality is preserved, and failures cannot produce synthetic or falsely complete data.
