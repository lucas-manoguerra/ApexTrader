# ApexTrader Agent Contract

## Mission

Deliver maintainable, secure, idiomatic Rust changes while preserving the project's documented architecture and quality gates.

## Mandatory rules

1. Read this file and the relevant documents under `docs/` before changing project structure or cross-module contracts.
2. Treat the domain as independent from UI frameworks, rendering backends, operating-system APIs and external market providers.
3. Keep responsibilities cohesive and dependencies directed inward.
4. Do not introduce comments in code. This includes Rustdoc, line comments, block comments, TODO/FIXME markers and commented-out code.
5. Put explanatory material in Markdown documentation, architecture decision records or API design documents.
6. Prefer explicit types, ownership, borrowing and typed errors.
7. Do not add dependencies without evaluating maintenance, license, security, compile-time and runtime costs.
8. Do not introduce `unsafe` without a dedicated review and documented safety case.
9. Add or update tests for changed behavior and edge cases.
10. Measure performance-sensitive changes with reproducible benchmarks.
11. Do not claim a backend, feature or validation is complete unless implementation and tests establish it.
12. Preserve unrelated user changes and avoid broad refactors outside the approved scope.

## Required workflow

1. Inspect the current implementation and its callers.
2. State the affected responsibilities, contracts and risks in the work plan.
3. Make the smallest coherent change.
4. Update external documentation and ADRs when architecture or behavior changes.
5. Run formatting, compilation, Clippy, tests and applicable repository checks.
6. Report commands executed, results, limitations and follow-up risks.

## Review checklist

- Are dependencies directed toward stable abstractions?
- Does each module own one coherent responsibility?
- Are invalid states difficult to represent?
- Are errors recoverable and observable?
- Are allocations and clones justified?
- Are tests deterministic and meaningful?
- Is performance measured where relevant?
- Is the source free of comments?
- Does external documentation match the implementation?
