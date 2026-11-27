# Component ownership and responsibility boundaries

## Purpose

This document defines how responsibilities are assigned across ApexTrader. It is an architectural constraint for implementation and review, not a target directory tree to populate mechanically.

The governing rule is: one component owns one cohesive capability and does that capability well. A component may collaborate with other components through explicit contracts, but it must not absorb their policies, state, infrastructure details or lifecycle.

## Responsibility test

A component is cohesive when all of the following are true:

1. Its name describes the capability it owns.
2. Its public API expresses only that capability.
3. Its internal state exists to uphold that capability's invariants.
4. Its dependencies are required to perform that capability, not convenient shortcuts.
5. A change in an unrelated policy does not normally require changing this component.
6. Its behavior can be tested without starting unrelated subsystems.

If one of these conditions fails, first inspect whether the component has multiple reasons to change. Split only along real ownership and invariant boundaries, not line count.

## Ownership matrix

| Component / layer | Owns | Must not own |
|---|---|---|
| Domain | Financial and market concepts, identities, units, invariants and domain policies | UI framework, network clients, persistence, logging setup, rendering, wall-clock access |
| Application use cases | User/business workflows, orchestration of domain operations, consistency policy for a use case | Widget state, drawing, provider SDKs, desktop event loop, concrete mock construction |
| Application ports | Capabilities required by use cases, expressed in application language | Concrete transport, vendor SDK, UI framework, adapter implementation |
| Market-data normalization | Validation, canonicalization and mapping of provider payloads into trusted market contracts | Chart layout, UI formatting, simulation policy, order execution |
| Market-data providers | Communication with a specific provider and translation of its protocol | Application workflow decisions, chart state, financial interpretation beyond provider contract |
| Simulation | Deterministic synthetic data generation and simulation lifecycle | Production fallback, implicit substitution for provider failures |
| Chart interaction | Input-independent interaction events and gesture/viewport policies | egui event polling, OS input, network data access |
| Chart layout | Plot and axis geometry and coordinate regions | Widget rendering, market data ingestion, financial calculations |
| Presentation adapters | UI framework integration, input translation, view state and user-facing composition | Authoritative financial rules, provider protocol, market data validation |
| Rendering scene | Framework-neutral renderable primitives and scene construction | Provider access, business workflows, application state mutation |
| Rendering backends | Mapping scene primitives to a concrete graphics API | Domain policy, data ingestion, application workflow |
| Infrastructure | Cross-cutting technical capabilities such as clock, logging sinks, persistence and transport | Owning business decisions or silently changing domain values |
| Bootstrap / composition root | Selecting concrete adapters and wiring the application graph | Business policy, data transformations, UI interaction logic |

## Dependency direction

- Domain is the innermost policy layer.
- Application use cases depend on domain and application-owned ports.
- Adapters implement ports and translate external representations at boundaries.
- Presentation depends on application use cases and presentation-facing models; application must not depend on presentation.
- Rendering consumes chart/render-scene contracts and must not control application workflows.
- Bootstrap is the only place allowed to know all concrete implementations needed to construct the running application.

A dependency is not justified merely because two modules share a crate. Visibility and imports must still respect ownership.

## Single-responsibility design rules

### State ownership

- Each mutable state value has one authoritative owner.
- Other components receive immutable views or explicit commands.
- Do not mirror mutable state across layers unless a synchronization contract and invalidation policy exist.
- A UI selection or text field is not the authoritative identity of a loaded market series.

### Validation ownership

- Validate external data at the boundary where it enters the trusted model.
- Domain constructors protect domain invariants regardless of caller.
- Do not rely on UI validation for correctness.
- Do not duplicate the same validation rules in multiple layers; share a domain contract or use a boundary mapper.

### Error ownership

- Errors belong to the capability that can explain and classify the failure.
- Lower layers must not import higher-level models just to enrich an error.
- Cross-boundary errors should carry stable structured context; presentation decides how to display them.

### Mapping ownership

- Each representation conversion has one explicit owner and direction.
- Provider payloads become normalized market data before being projected into chart data.
- Chart-specific types must not become the canonical representation for analytics, persistence or execution.
- Precision loss must be explicit and one-way at presentation/rendering boundaries.

### Resource and lifecycle ownership

- The component that creates a resource owns its release/cancellation lifecycle.
- Long-running streams need explicit cancellation, backpressure and shutdown semantics.
- Cache ownership includes invalidation rules and capacity limits.
- No component should poll, clone or recompute data owned by another layer without a measured need and explicit contract.

### Abstraction threshold

Introduce an abstraction when it protects an invariant, isolates a real variation, removes meaningful duplication or enables independent testing. Do not add a trait, manager, service, factory, generic wrapper or shared utility solely to make the tree look layered.

## Review questions

Every new or materially changed component should be reviewed against these questions:

1. What single capability does it own?
2. What state and invariants belong exclusively to it?
3. Which dependencies are essential, and why are they pointing in this direction?
4. What does it explicitly not do?
5. Can it be tested without unrelated infrastructure?
6. Does it preserve exactness, provenance and deterministic behavior where financial data is involved?
7. What are its resource, failure and cancellation semantics?
8. Does its name and location reveal its ownership at first glance?

A component that cannot answer these questions clearly is not ready to become a stable architectural boundary.
