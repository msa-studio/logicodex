#!/usr/bin/env python3
"""Render the Logicodex Gatekeeper summary from mandatory job results."""

from __future__ import annotations

import argparse
from pathlib import Path


GATES = {
    "phase": "Phase Compliance",
    "documentation": "Documentation Gate",
    "size": "PR Size Check",
    "architecture": "Architecture Change Control",
}

RESULTS = {
    "success": ("PASS", "Mandatory job completed successfully.", False),
    "failure": ("FAIL", "Mandatory job failed.", True),
    "cancelled": (
        "CANCELLED",
        "Mandatory job was interrupted before completion.",
        True,
    ),
}


def classify(result: str, intentional_skip: bool) -> tuple[str, str, bool]:
    normalized = result.strip().lower()

    if normalized == "skipped":
        if intentional_skip:
            return (
                "SKIPPED INTENTIONALLY",
                "Skip was explicitly allowed by policy.",
                False,
            )
        return (
            "SKIPPED ACCIDENTALLY",
            "Mandatory job did not run without an allowed skip.",
            True,
        )

    if normalized in RESULTS:
        return RESULTS[normalized]

    value = normalized or "(empty)"
    return "UNKNOWN", f"Unsupported job result: {value}.", True


def evaluate(
    results: dict[str, str],
    intentional_skips: set[str],
) -> tuple[list[tuple[str, str, str]], bool]:
    rows = []
    failed = False

    for key, label in GATES.items():
        display, explanation, gate_failed = classify(
            results[key],
            key in intentional_skips,
        )
        rows.append((label, display, explanation))
        failed |= gate_failed

    return rows, failed


def render(rows: list[tuple[str, str, str]], failed: bool) -> str:
    lines = [
        "# Logicodex Gatekeeper Policy Summary",
        "",
        "| Gate | Result | Interpretation |",
        "| --- | --- | --- |",
    ]
    lines.extend(
        f"| {label} | **{display}** | {explanation} |"
        for label, display, explanation in rows
    )

    overall = "FAIL" if failed else "PASS"
    detail = (
        "A mandatory job failed, was cancelled, was skipped accidentally, "
        "or returned an unknown result."
        if failed
        else "All mandatory jobs passed or used an allowed skip."
    )
    lines.extend(["", f"**Overall policy result: {overall}**", "", detail])
    return "\n".join(lines) + "\n"


def run(
    results: dict[str, str],
    intentional_skips: set[str],
    summary_file: str | None,
) -> int:
    rows, failed = evaluate(results, intentional_skips)
    markdown = render(rows, failed)

    if summary_file:
        with Path(summary_file).open("a", encoding="utf-8") as handle:
            handle.write(markdown)

    print(markdown, end="")
    return int(failed)


def self_test() -> None:
    success = {key: "success" for key in GATES}
    cases = (
        ("all_success", success, set(), 0, "Overall policy result: PASS"),
        (
            "intentional_skip",
            {**success, "documentation": "skipped"},
            {"documentation"},
            0,
            "SKIPPED INTENTIONALLY",
        ),
        (
            "failed_gate",
            {**success, "phase": "failure"},
            set(),
            1,
            "Overall policy result: FAIL",
        ),
        (
            "accidental_skip",
            {**success, "size": "skipped"},
            set(),
            1,
            "SKIPPED ACCIDENTALLY",
        ),
        (
            "cancelled_gate",
            {**success, "architecture": "cancelled"},
            set(),
            1,
            "CANCELLED",
        ),
        (
            "unknown_gate",
            {**success, "architecture": ""},
            set(),
            1,
            "UNKNOWN",
        ),
    )

    for name, results, skips, expected_rc, expected_text in cases:
        rows, failed = evaluate(results, skips)
        markdown = render(rows, failed)

        if int(failed) != expected_rc:
            raise RuntimeError(
                f"{name}: expected RC {expected_rc}, got {int(failed)}"
            )
        if expected_text not in markdown:
            raise RuntimeError(f"{name}: missing {expected_text!r}")

        print(f"{name}=PASS")

    print("policy_summary_self_test=PASS")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Evaluate Logicodex Gatekeeper results."
    )
    for key in GATES:
        parser.add_argument(f"--{key}")

    parser.add_argument("--summary-file")
    parser.add_argument(
        "--intentional-skip",
        action="append",
        default=[],
        choices=sorted(GATES),
    )
    parser.add_argument("--self-test", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()

    if args.self_test:
        self_test()
        return 0

    results = {key: getattr(args, key) for key in GATES}
    missing = [key for key, value in results.items() if value is None]

    if missing:
        print(
            "ERROR: missing mandatory result arguments: "
            + ", ".join(missing)
        )
        return 2

    return run(
        {key: str(value) for key, value in results.items()},
        set(args.intentional_skip),
        args.summary_file,
    )


if __name__ == "__main__":
    raise SystemExit(main())
