# Logicodex Agent Instructions

Before planning, editing, reviewing, or proposing work, read:

- [`docs/architecture/current-authority.md`](docs/architecture/current-authority.md)

That file is the single entry point for current authority and work sequence.
Live source plus executable tests remain primary truth for implemented behavior.

Do not infer current authority from:

- version numbers in historical documents;
- `docs/archive/**`, `scripts/_archive/**`, or `spec/v*/**`;
- unchecked roadmap issue labels or old sprint lists;
- dormant, legacy, or `FutureReserved` code.

Preserve canonical HIR and `semantic_gate` Meaning Authority. Do not reactivate
legacy paths or perform broad cleanup without an approved scoped task. Keep code,
tests, lifecycle records, changelog, and the current-authority entry point aligned.
Run the repository quick gate while iterating and full integrity before push or
task-complete delivery.
