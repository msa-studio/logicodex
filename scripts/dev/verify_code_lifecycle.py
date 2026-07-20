#!/usr/bin/env python3
"""Validate Logicodex code lifecycle inventory against source suppressions."""

from __future__ import annotations

import argparse
import collections
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

ALLOWED_STATUSES = {
    "Active",
    "FutureReserved",
    "Experimental",
    "LegacyReferenceOnly",
    "Deprecated",
    "OrphanCandidate",
    "OrphanBug",
}

TRACKED_LINT_PREFIXES = (
    "dead_code",
    "unused",
    "unreachable",
)

ALLOW_RE = re.compile(
    r"^\s*#(?P<crate>!)?\[allow\((?P<lints>[^)]*)\)\]\s*$"
)

CRATE_CELL_RE = re.compile(
    r"^`(?P<path>src/[^`]+)`:\s+crate-level\s+"
    r"`(?P<lint>[^`]+)`$"
)

MODULE_CELL_RE = re.compile(
    r"^`(?P<path>src/[^`]+)`:\s+"
    r"`#\[allow\((?P<lint>[^)]+)\)\]\s+"
    r"mod\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)`$"
)

ARTIFACT_CELL_RE = re.compile(
    r"^`(?P<artifact>src/[^`]+)`$"
)

FN_RE = re.compile(
    r"^(?:pub(?:\([^)]*\))?\s+)?"
    r"(?:(?:const|async|unsafe)\s+)*"
    r"(?:extern(?:\s+\"[^\"]+\")?\s+)?"
    r"fn\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)"
)

MOD_RE = re.compile(
    r"^(?:pub(?:\([^)]*\))?\s+)?"
    r"mod\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*;"
)

TYPE_RE = re.compile(
    r"^(?:pub(?:\([^)]*\))?\s+)?"
    r"(?:enum|struct|union|trait|type)\s+"
    r"(?P<name>[A-Za-z_][A-Za-z0-9_]*)"
)

IMPL_RE = re.compile(
    r"^impl(?:<[^>]+>)?\s+"
    r"(?P<name>[A-Za-z_][A-Za-z0-9_:]*)"
)

CLOSED_ORPHAN_REQUIREMENTS = (
    (
        "src/ast.rs::Type::storage_width_bits",
        "ast.rs",
        re.compile(r"\bfn\s+storage_width_bits\s*\("),
        (
            "- `src/ast.rs::Type::storage_width_bits`: `Delete` — "
            "canonical layout ownership belongs to `LayoutEngine`."
        ),
    ),
    (
        "src/types.rs::PrimitiveType::is_signed_int",
        "types.rs",
        re.compile(r"\bfn\s+is_signed_int\s*\("),
        (
            "- `src/types.rs::PrimitiveType::is_signed_int`: `Delete` — "
            "integer extension behavior is already closed by `int_bits` "
            "plus `is_unsigned_int`."
        ),
    ),
)

LOCKED_LEGACY_STATUSES = (
    (
        "src/semantic.rs::Analyzer::analyze",
        "src/semantic.rs",
        "analyze",
        "LegacyReferenceOnly",
    ),
)


class ValidationError(RuntimeError):
    """Raised when lifecycle validation finds one or more errors."""

    def __init__(self, errors: Iterable[str]) -> None:
        self.errors = list(errors)
        super().__init__("\n".join(self.errors))


@dataclass(frozen=True)
class SuppressionRecord:
    path: str
    scope: str
    lint: str
    kind: str
    name: str
    line: int

    @property
    def key(self) -> tuple[str, str, str, str, str]:
        return (
            self.path,
            self.scope,
            self.lint,
            self.kind,
            self.name,
        )

    @property
    def display(self) -> str:
        if self.scope == "crate":
            return f"{self.path}: crate-level {self.lint}"

        return (
            f"{self.path}: item {self.kind} {self.name} "
            f"({self.lint})"
        )


@dataclass(frozen=True)
class InventoryRow:
    record: SuppressionRecord
    status: str
    evidence: str
    owner_action: str
    condition: str
    raw: str


@dataclass(frozen=True)
class ValidationSummary:
    inventory_rows: int
    source_records: int
    crate_records: int
    item_records: int
    status_counts: dict[str, int]


def is_tracked_lint(lint: str) -> bool:
    normalized = lint.strip()

    return any(
        normalized == prefix or normalized.startswith(prefix + "_")
        for prefix in TRACKED_LINT_PREFIXES
    )


def next_declaration(
    lines: list[str],
    start_index: int,
) -> tuple[str, str]:
    for line in lines[start_index:]:
        stripped = line.strip()

        if not stripped:
            continue

        if stripped.startswith("//"):
            continue

        if stripped.startswith("/*") or stripped.startswith("*"):
            continue

        if stripped.startswith("#["):
            continue

        match = MOD_RE.match(stripped)

        if match:
            return "mod", match.group("name")

        match = FN_RE.match(stripped)

        if match:
            return "fn", match.group("name")

        match = TYPE_RE.match(stripped)

        if match:
            return "type", match.group("name")

        match = IMPL_RE.match(stripped)

        if match:
            return "impl", match.group("name")

        raise ValidationError(
            [
                "unable to classify declaration following suppression: "
                f"{stripped}"
            ]
        )

    raise ValidationError(
        ["suppression attribute has no following declaration"]
    )


def scan_source_suppressions(
    source_root: Path,
) -> list[SuppressionRecord]:
    errors: list[str] = []
    records: list[SuppressionRecord] = []

    if not source_root.is_dir():
        raise ValidationError(
            [f"source root does not exist: {source_root}"]
        )

    repository_root = source_root.parent

    for path in sorted(source_root.rglob("*.rs")):
        lines = path.read_text(
            encoding="utf-8",
            errors="strict",
        ).splitlines()

        relative = path.relative_to(repository_root).as_posix()

        for index, line in enumerate(lines):
            match = ALLOW_RE.match(line)

            if not match:
                continue

            lints = [
                lint.strip()
                for lint in match.group("lints").split(",")
                if lint.strip()
            ]

            tracked = [
                lint
                for lint in lints
                if is_tracked_lint(lint)
            ]

            if not tracked:
                continue

            if match.group("crate"):
                kind = "crate"
                name = ""
                scope = "crate"
            else:
                scope = "item"

                try:
                    kind, name = next_declaration(
                        lines,
                        index + 1,
                    )
                except ValidationError as error:
                    errors.extend(
                        f"{relative}:{index + 1}: {message}"
                        for message in error.errors
                    )
                    continue

            for lint in tracked:
                records.append(
                    SuppressionRecord(
                        path=relative,
                        scope=scope,
                        lint=lint,
                        kind=kind,
                        name=name,
                        line=index + 1,
                    )
                )

    key_counts = collections.Counter(
        record.key
        for record in records
    )

    duplicates = [
        key
        for key, count in key_counts.items()
        if count > 1
    ]

    for key in duplicates:
        errors.append(
            "duplicate source suppression record: "
            + ":".join(key)
        )

    if errors:
        raise ValidationError(errors)

    return records


def inventory_record_from_cell(
    cell: str,
    row_number: int,
) -> SuppressionRecord:
    match = CRATE_CELL_RE.match(cell)

    if match:
        return SuppressionRecord(
            path=match.group("path"),
            scope="crate",
            lint=match.group("lint"),
            kind="crate",
            name="",
            line=row_number,
        )

    match = MODULE_CELL_RE.match(cell)

    if match:
        return SuppressionRecord(
            path=match.group("path"),
            scope="item",
            lint=match.group("lint"),
            kind="mod",
            name=match.group("name"),
            line=row_number,
        )

    match = ARTIFACT_CELL_RE.match(cell)

    if not match:
        raise ValidationError(
            [
                f"inventory row {row_number}: "
                f"unsupported artifact cell: {cell}"
            ]
        )

    artifact = match.group("artifact")

    marker = ".rs::"

    if marker not in artifact:
        raise ValidationError(
            [
                f"inventory row {row_number}: "
                f"item artifact lacks .rs:: separator: {artifact}"
            ]
        )

    path_prefix, suffix = artifact.split(marker, 1)
    path = path_prefix + ".rs"

    if suffix.startswith("impl "):
        kind = "impl"
        name = suffix.removeprefix("impl ").strip()
    elif "::" in suffix:
        kind = "fn"
        name = suffix.rsplit("::", 1)[1]
    else:
        kind = "type"
        name = suffix

    return SuppressionRecord(
        path=path,
        scope="item",
        lint="dead_code",
        kind=kind,
        name=name,
        line=row_number,
    )


def extract_baseline_count(
    text: str,
    pattern: str,
    label: str,
) -> int:
    match = re.search(
        pattern,
        text,
        flags=re.MULTILINE,
    )

    if not match:
        raise ValidationError(
            [f"missing audit baseline count: {label}"]
        )

    return int(match.group(1))


def parse_inventory(
    inventory_path: Path,
) -> tuple[list[InventoryRow], tuple[int, int, int], str]:
    if not inventory_path.is_file():
        raise ValidationError(
            [f"inventory file does not exist: {inventory_path}"]
        )

    text = inventory_path.read_text(encoding="utf-8")
    errors: list[str] = []

    for status in sorted(ALLOWED_STATUSES):
        if f"- `{status}`" not in text:
            errors.append(
                f"official lifecycle status missing from document: "
                f"{status}"
            )

    required_policy_text = [
        (
            "No uncalled function, type, module, or trait "
            "may be deleted automatically."
        ),
        "Source removal is not authorized by SSM-D2.",
    ]

    for required in required_policy_text:
        if required not in text:
            errors.append(
                f"required lifecycle policy text missing: {required}"
            )

    total_count = extract_baseline_count(
        text,
        r"^-\s+(\d+)\s+explicit dead-code,",
        "total suppressions",
    )

    crate_count = extract_baseline_count(
        text,
        r"^-\s+(\d+)\s+crate-level suppression attributes;",
        "crate-level suppressions",
    )

    item_count = extract_baseline_count(
        text,
        r"^-\s+(\d+)\s+item-level suppression attributes;",
        "item-level suppressions",
    )

    rows: list[InventoryRow] = []

    for row_number, line in enumerate(
        text.splitlines(),
        start=1,
    ):
        if not line.startswith("| `src/"):
            continue

        cells = [
            cell.strip()
            for cell in line.strip("|").split("|")
        ]

        if len(cells) != 5:
            errors.append(
                f"inventory row {row_number}: "
                f"expected 5 cells, found {len(cells)}"
            )
            continue

        artifact_cell, status, evidence, owner_action, condition = cells

        if status not in ALLOWED_STATUSES:
            errors.append(
                f"inventory row {row_number}: "
                f"unknown lifecycle status: {status}"
            )

        try:
            record = inventory_record_from_cell(
                artifact_cell,
                row_number,
            )
        except ValidationError as error:
            errors.extend(error.errors)
            continue

        row = InventoryRow(
            record=record,
            status=status,
            evidence=evidence,
            owner_action=owner_action,
            condition=condition,
            raw=line,
        )

        rows.append(row)

        if not evidence or evidence.lower() in {"tbd", "none"}:
            errors.append(
                f"inventory row {row_number}: "
                "canonical evidence is empty or placeholder"
            )

        if not owner_action or owner_action.lower() in {"tbd", "none"}:
            errors.append(
                f"inventory row {row_number}: "
                "owner and intended action are missing"
            )

        if not condition or condition.lower() in {"tbd", "none"}:
            errors.append(
                f"inventory row {row_number}: "
                "activation or review condition is missing"
            )

        if status == "FutureReserved":
            future_text = (
                owner_action + " " + condition
            ).lower()

            future_terms = (
                "activate",
                "activation",
                "approved",
                "phase",
                "frontend",
            )

            if not any(term in future_text for term in future_terms):
                errors.append(
                    f"inventory row {row_number}: "
                    "FutureReserved entry lacks an explicit "
                    "activation condition"
                )

        if status == "OrphanCandidate":
            protection_text = (
                owner_action + " " + condition
            ).lower()

            protected = (
                "do not" in protection_text
                and any(
                    term in protection_text
                    for term in (
                        "delete",
                        "automatic",
                        "redundant",
                    )
                )
            )

            if not protected:
                errors.append(
                    f"inventory row {row_number}: "
                    "OrphanCandidate lacks explicit protection "
                    "against automatic deletion"
                )

        if status == "OrphanBug":
            evidence_text = evidence.lower()
            priority_text = (
                owner_action + " " + condition
            ).lower()

            evidence_present = (
                "evidence" in evidence_text
                and "no evidence" not in evidence_text
            )

            if not evidence_present:
                errors.append(
                    f"inventory row {row_number}: "
                    "OrphanBug lacks explicit activation-gap evidence"
                )

            if "priority" not in priority_text:
                errors.append(
                    f"inventory row {row_number}: "
                    "OrphanBug lacks a priority review marker"
                )

    key_counts = collections.Counter(
        row.record.key
        for row in rows
    )

    for key, count in key_counts.items():
        if count > 1:
            errors.append(
                "duplicate inventory record: "
                + ":".join(key)
            )

    if len(rows) != total_count:
        errors.append(
            "inventory row count does not match audit baseline: "
            f"rows={len(rows)} baseline={total_count}"
        )

    if crate_count + item_count != total_count:
        errors.append(
            "audit baseline counts are inconsistent: "
            f"crate={crate_count} item={item_count} "
            f"total={total_count}"
        )

    if errors:
        raise ValidationError(errors)

    return rows, (total_count, crate_count, item_count), text


def validate_repository(
    inventory_path: Path,
    source_root: Path,
) -> ValidationSummary:
    rows, baseline, inventory_text = parse_inventory(inventory_path)
    source_records = scan_source_suppressions(source_root)

    total_expected, crate_expected, item_expected = baseline
    errors: list[str] = []

    for artifact, source_path, declaration_re, resolution_record in (
        CLOSED_ORPHAN_REQUIREMENTS
    ):
        if resolution_record not in inventory_text:
            errors.append(
                f"closed orphan resolution missing: {artifact}"
            )

        path = source_root / source_path

        if path.is_file() and declaration_re.search(
            path.read_text(encoding="utf-8")
        ):
            errors.append(
                f"closed orphan artifact reintroduced: {artifact}"
            )

    for artifact, path, name, expected_status in LOCKED_LEGACY_STATUSES:
        matching_rows = [
            row
            for row in rows
            if row.record.path == path and row.record.name == name
        ]

        actual_status = (
            matching_rows[0].status
            if len(matching_rows) == 1
            else "missing"
        )

        if actual_status != expected_status:
            errors.append(
                "locked legacy status mismatch: "
                f"{artifact} must remain {expected_status}; "
                f"found {actual_status}"
            )

    inventory_counter = collections.Counter(
        row.record.key
        for row in rows
    )

    source_counter = collections.Counter(
        record.key
        for record in source_records
    )

    missing_from_inventory = source_counter - inventory_counter
    stale_inventory = inventory_counter - source_counter

    for key, count in sorted(missing_from_inventory.items()):
        errors.append(
            "source suppression missing from inventory: "
            f"{':'.join(key)} count={count}"
        )

    for key, count in sorted(stale_inventory.items()):
        errors.append(
            "stale inventory record has no source suppression: "
            f"{':'.join(key)} count={count}"
        )

    source_crate = sum(
        record.scope == "crate"
        for record in source_records
    )

    source_item = sum(
        record.scope == "item"
        for record in source_records
    )

    if len(source_records) != total_expected:
        errors.append(
            "source suppression count does not match audit baseline: "
            f"source={len(source_records)} "
            f"baseline={total_expected}"
        )

    if source_crate != crate_expected:
        errors.append(
            "crate-level source suppression count mismatch: "
            f"source={source_crate} baseline={crate_expected}"
        )

    if source_item != item_expected:
        errors.append(
            "item-level source suppression count mismatch: "
            f"source={source_item} baseline={item_expected}"
        )

    if errors:
        raise ValidationError(errors)

    status_counts = collections.Counter(
        row.status
        for row in rows
    )

    return ValidationSummary(
        inventory_rows=len(rows),
        source_records=len(source_records),
        crate_records=source_crate,
        item_records=source_item,
        status_counts=dict(status_counts),
    )


def build_parser() -> argparse.ArgumentParser:
    script_path = Path(__file__).resolve()
    repository_root = script_path.parent.parent.parent

    parser = argparse.ArgumentParser(
        description=(
            "Validate the Logicodex lifecycle inventory against "
            "source suppression attributes."
        )
    )

    parser.add_argument(
        "--inventory",
        type=Path,
        default=(
            repository_root
            / "docs/architecture/code-lifecycle-inventory.md"
        ),
    )

    parser.add_argument(
        "--source-root",
        type=Path,
        default=repository_root / "src",
    )

    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()

    try:
        summary = validate_repository(
            args.inventory.resolve(),
            args.source_root.resolve(),
        )
    except ValidationError as error:
        for message in error.errors:
            print(
                f"ERROR: {message}",
                file=sys.stderr,
            )

        print(
            "code_lifecycle_validation=FAIL",
            file=sys.stderr,
        )

        return 1
    except Exception as error:
        print(
            f"ERROR: unexpected validator failure: {error}",
            file=sys.stderr,
        )

        print(
            "code_lifecycle_validation=FAIL",
            file=sys.stderr,
        )

        return 2

    print(
        f"lifecycle_inventory_rows={summary.inventory_rows}"
    )
    print(
        f"source_suppression_records={summary.source_records}"
    )
    print(
        f"crate_level_records={summary.crate_records}"
    )
    print(
        f"item_level_records={summary.item_records}"
    )

    for status in sorted(summary.status_counts):
        print(
            f"status_{status}="
            f"{summary.status_counts[status]}"
        )

    print("code_lifecycle_validation=PASS")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
