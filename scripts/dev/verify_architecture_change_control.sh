#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(
  cd "$(dirname "${BASH_SOURCE[0]}")" &&
  pwd
)"

REPO_ROOT="$(
  cd "$SCRIPT_DIR/../.." &&
  pwd
)"

cd "$REPO_ROOT"

echo "---------->>> VERIFY ARCHITECTURE CHANGE CONTROL START"

echo "---------->>> WHITESPACE"
git diff --check

echo "---------->>> REQUIRED FILES"
test -s docs/governance/architecture-freeze-exit-2026-07-13.md
test -s docs/governance/architecture-change-control.md
test -s .github/workflows/gatekeeper.yml
test -s scripts/dev/evaluate_policy_summary.py
test -s ROADMAP_v2.md
test -s SPECIFICATION.md

echo "required_files=PASS"

echo "---------->>> GOVERNANCE DOCUMENT CONTENT"

grep -Fqx \
  '**Decision:** APPROVED' \
  docs/governance/architecture-freeze-exit-2026-07-13.md

grep -Fqx \
  '## Architect ratification' \
  docs/governance/architecture-freeze-exit-2026-07-13.md

grep -Fqx \
  '## Locked invariants' \
  docs/governance/architecture-change-control.md

grep -Fqx -- \
  '- `architecture-change`' \
  docs/governance/architecture-change-control.md

grep -Fqx -- \
  '- `size-exception`' \
  docs/governance/architecture-change-control.md

grep -Fqx -- \
  '- `rfc-required`' \
  docs/governance/architecture-change-control.md

grep -Fqx -- \
  '- `rfc-approved`' \
  docs/governance/architecture-change-control.md

grep -Fqx \
  '`rfc-approved` must never be applied only to bypass the PR-size gate.' \
  docs/governance/architecture-change-control.md

echo "governance_documents=PASS"

echo "---------->>> ACTIVE VERSION AUTHORITY"

CURRENT_VERSION="$(
  grep -m1 '^version[[:space:]]*=' Cargo.toml |
    sed 's/.*"\([^"]*\)".*/\1/'
)"

CURRENT_STATUS="$(
  grep -m1 '^\*\*Status:\*\*' ROADMAP_v2.md |
    sed 's/^\*\*Status:\*\*[[:space:]]*//' |
    xargs
)"

test "$CURRENT_VERSION" = "0.46.0-alpha"

echo "$CURRENT_STATUS" |
  grep -q 'historical freeze exited'

echo "current_version=$CURRENT_VERSION"
echo "current_status=$CURRENT_STATUS"

echo "---------->>> YAML AND JOB STRUCTURE"

python3 - <<'PY'
from pathlib import Path
import yaml

path = Path(".github/workflows/gatekeeper.yml")
text = path.read_text(encoding="utf-8")
data = yaml.safe_load(text)

if not isinstance(data, dict):
    raise SystemExit("ERROR: workflow root is not a mapping")

jobs = data.get("jobs")

if not isinstance(jobs, dict):
    raise SystemExit("ERROR: jobs mapping missing")

expected_jobs = {
    "check_phase_compliance",
    "check_documentation",
    "check_size",
    "check_architecture_control",
    "summarize",
}

missing = sorted(expected_jobs - set(jobs))

if missing:
    raise SystemExit(f"ERROR: missing workflow jobs: {missing}")

summary = jobs["summarize"]
expected_needs = [
    "check_phase_compliance",
    "check_documentation",
    "check_size",
    "check_architecture_control",
]

if summary.get("needs") != expected_needs:
    raise SystemExit(
        "ERROR: Policy Summary needs do not match mandatory jobs"
    )

expected_if = "${{ always() && github.event.pull_request }}"

if summary.get("if") != expected_if:
    raise SystemExit(
        f"ERROR: Policy Summary if must be {expected_if!r}"
    )

summary_steps = summary.get("steps", [])

if not isinstance(summary_steps, list):
    raise SystemExit("ERROR: Policy Summary steps missing")

summary_runs = [
    step.get("run", "")
    for step in summary_steps
    if isinstance(step, dict)
]

if not any(
    "scripts/dev/evaluate_policy_summary.py" in run
    for run in summary_runs
):
    raise SystemExit(
        "ERROR: Policy Summary evaluator is not wired"
    )

if "check_freeze" in jobs:
    raise SystemExit("ERROR: stale check_freeze job remains")

required_text = [
    "ARCH_SENSITIVE_FILES:",
    "check_architecture_control:",
    'grep -x "architecture-change"',
    'grep -x "rfc-approved"',
    "check_size, check_architecture_control",
    "scripts/dev/evaluate_policy_summary.py",
    "needs.check_phase_compliance.result",
    "needs.check_documentation.result",
    "needs.check_size.result",
    "needs.check_architecture_control.result",
    "GITHUB_STEP_SUMMARY",
    "ROADMAP_v2.md",
    "docs/governance/architecture-change-control.md",
]

for item in required_text:
    if item not in text:
        raise SystemExit(f"ERROR: required workflow text missing: {item}")

stale_text = [
    "check_freeze:",
    "FROZEN_FILES",
    "freeze-override",
    "Architecture Freeze Enforcement",
    "Post welcome / policy reminder",
    "github.event.action == 'opened' && github.event.pull_request",
]

for item in stale_text:
    if item in text:
        raise SystemExit(f"ERROR: stale freeze mechanic remains: {item}")

print("yaml_jobs=" + ",".join(sorted(jobs)))
print("workflow_structure=PASS")
PY

echo "---------->>> LABEL POLICY SCENARIOS"

python3 - <<'PY'
def policy_state(labels: set[str]) -> str:
    architecture_change = "architecture-change" in labels
    size_exception = "size-exception" in labels
    rfc_required = "rfc-required" in labels
    rfc_approved = "rfc-approved" in labels

    if rfc_required and rfc_approved:
        return "INVALID"

    if rfc_approved and not architecture_change:
        return "INVALID"

    if rfc_required and not architecture_change:
        return "INVALID"

    if architecture_change:
        if rfc_approved:
            return "ARCHITECTURE_APPROVED"

        if rfc_required:
            return "ARCHITECTURE_PENDING"

        return "INVALID"

    if size_exception:
        return "SIZE_EXCEPTION"

    return "ROUTINE"


scenarios = {
    "routine": (
        set(),
        "ROUTINE",
    ),
    "roadmap_aligned_implementation": (
        set(),
        "ROUTINE",
    ),
    "large_non_architecture": (
        {"size-exception"},
        "SIZE_EXCEPTION",
    ),
    "legacy_size_rfc_conflation": (
        {"rfc-required"},
        "INVALID",
    ),
    "architecture_without_lifecycle": (
        {"architecture-change"},
        "INVALID",
    ),
    "architecture_pending": (
        {
            "architecture-change",
            "rfc-required",
        },
        "ARCHITECTURE_PENDING",
    ),
    "architecture_approved": (
        {
            "architecture-change",
            "rfc-approved",
        },
        "ARCHITECTURE_APPROVED",
    ),
    "large_architecture_approved": (
        {
            "architecture-change",
            "rfc-approved",
            "size-exception",
        },
        "ARCHITECTURE_APPROVED",
    ),
    "approved_without_architecture": (
        {"rfc-approved"},
        "INVALID",
    ),
    "pending_and_approved_together": (
        {
            "architecture-change",
            "rfc-required",
            "rfc-approved",
        },
        "INVALID",
    ),
    "size_and_rfc_without_architecture": (
        {
            "size-exception",
            "rfc-required",
        },
        "INVALID",
    ),
}

for name, (labels, expected) in scenarios.items():
    actual = policy_state(labels)

    print(
        f"SCENARIO[{name}]="
        f"{actual}"
    )

    if actual != expected:
        raise SystemExit(
            f"{name}: expected {expected}, got {actual}"
        )

print("LABEL_POLICY_SCENARIOS=PASS")
PY

echo "---------->>> POLICY SUMMARY EVALUATOR"

python3 scripts/dev/evaluate_policy_summary.py --self-test

echo "policy_summary_evaluator=PASS"

echo "---------->>> STALE ACTIVE POLICY TEXT"

if git grep -nEi \
  'Architecture Freeze is active|FROZEN_FILES|freeze-override|Current: v1\.45|Architecture Freeze Enforcement' \
  -- \
  .github/workflows/gatekeeper.yml \
  .github/pull_request_template.md \
  .github/ISSUE_TEMPLATE/feature_request.md \
  ROADMAP_v2.md \
  SPECIFICATION.md \
  README.md
then
  echo "ERROR: stale active freeze wording found"
  exit 1
fi

echo "stale_active_policy_text=PASS"

echo "---------->>> VERIFY ARCHITECTURE CHANGE CONTROL END: OK"
