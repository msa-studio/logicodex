---
name: Feature Request
about: Suggest a new feature or improvement
title: '[FEATURE] '
labels: enhancement
assignees: ''
---

## Feature Description

A clear description of the proposed feature.

## Motivation

Why is this feature needed? What problem does it solve?

## Alignment Checks

Logicodex is under [Architecture Freeze](https://github.com/mymsa/logicodex/blob/main/SPECIFICATION.md#roadmap). Before proposing a feature, check alignment:

- [ ] **Static Topology** — Does this preserve compile-time structure verification?
- [ ] **Explicit Ownership** — Does this preserve zero-cost memory safety?
- [ ] **Shard Isolation** — Does this preserve deterministic concurrency?
- [ ] **Deterministic Behavior** — Does this preserve reproducible execution?

If 3+ checks pass, consider submitting an [RFC](https://github.com/mymsa/logicodex/blob/main/docs/RFC_TEMPLATE.md).

## Proposed Design

Brief description of how this could be implemented. Link to RFC if submitted.

## Alternatives

What alternatives have you considered?

## Additional Context

Any other information (references to other languages, academic papers, etc.).
