#!/usr/bin/env python3
"""Self-tests for the Logicodex code lifecycle validator."""

from __future__ import annotations

import tempfile
from pathlib import Path

from verify_code_lifecycle import (
    ValidationError,
    validate_repository,
)


def fixture_inventory() -> str:
    return """# Code Lifecycle Inventory

Status: Active SSM-D2 lifecycle record

## Lifecycle statuses

- `Active`
- `FutureReserved`
- `Experimental`
- `LegacyReferenceOnly`
- `Deprecated`
- `OrphanCandidate`
- `OrphanBug`

## Audit baseline

- 3 explicit dead-code, unused-variable, or related suppression attributes;
- 1 crate-level suppression attributes;
- 2 item-level suppression attributes;

## Suppression and lifecycle inventory

| Suppressed artifact | Status | Canonical evidence | Owner and intended action | Activation or review condition |
|---|---|---|---|---|
| `src/a.rs`: crate-level `dead_code` | Active | Active compatibility surface. | Test owner. Retain. | Review with focused tests. |
| `src/a.rs::FutureHook` | FutureReserved | Dormant hook evidence. | Test owner. Preserve for approved phase. | Activate only during an approved phase. |
| `src/b.rs::Candidate` | OrphanCandidate | No caller evidence. | Test owner. Do not delete automatically. | Review ownership before any deletion. |

## SSM-D2 decisions

No uncalled function, type, module, or trait may be deleted automatically.
Source removal is not authorized by SSM-D2.
"""


def expect_failure(
    name: str,
    inventory_path: Path,
    source_root: Path,
    expected_text: str,
) -> None:
    try:
        validate_repository(
            inventory_path,
            source_root,
        )
    except ValidationError as error:
        message = str(error)

        if expected_text not in message:
            raise RuntimeError(
                f"{name}: expected error containing "
                f"{expected_text!r}, got {message!r}"
            ) from error

        print(f"self_test_{name}=PASS")
        return

    raise RuntimeError(
        f"{name}: validation unexpectedly passed"
    )


def run_self_test() -> None:
    with tempfile.TemporaryDirectory(
        prefix="logicodex-lifecycle-self-test-"
    ) as temp:
        root = Path(temp)
        source_root = root / "src"
        source_root.mkdir()

        (source_root / "a.rs").write_text(
            "#![allow(dead_code)]\n"
            "\n"
            "#[allow(dead_code)]\n"
            "pub struct FutureHook;\n",
            encoding="utf-8",
        )

        (source_root / "b.rs").write_text(
            "#[allow(dead_code)]\n"
            "pub struct Candidate;\n",
            encoding="utf-8",
        )

        inventory_path = root / "inventory.md"
        valid_text = fixture_inventory()

        inventory_path.write_text(
            valid_text,
            encoding="utf-8",
        )

        summary = validate_repository(
            inventory_path,
            source_root,
        )

        if summary.inventory_rows != 3:
            raise RuntimeError(
                "valid fixture row count mismatch"
            )

        print("self_test_valid_fixture=PASS")

        inventory_path.write_text(
            valid_text.replace(
                "| FutureReserved |",
                "| UnknownStatus |",
                1,
            ),
            encoding="utf-8",
        )

        expect_failure(
            "unknown_status",
            inventory_path,
            source_root,
            "unknown lifecycle status",
        )

        candidate_row = (
            "| `src/b.rs::Candidate` | OrphanCandidate | "
            "No caller evidence. | Test owner. "
            "Do not delete automatically. | "
            "Review ownership before any deletion. |"
        )

        inventory_path.write_text(
            valid_text.replace(
                candidate_row,
                candidate_row + "\n" + candidate_row,
            ),
            encoding="utf-8",
        )

        expect_failure(
            "duplicate_record",
            inventory_path,
            source_root,
            "duplicate inventory record",
        )

        inventory_path.write_text(
            valid_text.replace(
                "Activate only during an approved phase.",
                "TBD",
                1,
            ),
            encoding="utf-8",
        )

        expect_failure(
            "future_reserved_condition",
            inventory_path,
            source_root,
            "activation or review condition is missing",
        )

        inventory_path.write_text(
            valid_text.replace(
                "Do not delete automatically.",
                "Review later.",
                1,
            ),
            encoding="utf-8",
        )

        expect_failure(
            "orphan_candidate_protection",
            inventory_path,
            source_root,
            "OrphanCandidate lacks explicit protection",
        )

        inventory_path.write_text(
            valid_text.replace(
                "| OrphanCandidate | No caller evidence. | "
                "Test owner. Do not delete automatically. |",
                "| OrphanBug | No evidence. | "
                "Test owner. Do not delete automatically. |",
                1,
            ),
            encoding="utf-8",
        )

        expect_failure(
            "orphan_bug_evidence",
            inventory_path,
            source_root,
            "OrphanBug lacks explicit activation-gap evidence",
        )

        inventory_path.write_text(
            valid_text,
            encoding="utf-8",
        )

        (source_root / "c.rs").write_text(
            "#[allow(dead_code)]\n"
            "pub fn unlisted() {}\n",
            encoding="utf-8",
        )

        expect_failure(
            "source_inventory_drift",
            inventory_path,
            source_root,
            "source suppression missing from inventory",
        )

    print("code_lifecycle_self_test=PASS")

def main() -> int:
    run_self_test()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
