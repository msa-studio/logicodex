# Diagnostics Fail-Fast Inventory

> **Evidence note:** Embedded versioned excerpts are preserved as historical audit evidence and are not current compiler authority.

Status: inventory only.

Goal: identify compiler/codegen paths where unsupported behavior may silently
succeed, especially by producing `0`, instead of producing a structured
diagnostic.

## Policy

Production-grade compiler behavior must not silently lower unsupported language
features into successful default values.

Unsupported or unimplemented paths should become one of:

- structured diagnostic error
- explicit unsupported-feature diagnostic
- guarded invariant panic only for impossible internal compiler bugs
- documented intentionally-zero semantic value, such as `None = 0`

## Categories to classify

### Allowed zero values

Examples:

- `None = 0`
- false-like boolean `0`
- numeric literal `0`
- explicit user value `0`

### Suspicious fallback-zero paths

Examples to audit:

- unsupported expression lowered as `0`
- unsupported address/deref lowered as `0`
- missing layout lowered as `0`
- unknown call return lowered as `0`
- array literal expression fallback outside supported initializer path

### Acceptable invariant panics

Examples:

- impossible internal compiler state
- invalid `TypeId`
- wrong type unwrap inside compiler internals

### Deferred feature diagnostics

Examples:

- enum layout size/alignment
- unsupported pointer dereference
- unsupported address-of
- unsupported heap-backed/generic collection operation

## Snapshot counts

- zero fallback grep: 42
- unsupported / placeholder grep: 360
- diagnostics grep: 360

## Next action

Create a classified table in a follow-up patch.

| File | Line/context | Category | Action |
|---|---|---|---|
| TBD | TBD | TBD | TBD |

Do not change compiler behavior until the inventory is classified.

## P0-A classification

This first fail-fast batch intentionally targets only the clearest suspicious
paths. Runtime ABI defaults and intentionally-zero semantic values are left
unchanged until a later classification pass.

| File/context | Category | Action |
|---|---|---|
| `src/codegen.rs` `UnaryOpAst::AddressOf` | Suspicious placeholder-zero | fail-fast with unsupported codegen diagnostic |
| `src/codegen.rs` `UnaryOpAst::Deref` | Suspicious placeholder-zero | fail-fast with unsupported codegen diagnostic |
| `src/codegen.rs` unresolved field layout | Suspicious fallback-zero | fail-fast because field access requires resolved struct layout |
| `src/codegen.rs` direct `HirExprKind::ArrayLiteral` expression | Suspicious fallback-zero | fail-fast; currently only typed local initializers are supported |
| `src/codegen.rs` `Color(...)` non-literal byte arguments | Suspicious fallback-zero | fail-fast until constructor semantics support evaluated byte arguments |
| `src/codegen.rs` unknown struct constructor layout | Suspicious fallback-zero | fail-fast because constructor requires resolved struct layout |

Left for later batches:

- actor/channel runtime return defaults
- `None = 0`
- false-like boolean values
- runtime call return-value normalization
- implicit function return policy
- global symbol expression policy


## Snapshot: zero fallback grep

```text
src/codegen.rs:264:            .build_int_compare(IntPredicate::NE, value, self.i64_type.const_zero(), name)
src/codegen.rs:581:                let zero = self.i64_type.const_int(0, false);
src/codegen.rs:955:                LiteralAst::Unit => Ok(self.i64_type.const_int(0, false)),
src/codegen.rs:971:                Ok(self.i64_type.const_int(0, false))
src/codegen.rs:1030:                    UnaryOpAst::AddressOf => Ok(self.i64_type.const_int(0, false)), // placeholder
src/codegen.rs:1031:                    UnaryOpAst::Deref => Ok(self.i64_type.const_int(0, false)),     // placeholder
src/codegen.rs:1071:            HirExprKind::OptionNone => Ok(self.i64_type.const_int(0, false)),
src/codegen.rs:1083:                    None => return Ok(self.i64_type.const_int(0, false)),
src/codegen.rs:1173:                    let i64_zero = self.i64_type.const_int(0, false);
src/codegen.rs:1215:                            let wzero = self.i64_type.const_int(0, false);
src/codegen.rs:1274:                    _ => self.i64_type.const_int(0, false),
src/codegen.rs:1314:                    None => self.i64_type.const_int(0, false),
src/codegen.rs:1322:                    _ => Ok(self.i64_type.const_int(0, false)),
src/codegen.rs:1341:                    _ => Ok(self.i64_type.const_int(0, false)),
src/codegen.rs:1361:                    _ => Ok(self.i64_type.const_int(0, false)),
src/codegen.rs:1453:                        _ => Ok(self.i64_type.const_int(0, false)), // default: 0 (None)
src/codegen.rs:1455:                    None => Ok(self.i64_type.const_int(0, false)),
src/codegen.rs:1468:                        _ => Ok(self.i64_type.const_int(0, false)),
src/codegen.rs:1470:                    None => Ok(self.i64_type.const_int(0, false)),
src/codegen.rs:1486:                        _ => Ok(self.i64_type.const_int(0, false)),
src/codegen.rs:1488:                    None => Ok(self.i64_type.const_int(0, false)),
src/codegen.rs:1521:                        _ => Ok(self.i64_type.const_int(0, false)),
src/codegen.rs:1523:                    None => Ok(self.i64_type.const_int(0, false)),
src/codegen.rs:1527:            HirExprKind::ArrayLiteral { .. } => Ok(self.i64_type.const_int(0, false)),
src/codegen.rs:1545:            let mut last = self.i64_type.const_int(0, false);
src/codegen.rs:1567:                self.i64_type.const_int(0, false)
src/codegen.rs:1575:                _ => Ok(self.i64_type.const_int(0, false)),
src/codegen.rs:1587:                _ => Ok(self.i64_type.const_int(0, false)),
src/codegen.rs:1656:            _ => Ok(self.i64_type.const_int(0, false)),
src/codegen.rs:1832:        let zero = self.i64_type.const_int(0, false);
src/codegen.rs:1858:        let zero = self.i64_type.const_int(0, false);
src/codegen.rs:1880:        let zero = self.i64_type.const_int(0, false);
src/codegen.rs:1961:        // TODO(inkwell-0.4.0): StructType::set_name() is not available; name set via symbol table only
src/codegen.rs:1994:                            return Ok(self.i64_type.const_int(0, false));
src/codegen.rs:2110:                    None => return Ok(self.i64_type.const_int(0, false)),
tests/stdlib_core_prelude.rs:100:         PAPAR core.prelude.fallback_i64(0, 9);\n\
tests/stdlib_core_prelude.rs:114:         PAPAR core.assert.eq_i64(core.prelude.fallback_i64(0, 8), 8);\n",
docs/archive/MAINTENANCE_v1441.md:83:| TODO/FIXME/HACK/XXX | **1** (ctl_mapper.rs:460) | Documented placeholder — acceptable |
docs/archive/MAINTENANCE_v1441.md:86:| `todo!()` macros | **0** | Clean |
docs/archive/CHANGELOG_ANALYSIS_v121_to_v145.md:285:| `todo!()` | Mungkin ada | **0** |
docs/release/V130_MAIN_READINESS.md:58:| Remaining `todo` markers in audited v1.30 modules | None in the readiness audit | Runtime TODO placeholders in the audited v1.30 surface were removed before merge. |
docs/white-paper/src/ch03-evolusi.md:360:| **Dead Code Audit** | 1 TODO (by design), 0 `#[allow(unused)]`, 0 `todo!()` |
```

## Snapshot: unsupported / placeholder grep

```text
src/contract_metadata.rs:15:#[derive(Debug, Clone, Copy, PartialEq, Eq)]
src/contract_metadata.rs:28:#[derive(Debug, Clone, Copy, PartialEq, Eq)]
src/contract_metadata.rs:58:#[derive(Debug, Clone, PartialEq, Eq)]
src/types.rs:8:#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
src/types.rs:11:#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
src/types.rs:14:#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
src/types.rs:17:#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
src/types.rs:26:#[derive(Debug, Clone, PartialEq, Eq)]
src/types.rs:36:#[derive(Debug, Clone, PartialEq, Eq)]
src/types.rs:45:#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
src/types.rs:50:#[derive(Debug, Clone, PartialEq, Eq, Hash)]
src/types.rs:61:    /// match lowering, and future LDX-DIP debugging metadata.
src/types.rs:87:            other => panic!("unwrap_struct called on {:?}", other),
src/types.rs:96:            other => panic!("unwrap_enum called on {:?}", other),
src/types.rs:101:#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
src/types.rs:158:#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
src/types.rs:222:#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
src/types.rs:228:#[derive(Debug, Clone)]
src/types.rs:241:#[derive(Debug, Clone, Copy, PartialEq, Eq)]
src/types.rs:388:            .unwrap_or_else(|| panic!("BUG: invalid TypeId({})", id.0))
src/types.rs:416:                        panic!(
src/types.rs:417:                            "BUG: StructLayoutId({}) not found. Register struct with intern_struct() first.",
src/types.rs:423:                // Sprint 2.5: Enum layout not yet implemented
src/types.rs:424:                panic!("TypeRegistry::get_size for Enum not yet implemented (Sprint 2.5)")
src/types.rs:460:                        panic!(
src/types.rs:461:                            "BUG: StructLayoutId({}) not found. Register struct with intern_struct() first.",
src/types.rs:467:                // Sprint 2.5: Enum layout not yet implemented
src/types.rs:468:                panic!("TypeRegistry::get_align for Enum not yet implemented (Sprint 2.5)")
src/types.rs:552:    /// Look up a type by name (placeholder)
src/types.rs:554:        None // TODO: implement reverse lookup
src/types.rs:557:    /// Find a callable by name (placeholder)
src/types.rs:562:        None // TODO: implement callable lookup
src/types.rs:644:                panic!(
src/types.rs:667:                panic!("const_char_ptr not pre-interned: call c_const_char_ptr() during TypeRegistry construction")
src/types.rs:674:#[derive(Debug, Clone, Copy, PartialEq, Eq)]
src/main.rs:65:#[derive(Debug, ClapParser)]
src/main.rs:86:#[derive(Debug, Clone, Copy, PartialEq, Eq)]
src/main.rs:109:    /// Profiles whose runtime is not yet implemented return Some(reason).
src/main.rs:126:#[derive(Debug, Subcommand)]
src/main.rs:691:    // v1.38 G1: Compute a simple module integrity hash (placeholder for SHA-256)
src/main.rs:696:        "# Logicodex Runtime Memory Integrity Verification Plan\n\nTarget artifact: `{}`\nModule hash (placeholder): `{:x}`\n\nSecurity roadmap: v1.21-alpha specification baseline and practical severity model.\n\nThe `--secure` compilation path computes a module integrity hash and records the security roadmap. v1.38: Basic hash computation via simple folding (placeholder for SHA-256).\n\nMemory integrity plan: `{:?}`\n\n## Verification Steps\n1. Recompile with `--secure` flag\n2. Compare module hash against known-good value\n3. Hash mismatch → fail-stop (hosted: process termination, freestanding: halt)\n",
src/main.rs:716:/// Placeholder — production would use SHA-256 over the .text section.
src/main.rs:792:        TargetArch::X86_64 // Default fallback
src/main.rs:1096:#[derive(Debug, Default, Clone)]
src/ast.rs:10:#[derive(Debug, Clone, PartialEq, Eq)]
src/ast.rs:21:#[derive(Debug, Clone, PartialEq, Eq)]
src/ast.rs:132:#[derive(Debug, Clone, PartialEq, Eq)]
src/ast.rs:139:#[derive(Debug, Clone, PartialEq, Eq)]
src/ast.rs:159:#[derive(Debug, Clone, PartialEq, Eq)]
src/ast.rs:166:#[derive(Debug, Clone, PartialEq, Eq)]
src/ast.rs:172:#[derive(Debug, Clone, PartialEq, Eq)]
src/ast.rs:304:#[derive(Debug, Copy, Clone, PartialEq, Eq)]
src/ast.rs:326:#[derive(Debug, Clone, PartialEq, Eq)]
src/span.rs:10:#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
src/span.rs:13:#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
src/span.rs:50:#[derive(Debug, Clone, PartialEq, Eq)]
src/span.rs:56:#[derive(Debug, Clone, PartialEq, Eq)]
src/span.rs:66:#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
src/span.rs:73:#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
src/span.rs:75:    ParserUnsupportedFeature,
src/span.rs:84:#[derive(Debug, Clone, PartialEq, Eq)]
src/lod.rs:46:#[derive(Debug, Default, Clone, PartialEq)]
src/lod.rs:54:#[derive(Debug, Default, Clone, PartialEq)]
src/lod.rs:60:#[derive(Debug, Default, Clone, PartialEq)]
src/lod.rs:67:#[derive(Debug, Default, Clone, PartialEq)]
src/lod.rs:264:#[derive(Debug, PartialEq)]
src/lod.rs:289:#[derive(Debug)]
src/parser.rs:14:#[derive(Debug, Clone, Copy, PartialEq, Eq)]
src/parser.rs:41:#[derive(Debug, Error)]
src/parser.rs:348:    /// BUGFIX #2: Peek ahead to detect `buf[index] = value` assignment pattern
src/parser.rs:459:    /// BUGFIX #2: Parse `buf[index] = value` as Stmt::Assign
src/parser.rs:535:            // BUGFIX #2: buf[index] = value assignment syntax
src/parser.rs:963:                _ => unreachable!(),
src/parser.rs:987:                _ => unreachable!(),
src/parser.rs:1005:                _ => unreachable!(),
src/parser.rs:1415:        // Fallback: a word-like keyword used as a namespaced base, e.g. the gate
src/codegen.rs:26:#[derive(Debug, Clone)]
src/codegen.rs:45:#[derive(Debug, Clone)]
src/codegen.rs:76:#[derive(Debug, Clone, Copy)]
src/codegen.rs:200:                "type_id_to_llvm: unsupported type kind: {:?}",
src/codegen.rs:1030:                    UnaryOpAst::AddressOf => Ok(self.i64_type.const_int(0, false)), // placeholder
src/codegen.rs:1031:                    UnaryOpAst::Deref => Ok(self.i64_type.const_int(0, false)),     // placeholder
src/codegen.rs:1925:            .unwrap_or_else(|| TypeId(6)) // fallback
src/codegen.rs:1961:        // TODO(inkwell-0.4.0): StructType::set_name() is not available; name set via symbol table only
src/semantic.rs:16:#[derive(Debug, Clone, Copy, PartialEq, Eq)]
src/semantic.rs:40:#[derive(Debug, Error)]
src/semantic.rs:151:#[derive(Debug, Default)]
src/semantic.rs:235:        // BUGFIX #3: Clear moved_vars and buffer_registry for variables going out of scope
src/semantic.rs:306:                // BUGFIX #1: Register Buffer types in buffer_registry for provenance tracking
src/semantic.rs:337:                // BUGFIX #4: Detect ownership move — let buf2 = buf
src/semantic.rs:355:                        // BUGFIX #2: buf[index] = value assignment
src/semantic.rs:887:                    // Fallback: check variable type
src/ffi.rs:21:#[derive(Debug, Clone, PartialEq, Eq)]
src/ffi.rs:32:#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
src/ffi.rs:40:#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
src/ffi.rs:46:#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
src/ffi.rs:52:#[derive(Debug, Default, Clone)]
src/ffi.rs:103:#[derive(Debug, Clone)]
src/ffi.rs:167:            // Placeholder for the future ffi.allow_lib path. Until lod wires the
src/layout.rs:14:#[derive(Debug, Clone, PartialEq, Eq)]
src/layout.rs:22:#[derive(Debug, Clone, PartialEq, Eq)]
src/layout.rs:29:#[derive(Debug, Clone, Copy, PartialEq, Eq)]
src/layout.rs:40:#[derive(Debug, Clone, Copy, PartialEq, Eq)]
src/layout.rs:46:#[derive(Debug, Clone, PartialEq, Eq)]
src/layout.rs:92:    /// Create with native target (placeholder)
src/layout.rs:223:#[derive(Debug, Clone, PartialEq, Eq)]
src/layout.rs:232:#[derive(Debug, Clone, PartialEq, Eq)]
src/layout.rs:239:#[derive(Debug, Clone, PartialEq, Eq)]
src/layout.rs:246:#[derive(Debug, Clone, Copy, PartialEq, Eq)]
src/layout.rs:257:#[derive(Debug, Default)]
src/lexer.rs:15:#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
src/lexer.rs:131:#[derive(Debug, Clone, PartialEq, Eq)]
src/lexer.rs:150:#[derive(Debug, Deserialize)]
src/lexer.rs:155:#[derive(Debug, Deserialize)]
src/lexer.rs:162:#[derive(Debug, Deserialize)]
src/lexer.rs:170:#[derive(Debug, Deserialize)]
src/lexer.rs:179:#[derive(Debug, Clone)]
src/lexer.rs:185:#[derive(Debug, Error)]
src/hir.rs:20:#[derive(Debug, Clone, PartialEq)]
src/hir.rs:25:#[derive(Debug, Clone, PartialEq)]
src/hir.rs:33:#[derive(Debug, Clone, PartialEq)]
src/hir.rs:46:#[derive(Debug, Clone, PartialEq)]
src/hir.rs:52:#[derive(Debug, Clone, PartialEq)]
src/hir.rs:57:#[derive(Debug, Clone, PartialEq)]
src/hir.rs:93:#[derive(Debug, Clone, PartialEq)]
src/hir.rs:186:#[derive(Debug, Clone, PartialEq, Eq)]
src/hir.rs:194:#[derive(Debug, Clone, PartialEq, Eq)]
src/hir.rs:215:#[derive(Debug, Clone, Copy, PartialEq, Eq)]
src/hir.rs:237:#[derive(Debug, Clone, Copy, PartialEq, Eq)]
src/hir.rs:245:#[derive(Debug, Clone, PartialEq)]
src/hir.rs:252:#[derive(Debug, Clone, PartialEq)]
src/hir.rs:259:#[derive(Debug, Clone, Copy, PartialEq, Eq)]
src/hir.rs:265:#[derive(Debug, Clone, PartialEq, Eq)]
src/hir.rs:271:#[derive(Debug, Clone, PartialEq)]
src/hir.rs:278:#[derive(Debug, Clone, PartialEq)]
src/hir.rs:284:#[derive(Debug, Clone, PartialEq)]
src/hir.rs:291:#[derive(Debug, Clone, PartialEq)]
src/hir.rs:297:#[derive(Debug, Clone, PartialEq)]
src/hir.rs:305:#[derive(Debug, Clone, PartialEq)]
src/hir.rs:310:#[derive(Debug, Clone, PartialEq)]
src/hir.rs:318:#[derive(Debug, Clone, PartialEq)]
src/hir.rs:328:#[derive(Debug, Clone, PartialEq)]
src/hir.rs:334:#[derive(Debug, Clone, PartialEq)]
src/hir.rs:340:#[derive(Debug, Clone, PartialEq)]
src/hir.rs:346:#[derive(Debug, Clone, PartialEq)]
src/hir.rs:361:#[derive(Debug, Clone, PartialEq)]
src/hir.rs:367:#[derive(Debug, Clone, PartialEq)]
src/hir.rs:373:#[derive(Debug, Clone, PartialEq)]
src/hir.rs:378:#[derive(Debug, Clone, PartialEq)]
src/hir.rs:418:#[derive(Debug, Clone, PartialEq)]
src/hir.rs:425:#[derive(Debug, Clone, PartialEq)]
src/hir.rs:521:#[derive(Debug, Default)]
src/hir.rs:534:#[derive(Debug, Clone, Copy, PartialEq, Eq)]
src/hir.rs:632:#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
src/hir.rs:635:#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
src/hir.rs:920:                        code: DiagnosticCode::ParserUnsupportedFeature,
src/hir.rs:965:                            code: DiagnosticCode::ParserUnsupportedFeature,
src/hir.rs:1228:    /// TODO(type-check): verify the resolved local has type Channel<_, _, I64>
src/hir.rs:1677:            code: DiagnosticCode::ParserUnsupportedFeature,
src/hir.rs:2190:        // lowered to Unit, so `-5` compiled to 0 -- a correctness bug.
src/hir.rs:2255:            other => panic!("expected Unary(Negate), got {other:?}"),
src/hir.rs:2267:            other => panic!("expected Unary(LogicalNot), got {other:?}"),
src/hir.rs:2312:            other => panic!("unexpected item: {other:?}"),
src/hir.rs:2346:            DiagnosticCode::ParserUnsupportedFeature
src/module_loader.rs:41:#[derive(Debug, Clone)]
src/module_loader.rs:54:#[derive(Debug)]
src/module_loader.rs:61:#[derive(Debug)]
src/module_loader.rs:97:///   3. `./lib`              -- dev/test fallback (repo-root `lib/`)
src/module_loader.rs:482:            other => panic!("expected ModuleNotFound, got {other:?}"),
src/tier2/gate.rs:26:#[derive(Debug, Clone, PartialEq, Eq, Hash)]
src/tier2/gate.rs:127:impl fmt::Debug for GateRef {
src/tier2/gate.rs:144:#[derive(Debug, Clone, PartialEq, Eq)]
src/tier2/gate.rs:176:#[derive(Debug, Clone)]
src/tier2/capability_ir.rs:31:#[derive(Debug, Clone, Copy, PartialEq, Eq)]
src/tier2/capability_ir.rs:64:#[derive(Debug, Clone, PartialEq, Eq, Hash)]
src/tier2/capability_ir.rs:122:#[derive(Debug, Clone)]
src/tier2/capability_ir.rs:190:#[derive(Debug, Clone)]
src/tier2/capability_ir.rs:215:#[derive(Debug, Clone)]
src/tier2/capability_ir.rs:225:#[derive(Debug, Clone)]
src/tier2/capability_ir.rs:234:#[derive(Debug)]
src/tier2/capability_ir.rs:605:#[derive(Debug, Clone)]
src/tier2/capability_ir.rs:617:#[derive(Debug, Clone)]
src/tier2/shard.rs:27:#[derive(Debug, Clone)]
src/tier2/shard.rs:78:#[derive(Debug, Clone)]
src/tier2/shard.rs:125:#[derive(Debug, Clone)]
src/tier2/shard.rs:141:#[derive(Debug, Clone, PartialEq, Eq)]
src/tier2/shard.rs:182:#[derive(Debug, Default)]
src/tier2/shard.rs:229:#[derive(Debug)]
src/tier2/shard.rs:240:#[derive(Debug, Clone)]
src/tier2/shard.rs:260:#[derive(Debug, Clone)]
src/tier2/shard.rs:272:#[derive(Debug, Clone)]
src/tier2/topology.rs:20:#[derive(Debug, Default)]
src/tier2/topology.rs:70:#[derive(Debug, Clone)]
src/tier2/topology.rs:81:#[derive(Debug, Clone)]
src/tier2/topology.rs:372:#[derive(Debug, Clone)]
src/tier2/pass.rs:27:#[derive(Debug)]
src/tier2/pass.rs:53:#[derive(Debug, Clone, Copy, PartialEq, Eq)]
src/tier2/pass.rs:211:                    // Placeholder: discard_function_body(name);
src/tier2/metadata.rs:81:impl std::fmt::Debug for Capability {
src/tier2/metadata.rs:113:#[derive(Debug, Clone, Copy, PartialEq, Eq)]
src/tier2/metadata.rs:148:#[derive(Debug, Clone)]
src/tier2/metadata.rs:239:#[derive(Debug, Default)]
src/tier2/metadata.rs:371:#[derive(Debug)]
src/tier2/ctl_mapper.rs:31:#[derive(Debug, Clone, PartialEq, Eq)]
src/tier2/ctl_mapper.rs:96:#[derive(Debug, Clone, PartialEq, Eq)]
src/tier2/ctl_mapper.rs:223:#[derive(Debug)]
src/tier2/ctl_mapper.rs:295:            // Fallback: just use the operation name
src/tier2/ctl_mapper.rs:417:                        // Fallback
src/tier2/ctl_mapper.rs:456:                "    // TODO: Implement {}.{} host-side logic",
src/tier2/ctl_mapper.rs:501:#[derive(Debug, Clone)]
src/semantic/type_checker.rs:15:/// Placeholder impl for TypeRegistry methods referenced by type_checker.
src/semantic/type_checker.rs:18:    /// TODO: implement full mapping for all interned types.
src/semantic/type_checker.rs:33:#[derive(Debug, Clone, PartialEq, Eq)]
src/semantic/type_checker.rs:184:            // TODO: Expr::FloatLiteral does not exist in Expr enum; no F64 literal variant.
src/semantic/type_checker.rs:198:            // TODO: Expr::Unary variant does not exist in Expr enum
src/semantic/type_checker.rs:209:                        return None; // TODO: refine argument-type inference
src/semantic/type_checker.rs:242:            // TODO: validate each argument type against field type
src/semantic/type_checker.rs:249:        // TODO: integrate with CallableRegistry for full function checking
src/semantic/type_checker.rs:380:        // TODO: Expr::FloatLiteral does not exist in Expr enum
src/semantic/coercion.rs:14:#[derive(Debug, Clone, Copy, PartialEq, Eq)]
src/net/event.rs:10:#[derive(Debug, Clone, Copy, PartialEq, Eq)]
src/net/event.rs:25:#[derive(Debug, Clone)]
src/net/event.rs:58:#[derive(Debug, Clone, Copy, PartialEq, Eq)]
src/net/shard_local_pool.rs:28:#[derive(Debug, Clone, PartialEq, Eq)]
src/net/shard_local_pool.rs:152:#[derive(Debug, Clone)]
src/net/host_reactor.rs:23:#[derive(Debug, Clone, PartialEq)]
src/net/host_reactor.rs:57:#[derive(Debug, Clone, Default)]
src/net/host_reactor.rs:88:#[derive(Debug, Clone, Default)]
src/net/host_reactor.rs:130:#[derive(Debug, Clone, Default)]
src/net/host_reactor.rs:370:#[derive(Debug, Clone, Copy, PartialEq, Eq)]
src/net/host_reactor.rs:378:#[derive(Debug, Clone)]
src/net/host_reactor.rs:386:#[derive(Debug, Clone)]
src/net/reactor.rs:26:#[derive(Debug, Clone, Copy)]
src/net/reactor.rs:287:                // Stub mode — fallback
src/net/reactor.rs:410:#[derive(Debug, Clone, PartialEq, Eq)]
src/net/affinity.rs:15:#[derive(Debug, Clone)]
src/net/affinity.rs:17:    UnsupportedPlatform,
src/net/affinity.rs:25:            AffinityError::UnsupportedPlatform => {
src/net/affinity.rs:83:    // Fallback: teruskan tanpa affinity — log warning.
src/net/affinity.rs:89:    Err(AffinityError::UnsupportedPlatform)
src/net/affinity.rs:107:    Err(AffinityError::UnsupportedPlatform)
src/net/affinity.rs:110:// ─── Fallback untuk platform lain ───
src/net/affinity.rs:117:    Err(AffinityError::UnsupportedPlatform)
src/net/affinity.rs:126:        .unwrap_or(4) // fallback: 4 cores
src/net/affinity.rs:142:    0 // fallback
src/net/sharded_reactor.rs:111:#[derive(Debug, Clone)]
src/net/sharded_reactor.rs:325:#[derive(Debug, Clone)]
src/net/policy.rs:17:#[derive(Debug, Clone, Copy, PartialEq, Eq)]
src/net/policy.rs:75:#[derive(Debug, Clone, Copy, PartialEq, Eq)]
src/net/policy.rs:90:#[derive(Debug, Clone)]
src/net/connection.rs:22:#[derive(Debug, Clone, Copy, PartialEq, Eq)]
src/net/connection.rs:275:#[derive(Debug, Clone, PartialEq, Eq)]
src/net/connection.rs:305:#[derive(Debug, Default)]
src/net/service.rs:29:#[derive(Debug, Clone)]
src/net/service.rs:108:#[derive(Debug, Default)]
src/net/service.rs:240:#[derive(Debug, Clone)]
src/net/service.rs:250:#[derive(Debug, Clone, PartialEq, Eq)]
src/net/service.rs:272:#[derive(Debug, Clone)]
src/ffi/raylib.rs:19:#[derive(Debug, Clone, Copy)]
src/ffi/raylib.rs:197:#[derive(Debug, Clone, Copy)]
src/ffi/raylib_sys.rs:17:#[derive(Debug, Clone, Copy, PartialEq, Eq)]
src/ffi/raylib_sys.rs:27:#[derive(Debug, Clone, Copy, PartialEq)]
src/ffi/raylib_sys.rs:35:#[derive(Debug, Clone, Copy, PartialEq)]
src/ffi/raylib_sys.rs:44:#[derive(Debug, Clone, Copy, PartialEq)]
src/ffi/raylib_sys.rs:54:#[derive(Debug, Clone, Copy)]
src/ffi/raylib_sys.rs:65:#[derive(Debug, Clone, Copy)]
src/ffi/raylib_sys.rs:325:#[derive(Debug, Clone, Copy)]
src/ffi/raylib_sys.rs:336:#[derive(Debug, Clone, Copy)]
src/ffi/raylib_sys.rs:344:#[derive(Debug, Clone, Copy)]
src/ffi/raylib_sys.rs:355:#[derive(Debug, Clone, Copy)]
src/os/panic.rs:4:// When `panic!()` is called in a bare-metal environment (no OS),
src/os/panic.rs:92:// Hosted fallback — use eprintln
src/os/uart.rs:5:// Used for debug output in bare-metal environments where there's no OS
src/os/uart.rs:15:// Also provides VGA text mode output (0xB8000) as fallback/alternative.
src/os/uart.rs:182:#[derive(Clone, Copy, Debug)]
src/os/uart.rs:249:                // Non-printable: print placeholder
src/os/syscall.rs:126:    /// v1.38 F1: Graceful fallback — returns error instead of panicking.
src/os/syscall.rs:132:    /// v1.38 F1: Windows fallback for sys_recv — not applicable on Windows.
src/os/syscall.rs:133:    pub fn win_recv_fallback(_fd: i32, _buf: &mut [u8]) -> Result<usize, i32> {
src/os/syscall.rs:140:    /// v1.38 F1: Windows fallback for sys_send — not applicable on Windows.
src/os/syscall.rs:141:    pub fn win_send_fallback(_fd: i32, _buf: &[u8]) -> Result<usize, i32> {
src/os/syscall.rs:163:        -1 // Unsupported platform
src/os/syscall.rs:167:#[derive(Debug, Clone, Copy)]
src/os/syscall.rs:291:        // Fallback to Rust std
src/os/interrupts.rs:110:    "Debug",
src/os/interrupts.rs:280:exception_handler!(exc_debug, 1);
src/os/interrupts.rs:295:    exc_divide_by_zero, exc_debug, exc_nmi, exc_breakpoint,
src/os/target.rs:15:#[derive(Debug, Clone, Copy, PartialEq, Eq)]
src/os/target.rs:51:#[derive(Debug, Clone, Copy, PartialEq, Eq)]
src/os/target.rs:73:                        "unsupported freestanding architecture `{other}`; \
src/os/target.rs:88:                "unsupported Logicodex target `{other}`; \
src/os/target.rs:130:#[derive(Debug, Clone, Copy)]
src/os/startup.rs:114:// ─── Hosted fallback ───
tests/compiler_result_option_foundation.rs:112:        "public function unwrap_or_i64(x: Result<I64, I64>, fallback: I64) -> I64 begin\n\
tests/compiler_result_option_foundation.rs:118:                     return fallback;\n\
tests/compiler_result_option_foundation.rs:135:        "public function unwrap_err_or_i64(x: Result<I64, I64>, fallback: I64) -> I64 begin\n\
tests/compiler_result_option_foundation.rs:138:                     return fallback;\n\
tests/compiler_result_option_foundation.rs:158:        "public function unwrap_or_i64(x: Result<I64, I64>, fallback: I64) -> I64 begin\n\
tests/compiler_result_option_foundation.rs:164:                     return fallback;\n\
tests/compiler_result_option_foundation.rs:179:        "public function unwrap_or_i64(x: Option<I64>, fallback: I64) -> I64 begin\n\
tests/compiler_result_option_foundation.rs:185:                     return fallback;\n\
tests/compiler_result_option_foundation.rs:215:        "public function unwrap_or_i64(x: Option<I64>, fallback: I64) -> I64 begin\n\
tests/compiler_result_option_foundation.rs:221:                     return fallback;\n\
tests/freestanding_v144_gaps.rs:453:    assert!(build.contains("raylib_no_link"), "G15: graceful fallback");
tests/stdlib_core_prelude.rs:100:         PAPAR core.prelude.fallback_i64(0, 9);\n\
tests/stdlib_core_prelude.rs:101:         PAPAR core.prelude.fallback_i64(7, 9);\n",
tests/stdlib_core_prelude.rs:114:         PAPAR core.assert.eq_i64(core.prelude.fallback_i64(0, 8), 8);\n",
docs/VS_CODE_EXTENSION.md:18:| `extensions/vscode-logicodex/resources/core_map.json` | Fallback dictionary snapshot used when the extension runs outside the repository workspace. |
docs/VS_CODE_EXTENSION.md:77:| `logicodexSideView.dictionaryPath` | `""` | Custom path to `core_map.json`; when empty, the extension tries workspace `dict/core_map.json` and then the bundled fallback. |
docs/DOCUMENTATION_POLICY.md:41:> **Status: PLANNED — not yet implemented.** Describes intended design; code here
docs/HANDBOOK.md:616:| `unwrap on None` | This is a Logicodex bug — please report with backtrace |
docs/GAPS_ASSESSMENT.md:72:- `.github/ISSUE_TEMPLATE/bug_report.md`
docs/architecture/runtime-doctrine.md:106:- The **profile layer is doctrine, not yet implemented.** `--target wasm` and the
docs/architecture/CONTRACT_EXTENSION_ARCHITECTURE.md:70:silent extension fallback
docs/architecture/CONTRACT_EXTENSION_ARCHITECTURE.md:87:No Silent Fallback:
docs/architecture/CONTRACT_EXTENSION_ARCHITECTURE.md:88:  unsupported behavior must fail explicitly.
docs/architecture/CONTRACT_EXTENSION_ARCHITECTURE.md:188:prevent silent fallback to unknown libraries
docs/architecture/CONTRACT_EXTENSION_ARCHITECTURE.md:249:unsupported constructs
docs/architecture/CONTRACT_EXTENSION_ARCHITECTURE.md:261:reject unsupported semantics explicitly
docs/architecture/CONTRACT_EXTENSION_ARCHITECTURE.md:297:reject invalid or unsupported IL
docs/architecture/CONTRACT_EXTENSION_ARCHITECTURE.md:337:avoid vague fallback errors
docs/architecture/CONTRACT_EXTENSION_ARCHITECTURE.md:462:This makes Logicodex easier for humans and agents to debug without making agents the authority.
docs/architecture/CONTRACT_EXTENSION_ARCHITECTURE.md:466:## 8. No Silent Fallback Rule
docs/architecture/CONTRACT_EXTENSION_ARCHITECTURE.md:475:unsupported import resolved as a different module
docs/architecture/stdlib-migration-status.md:21:- `core.prelude` — CPB Phase 1 explicit-import scalar bootstrap helpers (`id_i64`, `zero_i64`, `one_i64`, `truthy_i64`, `fallback_i64`), stage 1 contract-backed. This is not a magic auto-prelude and does not depend on re-export/delegation.
docs/architecture/cpb-next-roadmap-blockers.md:47:- no silent success for unsupported code paths
docs/architecture/cpb-next-roadmap-blockers.md:48:- audit and remove unsafe fallback-to-zero behavior in codegen/semantic lowering
docs/architecture/cpb-next-roadmap-blockers.md:50:Unsupported expressions or unimplemented features must become structured compile
docs/architecture/cpb-next-roadmap-blockers.md:62:- clearer diagnostics for unsupported generic or heap-backed patterns
docs/architecture/hir-decision.md:17:- Removing the dual-path ambiguity eliminates a class of "which path ran?" bugs.
docs/architecture/stdlib-core-design-doctrine.md:64:- silently changing semantics to match old bugs
docs/architecture/stdlib-core-design-doctrine.md:182:- define unsupported behavior
docs/architecture/result-option-foundation.md:182:### Debuggability and smart-compiler constraints
docs/architecture/result-option-foundation.md:187:AI-queryable debugging, and future safe self-treatment.
docs/architecture/result-option-foundation.md:193:- Unsupported match patterns must not be silently ignored during lowering.
docs/archive/MANUAL.md:66:./target/x86_64-unknown-linux-gnu/debug/logicodex logo
docs/archive/MANUAL.md:67:./target/x86_64-unknown-linux-gnu/debug/logicodex tokens examples/01_tambah_pemula.ldx
docs/archive/MANUAL.md:68:./target/x86_64-unknown-linux-gnu/debug/logicodex check examples/01_tambah_pemula.ldx
docs/archive/MAINTENANCE_v1441.md:6:**Result:** All 27 validators passing, 5 untested modules identified, 0 production unwrap(), 1 stale TODO
docs/archive/MAINTENANCE_v1441.md:45:validate_buffer_bugfixes.py             — Buffer provenance fixes
docs/archive/MAINTENANCE_v1441.md:83:| TODO/FIXME/HACK/XXX | **1** (ctl_mapper.rs:460) | Documented placeholder — acceptable |
docs/archive/MAINTENANCE_v1441.md:86:| `todo!()` macros | **0** | Clean |
docs/archive/MAINTENANCE_v1441.md:89:**The single TODO:**
docs/archive/MAINTENANCE_v1441.md:92:"    // TODO: Implement {}.{} host-side logic",
docs/archive/MAINTENANCE_v1441.md:94:This is an **intentional placeholder string** inside the WIT template generator. It is emitted into generated WIT files as documentation for host-side implementers, not an unimplemented code path. Status: **BY DESIGN** — should not be removed.
docs/archive/MAINTENANCE_v1441.md:107:| `src/os/windows.rs` | ~50 | NO | Low | Windows stub — Linux dev environment, runtime_assembly fallback |
docs/archive/MAINTENANCE_v1441.md:203:| ctl_mapper.rs TODO | String in WIT template | **BY DESIGN** — host-side guidance |
docs/archive/MAINTENANCE_v1441.md:215:| **Code Cleanliness** | 1 TODO (by design) | No stale comments, no unused code |
docs/archive/REPOS_CONTEXT.md:58:- **Fail-fast**: v1.21 codegen has `unreachable!()` safety nets that panic if v1.30 AST nodes leak through.
docs/archive/REPOS_CONTEXT.md:65:- `src/codegen.rs`: `unreachable!()` safety net, `CodegenBackend` trait, `compile_v130()` entry point.
docs/archive/CHANGELOG_ANALYSIS_v121_to_v145.md:230:| **F** (Platform) | F1: Windows syscall fallback | ✅ 1/1 resolved |
docs/archive/CHANGELOG_ANALYSIS_v121_to_v145.md:283:| TODO dalam kod | Berisiko | **1** — by design (WIT template placeholder) |
docs/archive/CHANGELOG_ANALYSIS_v121_to_v145.md:285:| `todo!()` | Mungkin ada | **0** |
docs/archive/GrammarandDictionary.md:49:| Logical OR | `||`, `or` | `ready || fallback` |
docs/archive/GETTING_STARTED_dual_engine.md:768:#[debug]
docs/archive/GETTING_STARTED_dual_engine.md:790:#[debug]
docs/archive/GETTING_STARTED.md:47:| Slow debugging | Deterministic behavior — same input = same output, every time |
docs/archive/GETTING_STARTED.md:390:| Difficult to debug | Easy to reason about |
docs/archive/GETTING_STARTED.md:843:# For development, use debug mode (faster compile)
docs/archive/GRAMMAR_ANALYSIS.md:175:**THIS IS THE BUG!**
```

## Snapshot: diagnostics grep

```text
src/codegen_contract.rs:41:    use crate::span::{Span, Spanned};
src/codegen_contract.rs:96:            items: vec![Spanned {
src/codegen_contract.rs:107:                span: Span::unknown(),
src/contract_metadata.rs:22:    Diagnostic,
src/types.rs:60:    /// Result<I64, I64>, but the type identity is preserved for diagnostics,
src/types.rs:67:    /// Option<I64>, but the type identity is preserved for diagnostics and
src/main.rs:23:mod span;
src/main.rs:483:    let mut merged: Vec<span::Spanned<hir::HirItem>> = Vec::new();
src/main.rs:529:            diagnostics: Vec::new(),
src/main.rs:598:        diagnostics: Vec::new(),
src/main.rs:606:        .map_err(|diagnostics| anyhow::anyhow!(format_v130_diagnostics(&diagnostics)))?;
src/main.rs:868:            diagnostics: Vec::new(),
src/main.rs:891:        diagnostics: Vec::new(),
src/main.rs:899:        .map_err(|diagnostics| anyhow::anyhow!(format_v130_diagnostics(&diagnostics)))?;
src/main.rs:933:                        span: span::Span::unknown(),
src/main.rs:938:                        span: span::Span::unknown(),
src/main.rs:942:                span: span::Span::unknown(),
src/main.rs:944:            .map_err(|diagnostic| anyhow::anyhow!(format_v130_diagnostic(&diagnostic)))?;
src/main.rs:949:        items: vec![span::Spanned {
src/main.rs:955:                    statements: vec![span::Spanned {
src/main.rs:958:                                statements: vec![span::Spanned {
src/main.rs:960:                                    span: span::Span::unknown(),
src/main.rs:964:                        span: span::Span::unknown(),
src/main.rs:970:            span: span::Span::unknown(),
src/main.rs:977:        diagnostics: Vec::new(),
src/main.rs:982:        .map_err(|diagnostics| anyhow::anyhow!(format_v130_diagnostics(&diagnostics)))?;
src/main.rs:988:        diagnostics: Vec::new(),
src/main.rs:996:        .map_err(|diagnostics| anyhow::anyhow!(format_v130_diagnostics(&diagnostics)))?;
src/main.rs:1001:fn format_v130_diagnostics(diagnostics: &[span::Diagnostic]) -> String {
src/main.rs:1002:    diagnostics
src/main.rs:1004:        .map(format_v130_diagnostic)
src/main.rs:1009:fn format_v130_diagnostic(diagnostic: &span::Diagnostic) -> String {
src/main.rs:1010:    format!("{} / {}", diagnostic.message_ms, diagnostic.message_en)
src/main.rs:1134:    // is preserved in the struct and surfaced in diagnostics/docs.
src/span.rs:5:// Source spans and diagnostics.
src/span.rs:14:pub struct Span {
src/span.rs:22:impl Span {
src/span.rs:51:pub struct Spanned<T> {
src/span.rs:53:    pub span: Span,
src/span.rs:57:pub struct Diagnostic {
src/span.rs:58:    pub code: DiagnosticCode,
src/span.rs:62:    pub primary_span: Span,
src/span.rs:63:    pub notes: Vec<DiagnosticNote>,
src/span.rs:74:pub enum DiagnosticCode {
src/span.rs:85:pub struct DiagnosticNote {
src/span.rs:86:    pub span: Option<Span>,
src/lod.rs:93:    pub fn from_toml_str(text: &str) -> Result<Self, ParseError> {
src/lod.rs:216:fn split_key_value(line: &str, lineno: usize) -> Result<(&str, &str), ParseError> {
src/lod.rs:219:        .ok_or_else(|| ParseError::new(lineno, "expected `key = value`"))?;
src/lod.rs:223:        return Err(ParseError::new(lineno, "empty key"));
src/lod.rs:229:fn parse_string(value: &str, lineno: usize) -> Result<String, ParseError> {
src/lod.rs:234:        .ok_or_else(|| ParseError::new(lineno, "expected a quoted string"))?;
src/lod.rs:236:        return Err(ParseError::new(lineno, "unexpected quote inside string"));
src/lod.rs:242:fn parse_string_array(value: &str, lineno: usize) -> Result<Vec<String>, ParseError> {
src/lod.rs:247:        .ok_or_else(|| ParseError::new(lineno, "expected a `[...]` array"))?;
src/lod.rs:263:/// A manifest parse error with a line number for actionable diagnostics.
src/lod.rs:265:pub struct ParseError {
src/lod.rs:270:impl ParseError {
src/lod.rs:272:        ParseError {
src/lod.rs:279:impl std::fmt::Display for ParseError {
src/lod.rs:285:impl std::error::Error for ParseError {}
src/semantic_gate.rs:16:use crate::span::{Diagnostic, DiagnosticCode, Severity, Span, Spanned};
src/semantic_gate.rs:23:    pub diagnostics: Vec<Diagnostic>,
src/semantic_gate.rs:39:    pub fn check_module(&mut self, module: &HirModule) -> Result<(), Vec<Diagnostic>> {
src/semantic_gate.rs:46:                        DiagnosticCode::DuplicateDefinition,
src/semantic_gate.rs:47:                        item.span,
src/semantic_gate.rs:68:        if self.diagnostics.is_empty() {
src/semantic_gate.rs:71:            Err(self.diagnostics.clone())
src/semantic_gate.rs:75:    fn check_item(&mut self, item: &Spanned<HirItem>) {
src/semantic_gate.rs:93:    fn check_statement(&mut self, stmt: &Spanned<HirStmt>) {
src/semantic_gate.rs:128:                    DiagnosticCode::UnsafeBoundaryViolation,
src/semantic_gate.rs:129:                    stmt.span,
src/semantic_gate.rs:170:                            DiagnosticCode::DivisionByZero,
src/semantic_gate.rs:171:                            expr.span,
src/semantic_gate.rs:188:                        DiagnosticCode::TypeMismatch,
src/semantic_gate.rs:189:                        expr.span,
src/semantic_gate.rs:221:                        gate.validate_call(signature, args, self.safety_context, expr.span);
src/semantic_gate.rs:222:                    if let Err(diagnostic) = result {
src/semantic_gate.rs:223:                        self.diagnostics.push(diagnostic);
src/semantic_gate.rs:232:                        self.diagnostics.push(crate::span::Diagnostic {
src/semantic_gate.rs:233:                            code: DiagnosticCode::FfiBoundaryViolation,
src/semantic_gate.rs:243:                            primary_span: expr.span,
src/semantic_gate.rs:282:        code: DiagnosticCode,
src/semantic_gate.rs:283:        span: Span,
src/semantic_gate.rs:287:        self.diagnostics.push(Diagnostic {
src/semantic_gate.rs:292:            primary_span: span,
src/semantic_gate.rs:305:    fn spanned<T>(node: T) -> Spanned<T> {
src/semantic_gate.rs:306:        Spanned {
src/semantic_gate.rs:308:            span: Span::unknown(),
src/semantic_gate.rs:318:            span: Span::unknown(),
src/semantic_gate.rs:327:            diagnostics: Vec::new(),
src/semantic_gate.rs:339:            items: vec![spanned(HirItem::Function(HirFunction {
src/semantic_gate.rs:347:                    statements: vec![spanned(HirStmt::Break { target_depth: 0 })],
src/semantic_gate.rs:353:        let diagnostics = ctx
src/semantic_gate.rs:356:        assert_eq!(diagnostics[0].code, DiagnosticCode::UnsafeBoundaryViolation);
src/semantic_gate.rs:363:            items: vec![spanned(HirItem::Function(HirFunction {
src/semantic_gate.rs:371:                    statements: vec![spanned(HirStmt::Loop {
src/semantic_gate.rs:373:                            statements: vec![spanned(HirStmt::Break { target_depth: 0 })],
src/semantic_gate.rs:406:            span: Span::unknown(),
src/semantic_gate.rs:409:            items: vec![spanned(HirItem::Function(HirFunction {
src/semantic_gate.rs:415:                    statements: vec![spanned(HirStmt::Expr(call))],
src/semantic_gate.rs:421:        let diagnostics = ctx
src/semantic_gate.rs:424:        assert_eq!(diagnostics[0].code, DiagnosticCode::FfiBoundaryViolation);
src/semantic_gate.rs:446:            span: Span::unknown(),
src/semantic_gate.rs:449:            items: vec![spanned(HirItem::Function(HirFunction {
src/semantic_gate.rs:455:                    statements: vec![spanned(HirStmt::UnsafeBlock(HirBlock {
src/semantic_gate.rs:456:                        statements: vec![spanned(HirStmt::Expr(call))],
src/semantic_gate.rs:472:/// Returns Ok(()) if no issues found, or a list of diagnostics otherwise.
src/semantic_gate.rs:473:pub fn validate_module(module: &HirModule, types: TypeRegistry) -> Result<(), Vec<Diagnostic>> {
src/semantic_gate.rs:479:        diagnostics: Vec::new(),
src/semantic_gate.rs:488:/// Run the semantic gatekeeper and print any diagnostics.
src/semantic_gate.rs:496:        Err(diagnostics) => {
src/semantic_gate.rs:499:                diagnostics.len()
src/semantic_gate.rs:501:            for d in &diagnostics {
src/parser.rs:42:pub enum ParseError {
src/parser.rs:86:    pub fn parse(&mut self) -> Result<Program, ParseError> {
src/parser.rs:105:    fn declaration_or_statement(&mut self) -> Result<Stmt, ParseError> {
src/parser.rs:114:            return Err(ParseError::Expected {
src/parser.rs:148:    fn struct_declaration(&mut self) -> Result<Stmt, ParseError> {
src/parser.rs:175:    fn enum_declaration(&mut self) -> Result<Stmt, ParseError> {
src/parser.rs:197:    fn unsafe_block(&mut self) -> Result<Stmt, ParseError> {
src/parser.rs:203:    fn extern_block(&mut self) -> Result<Stmt, ParseError> {
src/parser.rs:257:    fn use_declaration(&mut self) -> Result<Stmt, ParseError> {
src/parser.rs:272:                return Err(ParseError::Expected {
src/parser.rs:286:    fn hardware_zone_block(&mut self) -> Result<Stmt, ParseError> {
src/parser.rs:293:    fn hardware_declaration(&mut self) -> Result<Stmt, ParseError> {
src/parser.rs:308:    fn function_definition(&mut self, is_public: bool) -> Result<Stmt, ParseError> {
src/parser.rs:359:    fn actor_statement(&mut self) -> Result<Stmt, ParseError> {
src/parser.rs:393:    fn service_statement(&mut self) -> Result<Stmt, ParseError> {
src/parser.rs:460:    fn index_assignment_statement(&mut self) -> Result<Stmt, ParseError> {
src/parser.rs:490:    fn variable_assignment_statement(&mut self) -> Result<Stmt, ParseError> {
src/parser.rs:504:    fn statement(&mut self) -> Result<Stmt, ParseError> {
src/parser.rs:559:    fn match_statement(&mut self) -> Result<Stmt, ParseError> {
src/parser.rs:579:    fn match_arm(&mut self) -> Result<MatchArm, ParseError> {
src/parser.rs:615:                .map_err(|_| ParseError::InvalidInteger {
src/parser.rs:626:            return Err(ParseError::Expected {
src/parser.rs:646:    fn let_statement(&mut self, beginner: bool) -> Result<Stmt, ParseError> {
src/parser.rs:666:    fn print_statement(&mut self, beginner: bool) -> Result<Stmt, ParseError> {
src/parser.rs:672:    fn return_statement(&mut self, beginner: bool) -> Result<Stmt, ParseError> {
src/parser.rs:678:    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
src/parser.rs:697:    fn while_statement(&mut self) -> Result<Stmt, ParseError> {
src/parser.rs:704:    fn loop_statement(&mut self) -> Result<Stmt, ParseError> {
src/parser.rs:710:    fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
src/parser.rs:725:    fn parse_type(&mut self) -> Result<Type, ParseError> {
src/parser.rs:784:                .map_err(|_| ParseError::Expected {
src/parser.rs:878:        Err(ParseError::Expected {
src/parser.rs:886:    fn expression(&mut self) -> Result<Expr, ParseError> {
src/parser.rs:890:    fn logical_or(&mut self) -> Result<Expr, ParseError> {
src/parser.rs:903:    fn logical_and(&mut self) -> Result<Expr, ParseError> {
src/parser.rs:916:    fn bit_or(&mut self) -> Result<Expr, ParseError> {
src/parser.rs:931:    fn bit_xor(&mut self) -> Result<Expr, ParseError> {
src/parser.rs:944:    fn bit_and(&mut self) -> Result<Expr, ParseError> {
src/parser.rs:957:    fn equality(&mut self) -> Result<Expr, ParseError> {
src/parser.rs:975:    fn comparison(&mut self) -> Result<Expr, ParseError> {
src/parser.rs:999:    fn shift(&mut self) -> Result<Expr, ParseError> {
src/parser.rs:1017:    fn term(&mut self) -> Result<Expr, ParseError> {
src/parser.rs:1035:    fn unary(&mut self) -> Result<Expr, ParseError> {
src/parser.rs:1054:    fn factor(&mut self) -> Result<Expr, ParseError> {
src/parser.rs:1081:    fn parse_postfix(&mut self, mut expr: Expr) -> Result<Expr, ParseError> {
src/parser.rs:1088:                return Err(ParseError::Expected {
src/parser.rs:1117:    fn primary(&mut self) -> Result<Expr, ParseError> {
src/parser.rs:1123:                .map_err(|_| ParseError::InvalidInteger {
src/parser.rs:1436:        Err(ParseError::Unexpected {
src/parser.rs:1443:    fn consume_integer_literal(&mut self, expected: &str) -> Result<i64, ParseError> {
src/parser.rs:1448:            .map_err(|_| ParseError::InvalidInteger {
src/parser.rs:1482:    ) -> Result<(), ParseError> {
src/parser.rs:1519:    fn consume(&mut self, kind: TokenKind, expected: &str) -> Result<&Token, ParseError> {
src/parser.rs:1524:        Err(ParseError::Expected {
src/parser.rs:1539:    fn consume_member_name(&mut self, ctx: &str) -> Result<String, ParseError> {
src/parser.rs:1551:            Err(ParseError::Expected {
src/codegen.rs:360:        if let Err(diagnostics) = crate::semantic_gate::validate_module(hir_module, types_clone) {
src/codegen.rs:363:                diagnostics.len()
src/codegen.rs:365:            for d in &diagnostics {
src/lib.rs:51:pub mod span;
src/semantic.rs:41:pub enum SemanticError {
src/semantic.rs:194:    pub fn analyze(program: &Program) -> Result<(), SemanticError> {
src/semantic.rs:202:    ) -> Result<(), SemanticError> {
src/semantic.rs:209:    ) -> Result<(), SemanticError> {
src/semantic.rs:225:    fn block(&mut self, statements: &[Stmt]) -> Result<(), SemanticError> {
src/semantic.rs:232:    fn scoped_block(&mut self, statements: &[Stmt]) -> Result<(), SemanticError> {
src/semantic.rs:246:    fn statement(&mut self, stmt: &Stmt) -> Result<(), SemanticError> {
src/semantic.rs:257:                    return Err(SemanticError::DuplicateHardwareRegion(name.clone()));
src/semantic.rs:260:                    return Err(SemanticError::InvalidPointerInitializer { name: name.clone() });
src/semantic.rs:275:                    return Err(SemanticError::DuplicateFunction(name.clone()));
src/semantic.rs:340:                        return Err(SemanticError::UseAfterMove {
src/semantic.rs:365:                                    return Err(SemanticError::ElementTypeMismatch {
src/semantic.rs:373:                            other => Err(SemanticError::TypeMismatch {
src/semantic.rs:384:                            return Err(SemanticError::UseAfterMove { name: name.clone() });
src/semantic.rs:402:                    return Err(SemanticError::ReturnOutsideFunction);
src/semantic.rs:406:                        return Err(SemanticError::ReturnTypeMismatch {
src/semantic.rs:422:                    return Err(SemanticError::NonBooleanCondition(ty));
src/semantic.rs:430:                    return Err(SemanticError::NonBooleanCondition(ty));
src/semantic.rs:445:                    Err(SemanticError::BreakOutsideLoop)
src/semantic.rs:452:                    Err(SemanticError::ContinueOutsideLoop)
src/semantic.rs:460:                    return Err(SemanticError::DuplicateActor { name: name.clone() });
src/semantic.rs:499:                    return Err(SemanticError::DuplicateActor { name: name.clone() });
src/semantic.rs:503:                    return Err(SemanticError::CapabilityContractViolation {
src/semantic.rs:517:                    return Err(SemanticError::CapabilityContractViolation {
src/semantic.rs:544:                        return Err(SemanticError::NonExhaustiveMatch {
src/semantic.rs:571:    ) -> Result<(), SemanticError> {
src/semantic.rs:574:                return Err(SemanticError::HardwareMutationOutsideZone);
src/semantic.rs:583:                Expr::AddressOfLiteral(_) => Err(SemanticError::BareAddressRejected {
src/semantic.rs:587:                _ => Err(SemanticError::InvalidPointerInitializer {
src/semantic.rs:594:                return Err(SemanticError::NumericBounds {
src/semantic.rs:603:            return Err(SemanticError::DeclaredTypeMismatch {
src/semantic.rs:671:    ) -> Result<(), SemanticError> {
src/semantic.rs:674:            return Err(SemanticError::UseAfterMove {
src/semantic.rs:683:                .ok_or_else(|| SemanticError::NotABuffer {
src/semantic.rs:690:                return Err(SemanticError::BufferOverflow {
src/semantic.rs:702:    fn expression(&mut self, expr: &Expr) -> Result<Type, SemanticError> {
src/semantic.rs:710:                    return Err(SemanticError::UseAfterSend { name: name.clone() });
src/semantic.rs:716:                    return Err(SemanticError::HardwareMutationOutsideZone);
src/semantic.rs:720:                    return Err(SemanticError::MissingProvenance(*addr));
src/semantic.rs:746:                    return Err(SemanticError::TypeMismatch {
src/semantic.rs:763:                    other => Err(SemanticError::TypeMismatch {
src/semantic.rs:779:                    return Err(SemanticError::HandleNotOpen {
src/semantic.rs:787:                            return Err(SemanticError::HandlePermissionDenied {
src/semantic.rs:795:                            return Err(SemanticError::HandlePermissionDenied {
src/semantic.rs:837:                    return Err(SemanticError::ActorNotFound {
src/semantic.rs:865:                        return Err(SemanticError::UseAfterSend {
src/semantic.rs:891:                        Err(SemanticError::UndefinedVariable(channel_name.clone()))
src/semantic.rs:897:                    return Err(SemanticError::ActorNotFound {
src/semantic.rs:921:                        return Err(SemanticError::UseAfterSend {
src/semantic.rs:963:                    return Err(SemanticError::TypeMismatch {
src/semantic.rs:987:                    return Err(SemanticError::TypeMismatch {
src/semantic.rs:1033:                    return Err(SemanticError::DivisionByZero);
src/semantic.rs:1051:                            Err(SemanticError::TypeMismatch {
src/semantic.rs:1066:                            Err(SemanticError::TypeMismatch {
src/semantic.rs:1078:                            Err(SemanticError::TypeMismatch {
src/semantic.rs:1090:                            Err(SemanticError::TypeMismatch {
src/semantic.rs:1104:    fn define(&mut self, name: &str, ty: Type) -> Result<(), SemanticError> {
src/semantic.rs:1110:            return Err(SemanticError::DuplicateVariable(name.to_string()));
src/semantic.rs:1116:    fn resolve(&self, name: &str) -> Result<Type, SemanticError> {
src/semantic.rs:1122:        Err(SemanticError::UndefinedVariable(name.to_string()))
src/semantic.rs:1205:    pub fn verify_audio_callbacks(&self, program: &Program) -> Result<(), SemanticError> {
src/semantic.rs:1217:    fn verify_audio_safety(&self, func_name: &str, stmts: &[Stmt]) -> Result<(), SemanticError> {
src/semantic.rs:1224:    fn verify_audio_stmt(&self, func_name: &str, stmt: &Stmt) -> Result<(), SemanticError> {
src/semantic.rs:1226:            Stmt::Print { .. } => Err(SemanticError::AudioViolationIo {
src/semantic.rs:1254:                Err(SemanticError::AudioViolationUnboundedLoop)
src/semantic.rs:1284:    fn verify_audio_expr(&self, func_name: &str, expr: &Expr) -> Result<(), SemanticError> {
src/semantic.rs:1290:                        return Err(SemanticError::AudioViolationIo {
src/semantic.rs:1296:                        return Err(SemanticError::AudioViolationRecursion {
src/semantic.rs:1302:                        return Err(SemanticError::AudioViolationForbiddenCall {
src/ffi.rs:17:use crate::span::{Diagnostic, DiagnosticCode, Severity, Span};
src/ffi.rs:200:        call_span: Span,
src/ffi.rs:201:    ) -> Result<(), Diagnostic> {
src/ffi.rs:207:                call_span,
src/ffi.rs:222:                call_span,
src/ffi.rs:240:                call_span,
src/ffi.rs:261:                    actual.span,
src/ffi.rs:289:                        call_span,
src/ffi.rs:351:/// Helper: create an FFI diagnostic with dual messages.
src/ffi.rs:352:fn ffi_error(span: Span, mus_msg: String, eng_msg: String) -> Diagnostic {
src/ffi.rs:353:    Diagnostic {
src/ffi.rs:354:        code: DiagnosticCode::FfiBoundaryViolation,
src/ffi.rs:358:        primary_span: span,
src/layout.rs:9:use crate::span::{Diagnostic, DiagnosticCode, Severity, Span};
src/layout.rs:19:    pub span: Span,
src/layout.rs:26:    pub span: Span,
src/layout.rs:103:    ) -> Result<StructLayout, Diagnostic> {
src/layout.rs:110:            let (size_bytes, natural_alignment) = self.size_and_align(field.ty, field.span)?;
src/layout.rs:133:    fn size_and_align(&self, ty: TypeId, span: Span) -> Result<(usize, usize), Diagnostic> {
src/layout.rs:149:                let (element_size, element_align) = self.size_and_align(*element, span)?;
src/layout.rs:157:                        span,
src/layout.rs:175:                span,
src/layout.rs:212:fn layout_error(span: Span, message_ms: String, message_en: String) -> Diagnostic {
src/layout.rs:213:    Diagnostic {
src/layout.rs:214:        code: DiagnosticCode::LayoutError,
src/layout.rs:218:        primary_span: span,
src/layout.rs:266:    use crate::span::Span;
src/layout.rs:284:                        span: Span::unknown(),
src/layout.rs:289:                        span: Span::unknown(),
src/layout.rs:293:                span: Span::unknown(),
src/layout.rs:319:                        span: Span::unknown(),
src/layout.rs:324:                        span: Span::unknown(),
src/layout.rs:328:                span: Span::unknown(),
src/layout.rs:344:        let diagnostic = engine
src/layout.rs:350:                    span: Span::unknown(),
src/layout.rs:353:                span: Span::unknown(),
src/layout.rs:357:        assert_eq!(diagnostic.code, DiagnosticCode::LayoutError);
src/layout.rs:358:        assert!(diagnostic.message_ms.contains("Ralat:"));
src/layout.rs:359:        assert!(diagnostic.message_en.contains("Error:"));
src/hir.rs:14:use crate::span::{Diagnostic, DiagnosticCode, Severity, Span, Spanned};
src/hir.rs:22:    pub items: Vec<Spanned<ItemAst>>,
src/hir.rs:54:    pub statements: Vec<Spanned<StmtAst>>,
src/hir.rs:139:    /// constructor identity is preserved for diagnostics and match lowering.
src/hir.rs:203:    /// HIR keeps the meaning for match lowering and diagnostics.
src/hir.rs:307:    pub items: Vec<Spanned<HirItem>>,
src/hir.rs:375:    pub statements: Vec<Spanned<HirStmt>>,
src/hir.rs:422:    pub span: Span,
src/hir.rs:641:    pub diagnostics: Vec<Diagnostic>,
src/hir.rs:677:    pub fn lower_program(&mut self, program: ast::Program) -> Result<HirModule, Vec<Diagnostic>> {
src/hir.rs:685:        let mut functions: Vec<Spanned<ItemAst>> = Vec::new();
src/hir.rs:686:        let mut top_level_stmts: Vec<Spanned<StmtAst>> = Vec::new();
src/hir.rs:697:                    functions.push(Spanned {
src/hir.rs:711:                                    .map(|s| Spanned {
src/hir.rs:713:                                        span: Span::unknown(),
src/hir.rs:720:                        span: Span::unknown(),
src/hir.rs:724:                    functions.push(Spanned {
src/hir.rs:737:                        span: Span::unknown(),
src/hir.rs:741:                    functions.push(Spanned {
src/hir.rs:753:                        span: Span::unknown(),
src/hir.rs:760:                    functions.push(Spanned {
src/hir.rs:788:                        span: Span::unknown(),
src/hir.rs:799:                    functions.push(Spanned {
src/hir.rs:817:                                    .map(|s| Spanned {
src/hir.rs:819:                                        span: Span::unknown(),
src/hir.rs:826:                        span: Span::unknown(),
src/hir.rs:830:                    top_level_stmts.push(Spanned {
src/hir.rs:832:                        span: Span::unknown(),
src/hir.rs:840:            functions.push(Spanned {
src/hir.rs:851:                span: Span::unknown(),
src/hir.rs:868:    /// statement) is rejected with a clear bilingual diagnostic, because
src/hir.rs:875:    ) -> Result<HirModule, Vec<Diagnostic>> {
src/hir.rs:877:        let mut functions: Vec<Spanned<ItemAst>> = Vec::new();
src/hir.rs:878:        let mut errors: Vec<Diagnostic> = Vec::new();
src/hir.rs:888:                    functions.push(Spanned {
src/hir.rs:902:                                    .map(|s| Spanned {
src/hir.rs:904:                                        span: Span::unknown(),
src/hir.rs:911:                        span: Span::unknown(),
src/hir.rs:919:                    errors.push(Diagnostic {
src/hir.rs:920:                        code: DiagnosticCode::ParserUnsupportedFeature,
src/hir.rs:928:                        primary_span: Span::unknown(),
src/hir.rs:949:    ) -> Result<HirModule, Vec<Diagnostic>> {
src/hir.rs:956:    pub fn lower_module(&mut self, module: ModuleAst) -> Result<HirModule, Vec<Diagnostic>> {
src/hir.rs:964:                        self.diagnostics.push(Diagnostic {
src/hir.rs:965:                            code: DiagnosticCode::ParserUnsupportedFeature,
src/hir.rs:975:                            primary_span: item.span,
src/hir.rs:1070:        if self.diagnostics.is_empty() {
src/hir.rs:1073:            Err(std::mem::take(&mut self.diagnostics))
src/hir.rs:1077:    fn lower_item(&mut self, item: Spanned<ItemAst>) -> Option<Spanned<HirItem>> {
src/hir.rs:1078:        let span = item.span;
src/hir.rs:1158:        Some(Spanned { node, span })
src/hir.rs:1161:    fn lower_statement(&mut self, stmt: Spanned<StmtAst>) -> Option<Spanned<HirStmt>> {
src/hir.rs:1162:        let span = stmt.span;
src/hir.rs:1165:                let value = value.map(|expr| self.lower_expr(expr, span));
src/hir.rs:1174:                target: self.lower_expr(target, span),
src/hir.rs:1175:                value: self.lower_expr(value, span),
src/hir.rs:1182:                condition: self.lower_expr(condition, span),
src/hir.rs:1187:                condition: self.lower_expr(condition, span),
src/hir.rs:1204:            StmtAst::Expr(expr) => HirStmt::Expr(self.lower_expr(expr, span)),
src/hir.rs:1205:            StmtAst::Return(expr) => HirStmt::Return(expr.map(|expr| self.lower_expr(expr, span))),
src/hir.rs:1207:        Some(Spanned { node, span })
src/hir.rs:1230:    fn lower_channel_ref(&mut self, name: &str, span: Span) -> HirExpr {
src/hir.rs:1235:                span,
src/hir.rs:1246:                span,
src/hir.rs:1260:            // Emit handle 0 so the rest of lowering proceeds; the diagnostic
src/hir.rs:1265:                span,
src/hir.rs:1270:    fn lower_expr(&mut self, expr: ExprAst, span: Span) -> HirExpr {
src/hir.rs:1275:                span,
src/hir.rs:1282:                        span,
src/hir.rs:1288:                        span,
src/hir.rs:1292:                        span,
src/hir.rs:1299:                        span,
src/hir.rs:1304:                let left = self.lower_expr(*left, span);
src/hir.rs:1305:                let right = self.lower_expr(*right, span);
src/hir.rs:1314:                    span,
src/hir.rs:1318:                let lowered = self.lower_expr(*expr, span);
src/hir.rs:1338:                    span,
src/hir.rs:1344:                    .map(|arg| self.lower_expr(arg, span))
```

## P0-B classification

Second fail-fast batch.

| File/context | Category | Action |
|---|---|---|
| `src/codegen.rs` `HirExprKind::Global` | Suspicious fallback-zero | fail-fast; global symbol expressions are not implemented yet |
| `src/codegen.rs` builtin `logicodex_sleep` missing argument | Suspicious fallback-zero | fail-fast; runtime sleep requires a duration argument |

Still deferred:

| File/context | Category | Reason |
|---|---|---|
| implicit function return `0` | Deferred policy | requires return-type policy audit before behavior change |
| `LiteralAst::Unit` | Allowed zero | unit-like value currently encoded as `0` |
| `OptionNone` | Allowed zero | semantic encoding: `None = 0` |
| actor/channel runtime return defaults | Deferred ABI policy | may be runtime status/handle normalization, not necessarily fallback |
| GEP index zero values | Allowed zero | LLVM indexing helper value, not semantic fallback |
| `print` builtin initial `last = 0` | Deferred builtin policy | only applies to empty print argument list; parser/semantic policy should decide |
| generic non-i64 call return default | Deferred ABI policy | may require callable return typing before behavior change |

## P0 final status

This PR converts the clearest unsupported codegen fallback-zero cases into
fail-fast errors and records the remaining zero-producing paths for future
policy work.

### Converted to fail-fast

- address-of expression placeholder
- dereference expression placeholder
- unresolved field layout fallback
- direct array literal expression fallback
- non-literal `Color(...)` byte arguments
- unresolved struct constructor layout fallback
- global symbol expression fallback
- missing `logicodex_sleep` duration argument fallback

### Intentionally not changed in this PR

- `None = 0`
- unit/false/literal-zero encodings
- LLVM GEP index zero helper values
- actor/channel runtime ABI default handling
- implicit function return policy
- generic non-i64 call return policy
- empty `print` argument policy

Those remaining items need separate semantic/ABI policy before behavior changes.
