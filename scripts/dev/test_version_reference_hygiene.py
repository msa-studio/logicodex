#!/usr/bin/env python3
"""Self-tests for version-reference hygiene validation."""

from __future__ import annotations

from pathlib import Path
import tempfile

from verify_version_reference_hygiene import (
    ValidationError,
    validate_repository,
)


def write(
    root: Path,
    relative: str,
    text: str,
) -> None:
    path = root / relative
    path.parent.mkdir(
        parents=True,
        exist_ok=True,
    )
    path.write_text(
        text,
        encoding="utf-8",
    )


def create_fixture(root: Path) -> None:
    version = "0.46.0-alpha"

    write(
        root,
        "Cargo.toml",
        '[package]\n'
        'name = "logicodex"\n'
        f'version = "{version}"\n',
    )

    write(
        root,
        ".github/SECURITY.md",
        "| Version | Status |\n"
        "|---|---|\n"
        f"| v{version} (current Cargo release authority) "
        "| ✅ Current — best-effort alpha security support |\n",
    )

    write(
        root,
        "src/main.rs",
        'env!("CARGO_PKG_VERSION");\n'
        'env!("CARGO_PKG_VERSION");\n'
        'env!("CARGO_PKG_VERSION");\n'
        'default_value = "v1.30"\n'
        'name = "v130-check"\n'
        "CompilerPipeline::V130\n",
    )

    write(
        root,
        "docs/VS_CODE_EXTENSION.md",
        f"current Logicodex v{version}\n",
    )

    write(
        root,
        "docs/examples/REFLEX_ENGINE_EXAMPLES.md",
        "current single HIR compiler path; "
        "v1.21 is a deprecated compatibility alias\n",
    )

    write(
        root,
        "docs/guide/src/title.md",
        "**Historical guide baseline:** v1.45.0-alpha "
        "(non-authoritative)\n",
    )

    write(
        root,
        "docs/wiki/README.md",
        f"Logicodex Documentation — v{version}\n",
    )

    write(
        root,
        "extensions/vscode-logicodex/README.md",
        f"current Logicodex v{version}\n",
    )

    write(
        root,
        "extensions/vscode-logicodex/package.json",
        '{"description":"current Logicodex '
        f'v{version}"}}\n',
    )

    write(
        root,
        "extensions/vscode-logicodex/"
        "src/previewNormalizer.ts",
        "current single HIR compiler path "
        "is authoritative\n",
    )

    for hook in (
        "scripts/dev/verify_quick_integrity.sh",
        "scripts/dev/verify_full_integrity.sh",
    ):
        write(
            root,
            hook,
            "python3 -B scripts/dev/"
            "test_version_reference_hygiene.py\n"
            "python3 -B scripts/dev/"
            "verify_version_reference_hygiene.py\n",
        )

    write(
        root,
        ".github/workflows/ci.yml",
        "\n".join(
            f"grep -Fq '{script}' {hook}"
            for hook in (
                "scripts/dev/verify_quick_integrity.sh",
                "scripts/dev/verify_full_integrity.sh",
            )
            for script in (
                "test_version_reference_hygiene.py",
                "verify_version_reference_hygiene.py",
            )
        )
        + "\n",
    )


def expect_failure(
    name: str,
    root: Path,
    expected: str,
) -> None:
    try:
        validate_repository(root)
    except ValidationError as error:
        message = str(error)

        if expected not in message:
            raise RuntimeError(
                f"{name}: expected {expected!r}, "
                f"got {message!r}"
            ) from error

        print(f"self_test_{name}=PASS")
        return

    raise RuntimeError(
        f"{name}: validation unexpectedly passed"
    )


def run_self_test() -> None:
    with tempfile.TemporaryDirectory(
        prefix="logicodex-version-hygiene-",
    ) as temp:
        root = Path(temp)
        create_fixture(root)

        summary = validate_repository(root)

        if summary.cargo_version != "0.46.0-alpha":
            raise RuntimeError(
                "valid fixture Cargo version mismatch"
            )

        print("self_test_valid_fixture=PASS")

        security = root / ".github/SECURITY.md"
        valid_security = security.read_text(
            encoding="utf-8",
        )

        security.write_text(
            valid_security
            + "current Logicodex v1.21-alpha\n",
            encoding="utf-8",
        )

        expect_failure(
            "stale_current_claim",
            root,
            "stale active version claim",
        )

        security.write_text(
            valid_security,
            encoding="utf-8",
        )

        main = root / "src/main.rs"
        valid_main = main.read_text(
            encoding="utf-8",
        )

        main.write_text(
            valid_main
            + 'version = "1.30.0-alpha"\n',
            encoding="utf-8",
        )

        expect_failure(
            "hardcoded_cli_version",
            root,
            "stale active version claim",
        )

        main.write_text(
            valid_main.replace(
                'default_value = "v1.30"\n',
                "",
            ),
            encoding="utf-8",
        )

        expect_failure(
            "compatibility_selector_removed",
            root,
            "compatibility marker missing",
        )

        main.write_text(
            valid_main,
            encoding="utf-8",
        )

        guide = root / "docs/guide/src/title.md"

        guide.write_text(
            "**Historical guide baseline:** "
            "v1.45.0-alpha (non-authoritative)\n",
            encoding="utf-8",
        )

        validate_repository(root)
        print(
            "self_test_historical_reference_allowed=PASS"
        )

    print(
        "version_reference_hygiene_self_test=PASS"
    )


def main() -> int:
    run_self_test()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
