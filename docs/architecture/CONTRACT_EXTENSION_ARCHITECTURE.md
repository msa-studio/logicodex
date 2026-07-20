# Logicodex Contract Extension Architecture

Status: architecture direction lock
Scope: stdlib, framework, FFI/library imports, runtime profiles, language frontends, IL semantic adapters, diagnostics, and agent-facing tooling.

This subordinate architecture record defines the general Logicodex pattern for
contract-compliant static extensions. Current cross-project authority and work
sequence are owned by [`current-authority.md`](current-authority.md).

It complements:

```
docs/stdlib/CONTRACT.md
```

The purpose is to prevent Logicodex from becoming a compiler full of one-off hardcoded features while still allowing the ecosystem to grow.

---

## 1. Core Decision

Logicodex integrates extension categories, not every individual implementation.

The compiler may understand stable categories such as:

```
official library namespaces
runtime profile categories
FFI contract categories
language frontend families
IL semantic adapters
diagnostic contract families
agent feature contracts
```

The compiler must not hardcode every module, package, native library, framework, frontend, backend, or tool integration.

Canonical rule:

```
New module/package/adapter = implementation + contract metadata.
New semantic capability = compiler/runtime design change.
```

A module that only adds behavior inside an existing category should be declared through contract metadata.

A feature that changes language meaning, type rules, runtime requirements, capability gates, ABI behavior, or diagnostics belongs in compiler/runtime architecture.

---

## 2. Not a Runtime Plugin System

This architecture is not a dynamic runtime plugin system.

Preferred terms:

```
contract-compliant static extension
contract extension
contract-bound adapter
contract-declared capability
contract-verified module
```

Avoid using these as the main architecture terms:

```
runtime plugin
arbitrary plugin mutation
dynamic monkey patch
compiler hook without contract
silent extension fallback
```

Extensions must not mutate compiler meaning through undocumented hooks.

If an extension affects language meaning, type checking, capability behavior, code generation, runtime behavior, or diagnostics, it must do so through a declared contract and an approved category.

---

## 3. Logicodex Identity Invariants

Every contract extension must preserve these Logicodex invariants:

```
Meaning Authority:
  Logicodex preserves declared meaning instead of silently guessing.

No Silent Fallback:
  unsupported behavior must fail explicitly.

Implementation >= Claim:
  a module or adapter must not claim support beyond what it validates.

Profile-Aware:
  behavior must declare assumptions such as bare, std, safe, actor, or service.

Capability-Aware:
  unsafe or external behavior must declare capabilities.

Low Runtime Overhead:
  validation should happen at compile, dev, or CI time where possible.

Evidence-Backed:
  claims should be backed by contract metadata, tests, examples, and diagnostics.

AI-Queryable:
  future diagnostics and contracts should be structured enough for tools and agents to query.
```

---

## 4. Extension Families

The same contract pattern should be reused across these families.

### 4.1 Standard Library and Framework Contracts

Purpose:

```
declare official core.*, std.*, and framework.* modules
```

Examples:

```
core.math
core.assert
std.io
std.fs
framework.http
```

Expected files:

```
.ldx implementation
.std.toml or equivalent contract metadata
acceptance tests
```

Compiler responsibility:

```
recognize official namespace categories
resolve library roots
validate contracts in explicit validation mode
avoid per-module hardcoding
```

### 4.2 FFI and Native Library Contracts

Purpose:

```
import external C/native libraries safely and explicitly
```

Examples:

```
libm
sqlite3
raylib
platform SDKs
device SDKs
```

Expected contract fields:

```
library name
ABI
symbols
types
calling convention
version/hash expectations
capability requirement
profile requirement
unsafe boundary
diagnostics
```

Compiler responsibility:

```
enforce ABI declaration
reject undeclared symbol usage
prevent silent fallback to unknown libraries
connect with capability gates
```

### 4.3 Runtime Profile Contracts

Purpose:

```
declare assumptions for bare, std, safe, actor, and service profiles
```

Examples:

```
bare-compatible core module
std module that needs OS exit
actor module that needs actor runtime
service module that needs networking
```

Expected contract fields:

```
profile
required services
forbidden imports
runtime assumptions
diagnostics
```

Compiler/runtime responsibility:

```
refuse invalid profile combinations
surface clear diagnostics
keep bare builds clean
```

### 4.4 Language Frontend Contracts

Purpose:

```
allow future frontends to map external languages into Logicodex meaning
```

Examples:

```
C frontend
COBOL frontend
future Fortran subset
educational DSL frontend
```

Expected contract fields:

```
source language
supported subset
unsupported constructs
mapping to HIR/IL
type semantics
memory semantics
safety/capability implications
test corpus
```

Compiler responsibility:

```
accept frontend output only through approved IR/HIR boundaries
reject unsupported semantics explicitly
avoid pretending incomplete translation is complete
```

### 4.5 IL Semantic Contracts

Purpose:

```
let Logicodex understand intermediate language meaning as a stable contract, not only opaque codegen output
```

Examples:

```
public HIR export/import
LDB bytecode
semantic operator tables
typed dispatch metadata
```

Expected contract fields:

```
operator set
type rules
control-flow rules
capability annotations
profile annotations
diagnostics metadata
```

Compiler responsibility:

```
preserve meaning
reject invalid or unsupported IL
make diagnostics queryable
```

### 4.6 Diagnostic and Agent Contracts

Purpose:

```
make diagnostics structured, causal, and AI-queryable.
```

Examples:

```
LDX-DIP error families
contract validation errors
capability gate errors
profile mismatch errors
FFI ABI errors
reviewer/planner/coder agent boundaries
```

Expected contract fields:

```
error code
stage
component
source span
rule violated
evidence
safe fix suggestion
machine-readable fields
```

Compiler/tooling responsibility:

```
emit causal diagnostics
avoid vague fallback errors
connect errors to contracts where possible
avoid granting authority to unverified agent claims
```

---

## 5. General Contract Shape

A contract extension should declare at least:

```
identity:
  name, kind, version, stage, status

scope:
  what it supports
  what it does not support

layer/category:
  core, std, framework, ffi, frontend, il, runtime-profile, diagnostic, agent

inputs:
  files, symbols, imports, source language, or metadata consumed

outputs:
  exports, generated IR, diagnostics, symbols, runtime services

profiles:
  bare, std, safe, actor, service, or future profiles

capabilities:
  required capabilities and forbidden capabilities

constraints:
  allowed imports
  forbidden imports
  allowed unsafe boundary
  forbidden features

validation:
  metadata checks
  static checks
  compile checks
  run checks
  negative checks
  stress/property checks if enabled

limits:
  timeout
  output size
  max cases
  resource constraints

diagnostics:
  expected error codes
  structured fields
  safe fix suggestions

evidence:
  tests
  examples
  hashes
  version pins
  known limitations
```

---

## 6. Validation Modes

Validation should be feature-activated. Normal compilation should stay fast.

General validation modes:

```
quick:
  metadata and shape validation

standard:
  metadata, static checks, and compile validation

full:
  standard plus negative tests and compatibility checks

stress:
  reserved for bounded stress/property tests
```

Heavy validation belongs in explicit verify commands, CI, or developer tooling.

---

## 7. Relationship with LDX-DIP

The Logicodex Diagnostic Intelligence Pipeline should consume contract facts and produce structured diagnostics.

Contracts provide:

```
declared intent
capability requirements
profile assumptions
allowed and forbidden behavior
test/evidence expectations
```

Diagnostics provide:

```
actual failure
causal explanation
rule violated
source span if available
suggested safe fix
machine-readable facts
```

Together:

```
contract = declared meaning and expected behavior
diagnostic = structured evidence when behavior violates meaning
```

This makes Logicodex easier for humans and agents to debug without making agents the authority.

---

## 8. No Silent Fallback Rule

When contract validation fails, Logicodex must not pretend success.

Forbidden:

```
missing contract treated as official support
unknown field ignored in strict mode
unsupported import resolved as a different module
missing export treated as valid
FFI symbol guessed
frontend semantic gap silently lowered
runtime profile mismatch ignored
diagnostic evidence omitted
```

Required:

```
explicit error
clear rule
component name
contract path if available
safe next action
```

---

## 9. Community vs Enterprise Boundary

Community-safe contract architecture may include:

```
parser/AST/HIR/type checker/module system support
stdlib contract metadata
basic capability gates
basic FFI contract foundation
public diagnostics
public tests
public docs and examples
```

Future Enterprise/Assurance features may extend the pattern with stronger enforcement, signing, attestation, governance, registry, and audit evidence.

Do not implement private Enterprise enforcement inside Community before the project split.

Community should remain production-grade, not a toy, but its contract architecture must stay public-safe.

---

## 10. Historical Foundation Order

Recorded bootstrap order for this subsystem (not current project work sequence):

```
1. stdlib contract docs
2. contract extension architecture doc
3. rename std_root concept to library_root
4. add framework namespace reservation
5. add lib/core/math.std.toml
6. add lib/core/assert.std.toml
7. add minimal stdlib contract parser/loader
8. add bounded stdout oracle tests
9. archive stale lib/core and lib/std drafts
10. integrate structured contract diagnostics later
```

---

## 11. Definition of Done

The Contract Extension Architecture is established when:

```
docs/architecture/CONTRACT_EXTENSION_ARCHITECTURE.md exists
docs/stdlib/CONTRACT.md exists
active stdlib modules have contracts
official categories are documented
compiler avoids per-module hardcoding
validation can be feature-activated
failures are explicit
diagnostics can point to violated contracts
future extension families share one pattern
```

Final status target:

```
Logicodex has one reusable contract extension pattern.
Stdlib is the first concrete implementation.
FFI, runtime profiles, frontends, IL, diagnostics, and agent features can follow the same pattern without mutating compiler identity.
```

