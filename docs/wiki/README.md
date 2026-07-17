# Logicodex Documentation

Logicodex documentation is organized into **4 core documents** and **2 deep wikis**.

## Core Documents (Start Here)

| Document | Scope | Path |
|---|---|---|
| **README** | Project entry point, quick start, document map | `README.md` |
| **Specification** | The contract: language spec, architecture, roadmap, governance | `SPECIFICATION.md` |
| **Changelog** | Version history, decision log | `CHANGELOG.md` |
| **Handbook** | User guide: tutorials, API reference, recipes, troubleshooting | `docs/HANDBOOK.md` |

## Deep Wikis (Detailed Reference)

| Wiki | Scope | Build |
|---|---|---|
| **Experimental Compiler Philosophy** | Detailed justifications for every architectural decision | `cd docs/white-paper && mdbook build` |
| **Functions And Guide** | Comprehensive function reference for all APIs | `cd docs/guide && mdbook build` |

## What Each Document Covers

| Question | Document |
|---|---|
| "What is Logicodex?" | `SPECIFICATION.md` § 1–2 |
| "How do I install it?" | `docs/HANDBOOK.md` § 1 |
| "How do I write a program?" | `docs/HANDBOOK.md` § 2–6 |
| "How do I use graphics/audio?" | `docs/HANDBOOK.md` § 9–10 |
| "How do actors/channels work?" | `docs/HANDBOOK.md` § 7 |
| "What is the language grammar?" | `SPECIFICATION.md` § 2.1 |
| "What is the architecture?" | `SPECIFICATION.md` § 3 |
| "What is the security model?" | `SPECIFICATION.md` § 4 |
| "What is the concurrency model?" | `SPECIFICATION.md` § 5 |
| "What targets are supported?" | `SPECIFICATION.md` § 6 |
| "What is the roadmap?" | `SPECIFICATION.md` § 7 |
| "What changed when?" | `CHANGELOG.md` |
| "Why was this decision made?" | `docs/white-paper/` (wiki) |
| "What functions are available?" | `docs/guide/` (wiki) |
| "How do I contribute?" | `SPECIFICATION.md` § 8 |
| "What is the license?" | `SPECIFICATION.md` § 8.1 |

## Archived Documents

Historical documents preserved in `docs/archive/`:

| Document | Why Archived |
|---|---|
| `WHITE_PAPER_v121.md` | Original v1.21 baseline — referenced from SPECIFICATION.md |
| `ROADMAP_v145.md` | Merged into SPECIFICATION.md § 7 |
| `ARCHITECTURE.md` | Merged into SPECIFICATION.md § 3 |
| `AUDIT_v141.md` | Stale (v1.41) — CHANGELOG.md has current data |
| `GETTING_STARTED_v145.md` | Merged into docs/HANDBOOK.md |
| `v1.30-` to `v1.37-*.md` | Merged into HANDBOOK.md + wikis |
| `CHANGELOG_ANALYSIS_v145.md` | One-time analysis document |
| `INCONSISTENCY_AUDIT_v145.md` | One-time audit document |
| `BENCHMARK_PLAN_v145.md` | Merged into SPECIFICATION.md § 7 |
| `MAINTENANCE_v1441.md` | One-time maintenance report |

## Document Count

| Category | Count | Purpose |
|---|---|---|
| Core documents (root) | 3 (README + SPECIFICATION + CHANGELOG) | Single sources of truth |
| Core documents (docs/) | 1 (HANDBOOK) | User guide |
| Deep wikis | 2 (white-paper + guide) | Detailed reference |
| Archived documents | 16 | Historical preservation |
| **Total active docs** | **4** | Manageable, specific scope each |

---

*Logicodex Documentation — v0.46.0-alpha*
