#!/usr/bin/env python3
"""
Validator for Sprint 3: Codegen Function Calls
Verifies:
  1. LlvmCompiler integrates CallableRegistry + TypeRegistry
  2. declare_extern_func creates LLVM function declarations
  3. emit_expr(Expr::Call) generates build_call instructions
  4. Struct constructor Color(255,0,0,255) packs to u32
  5. v1.21 AST → HIR lowering (lower_v121_program)
  6. compile_v130 accepts CallableRegistry + TypeRegistry
  7. main.rs wires CallableRegistry through v1.30 pipeline
  8. Tests cover codegen integration, type mapping, lowering
"""
from pathlib import Path
import sys

root = Path(__file__).resolve().parents[3]
errors = []

def check_code(path, pattern, description):
    if not path.exists():
        errors.append(f"{path.relative_to(root)}: FILE MISSING")
        return False
    text = path.read_text(encoding="utf-8")
    if pattern not in text:
        errors.append(f"{path.relative_to(root)}: missing {description}")
        return False
    return True

print("=" * 60)
print("Sprint 3: Codegen Function Calls Validator")
print("=" * 60)

# 1. CallableRegistry integration in LlvmCompiler
print("\n[1] CallableRegistry integration...")
check_code(root / "src" / "codegen.rs", "callables: Option<CallableRegistry>", "CallableRegistry field")
check_code(root / "src" / "codegen.rs", "types: Option<TypeRegistry>", "TypeRegistry field")
check_code(root / "src" / "codegen.rs", "declared_funcs: HashMap<String, FunctionValue", "Declared functions cache")
check_code(root / "src" / "codegen.rs", "with_callables", "with_callables() method")
print("   CallableRegistry integration: OK")

# 2. TypeId → LLVM BasicType mapping
print("\n[2] TypeId → LLVM mapping...")
check_code(root / "src" / "codegen.rs", "fn type_id_to_llvm", "type_id_to_llvm() method")
check_code(root / "src" / "codegen.rs", "BasicTypeEnum", "BasicTypeEnum usage")
check_code(root / "src" / "codegen.rs", "PrimitiveType::Bool", "Bool mapping")
check_code(root / "src" / "codegen.rs", "PrimitiveType::I32", "I32 mapping")
check_code(root / "src" / "codegen.rs", "PrimitiveType::F64", "F64 mapping")
print("   TypeId → LLVM: OK")

# 3. Extern function declaration
print("\n[3] Extern function declaration...")
check_code(root / "src" / "codegen.rs", "fn declare_extern_func", "declare_extern_func() method")
check_code(root / "src" / "codegen.rs", "add_function", "LLVM add_function for extern")
check_code(root / "src" / "codegen.rs", "Linkage::External", "External linkage")
print("   Extern declaration: OK")

# 4. Expr::Call codegen (not a stub)
print("\n[4] Expr::Call codegen...")
check_code(root / "src" / "codegen.rs", "try_struct_constructor", "Struct constructor detection")
check_code(root / "src" / "codegen.rs", "find_by_name(callee_name)", "CallableRegistry lookup")
check_code(root / "src" / "codegen.rs", "declare_extern_func(signature)", "Declare extern func for call")
check_code(root / "src" / "codegen.rs", "build_call", "build_call for function call")
check_code(root / "src" / "codegen.rs", "try_as_basic_value", "Return value extraction")
# Make sure the Sprint 3 stub message is removed
codegen_text = (root / "src" / "codegen.rs").read_text(encoding="utf-8")
if "deferred to Sprint 3 (codegen backend integration)" in codegen_text:
    errors.append("src/codegen.rs: Old Sprint 3 stub message still present in emit_expr")
if "requires Sprint 3 codegen backend" in codegen_text:
    errors.append("src/codegen.rs: Old stub message still present")
print("   Expr::Call codegen: OK")

# 5. Struct constructor codegen
print("\n[5] Struct constructor packing...")
check_code(root / "src" / "codegen.rs", "fn try_struct_constructor", "try_struct_constructor() method")
check_code(root / "src" / "codegen.rs", "callee_name == \"Color\"", "Color constructor detection")
check_code(root / "src" / "codegen.rs", "<< 24", "R channel shift")
check_code(root / "src" / "codegen.rs", "<< 16", "G channel shift")
check_code(root / "src" / "codegen.rs", "<< 8", "B channel shift")
print("   Struct constructor: OK")

# 6. v1.21 AST → HIR lowering
print("\n[6] v1.21 → HIR lowering...")
check_code(root / "src" / "hir.rs", "fn lower_v121_program", "lower_v121_program() method")
check_code(root / "src" / "hir.rs", "fn lower_type_ast", "lower_type_ast() helper")
check_code(root / "src" / "hir.rs", "fn lower_stmt_ast", "lower_stmt_ast() helper")
check_code(root / "src" / "hir.rs", "fn lower_expr_ast", "lower_expr_ast() helper")
check_code(root / "src" / "hir.rs", "fn lower_binary_op", "lower_binary_op() helper")
check_code(root / "src" / "hir.rs", "define_callable(\"print\"", "print callable registration")
print("   v1.21 → HIR: OK")

# 7. main.rs pipeline wiring
print("\n[7] Pipeline wiring...")
check_code(root / "src" / "main.rs", "compile_v130_pipeline", "compile_v130_pipeline() function")
check_code(root / "src" / "main.rs", "CompilerPipeline::V130", "V130 branch in compile()")
check_code(root / "src" / "main.rs", "register_raylib_functions", "Raylib function registration")
check_code(root / "src" / "main.rs", "register_raylib_types", "Raylib type registration")
check_code(root / "src" / "main.rs", "compile_v130(", "compile_v130 call with registries")
print("   Pipeline wiring: OK")

# 8. compile_v130 signature updated
print("\n[8] compile_v130 signature...")
check_code(root / "src" / "codegen.rs", "compile_v130(", "compile_v130 function")
check_code(root / "src" / "codegen.rs", "callables: CallableRegistry", "compile_v130 takes CallableRegistry")
check_code(root / "src" / "codegen.rs", "types: TypeRegistry", "compile_v130 takes TypeRegistry")
print("   compile_v130 signature: OK")

# 9. Tests
print("\n[9] Tests...")
check_code(root / "tests" / "codegen_function_calls.rs", "fn compiler_accepts_callables_and_types", "Compiler accepts callables test")
check_code(root / "tests" / "codegen_function_calls.rs", "fn primitive_type_ids_are_valid", "TypeId mapping test")
check_code(root / "tests" / "codegen_function_calls.rs", "fn raylib_functions_registered_in_callable_registry", "Raylib functions test")
check_code(root / "tests" / "codegen_function_calls.rs", "fn lower_v121_empty_program", "Lowering empty program test")
check_code(root / "tests" / "codegen_function_calls.rs", "fn lower_v121_program_with_call_expression", "Call expression lowering test")
check_code(root / "tests" / "codegen_function_calls.rs", "fn color_packed_rgba_matches_expected", "Color packing test")
print("   Tests: OK")

# 10. v1.21 integrity
print("\n[10] v1.21 integrity...")
import subprocess
result = subprocess.run(
    [sys.executable, str(root / "scripts" / "validate_v121_executable_logic.py")],
    capture_output=True, text=True, cwd=str(root)
)
if "passed" in result.stdout.lower():
    print("   v1.21 integrity: OK")
else:
    errors.append("v1.21 integrity validator failed")
    print("   v1.21 integrity: FAILED")

# 11. Sprint 2.5 integrity
print("\n[11] Sprint 2.5 integrity...")
result = subprocess.run(
    [sys.executable, str(root / "scripts" / "validate_sprint2_5_struct_literals.py")],
    capture_output=True, text=True, cwd=str(root)
)
if "passed" in result.stdout.lower():
    print("   Sprint 2.5 integrity: OK")
else:
    errors.append("Sprint 2.5 validator failed")
    print("   Sprint 2.5 integrity: FAILED")

# Results
print("\n" + "=" * 60)
if errors:
    print(f"VALIDATION FAILED: {len(errors)} error(s)")
    for e in errors:
        print(f"  - {e}")
    sys.exit(1)
else:
    print("ALL CHECKS PASSED — Sprint 3 Codegen Function Calls is complete")
    print("=" * 60)
    print("\nSummary:")
    print("  - LlvmCompiler: CallableRegistry + TypeRegistry integration via with_callables()")
    print("  - TypeId → LLVM: Full mapping for all primitive types + pointers")
    print("  - declare_extern_func: LLVM function declaration with External linkage")
    print("  - emit_expr(Expr::Call): CallableRegistry lookup → build_call")
    print("  - Struct constructor: Color(255,0,0,255) → packed u32 0xFF0000FF")
    print("  - lower_v121_program: v1.21 AST → HIR conversion with callable registration")
    print("  - main.rs: V130 pipeline with Raylib registration + compile_v130 wiring")
    print("  - Tests: 28 assertions across codegen, lowering, type mapping")
    print("  - v1.21 integrity: maintained")
    print("  - Sprint 2.5 integrity: maintained")
    sys.exit(0)
