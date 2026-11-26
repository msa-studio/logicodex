#!/usr/bin/env python3
"""Enforce canonical Logicodex version-reference hygiene."""

from __future__ import annotations

from dataclasses import dataclass
import os
from pathlib import Path
import re
import sys

from version_reference_hygiene_runtime import (
    APPROVED_LLVM_MAJOR,
    APPROVED_RUST_MINOR,
    RuntimeVersionValidationError,
    validate_active_runtime_labels,
    validate_toolchain_references,
)


PRIMARY_SURFACES = (
    ".github/SECURITY.md",
    "src/main.rs",
    "docs/VS_CODE_EXTENSION.md",
    "docs/examples/REFLEX_ENGINE_EXAMPLES.md",
    "docs/guide/src/title.md",
    "docs/wiki/README.md",
    "extensions/vscode-logicodex/README.md",
    "extensions/vscode-logicodex/package.json",
    "extensions/vscode-logicodex/src/previewNormalizer.ts",
)

HOOK_FILES = (
    "scripts/dev/verify_quick_integrity.sh",
    "scripts/dev/verify_full_integrity.sh",
)

HISTORICAL_ZONE_REQUIREMENTS = (
    (
        "docs/archive/.zone-status.md",
        "HistoricalProvenance",
    ),
    (
        "scripts/_archive/.zone-status.md",
        "ArchivedZone",
    ),
    (
        "spec/v1.11-alpha/.zone-status.md",
        "HistoricalProvenance",
    ),
    (
        "spec/v1.21-alpha/.zone-status.md",
        "HistoricalProvenance",
    ),
    (
        "spec/v1.30.0-alpha/.zone-status.md",
        "HistoricalProvenance",
    ),
)

FORBIDDEN_CURRENT_CLAIMS = (
    re.compile(
        r"current Logicodex v1\.(?:21|30|45)",
        re.IGNORECASE,
    ),
    re.compile(
        r"current logicodex v 1\.21",
        re.IGNORECASE,
    ),
    re.compile(
        r"v1\.30\.0-alpha \(current engine\)",
        re.IGNORECASE,
    ),
    re.compile(
        r"LOGICODEX COMPILER v1\.30\.0-alpha",
    ),
    re.compile(
        r"logicodex 1\.30\.0-alpha",
    ),
    re.compile(
        r'version\s*=\s*"1\.30\.0-alpha"',
    ),
)


RELEASE_WORKFLOW = ".github/workflows/release.yml"

RELEASE_BUILD_JOBS = {
    "build-linux": "ubuntu-24.04",
    "build-macos-x86": "macos-15-intel",
    "build-macos-aarch64": "macos-14",
}

RELEASE_PUBLISH_CONDITION = (
    "${{ (github.event_name == 'push' && "
    "startsWith(github.ref, 'refs/tags/v')) || "
    "(github.event_name == 'workflow_dispatch' && "
    "inputs.publish == true) }}"
)

RUST_TOOLCHAIN_ACTION = "dtolnay/rust-toolchain@1.100.0"
RELEASE_BUILD_COMMAND = "cargo build --release --locked"

# Exact executable shell lines (comments excluded) per job.
RELEASE_VERIFY_LINES = (
    "[[ \"$(rustc --version | awk '{print $2}')\" == '1.100.0' ]]",
    "[[ \"$(cargo --version | awk '{print $2}')\" == '1.100.0' ]]",
)

RELEASE_LINUX_ENV = {
    "LLVM_SYS_231_PREFIX": "/usr/lib/llvm-23",
    "LLVM_CONFIG_PATH": "/usr/lib/llvm-23/bin/llvm-config",
    "RUSTFLAGS": "-L/usr/lib/llvm-23/lib",
}

RELEASE_LINUX_INSTALL_LINES = (
    "sudo apt-get install -y llvm-23-dev libclang-23-dev "
    "libpolly-23-dev libzstd-dev pkg-config",
    '[[ "$version" =~ ^1:23\\.1\\.6~ ]]',
    "[[ \"$(/usr/lib/llvm-23/bin/llvm-config --version)\" "
    "== '23.1.6' ]]",
)

RELEASE_LINUX_APT_SOURCE = (
    "https://apt.llvm.org/noble/ llvm-toolchain-noble-23 main"
)

RELEASE_MACOS_INSTALL_COMMAND = "brew install llvm"

RELEASE_MACOS_ENV_LINES = (
    'prefix="$(brew --prefix llvm)"',
    'echo "LLVM_SYS_231_PREFIX=$prefix" >> "$GITHUB_ENV"',
    'echo "LDFLAGS=-L$prefix/lib" >> "$GITHUB_ENV"',
    'echo "CPPFLAGS=-I$prefix/include" >> "$GITHUB_ENV"',
    'echo "$prefix/bin" >> "$GITHUB_PATH"',
)

RELEASE_MACOS_VERIFY_LINE = "[[ \"${llvm_version%%.*}\" == '23' ]]"

RELEASE_CACHE_KEY_MARKERS = {
    "build-linux": "-ubuntu24.04-llvm23.1.6-rust1.100.0-cargo-release-",
    "build-macos-x86": "-llvm23-rust1.100.0-cargo-release-",
    "build-macos-aarch64": "-llvm23-rust1.100.0-cargo-release-",
}

RELEASE_TRIPLES = {
    "build-linux": "x86_64-unknown-linux-gnu",
    "build-macos-x86": "x86_64-apple-darwin",
    "build-macos-aarch64": "aarch64-apple-darwin",
}

RELEASE_BINARY_ARCH = {
    "build-linux": "x86-64",
    "build-macos-x86": "x86_64",
    "build-macos-aarch64": "arm64",
}

RELEASE_ACTION_PREFIX = "softprops/action-gh-release@"

PUBLISH_ONLY_MARKERS = (
    "contents: write",
    RELEASE_ACTION_PREFIX,
    "secrets.",
    "environment:",
)


class ValidationError(RuntimeError):
    """Raised when version-reference hygiene fails."""

    def __init__(self, errors: list[str]) -> None:
        self.errors = errors
        super().__init__("\n".join(errors))


@dataclass(frozen=True)
class ValidationSummary:
    cargo_version: str
    checked_surfaces: int
    compatibility_selectors: int
    active_runtime_surfaces: int
    generated_version_authorities: int
    historical_zone_markers: int
    toolchain_scanned_paths: int
    toolchain_staged_blobs: int
    toolchain_historical_exceptions: int
    toolchain_base_sha: str | None
    release_workflow_jobs: int


def read_text(root: Path, relative: str) -> str:
    path = root / relative

    if not path.is_file():
        raise ValidationError(
            [f"required file is missing: {relative}"]
        )

    return path.read_text(
        encoding="utf-8",
        errors="strict",
    )


def cargo_version(root: Path) -> str:
    text = read_text(root, "Cargo.toml")

    match = re.search(
        r'(?m)^version\s*=\s*"([^"]+)"\s*$',
        text,
    )

    if not match:
        raise ValidationError(
            ["Cargo.toml package version is missing"]
        )

    return match.group(1)


class WorkflowParseError(ValueError):
    """Raised when a workflow leaves the supported block-YAML subset."""


_YAML_KEY_RE = re.compile(
    r"^(?P<key>[A-Za-z0-9_][A-Za-z0-9_.-]*|'[^']*'|\"[^\"]*\")"
    r":(?:[ ]+(?P<rest>.*))?$"
)


def _strip_yaml_comment(text: str) -> str:
    """Drop a trailing ``#`` comment from a plain or quoted scalar."""

    if text.startswith(("'", '"')):
        quote = text[0]
        index = 1

        while index < len(text):
            if text[index] == quote:
                if quote == "'" and text[index + 1 : index + 2] == "'":
                    index += 2
                    continue
                break

            if quote == '"' and text[index] == "\\":
                index += 1

            index += 1
        else:
            raise WorkflowParseError(f"unterminated quoted scalar: {text!r}")

        tail = text[index + 1 :]

        if tail.strip() and not tail.lstrip().startswith("#"):
            raise WorkflowParseError(f"text after quoted scalar: {text!r}")

        return text[: index + 1]

    if text.startswith("#"):
        return ""

    match = re.search(r"\s#", text)
    return (text[: match.start()] if match else text).rstrip()


def _yaml_scalar(text: str) -> str:
    if not text:
        return ""

    if text[0] in "[{&*!|>%@`":
        raise WorkflowParseError(f"unsupported YAML construct: {text!r}")

    if text.startswith("'"):
        return text[1:-1].replace("''", "'")

    if text.startswith('"'):
        if "\\" in text:
            raise WorkflowParseError(f"unsupported escape in: {text!r}")
        return text[1:-1]

    return text


class _BlockYaml:
    """Strict parser for the block-style YAML subset used by workflows.

    Supports nested mappings, ``-`` sequences, plain/quoted scalars and
    ``|``/``>`` block scalars. Comments are discarded; anything else
    (flow collections, anchors, tags, tabs, duplicate keys) fails
    closed rather than being guessed at.
    """

    def __init__(self, text: str) -> None:
        self.lines = text.split("\n")
        self.index = 0

    def _skip_blank(self) -> None:
        while self.index < len(self.lines):
            stripped = self.lines[self.index].strip()

            if stripped and not stripped.startswith("#"):
                return

            self.index += 1

    def _peek(self) -> tuple[int, str] | None:
        self._skip_blank()

        if self.index >= len(self.lines):
            return None

        raw = self.lines[self.index]
        content = raw.lstrip(" ")

        if content.startswith("\t"):
            raise WorkflowParseError(
                f"line {self.index + 1}: tab indentation"
            )

        return len(raw) - len(content), content.rstrip()

    def parse(self) -> object:
        first = self._peek()

        if first is None:
            raise WorkflowParseError("empty workflow")

        node = self._node(first[0])

        if self._peek() is not None:
            raise WorkflowParseError(
                f"line {self.index + 1}: unexpected indentation"
            )

        return node

    def _node(self, indent: int) -> object:
        peeked = self._peek()
        assert peeked is not None

        if peeked[1] == "-" or peeked[1].startswith("- "):
            return self._sequence(indent)

        return self._mapping(indent, None)

    def _sequence(self, indent: int) -> list[object]:
        items: list[object] = []

        while (peeked := self._peek()) and peeked[0] == indent:
            content = peeked[1]

            if not (content == "-" or content.startswith("- ")):
                break

            rest = content[1:].lstrip(" ")

            if not rest or rest.startswith("#"):
                self.index += 1
                items.append(self._child(indent))
            elif _YAML_KEY_RE.match(rest):
                items.append(
                    self._mapping(indent + len(content) - len(rest), rest)
                )
            else:
                self.index += 1
                items.append(_yaml_scalar(_strip_yaml_comment(rest)))

        return items

    def _child(self, indent: int) -> object:
        peeked = self._peek()

        if peeked is None:
            return ""

        if peeked[0] > indent:
            return self._node(peeked[0])

        if peeked[0] == indent and (
            peeked[1] == "-" or peeked[1].startswith("- ")
        ):
            return self._sequence(indent)

        return ""

    def _mapping(self, indent: int, first: str | None) -> dict[str, object]:
        mapping: dict[str, object] = {}

        while True:
            if first is None:
                peeked = self._peek()

                if peeked is None or peeked[0] != indent:
                    break

                content = peeked[1]

                if content == "-" or content.startswith("- "):
                    break
            else:
                content, first = first, None

            match = _YAML_KEY_RE.match(content)

            if not match:
                raise WorkflowParseError(
                    f"line {self.index + 1}: expected 'key: value', "
                    f"got {content!r}"
                )

            key = _yaml_scalar(match.group("key"))

            if key in mapping:
                raise WorkflowParseError(
                    f"line {self.index + 1}: duplicate key {key!r}"
                )

            rest = _strip_yaml_comment(match.group("rest") or "")
            self.index += 1

            if not rest:
                mapping[key] = self._child(indent)
            elif rest[0] in "|>":
                if not re.fullmatch(r"[|>][+-]?", rest):
                    raise WorkflowParseError(
                        f"line {self.index}: unsupported block header "
                        f"{rest!r}"
                    )
                mapping[key] = self._block_scalar(indent)
            else:
                mapping[key] = _yaml_scalar(rest)

        return mapping

    def _block_scalar(self, indent: int) -> str:
        body: list[str] = []
        block_indent: int | None = None

        while self.index < len(self.lines):
            raw = self.lines[self.index]

            if not raw.strip():
                body.append("")
                self.index += 1
                continue

            current = len(raw) - len(raw.lstrip(" "))

            if current <= indent:
                break

            if block_indent is None:
                block_indent = current

            if current < block_indent:
                raise WorkflowParseError(
                    f"line {self.index + 1}: block scalar under-indented"
                )

            body.append(raw[block_indent:])
            self.index += 1

        return "\n".join(body).strip("\n")


def parse_workflow(text: str) -> dict[str, object]:
    document = _BlockYaml(text).parse()

    if not isinstance(document, dict):
        raise WorkflowParseError("workflow root must be a mapping")

    return document


def _shell_lines(script: object) -> list[str]:
    """Executable shell lines with comments and spacing normalised."""

    if not isinstance(script, str):
        return []

    return [
        " ".join(line.split())
        for line in script.splitlines()
        if line.strip() and not line.strip().startswith("#")
    ]


def _strings(node: object, prefix: str = "") -> list[str]:
    """Flatten parsed YAML to ``key: value`` lines (comments excluded)."""

    if isinstance(node, dict):
        out: list[str] = []

        for key, value in node.items():
            if isinstance(value, str):
                lines = (
                    _shell_lines(value) if key == "run" else [value]
                )
                out.extend(f"{key}: {line}" for line in lines)
            else:
                out.append(f"{key}:")
                out.extend(_strings(value))

        return out

    if isinstance(node, list):
        return [line for item in node for line in _strings(item)]

    return [str(node)]


def _mapping_at(node: object, *path: str) -> dict[str, object]:
    for key in path:
        node = node.get(key) if isinstance(node, dict) else None

    return node if isinstance(node, dict) else {}


def _step_index(
    steps: list[dict[str, object]],
    predicate,
) -> int | None:
    return next(
        (index for index, step in enumerate(steps) if predicate(step)),
        None,
    )


def _runs_lines(step: dict[str, object], *lines: str) -> bool:
    executable = _shell_lines(step.get("run"))
    return all(line in executable for line in lines)


def _validate_build_job(
    job_name: str,
    job: dict[str, object],
    prefix: str,
) -> list[str]:
    errors: list[str] = []
    runner = RELEASE_BUILD_JOBS[job_name]
    linux = job_name == "build-linux"

    if job.get("runs-on") != runner:
        errors.append(prefix + f"{job_name} must run on {runner}")

    if "if" in job:
        errors.append(
            prefix + f"{job_name} must not be conditional; build jobs "
            "always run"
        )

    flat = "\n".join(_strings(job))

    for marker in PUBLISH_ONLY_MARKERS:
        if marker in flat:
            errors.append(
                prefix + f"{job_name} must not reference {marker!r}"
            )

    raw_steps = job.get("steps")
    steps = (
        [step for step in raw_steps if isinstance(step, dict)]
        if isinstance(raw_steps, list)
        else []
    )

    if not steps or len(steps) != len(raw_steps):
        errors.append(prefix + f"{job_name} steps must be a list of mappings")

    def missing(marker: str) -> None:
        errors.append(
            prefix + f"{job_name} missing toolchain marker {marker!r}"
        )

    triple_line = (
        "[[ \"$(rustc -vV | awk '$1 == \"host:\" {print $2}')\" "
        f"== '{RELEASE_TRIPLES[job_name]}' ]]"
    )
    arch_line = (
        "file target/release/logicodex | grep -Fq "
        f"'{RELEASE_BINARY_ARCH[job_name]}'"
    )
    cache_marker = RELEASE_CACHE_KEY_MARKERS[job_name]

    if linux:
        env = _mapping_at(job, "env")

        for key, value in RELEASE_LINUX_ENV.items():
            if env.get(key) != value:
                missing(f"{key}: {value}")

        llvm_install = _step_index(
            steps,
            lambda step: _runs_lines(step, *RELEASE_LINUX_INSTALL_LINES)
            and any(
                RELEASE_LINUX_APT_SOURCE in line
                for line in _shell_lines(step.get("run"))
            ),
        )

        if llvm_install is None:
            for line in RELEASE_LINUX_INSTALL_LINES + (
                RELEASE_LINUX_APT_SOURCE,
            ):
                missing(line)

        llvm_verify_lines: tuple[str, ...] = (
            RELEASE_LINUX_INSTALL_LINES[-1],
        )
    else:
        brew = _step_index(
            steps,
            lambda step: _shell_lines(step.get("run"))
            == [RELEASE_MACOS_INSTALL_COMMAND],
        )
        llvm_env = _step_index(
            steps,
            lambda step: _runs_lines(step, *RELEASE_MACOS_ENV_LINES),
        )

        if brew is None:
            missing(f"run: {RELEASE_MACOS_INSTALL_COMMAND}")

        if llvm_env is None:
            for line in RELEASE_MACOS_ENV_LINES:
                missing(line)

        if None not in (brew, llvm_env) and not brew < llvm_env:
            errors.append(
                prefix + f"{job_name} must install LLVM before "
                "exporting its environment"
            )

        llvm_install = llvm_env
        llvm_verify_lines = (RELEASE_MACOS_VERIFY_LINE,)

    rust_install = _step_index(
        steps,
        lambda step: step.get("uses") == RUST_TOOLCHAIN_ACTION,
    )
    verify = _step_index(
        steps,
        lambda step: _runs_lines(
            step,
            *RELEASE_VERIFY_LINES,
            *llvm_verify_lines,
            triple_line,
        ),
    )
    cache = _step_index(
        steps,
        lambda step: str(step.get("uses", "")).startswith("actions/cache@")
        and cache_marker in str(_mapping_at(step, "with").get("key", ""))
        and cache_marker in str(
            _mapping_at(step, "with").get("restore-keys", "")
        ),
    )
    build = _step_index(
        steps,
        lambda step: _shell_lines(step.get("run")) == [RELEASE_BUILD_COMMAND],
    )
    arch = _step_index(
        steps,
        lambda step: _runs_lines(step, arch_line),
    )

    if rust_install is None:
        missing(f"uses: {RUST_TOOLCHAIN_ACTION}")

    if verify is None:
        for line in (*RELEASE_VERIFY_LINES, *llvm_verify_lines, triple_line):
            missing(line)

    if cache is None:
        missing(cache_marker)

    if build is None:
        missing(f"run: {RELEASE_BUILD_COMMAND}")

    if arch is None:
        missing(arch_line)

    order = [llvm_install, rust_install, verify, build, arch]

    if None not in order and order != sorted(order):
        errors.append(
            prefix + f"{job_name} steps must run LLVM setup, Rust "
            "install, toolchain verification, release build, then "
            "binary verification in order"
        )

    return errors


def validate_release_workflow(root: Path) -> list[str]:
    """Check the release toolchain and fail-closed publication gate.

    The workflow is parsed into its job/step structure, so markers in
    comments, shell comments, or unrelated steps do not satisfy it.
    """

    prefix = f"{RELEASE_WORKFLOW}: "

    try:
        text = read_text(root, RELEASE_WORKFLOW)
    except ValidationError as error:
        return error.errors

    try:
        workflow = parse_workflow(text)
    except WorkflowParseError as error:
        return [prefix + f"unparseable workflow structure: {error}"]

    errors: list[str] = []
    jobs = _mapping_at(workflow, "jobs")

    if not jobs:
        return [prefix + "jobs section missing"]

    if workflow.get("permissions") != {"contents": "read"}:
        errors.append(prefix + "global permissions must be contents: read")

    if "contents: write" in "\n".join(
        _strings({key: value for key, value in workflow.items() if key != "jobs"})
    ):
        errors.append(prefix + "contents: write outside the release job")

    inputs = _mapping_at(workflow, "on", "workflow_dispatch", "inputs")
    tag = _mapping_at(inputs, "tag")
    publish = _mapping_at(inputs, "publish")

    if not (tag.get("required") == "true" and tag.get("type") == "string"):
        errors.append(prefix + "dispatch input tag must be a required string")

    if not (
        publish.get("required") == "true"
        and publish.get("type") == "boolean"
        and publish.get("default") == "false"
    ):
        errors.append(
            prefix
            + "dispatch input publish must be a required boolean "
            "defaulting to false"
        )

    expected_jobs = set(RELEASE_BUILD_JOBS) | {"release"}

    if set(jobs) != expected_jobs:
        errors.append(
            prefix
            + f"jobs must be exactly {sorted(expected_jobs)}, "
            f"found {sorted(jobs)}"
        )

    for job_name in RELEASE_BUILD_JOBS:
        job = jobs.get(job_name)

        if not isinstance(job, dict):
            errors.append(prefix + f"{job_name} job missing")
            continue

        errors.extend(_validate_build_job(job_name, job, prefix))

    release = jobs.get("release")
    release = release if isinstance(release, dict) else {}

    if release.get("if") != RELEASE_PUBLISH_CONDITION:
        errors.append(
            prefix + "release job must carry the exact fail-closed "
            "publication condition"
        )

    needs = release.get("needs")

    if not (
        isinstance(needs, list)
        and sorted(needs) == sorted(RELEASE_BUILD_JOBS)
    ):
        errors.append(prefix + "release job must need every build job")

    if release.get("environment") != "alpha-releases":
        errors.append(prefix + "release job missing 'environment: alpha-releases'")

    if release.get("permissions") != {"contents": "write"}:
        errors.append(
            prefix + "release job missing 'permissions:\\n  contents: write'"
        )

    release_steps = release.get("steps")
    publish_steps = [
        step
        for step in (release_steps if isinstance(release_steps, list) else [])
        if isinstance(step, dict)
        and str(step.get("uses", "")).startswith(RELEASE_ACTION_PREFIX)
    ]

    if len(publish_steps) != 1:
        errors.append(
            prefix + f"release job must publish with exactly one "
            f"{RELEASE_ACTION_PREFIX} step"
        )

    for job_name, job in jobs.items():
        if job_name == "release" or job_name in RELEASE_BUILD_JOBS:
            continue

        flat = "\n".join(_strings(job))

        for marker in PUBLISH_ONLY_MARKERS:
            if marker in flat:
                errors.append(
                    prefix + f"{marker!r} must appear only inside the "
                    "release job"
                )

    return errors


def validate_repository(
    root: Path,
    base_sha: str | None = None,
) -> ValidationSummary:
    errors: list[str] = []
    version = cargo_version(root)

    try:
        toolchain_summary = validate_toolchain_references(
            root,
            base_sha,
        )
    except RuntimeVersionValidationError as error:
        errors.extend(error.errors)
        toolchain_summary = None

    errors.extend(validate_release_workflow(root))

    try:
        runtime_summary = (
            validate_active_runtime_labels(root)
        )
    except RuntimeVersionValidationError as error:
        errors.extend(error.errors)
        runtime_summary = None

    surfaces: dict[str, str] = {}

    for relative in PRIMARY_SURFACES:
        try:
            surfaces[relative] = read_text(
                root,
                relative,
            )
        except ValidationError as error:
            errors.extend(error.errors)

    for relative, text in surfaces.items():
        for pattern in FORBIDDEN_CURRENT_CLAIMS:
            for match in pattern.finditer(text):
                line = text.count(
                    "\n",
                    0,
                    match.start(),
                ) + 1

                errors.append(
                    f"{relative}:{line}: stale active "
                    f"version claim: {match.group(0)!r}"
                )

    main = surfaces.get("src/main.rs", "")

    if main.count('env!("CARGO_PKG_VERSION")') < 3:
        errors.append(
            "src/main.rs must derive banner, long version, "
            "and Clap version from CARGO_PKG_VERSION"
        )

    compatibility_markers = (
        'default_value = "v1.30"',
        'name = "v130-check"',
        "CompilerPipeline::V130",
    )

    for marker in compatibility_markers:
        if marker not in main:
            errors.append(
                "src/main.rs compatibility marker missing: "
                + marker
            )

    security = surfaces.get(
        ".github/SECURITY.md",
        "",
    )

    expected_security = (
        f"| v{version} (current Cargo release authority) "
        "| ✅ Current — best-effort alpha security support |"
    )

    if expected_security not in security:
        errors.append(
            ".github/SECURITY.md does not identify the "
            "Cargo version as current authority"
        )

    expected_current_markers = {
        "docs/VS_CODE_EXTENSION.md":
            f"current Logicodex v{version}",
        "docs/wiki/README.md":
            f"Logicodex Documentation — v{version}",
        "extensions/vscode-logicodex/README.md":
            f"current Logicodex v{version}",
        "extensions/vscode-logicodex/package.json":
            f"current Logicodex v{version}",
    }

    for relative, marker in expected_current_markers.items():
        if marker not in surfaces.get(relative, ""):
            errors.append(
                f"{relative}: canonical version marker missing"
            )

    guide = surfaces.get(
        "docs/guide/src/title.md",
        "",
    )

    if (
        "**Historical guide baseline:** v1.45.0-alpha "
        "(non-authoritative)"
        not in guide
    ):
        errors.append(
            "stale guide must label v1.45 as a historical, "
            "non-authoritative baseline"
        )

    reflex = surfaces.get(
        "docs/examples/REFLEX_ENGINE_EXAMPLES.md",
        "",
    )

    if (
        "current single HIR compiler path"
        not in reflex
        or "deprecated compatibility alias"
        not in reflex
    ):
        errors.append(
            "reflex examples must describe the current "
            "single-engine compatibility boundary"
        )

    preview = surfaces.get(
        "extensions/vscode-logicodex/"
        "src/previewNormalizer.ts",
        "",
    )

    if (
        "current single HIR compiler path is authoritative"
        not in preview
    ):
        errors.append(
            "Side View notice must point to the current "
            "single HIR compiler path"
        )

    for relative, expected_classification in (
        HISTORICAL_ZONE_REQUIREMENTS
    ):
        try:
            marker = read_text(root, relative)
        except ValidationError:
            errors.append(
                f"historical zone marker missing: {relative}"
            )
            continue

        expected_marker = (
            "Classification: "
            f"`{expected_classification}`"
        )

        if expected_marker not in marker:
            errors.append(
                "historical zone classification mismatch: "
                f"{relative} must contain {expected_marker!r}"
            )

    for relative in HOOK_FILES:
        try:
            hook = read_text(root, relative)
        except ValidationError as error:
            errors.extend(error.errors)
            continue

        for script in (
            "test_version_reference_hygiene.py",
            "verify_version_reference_hygiene.py",
        ):
            if script not in hook:
                errors.append(
                    f"{relative}: missing gate for {script}"
                )

        if 'BASE_SHA="${BASE_SHA:-}"' not in hook:
            errors.append(
                f"{relative}: version hygiene gate must forward BASE_SHA"
            )

    ci = read_text(
        root,
        ".github/workflows/ci.yml",
    )

    for hook in HOOK_FILES:
        for script in (
            "test_version_reference_hygiene.py",
            "verify_version_reference_hygiene.py",
        ):
            expected = (
                f"grep -Fq '{script}' {hook}"
            )

            if expected not in ci:
                errors.append(
                    ".github/workflows/ci.yml: "
                    f"missing hook assertion: {expected}"
                )

        expected = f"grep -Fq 'BASE_SHA=\"${{BASE_SHA:-}}\"' {hook}"

        if expected not in ci:
            errors.append(
                ".github/workflows/ci.yml: "
                f"missing hook assertion: {expected}"
            )

    for script in (
        "test_version_reference_hygiene.py",
        "verify_version_reference_hygiene.py",
    ):
        expected = f"python3 -B scripts/dev/{script}"

        if expected not in ci:
            errors.append(
                ".github/workflows/ci.yml: "
                f"must execute {expected}"
            )

    if errors:
        raise ValidationError(errors)

    assert runtime_summary is not None
    assert toolchain_summary is not None

    return ValidationSummary(
        cargo_version=version,
        checked_surfaces=len(surfaces),
        compatibility_selectors=3,
        active_runtime_surfaces=(
            runtime_summary.checked_surfaces
        ),
        generated_version_authorities=(
            runtime_summary.generated_version_authorities
        ),
        historical_zone_markers=len(
            HISTORICAL_ZONE_REQUIREMENTS
        ),
        toolchain_scanned_paths=(
            toolchain_summary.scanned_paths
        ),
        toolchain_staged_blobs=(
            toolchain_summary.staged_blobs
        ),
        toolchain_historical_exceptions=(
            toolchain_summary.historical_exceptions
        ),
        toolchain_base_sha=toolchain_summary.base_sha,
        release_workflow_jobs=len(RELEASE_BUILD_JOBS) + 1,
    )


def main() -> int:
    root = Path(
        sys.argv[1]
        if len(sys.argv) > 1
        else "."
    ).resolve()

    base_sha = os.environ.get("BASE_SHA", "").strip() or None

    try:
        summary = validate_repository(root, base_sha)
    except ValidationError as error:
        for message in error.errors:
            print(
                f"version_reference_hygiene_error={message}",
                file=sys.stderr,
            )

        return 1

    print(
        "version_reference_hygiene=PASS"
    )
    print(
        f"cargo_version={summary.cargo_version}"
    )
    print(
        f"checked_surfaces={summary.checked_surfaces}"
    )
    print(
        "compatibility_selectors_preserved="
        f"{summary.compatibility_selectors}"
    )
    print(
        "active_runtime_surfaces="
        f"{summary.active_runtime_surfaces}"
    )
    print(
        "generated_version_authorities="
        f"{summary.generated_version_authorities}"
    )
    print(
        "historical_zone_markers="
        f"{summary.historical_zone_markers}"
    )
    print(
        "toolchain_scanned_paths="
        f"{summary.toolchain_scanned_paths}"
    )
    print(
        "toolchain_policy="
        f"llvm{APPROVED_LLVM_MAJOR},rust{APPROVED_RUST_MINOR}"
    )
    print(
        "toolchain_staged_blobs="
        f"{summary.toolchain_staged_blobs}"
    )
    print(
        "toolchain_historical_exceptions="
        f"{summary.toolchain_historical_exceptions}"
    )
    print(
        "toolchain_base_sha="
        f"{summary.toolchain_base_sha or 'unset'}"
    )
    print(
        "release_workflow_contract=PASS "
        f"jobs={summary.release_workflow_jobs}"
    )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
