# Roadmap

The lab progresses one compiler boundary at a time. Each slice should remain small enough to validate semantics and dataflow before the next representation is introduced.

## Stage 1 — Lexical front end

- [x] **1.1 Copy-conscious lexer foundation** — authoritative source buffer, compact byte spans, deterministic tokens and diagnostics, comments/keywords/operators, focused CI.
- [x] **1.2 Source locations and diagnostic rendering** — lazy line index built once per source, stable one-based line/character lookup, borrowed snippets and on-demand diagnostic rendering without attaching copied source text to diagnostics.

## Stage 2 — Parser

- [ ] **2.1 Expression parser** — literals, names, unary/binary operators and grouping with explicit precedence.
- [ ] **2.2 Statement/function parser** — bindings, returns, blocks and functions with deterministic recovery boundaries.
- [ ] **2.3 Parser corpus** — valid/invalid fixtures, exact diagnostics and replayable parse fingerprints.

## Stage 3 — AST and semantic analysis

- [ ] **3.1 Compact AST arena** — stable node IDs and source spans; avoid recursive owned strings and subtree cloning.
- [ ] **3.2 Name resolution** — scopes, bindings and deterministic unresolved/duplicate-name diagnostics.
- [ ] **3.3 Type checking** — explicit type representation, inference only where evidence remains inspectable.

## Stage 4 — Intermediate representation

- [ ] **4.1 Lower AST to IR** — stable value/block IDs with source provenance.
- [ ] **4.2 Control-flow validation** — predecessor/successor consistency, dominance-ready structure and verifier failures that fail closed.
- [ ] **4.3 IR observability** — deterministic text form and fingerprints for before/after pass comparison.

## Stage 5 — Optimization

- [ ] **5.1 Constant folding and propagation** with semantic equivalence tests.
- [ ] **5.2 Dead-code elimination** with explicit liveness evidence.
- [ ] **5.3 Pass manager** that records which pass changed what without cloning the whole IR between passes.

## Stage 6 — Execution and code generation

- [ ] **6.1 Reference interpreter** for the IR.
- [ ] **6.2 Bytecode VM** with deterministic instruction traces.
- [ ] **6.3 Native/Wasm code-generation experiment** behind equivalence tests against the interpreter.

## Cross-cutting constraints

- Source text is authoritative. Derived representations reference source spans or stable IDs unless ownership is semantically necessary.
- Do not make snapshots, pretty-printed forms or full-representation clones part of ordinary compilation; materialize them only when explicitly requested for diagnostics, debugging or persistence.
- Before imposing runtime budgets, instrument multiplicative costs: bytes copied, nodes/tokens materialized, pass visits and repeated analyses.
- Every compiler stage must be deterministic, replayable and independently testable.
- GitHub Pages presents project/evidence state through `github-pages-template`; it does not become an authority for compiler semantics or performance verdicts.
