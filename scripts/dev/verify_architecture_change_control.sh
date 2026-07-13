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
  '- `rfc-approved`' \
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

if "check_freeze" in jobs:
    raise SystemExit("ERROR: stale check_freeze job remains")

required_text = [
    "ARCH_SENSITIVE_FILES:",
    "check_architecture_control:",
    'grep -x "architecture-change"',
    'grep -x "rfc-approved"',
    "check_size, check_architecture_control",
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
]

for item in stale_text:
    if item in text:
        raise SystemExit(f"ERROR: stale freeze mechanic remains: {item}")

print("yaml_jobs=" + ",".join(sorted(jobs)))
print("workflow_structure=PASS")
PY

echo "---------->>> LABEL POLICY SCENARIOS"

python3 - <<'PY'
def policy_result(labels: set[str]) -> str:
    architecture_change = "architecture-change" in labels
    rfc_approved = "rfc-approved" in labels

    if architecture_change and not rfc_approved:
        return "FAIL"

    if rfc_approved and not architecture_change:
        return "FAIL"

    return "PASS"


scenarios = {
    "routine": (set(), "PASS"),
    "architecture_without_rfc": (
        {"architecture-change"},
        "FAIL",
    ),
    "rfc_without_declaration": (
        {"rfc-approved"},
        "FAIL",
    ),
    "approved_architecture_change": (
        {"architecture-change", "rfc-approved"},
        "PASS",
    ),
}

for name, (labels, expected) in scenarios.items():
    actual = policy_result(labels)

    if actual != expected:
        raise SystemExit(
            f"ERROR: scenario {name}: "
            f"expected {expected}, got {actual}"
        )

    print(f"{name}={actual}")

print("label_policy_scenarios=PASS")
PY

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
