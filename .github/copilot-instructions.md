# ApexTrader repository instructions

Read the repository-root `AGENTS.md` before making changes. Treat it as the canonical agent contract and follow the engineering standards in `docs/engineering/standards.md`.

Keep the domain independent from UI, rendering, operating-system and provider infrastructure. Preserve cohesive module boundaries and inward dependency direction. Do not add code comments of any kind, including Rustdoc, TODO/FIXME or commented-out code. Put explanations in external Markdown documentation.

Use typed errors, explicit ownership and borrowing, focused tests, and measured performance changes. Do not introduce dependencies, unsafe code, crates or architectural abstractions without evaluating their necessity and risks.

Before finishing, review the complete diff and run the applicable formatting, compilation, Clippy, test, architecture, security and comment-policy checks. Report any checks that could not be run.
