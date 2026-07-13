## Description

Brief description of the changes.

## Type of Change

- [ ] Bug fix
- [ ] New feature
- [ ] Documentation update
- [ ] Performance improvement
- [ ] Refactoring
- [ ] Other (describe):

## Checklist

- [ ] Tier A validators pass: `python3 scripts/validators/tier_a_core/*.py`
- [ ] Code is formatted: `cargo fmt`
- [ ] Clippy is clean: `cargo clippy -- -D warnings`
- [ ] Documentation is updated (if applicable)
- [ ] CHANGELOG.md is updated (if applicable)

## Architecture Change Declaration

- [ ] This PR preserves canonical execution, Meaning Authority, and public
      compiler contracts.
- [ ] This PR is an architectural change and carries the
      `architecture-change` label.

For a declared architectural change:

- [ ] RFC submitted and approved
- [ ] `rfc-approved` label applied
- [ ] Compatibility and migration impact documented
- [ ] Canonical HIR and Meaning Authority impact documented
- [ ] Architect or maintainer approval recorded

Policy:

`docs/governance/architecture-change-control.md`

## Testing

Describe how you tested this change.

## Related Issues

Fixes #(issue number) or references #(issue number).
