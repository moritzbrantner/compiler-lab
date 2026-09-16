# compiler-lab

A focused laboratory for language implementation experiments, progressing from parsing foundations through ASTs and IRs to optimization and execution/code generation.

## Design principles

- Keep source text authoritative; derived structures should reference spans instead of copying text by default.
- Make every compiler stage deterministic and independently testable.
- Preserve explicit diagnostics rather than silently recovering across semantic boundaries.
- Prefer reusable, inspectable intermediate representations over framework-heavy abstractions.
- Measure materialization and recomputation before adding performance budgets.

See [ROADMAP.md](ROADMAP.md) for the staged plan.
