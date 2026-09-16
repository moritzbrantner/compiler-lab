# AGENTS.md

## Scope

These rules apply to the whole repository.

## Compiler architecture

- Keep the original source buffer authoritative throughout front-end work.
- Tokens, diagnostics and syntax/semantic nodes should reference source by compact spans or stable IDs. Do not store copied lexeme/source strings unless ownership is required by semantics.
- Avoid whole-representation cloning between compiler stages or optimization passes. Prefer mutation through explicit local changes or construction of the next genuinely different representation.
- Expensive derived views such as formatted AST/IR, snapshots and debug dumps are on-demand observability surfaces, not ordinary pipeline state.
- Cache a derived analysis only when repeated use is demonstrated and the invalidation boundary is explicit.
- Make stage boundaries deterministic and fail closed on malformed internal state.

## Validation

Before a slice is integrated, run:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

Add focused regression tests for semantic changes. Prefer tests that prove stable spans, IDs, diagnostics and replayable outputs over broad snapshots.

## Performance evidence

Do not start with micro-optimizations or arbitrary timing budgets. First make multiplicative costs observable: source/IR bytes copied, materializations, node/pass visits, repeated analyses and allocation-heavy boundaries. Runtime budgets should come after repeated representative measurements.
