// =========================================================================
// Project: Logicodex Language Engine (Phase 2 Deployment Integration)
// Version: v1.21-alpha (Specification Baseline & Practical Severity Roadmap)
// Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
// Copyright (c) 2026. All Rights Reserved.
// Licensed under permissive dual-license: MIT & Apache License 2.0
// =========================================================================

#[cfg(feature = "v1_30")]
use crate::ffi::{CallableRegistry, CallableSignature};
#[cfg(not(feature = "v1_30"))]
use crate::types::CallableId;

#[cfg(not(feature = "v1_30"))]
#[derive(Debug, Clone, Default)]
pub struct CallableRegistry {
    signatures: Vec<CallableSignature>,
}
#[cfg(not(feature = "v1_30"))]
impl CallableRegistry {
    pub fn new() -> Self { Self { signatures: Vec::new() } }
    pub fn find_by_name(&self, _name: &str) -> Option<(CallableId, CallableSignature)> { None }
    pub fn get(&self, _id: CallableId) -> Option<CallableSignature> { None }
    pub fn signatures(&self) -> std::slice::Iter<CallableSignature> { self.signatures.iter() }
}

#[cfg(not(feature = "v1_30"))]
#[derive(Debug, Clone)]
pub struct CallableSignature {
    pub name: String,
    pub params: Vec<crate::types::TypeId>,
    pub return_type: crate::types::TypeId,
    pub is_variadic: bool,
}
#[cfg(not(feature = "v1_30"))]
impl Default for CallableSignature {
    fn default() -> Self {
        Self {
            name: String::new(),
            params: Vec::new(),
            return_type: crate::types::TypeId(0),
            is_variadic: false,
        }
    }
}
use crate::os::target::{build_target_machine, build_target_machine_with_arch, CompilationTarget, OutputKind};
use crate::types::{PrimitiveType, TypeId, TypeKind, TypeRegistry};
use anyhow::{anyhow, Context as AnyhowContext, Result};
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicType, BasicTypeEnum, IntType};
use inkwell::AddressSpace;
use inkwell::values::{BasicValueEnum, FunctionValue, IntValue, PointerValue};
use inkwell::IntPredicate;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct MemoryIntegrityPlan {
    pub golden_hash_symbol: &'static str,
    pub text_segment_symbol: &'static str,
    pub uses_sha_aes_intrinsics: bool,
    pub panic_strategy: &'static str,
}

impl MemoryIntegrityPlan {
    pub fn hardened_default() -> Self {
        Self {
            golden_hash_symbol: "__logicodex_golden_text_hash",
            text_segment_symbol: "__logicodex_text_segment_bounds",
            uses_sha_aes_intrinsics: true,
            panic_strategy: "clear_sensitive_registers_and_target_specific_fail_stop",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PhysicalMemoryAccessPlan {
    pub raw_pointer_type: &'static str,
    pub vga_text_buffer: u64,
    pub serial_uart_port: u16,
    pub requires_unsafe_backend_gate: bool,
}

impl PhysicalMemoryAccessPlan {
    pub fn freestanding_default() -> Self {
        Self {
            raw_pointer_type: "*int",
            vga_text_buffer: 0xB8000,
            serial_uart_port: 0x3F8,
            requires_unsafe_backend_gate: true,
        }
    }
}

pub struct CodegenOptions {
    pub module_name: String,
    pub emit_ir: bool,
    pub secure: bool,
    pub target: CompilationTarget,
}

pub struct CodegenArtifact {
    pub object_path: std::path::PathBuf,
    pub ir_path: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, Copy)]
struct LoopTarget<'ctx> {
    continue_block: BasicBlock<'ctx>,
    break_block: BasicBlock<'ctx>,
}

pub struct LlvmCompiler<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    i64_type: IntType<'ctx>,
    bool_type: IntType<'ctx>,
    variables: Vec<HashMap<String, PointerValue<'ctx>>>,
    loop_targets: Vec<LoopTarget<'ctx>>,
    print_fn: FunctionValue<'ctx>,
    // Sprint 3: CallableRegistry integration for function call codegen
    callables: Option<CallableRegistry>,
    types: Option<TypeRegistry>,
    declared_funcs: HashMap<String, FunctionValue<'ctx>>,
    // v1.35+: HIR codegen support — local allocations and callable resolution
    hir_local_allocs: HashMap<u32, PointerValue<'ctx>>,
    // Declared type of each HIR local, for fixed-width wrapping on assignment.
    hir_local_types: HashMap<u32, crate::types::TypeRef>,
    hir_callable_funcs: HashMap<u32, FunctionValue<'ctx>>,
    // v1.36+: Struct registry — symbol_id → LLVM struct type
    hir_struct_types: HashMap<u32, inkwell::types::StructType<'ctx>>,
    hir_struct_names: HashMap<String, u32>, // name → symbol_id
    callable_names: HashMap<u32, String>, // CallableId.0 → name (call routing)
    // v1.38 A6: CallableRegistry predeclaration tracking
    callables_predeclared: bool,
    // v1.44 G12: Hardware zone depth counter (MMIO volatile semantics)
    hw_zone_depth: u32,
    // sret: caller-provided return buffer + its LLVM type, for the current
    // struct-returning function (None when the function returns a scalar).
    current_sret: Option<(PointerValue<'ctx>, inkwell::types::StructType<'ctx>)>,
}

/// Backend trait for version-gated codegen. v1.21 uses direct compilation;
/// v1.30+ uses this trait for HIR-based codegen.
#[cfg(feature = "v1_30")]
pub trait CodegenBackend {
    #[cfg(feature = "v1_30")]
    fn compile_hir_module(&mut self, module: &crate::hir::HirModule, options: &CodegenOptions) -> Result<CodegenArtifact>;
}

impl<'ctx> LlvmCompiler<'ctx> {
    fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        let i64_type = context.i64_type();
        let bool_type = context.bool_type();
        let print_type = context.void_type().fn_type(&[i64_type.into()], false);
        let print_fn = module.add_function("logicodex_print_i64", print_type, None);
        Self {
            context,
            module,
            builder,
            i64_type,
            bool_type,
            variables: vec![HashMap::new()],
            loop_targets: Vec::new(),
            print_fn,
            callables: None,
            types: None,
            declared_funcs: HashMap::new(),
            hir_local_allocs: HashMap::new(),
            hir_local_types: HashMap::new(),
            hir_callable_funcs: HashMap::new(),
            hir_struct_types: HashMap::new(),
            hir_struct_names: HashMap::new(),
            callable_names: HashMap::new(),
            callables_predeclared: false,
            hw_zone_depth: 0,
            current_sret: None,
        }
    }

    /// Attach a CallableRegistry for FFI function resolution (Sprint 3).
    pub fn with_callables(mut self, callables: CallableRegistry, types: TypeRegistry) -> Self {
        self.callables = Some(callables);
        self.types = Some(types);
        self
    }

    /// Attach the id->name map for callables (used to route HIR calls by name).
    pub fn with_callable_names(mut self, names: HashMap<u32, String>) -> Self {
        self.callable_names = names;
        self
    }

    /// Map a Logicodex TypeId to an LLVM BasicTypeEnum.
    fn type_id_to_llvm(&self, type_id: TypeId) -> Result<BasicTypeEnum<'ctx>> {
        let types = self.types.as_ref().ok_or_else(|| {
            anyhow!("type_id_to_llvm: TypeRegistry not attached — call with_callables()")
        })?;
        match types.resolve(type_id) {
            TypeKind::Primitive(PrimitiveType::Bool) => Ok(self.bool_type.into()),
            TypeKind::Primitive(PrimitiveType::I32) => Ok(self.context.i32_type().into()),
            TypeKind::Primitive(PrimitiveType::I64) => Ok(self.i64_type.into()),
            TypeKind::Primitive(PrimitiveType::U32) => Ok(self.context.i32_type().into()),
            TypeKind::Primitive(PrimitiveType::F32) => Ok(self.context.f32_type().into()),
            TypeKind::Primitive(PrimitiveType::F64) => Ok(self.context.f64_type().into()),
            TypeKind::Primitive(PrimitiveType::Unit) => Ok(self.context.i8_type().into()), // void represented as i8
            TypeKind::Pointer { .. } => Ok(self.context.i8_type().ptr_type(inkwell::AddressSpace::default()).into()),
            other => Err(anyhow!("type_id_to_llvm: unsupported type kind: {:?}", other)),
        }
    }

    /// Declare an extern function in the LLVM module (or retrieve existing declaration).
    fn declare_extern_func(&mut self, signature: &CallableSignature) -> Result<FunctionValue<'ctx>> {
        let name = &signature.name;
        if let Some(func) = self.declared_funcs.get(name) {
            return Ok(*func);
        }
        // Convert param types
        let mut llvm_param_types: Vec<BasicTypeEnum<'ctx>> = Vec::with_capacity(signature.params.len());
        for param_id in &signature.params {
            llvm_param_types.push(self.type_id_to_llvm(*param_id)?);
        }
        let ret_type = self.type_id_to_llvm(signature.return_type)?;
        let fn_type = self.basic_type_fn_type(ret_type, &llvm_param_types, signature.is_variadic);
        let func = self.module.add_function(name, fn_type, Some(inkwell::module::Linkage::External));
        self.declared_funcs.insert(name.clone(), func);
        Ok(func)
    }

    /// Declare a runtime function (for threading primitives).
    /// Caches the declaration to avoid duplicates.
    fn declare_runtime_func(
        &mut self,
        name: &str,
        fn_type: inkwell::types::FunctionType<'ctx>,
    ) -> FunctionValue<'ctx> {
        if let Some(func) = self.declared_funcs.get(name) {
            return *func;
        }
        let func = self.module.add_function(name, fn_type, Some(inkwell::module::Linkage::External));
        self.declared_funcs.insert(name.to_string(), func);
        func
    }

    /// Detect if a callee name is a struct constructor (e.g., "Color" → packed u32).
    fn compare_to_i64(
        &self,
        predicate: IntPredicate,
        left: IntValue<'ctx>,
        right: IntValue<'ctx>,
        name: &str,
    ) -> Result<IntValue<'ctx>> {
        let cmp = self
            .builder
            .build_int_compare(predicate, left, right, name)
            .context("failed to build comparison")?;
        Ok(self.bool_to_i64(cmp))
    }

    fn i64_to_bool(&self, value: IntValue<'ctx>, name: &str) -> Result<IntValue<'ctx>> {
        self.builder
            .build_int_compare(IntPredicate::NE, value, self.i64_type.const_zero(), name)
            .context("failed to normalize integer to bool")
    }

    fn bool_to_i64(&self, value: IntValue<'ctx>) -> IntValue<'ctx> {
        self.builder
            .build_int_z_extend(value, self.i64_type, "booltoint")
            .expect("zext from i1 to i64 is valid")
    }

    fn current_block_has_terminator(&self) -> bool {
        self.builder
            .get_insert_block()
            .and_then(|block| block.get_terminator())
            .is_some()
    }

    fn create_entry_alloca(&self, function: FunctionValue<'ctx>, name: &str) -> PointerValue<'ctx> {
        let entry_builder = self.context.create_builder();
        let entry = function
            .get_first_basic_block()
            .expect("function has entry block");
        match entry.get_first_instruction() {
            Some(first) => entry_builder.position_before(&first),
            None => entry_builder.position_at_end(entry),
        }
        entry_builder
            .build_alloca(self.i64_type, name)
            .expect("alloca in entry block is valid")
    }

    /// v1.42 P3: Create an alloca of a specific type (for struct construction).
    fn create_entry_alloca_typed(
        &self,
        function: FunctionValue<'ctx>,
        name: &str,
        ty: inkwell::types::BasicTypeEnum<'ctx>,
    ) -> PointerValue<'ctx> {
        let entry_builder = self.context.create_builder();
        let entry = function
            .get_first_basic_block()
            .expect("function has entry block");
        match entry.get_first_instruction() {
            Some(first) => entry_builder.position_before(&first),
            None => entry_builder.position_at_end(entry),
        }
        entry_builder
            .build_alloca(ty, name)
            .expect("typed alloca in entry block is valid")
    }

    fn basic_type_fn_type(
        &self,
        ret_type: BasicTypeEnum<'ctx>,
        param_types: &[BasicTypeEnum<'ctx>],
        is_variadic: bool,
    ) -> inkwell::types::FunctionType<'ctx> {
        let meta_params: Vec<inkwell::types::BasicMetadataTypeEnum<'ctx>> = param_types
            .iter()
            .map(|t| (*t).into())
            .collect();
        match ret_type {
            BasicTypeEnum::IntType(t) => t.fn_type(&meta_params, is_variadic),
            BasicTypeEnum::FloatType(t) => t.fn_type(&meta_params, is_variadic),
            BasicTypeEnum::PointerType(t) => t.fn_type(&meta_params, is_variadic),
            BasicTypeEnum::StructType(t) => t.fn_type(&meta_params, is_variadic),
            BasicTypeEnum::ArrayType(t) => t.fn_type(&meta_params, is_variadic),
            BasicTypeEnum::VectorType(t) => t.fn_type(&meta_params, is_variadic),
        }
    }

}

#[cfg(feature = "v1_30")]
/// Entry point for v1.30 HIR-to-object compilation.
/// This is the gate between the v1.21 AST path and the v1.30 HIR path.
/// v1.21 code never calls this function.
pub fn compile_v130(
    hir_module: &crate::hir::HirModule,
    object_path: &Path,
    options: &CodegenOptions,
    callables: CallableRegistry,
    types: TypeRegistry,
    callable_names: HashMap<u32, String>,
) -> Result<CodegenArtifact> {
    let context = Context::create();
    let mut compiler = LlvmCompiler::new(&context, &options.module_name)
        .with_callables(callables, types)
        .with_callable_names(callable_names);

    // v1.38 A6: Pre-declare all callable functions so they're available during HIR codegen
    compiler.predeclare_callables()
        .unwrap_or_else(|e| eprintln!("logicodex v1.38: predeclare_callables warning: {}", e));

    // v1.38 I1: Run semantic gatekeeper as final validation pass before codegen
    {
        let types_clone = compiler.types.as_ref()
            .map(|t| t.clone())
            .unwrap_or_else(TypeRegistry::new);
        if let Err(diagnostics) = crate::semantic_gate::validate_module(hir_module, types_clone) {
            eprintln!("logicodex v1.38: Semantic gatekeeper warnings ({}):", diagnostics.len());
            for d in &diagnostics {
                eprintln!("  [{:?}] {}", d.code, d.message_en);
            }
            // Non-fatal: continue codegen even if gatekeeper has warnings
        }
    }

    // v1.30 uses HIR items instead of v1.21 AST statements
    for item in &hir_module.items {
        match &item.node {
            crate::hir::HirItem::Function(function) => {
                compiler.emit_v130_function(function, options.target)?;
            }
            crate::hir::HirItem::Struct(struct_decl) => {
                compiler.register_hir_struct(struct_decl)
                    .unwrap_or_else(|e| eprintln!("logicodex v1.30: struct registration warning: {}", e));
            }
            crate::hir::HirItem::Enum(_) => {
                eprintln!("logicodex v1.30: enum items are processed at semantic time");
            }
            crate::hir::HirItem::ExternFunction(extern_fn) => {
                compiler.emit_v130_extern_function(extern_fn)?;
            }
        }
    }

    compiler
        .module
        .verify()
        .map_err(|e| anyhow!("LLVM module verification failed (v1.30): {e}"))?;

    let output_kind = if options.target.is_freestanding() {
        OutputKind::FreestandingObject
    } else if options.target.is_wasm() {
        OutputKind::WasmModule
    } else {
        OutputKind::Object
    };
    // v1.44 G8: Use architecture-specific target machine for freestanding
    let target_machine = if let Some(arch) = options.target.arch() {
        build_target_machine_with_arch(output_kind, arch)?
    } else {
        build_target_machine(output_kind)?
    };
    target_machine
        .write_to_file(
            &compiler.module,
            inkwell::targets::FileType::Object,
            object_path,
        )
        .map_err(|e| anyhow!("failed to emit {} file {}: {e}",
            if options.target.is_wasm() { "wasm" } else { "object" },
            object_path.display()))?;

    let ir_path = if options.emit_ir {
        let mut ir_path = object_path.to_path_buf();
        ir_path.set_extension("ll");
        compiler
            .module
            .print_to_file(&ir_path)
            .map_err(|e| anyhow!("failed to write LLVM IR {}: {e}", ir_path.display()))?;
        Some(ir_path)
    } else {
        None
    };

    Ok(CodegenArtifact {
        object_path: object_path.to_path_buf(),
        ir_path,
    })
}

// =========================================================================
// v1.36.0-alpha: HIR Function Codegen — Full LLVM IR Emission (A1 Critical)
//
// Replaces previous stubs with complete HIR → LLVM lowering for:
//   - Functions with parameters and return values
//   - Local variables (Let/Assign)
//   - Control flow (If/While/Loop/Break/Continue)
//   - Expressions (Literal, Local, Call, Binary, Unary, Cast)
//   - Extern function declarations
// =========================================================================
impl<'ctx> LlvmCompiler<'ctx> {
    /// Emit a HIR function definition into the LLVM module.
    #[cfg(feature = "v1_30")]
    /// If `ty` is a struct, rebuild its LLVM struct type from the registry layout.
    #[cfg(feature = "v1_30")]
    fn resolve_struct_llvm(&self, ty: crate::types::TypeRef) -> Result<Option<inkwell::types::StructType<'ctx>>> {
        let layout = match self.types.as_ref() {
            Some(t) => match t.resolve(ty.id) {
                crate::types::TypeKind::Struct(lid) => t.get_struct_layout(*lid).cloned(),
                _ => None,
            },
            None => None,
        };
        let layout = match layout { Some(l) => l, None => return Ok(None) };
        let mut field_llvm: Vec<BasicTypeEnum<'ctx>> = Vec::with_capacity(layout.fields.len());
        for f in &layout.fields {
            field_llvm.push(self.hir_type_to_llvm(crate::types::TypeRef { id: f.ty })?);
        }
        Ok(Some(self.context.struct_type(&field_llvm, false)))
    }

    fn emit_v130_function(
        &mut self,
        function: &crate::hir::HirFunction,
        _target: CompilationTarget,
    ) -> Result<()> {
        use crate::hir::{HirParam};

        // 1. Determine LLVM parameter types
        let mut param_types: Vec<BasicTypeEnum<'ctx>> = Vec::new();
        for param in &function.params {
            param_types.push(self.hir_type_to_llvm(param.ty)?);
        }

        // 1b. Struct-return ABI (sret): a struct-returning function takes a
        // hidden leading pointer to a caller-allocated buffer, fills it, and
        // returns that pointer (as i64).
        let sret_struct = self.resolve_struct_llvm(function.return_type)?;
        if let Some(st) = sret_struct {
            param_types.insert(0, st.ptr_type(AddressSpace::default()).into());
        }

        // 2. Determine return type. A Unit-returning function must be an LLVM
        // `void` function (not i8) so the implicit `ret void` terminator below
        // matches the declared type and passes module verification.
        let is_unit_return = function.return_type.id == self.unit_type_id();
        let fn_type = if is_unit_return {
            let meta_params: Vec<inkwell::types::BasicMetadataTypeEnum<'ctx>> =
                param_types.iter().map(|t| (*t).into()).collect();
            self.context.void_type().fn_type(&meta_params, false)
        } else {
            let ret_type = self.hir_type_to_llvm(function.return_type)?;
            self.basic_type_fn_type(ret_type, &param_types, false)
        };

        // 3. Create LLVM function
        let func = self.module.add_function(&function.name, fn_type, None);
        // Register so calls to this function (incl. recursion) resolve to it.
        let this_callable_id = self
            .callable_names
            .iter()
            .find(|(_, n)| n.as_str() == function.name)
            .map(|(id, _)| *id);
        if let Some(id) = this_callable_id {
            self.hir_callable_funcs.insert(id, func);
        }
        let entry = self.context.append_basic_block(func, "entry");
        self.builder.position_at_end(entry);

        // Record the sret buffer pointer (param 0) for the return handler.
        self.current_sret = sret_struct.map(|st| {
            let pv = func.get_nth_param(0).unwrap().into_pointer_value();
            (pv, st)
        });
        let param_offset = if self.current_sret.is_some() { 1 } else { 0 };

        // 4. Clear locals from previous function
        self.hir_local_allocs.clear();

        // 5. Allocate parameters as local variables
        for (idx, HirParam { local, ty: _, .. }) in function.params.iter().enumerate() {
            let param_val = func.get_nth_param((idx + param_offset) as u32)
                .ok_or_else(|| anyhow!("function param {} out of range", idx))?;
            let alloca = self.create_entry_alloca(func, &format!("param_{}", idx));
            self.builder.build_store(alloca, param_val)
                .context("failed to store parameter")?;
            self.hir_local_allocs.insert(local.0, alloca);
        }

        // 6. Emit function body
        self.emit_hir_block(&function.body, func)?;

        // 7. Ensure the function has a terminator (implicit return if needed)
        if !self.current_block_has_terminator() {
            if function.return_type.id == self.unit_type_id() {
                self.builder.build_return(None)
                    .context("failed to build implicit void return")?;
            } else if let Some((sret_ptr, _)) = self.current_sret {
                let ret = self.builder
                    .build_ptr_to_int(sret_ptr, self.i64_type, "sret_ret")
                    .context("sret implicit return")?;
                self.builder.build_return(Some(&ret))
                    .context("failed to build implicit sret return")?;
            } else {
                // All HIR expressions produce i64 — return 0 as default
                let zero = self.i64_type.const_int(0, false);
                self.builder.build_return(Some(&zero))
                    .context("failed to build implicit zero return")?;
            }
        }

        self.current_sret = None;
        Ok(())
    }

    /// Declare a HIR extern function in the LLVM module.
    #[cfg(feature = "v1_30")]
    fn emit_v130_extern_function(
        &mut self,
        extern_fn: &crate::hir::HirExternFunction,
    ) -> Result<()> {
        let signature = {
            let callables = self.callables.as_ref()
                .ok_or_else(|| anyhow!("extern function codegen: CallableRegistry not attached"))?;
            callables.get(extern_fn.callable)
                .ok_or_else(|| anyhow!("extern function CallableId({}) not found in registry", extern_fn.callable.0))?
                .clone()
        };
        let func = self.declare_extern_func(&signature)?;
        self.hir_callable_funcs.insert(extern_fn.callable.0, func);
        Ok(())
    }

    // ─── HIR Block / Statement / Expression Emitters ───

    #[cfg(feature = "v1_30")]
    fn emit_hir_block(
        &mut self,
        block: &crate::hir::HirBlock,
        func: FunctionValue<'ctx>,
    ) -> Result<()> {
        for stmt in &block.statements {
            if self.current_block_has_terminator() {
                break;
            }
            self.emit_hir_stmt(&stmt.node, func)?;
        }
        Ok(())
    }

    #[cfg(feature = "v1_30")]
    fn emit_hir_stmt(
        &mut self,
        stmt: &crate::hir::HirStmt,
        func: FunctionValue<'ctx>,
    ) -> Result<()> {
        use crate::hir::{HirStmt};
        match stmt {
            HirStmt::Let { local, ty, value } => {
                let alloca = self.create_entry_alloca(func, &format!("local_{}", local.0));
                if let Some(val_expr) = value {
                    let val = self.emit_hir_expr(val_expr, func)?;
                    let val = self.wrap_to_width(val, *ty)?;
                    self.builder.build_store(alloca, val)
                        .context("failed to store let value")?;
                }
                self.hir_local_allocs.insert(local.0, alloca);
                self.hir_local_types.insert(local.0, *ty);
                Ok(())
            }
            HirStmt::Assign { target, value } => {
                let val = self.emit_hir_expr(value, func)?;
                match &target.kind {
                    crate::hir::HirExprKind::Local(local_id) => {
                        let val = match self.hir_local_types.get(&local_id.0).copied() {
                            Some(lty) => self.wrap_to_width(val, lty)?,
                            None => val,
                        };
                        let ptr = self.hir_local_allocs.get(&local_id.0)
                            .ok_or_else(|| anyhow!("assign target local {} not found", local_id.0))?;
                        self.builder.build_store(*ptr, val)
                            .context("failed to store assign value")?;
                    }
                    crate::hir::HirExprKind::Field { base, field_index } => {
                        // p.field = val: int->ptr the struct i64, gep field, store.
                        let base_val = self.emit_hir_expr(base, func)?;
                        let layout = match self.types.as_ref() {
                            Some(t) => match t.resolve(base.ty.id) {
                                crate::types::TypeKind::Struct(lid) => t.get_struct_layout(*lid).cloned(),
                                _ => None,
                            },
                            None => None,
                        };
                        if let Some(layout) = layout {
                            let mut field_llvm: Vec<BasicTypeEnum<'ctx>> = Vec::with_capacity(layout.fields.len());
                            for f in &layout.fields {
                                field_llvm.push(self.hir_type_to_llvm(crate::types::TypeRef { id: f.ty })?);
                            }
                            let struct_type = self.context.struct_type(&field_llvm, false);
                            let ptr = self.builder
                                .build_int_to_ptr(base_val, struct_type.ptr_type(AddressSpace::default()), "assign_base_ptr")
                                .context("field assign int->ptr")?;
                            let field_ptr = unsafe {
                                self.builder.build_struct_gep(struct_type, ptr, *field_index as u32, "assign_field_ptr")
                                    .context("field assign gep")?
                            };
                            let fty = crate::types::TypeRef { id: layout.fields[*field_index].ty };
                            let val = self.wrap_to_width(val, fty)?;
                            self.builder.build_store(field_ptr, val)
                                .context("field assign store")?;
                        }
                    }
                    _ => {}
                }
                Ok(())
            }
            HirStmt::If { condition, then_branch, else_branch } => {
                self.emit_hir_if(condition, then_branch, else_branch.as_ref(), func)
            }
            HirStmt::While { condition, body } => {
                self.emit_hir_while(condition, body, func)
            }
            HirStmt::Loop { body } => {
                self.emit_hir_loop(body, func)
            }
            HirStmt::Break { .. } => {
                let target = self.loop_targets.last()
                    .ok_or_else(|| anyhow!("break outside loop"))?.break_block;
                self.builder.build_unconditional_branch(target)
                    .context("failed to build break")?;
                Ok(())
            }
            HirStmt::Continue { .. } => {
                let target = self.loop_targets.last()
                    .ok_or_else(|| anyhow!("continue outside loop"))?.continue_block;
                self.builder.build_unconditional_branch(target)
                    .context("failed to build continue")?;
                Ok(())
            }
            HirStmt::UnsafeBlock(block) => {
                self.emit_hir_block(block, func)
            }
            HirStmt::Expr(expr) => {
                let _ = self.emit_hir_expr(expr, func)?;
                Ok(())
            }
            HirStmt::Return(expr) => {
                if let Some(val_expr) = expr {
                    let val = self.emit_hir_expr(val_expr, func)?;
                    if let Some((sret_ptr, struct_ty)) = self.current_sret {
                        let src = self.builder
                            .build_int_to_ptr(val, struct_ty.ptr_type(AddressSpace::default()), "ret_src")
                            .context("sret src int->ptr")?;
                        let n = struct_ty.count_fields();
                        for i in 0..n {
                            let sf = unsafe {
                                self.builder.build_struct_gep(struct_ty, src, i, "ret_sf")
                                    .context("sret src gep")?
                            };
                            let df = unsafe {
                                self.builder.build_struct_gep(struct_ty, sret_ptr, i, "ret_df")
                                    .context("sret dst gep")?
                            };
                            let fty = struct_ty.get_field_type_at_index(i)
                                .ok_or_else(|| anyhow!("sret field type"))?;
                            let v = self.builder.build_load(fty, sf, "ret_fv")
                                .context("sret field load")?;
                            self.builder.build_store(df, v).context("sret field store")?;
                        }
                        let ret = self.builder
                            .build_ptr_to_int(sret_ptr, self.i64_type, "sret_ret")
                            .context("sret ptr->int")?;
                        self.builder.build_return(Some(&ret))
                            .context("failed to build sret return")?;
                    } else {
                        self.builder.build_return(Some(&val))
                            .context("failed to build return")?;
                    }
                } else {
                    self.builder.build_return(None)
                        .context("failed to build void return")?;
                }
                Ok(())
            }
        }
    }

    /// Emulate fixed-width integer semantics on the uniform i64 working value:
    /// truncate to the type's bit width then re-extend (sign- or zero-extend)
    /// back to i64, so the value wraps exactly as a register of that width would.
    /// A no-op for 64-bit ints and non-integer types.
    #[cfg(feature = "v1_30")]
    fn wrap_to_width(&self, value: IntValue<'ctx>, ty: crate::types::TypeRef) -> Result<IntValue<'ctx>> {
        let prim = match self.types.as_ref() {
            Some(t) => match t.resolve(ty.id) {
                TypeKind::Primitive(p) => *p,
                _ => return Ok(value),
            },
            None => return Ok(value),
        };
        let bits = match prim.int_bits() {
            Some(b) if b < 64 => b,
            _ => return Ok(value),
        };
        let narrow_ty = self.context.custom_width_int_type(bits);
        let truncated = self
            .builder
            .build_int_truncate(value, narrow_ty, "wrap_trunc")
            .context("wrap truncate")?;
        let extended = if prim.is_unsigned_int() {
            self.builder
                .build_int_z_extend(truncated, self.i64_type, "wrap_zext")
                .context("wrap zext")?
        } else {
            self.builder
                .build_int_s_extend(truncated, self.i64_type, "wrap_sext")
                .context("wrap sext")?
        };
        Ok(extended)
    }

    #[cfg(feature = "v1_30")]
    fn emit_hir_expr(
        &mut self,
        expr: &crate::hir::HirExpr,
        func: FunctionValue<'ctx>,
    ) -> Result<IntValue<'ctx>> {
        use crate::hir::{HirExprKind, BinaryOpAst, UnaryOpAst, LiteralAst};
        match &expr.kind {
            HirExprKind::Literal(lit) => match lit {
                LiteralAst::Integer(v) => Ok(self.i64_type.const_int(*v as u64, true)),
                LiteralAst::Boolean(v) => Ok(self.bool_to_i64(self.bool_type.const_int(*v as u64, false))),
                LiteralAst::String(s) => Ok(self.i64_type.const_int(s.len() as u64, false)),
                LiteralAst::Unit => Ok(self.i64_type.const_int(0, false)),
            }
            HirExprKind::Local(local_id) => {
                let ptr = self.hir_local_allocs.get(&local_id.0)
                    .ok_or_else(|| anyhow!("local {} not allocated", local_id.0))?;
                Ok(self.builder.build_load(self.i64_type, *ptr, &format!("local_{}", local_id.0))
                    .context("failed to load local")?
                    .into_int_value())
            }
            HirExprKind::Global(_symbol_id) => {
                // Global: try to resolve as a zero-argument function call
                Ok(self.i64_type.const_int(0, false))
            }
            HirExprKind::Binary { left, op, right } => {
                let l = self.emit_hir_expr(left, func)?;
                let r = self.emit_hir_expr(right, func)?;
                let result = (match op {
                    BinaryOpAst::Add => self.builder.build_int_add(l, r, "addtmp").context("add"),
                    BinaryOpAst::Sub => self.builder.build_int_sub(l, r, "subtmp").context("sub"),
                    BinaryOpAst::Mul => self.builder.build_int_mul(l, r, "multmp").context("mul"),
                    BinaryOpAst::Div => self.builder.build_int_signed_div(l, r, "divtmp").context("div"),
                    BinaryOpAst::Eq => self.compare_to_i64(IntPredicate::EQ, l, r, "eqtmp"),
                    BinaryOpAst::NotEq => self.compare_to_i64(IntPredicate::NE, l, r, "netmp"),
                    BinaryOpAst::Lt => self.compare_to_i64(IntPredicate::SLT, l, r, "lttmp"),
                    BinaryOpAst::Lte => self.compare_to_i64(IntPredicate::SLE, l, r, "letmp"),
                    BinaryOpAst::Gt => self.compare_to_i64(IntPredicate::SGT, l, r, "gttmp"),
                    BinaryOpAst::Gte => self.compare_to_i64(IntPredicate::SGE, l, r, "getmp"),
                    BinaryOpAst::LogicalAnd => {
                        let lb = self.i64_to_bool(l, "andlhs")?;
                        let rb = self.i64_to_bool(r, "andrhs")?;
                        let v = self.builder.build_and(lb, rb, "andtmp").context("and")?;
                        Ok(self.bool_to_i64(v))
                    }
                    BinaryOpAst::LogicalOr => {
                        let lb = self.i64_to_bool(l, "orlhs")?;
                        let rb = self.i64_to_bool(r, "orrhs")?;
                        let v = self.builder.build_or(lb, rb, "ortmp").context("or")?;
                        Ok(self.bool_to_i64(v))
                    }
                    BinaryOpAst::BitAnd => self.builder.build_and(l, r, "bitandtmp").context("bitand"),
                    BinaryOpAst::BitOr => self.builder.build_or(l, r, "bitortmp").context("bitor"),
                    BinaryOpAst::BitXor => self.builder.build_xor(l, r, "xortmp").context("xor"),
                    BinaryOpAst::ShiftLeft => self.builder.build_left_shift(l, r, "shltmp").context("shl"),
                    BinaryOpAst::ShiftRight => self.builder.build_right_shift(l, r, true, "shrtmp").context("shr"),
                })?;
                self.wrap_to_width(result, expr.ty)
            }
            HirExprKind::Unary { op, expr } => {
                let val = self.emit_hir_expr(expr, func)?;
                let result = (match op {
                    UnaryOpAst::Negate => self.builder.build_int_neg(val, "negtmp").context("neg"),
                    UnaryOpAst::LogicalNot => {
                        let b = self.i64_to_bool(val, "notcond")?;
                        let not_b = self.builder.build_not(b, "nottmp").context("not")?;
                        Ok(self.bool_to_i64(not_b))
                    }
                    UnaryOpAst::AddressOf => Ok(self.i64_type.const_int(0, false)), // placeholder
                    UnaryOpAst::Deref => Ok(self.i64_type.const_int(0, false)), // placeholder
                })?;
                self.wrap_to_width(result, expr.ty)
            }
            HirExprKind::Call { callee, args } => {
                self.emit_hir_call(*callee, args, func, expr.ty)
            }
            HirExprKind::Field { base, field_index } => {
                let base_val = self.emit_hir_expr(base, func)?;
                let layout = match self.types.as_ref() {
                    Some(t) => match t.resolve(base.ty.id) {
                        crate::types::TypeKind::Struct(lid) => t.get_struct_layout(*lid).cloned(),
                        _ => None,
                    },
                    None => None,
                };
                let layout = match layout {
                    Some(l) => l,
                    None => return Ok(self.i64_type.const_int(0, false)),
                };
                let mut field_llvm: Vec<BasicTypeEnum<'ctx>> = Vec::with_capacity(layout.fields.len());
                for f in &layout.fields {
                    field_llvm.push(self.hir_type_to_llvm(crate::types::TypeRef { id: f.ty })?);
                }
                let struct_type = self.context.struct_type(&field_llvm, false);
                let field_ty = field_llvm.get(*field_index)
                    .copied()
                    .unwrap_or_else(|| self.i64_type.into());
                let ptr = self.builder
                    .build_int_to_ptr(base_val, struct_type.ptr_type(AddressSpace::default()), "field_base_ptr")
                    .context("field base int->ptr")?;
                let field_ptr = unsafe {
                    self.builder.build_struct_gep(struct_type, ptr, *field_index as u32, "field_ptr")
                        .context("field gep")?
                };
                let loaded = self.builder.build_load(field_ty, field_ptr, "field_val")
                    .context("field load")?;
                Ok(loaded.into_int_value())
            }
            HirExprKind::Cast { expr, .. } => {
                // For now, emit the inner expression (casts are no-ops at LLVM level for compatible types)
                self.emit_hir_expr(expr, func)
            }
            // ─── v1.30 Threading Expressions (A3) — LLVM Codegen ───
            HirExprKind::Spawn { actor_name, args } => {
                // Declare runtime function: logicodex_spawn(actor_name: *const u8) -> i64
                let spawn_fn = self.declare_runtime_func(
                    "logicodex_spawn",
                    self.i64_type.fn_type(&[self.context.i8_type().ptr_type(AddressSpace::default()).into()], false),
                );
                // Evaluate args (passed as pointers or values)
                let mut llvm_args: Vec<BasicValueEnum<'ctx>> = Vec::new();
                for arg in args {
                    let val = self.emit_hir_expr(arg, func)?;
                    llvm_args.push(val.into());
                }
                // Pass actor name as a global string
                let name_ptr = self.builder.build_global_string_ptr(actor_name, "spawn_actor_name")
                    .context("spawn actor name")?
                    .as_pointer_value();
                let call_site = self.builder
                    .build_call(spawn_fn, &[name_ptr.into()], "spawn_call")
                    .context("spawn call")?;
                match call_site.try_as_basic_value().left() {
                    Some(val) => match val {
                        BasicValueEnum::IntValue(iv) => Ok(iv),
                        _ => Ok(self.i64_type.const_int(0, false)),
                    }
                    None => Ok(self.i64_type.const_int(0, false)),
                }
            }
            HirExprKind::Join { actor_name } => {
                let join_fn = self.declare_runtime_func(
                    "logicodex_join",
                    self.i64_type.fn_type(&[self.context.i8_type().ptr_type(AddressSpace::default()).into()], false),
                );
                let name_ptr = self.builder.build_global_string_ptr(actor_name, "join_actor_name")
                    .context("join actor name")?
                    .as_pointer_value();
                let call_site = self.builder
                    .build_call(join_fn, &[name_ptr.into()], "join_call")
                    .context("join call")?;
                match call_site.try_as_basic_value().left() {
                    Some(val) => match val {
                        BasicValueEnum::IntValue(iv) => Ok(iv),
                        _ => Ok(self.i64_type.const_int(0, false)),
                    }
                    None => Ok(self.i64_type.const_int(0, false)),
                }
            }
            HirExprKind::ChannelSend { channel_name, value } => {
                let send_fn = self.declare_runtime_func(
                    "logicodex_channel_send",
                    self.i64_type.fn_type(&[
                        self.context.i8_type().ptr_type(AddressSpace::default()).into(),
                        self.i64_type.into(),
                    ], false),
                );
                let val = self.emit_hir_expr(value, func)?;
                let name_ptr = self.builder.build_global_string_ptr(channel_name, "send_channel_name")
                    .context("send channel name")?
                    .as_pointer_value();
                let call_site = self.builder
                    .build_call(send_fn, &[name_ptr.into(), val.into()], "send_call")
                    .context("send call")?;
                match call_site.try_as_basic_value().left() {
                    Some(val) => match val {
                        BasicValueEnum::IntValue(iv) => Ok(iv),
                        _ => Ok(self.i64_type.const_int(0, false)),
                    }
                    None => Ok(self.i64_type.const_int(0, false)),
                }
            }
            HirExprKind::ChannelRecv { channel_name } => {
                let recv_fn = self.declare_runtime_func(
                    "logicodex_channel_recv",
                    self.i64_type.fn_type(&[self.context.i8_type().ptr_type(AddressSpace::default()).into()], false),
                );
                let name_ptr = self.builder.build_global_string_ptr(channel_name, "recv_channel_name")
                    .context("recv channel name")?
                    .as_pointer_value();
                let call_site = self.builder
                    .build_call(recv_fn, &[name_ptr.into()], "recv_call")
                    .context("recv call")?;
                match call_site.try_as_basic_value().left() {
                    Some(val) => match val {
                        BasicValueEnum::IntValue(iv) => Ok(iv),
                        _ => Ok(self.i64_type.const_int(0, false)),
                    }
                    None => Ok(self.i64_type.const_int(0, false)),
                }
            }
            // ─── v1.30 Phase 3: Backpressure + Scheduler (A4) ───
            HirExprKind::ChannelTrySend { channel_name, value } => {
                let send_fn = self.declare_runtime_func(
                    "logicodex_channel_try_send",
                    self.i64_type.fn_type(&[
                        self.context.i8_type().ptr_type(AddressSpace::default()).into(),
                        self.i64_type.into(),
                    ], false),
                );
                let val = self.emit_hir_expr(value, func)?;
                let name_ptr = self.builder.build_global_string_ptr(channel_name, "trysend_channel_name")
                    .context("trysend channel name")?
                    .as_pointer_value();
                let call_site = self.builder
                    .build_call(send_fn, &[name_ptr.into(), val.into()], "trysend_call")
                    .context("trysend call")?;
                match call_site.try_as_basic_value().left() {
                    Some(val) => match val {
                        BasicValueEnum::IntValue(iv) => Ok(iv),
                        _ => Ok(self.i64_type.const_int(1, false)), // default: true (success)
                    }
                    None => Ok(self.i64_type.const_int(1, false)),
                }
            }
            HirExprKind::ChannelTryRecv { channel_name } => {
                let recv_fn = self.declare_runtime_func(
                    "logicodex_channel_try_recv",
                    self.i64_type.fn_type(&[self.context.i8_type().ptr_type(AddressSpace::default()).into()], false),
                );
                let name_ptr = self.builder.build_global_string_ptr(channel_name, "tryrecv_channel_name")
                    .context("tryrecv channel name")?
                    .as_pointer_value();
                let call_site = self.builder
                    .build_call(recv_fn, &[name_ptr.into()], "tryrecv_call")
                    .context("tryrecv call")?;
                match call_site.try_as_basic_value().left() {
                    Some(val) => match val {
                        BasicValueEnum::IntValue(iv) => Ok(iv),
                        _ => Ok(self.i64_type.const_int(0, false)), // default: 0 (None)
                    }
                    None => Ok(self.i64_type.const_int(0, false)),
                }
            }
            HirExprKind::Yield => {
                let yield_fn = self.declare_runtime_func(
                    "logicodex_yield",
                    self.i64_type.fn_type(&[], false),
                );
                let call_site = self.builder
                    .build_call(yield_fn, &[], "yield_call")
                    .context("yield call")?;
                match call_site.try_as_basic_value().left() {
                    Some(val) => match val {
                        BasicValueEnum::IntValue(iv) => Ok(iv),
                        _ => Ok(self.i64_type.const_int(0, false)),
                    }
                    None => Ok(self.i64_type.const_int(0, false)),
                }
            }
            HirExprKind::Sleep { duration_ms } => {
                let sleep_fn = self.declare_runtime_func(
                    "logicodex_sleep",
                    self.i64_type.fn_type(&[self.i64_type.into()], false),
                );
                let dur = self.emit_hir_expr(duration_ms, func)?;
                let call_site = self.builder
                    .build_call(sleep_fn, &[dur.into()], "sleep_call")
                    .context("sleep call")?;
                match call_site.try_as_basic_value().left() {
                    Some(val) => match val {
                        BasicValueEnum::IntValue(iv) => Ok(iv),
                        _ => Ok(self.i64_type.const_int(0, false)),
                    }
                    None => Ok(self.i64_type.const_int(0, false)),
                }
            }
            HirExprKind::ChannelTimeoutRecv { channel_name, timeout_ms } => {
                let recv_fn = self.declare_runtime_func(
                    "logicodex_timeout_recv",
                    self.i64_type.fn_type(&[
                        self.context.i8_type().ptr_type(AddressSpace::default()).into(),
                        self.i64_type.into(),
                    ], false),
                );
                let to = self.emit_hir_expr(timeout_ms, func)?;
                let name_ptr = self.builder.build_global_string_ptr(channel_name, "torecv_channel_name")
                    .context("torecv channel name")?
                    .as_pointer_value();
                let call_site = self.builder
                    .build_call(recv_fn, &[name_ptr.into(), to.into()], "torecv_call")
                    .context("timeout_recv call")?;
                match call_site.try_as_basic_value().left() {
                    Some(val) => match val {
                        BasicValueEnum::IntValue(iv) => Ok(iv),
                        _ => Ok(self.i64_type.const_int(0, false)),
                    }
                    None => Ok(self.i64_type.const_int(0, false)),
                }
            }
        }
    }

    #[cfg(feature = "v1_30")]
    fn emit_hir_call(
        &mut self,
        callee: crate::types::CallableId,
        args: &[crate::hir::HirExpr],
        func: FunctionValue<'ctx>,
        ret_ty: crate::types::TypeRef,
    ) -> Result<IntValue<'ctx>> {
        // Resolve the callee by NAME (the HIR Call.callee is a SymbolTable id,
        // independent of the FFI registry), avoiding aliasing a builtin like
        // `print` onto an unrelated extern (e.g. InitWindow).
        let name = self.callable_names.get(&callee.0).cloned();

        // Builtin: `print` lowers to a Call here — route to the runtime print fn.
        if name.as_deref() == Some("print") {
            let mut last = self.i64_type.const_int(0, false);
            for arg in args {
                let val = self.emit_hir_expr(arg, func)?;
                self.builder
                    .build_call(self.print_fn, &[val.into()], "printtmp")
                    .context("failed to build print call")?;
                last = val;
            }
            return Ok(last);
        }

        // Struct constructor (detected by name via the type registry).
        if let Some(ref n) = name {
            let is_struct = self.types.as_ref()
                .map(|t| t.find_struct_by_name(n).is_some())
                .unwrap_or(false);
            if is_struct || self.hir_struct_names.contains_key(n) {
                return self.emit_hir_struct_constructor(n, args, func);
            }
        }

        // Resolve the LLVM function: cached HIR/user function first, else a
        // genuine FFI extern resolved by name.
        let llvm_func = if let Some(f) = self.hir_callable_funcs.get(&callee.0) {
            *f
        } else {
            let signature = name
                .as_deref()
                .and_then(|n| {
                    self.callables
                        .as_ref()
                        .and_then(|c| c.find_by_name(n).map(|(_, s)| s.clone()))
                })
                .ok_or_else(|| {
                    anyhow!(
                        "HIR call: callee '{}' (CallableId {}) not resolvable",
                        name.as_deref().unwrap_or("?"),
                        callee.0
                    )
                })?;
            self.declare_extern_func(&signature)?
        };

        // Struct-return ABI: if the callee returns a struct, allocate the result
        // buffer in the caller frame and pass it as the hidden sret argument.
        let sret_buf = match self.resolve_struct_llvm(ret_ty)? {
            Some(st) => Some(self.create_entry_alloca_typed(func, "sret_buf", st.into())),
            None => None,
        };

        // Evaluate arguments and emit the call.
        let mut llvm_args: Vec<BasicValueEnum<'ctx>> = Vec::new();
        if let Some(buf) = sret_buf {
            llvm_args.push(buf.into());
        }
        for arg in args {
            let val = self.emit_hir_expr(arg, func)?;
            llvm_args.push(val.into());
        }
        let label = name.as_deref().unwrap_or("call");
        let call_site = self.builder
            .build_call(
                llvm_func,
                &llvm_args.iter().map(|a| (*a).into()).collect::<Vec<inkwell::values::BasicMetadataValueEnum>>(),
                &format!("call_{}", label),
            )
            .with_context(|| format!("failed to build call to '{}'", label))?;

        match call_site.try_as_basic_value().left() {
            Some(BasicValueEnum::IntValue(iv)) => Ok(iv),
            _ => Ok(self.i64_type.const_int(0, false)),
        }
    }

    #[cfg(feature = "v1_30")]
    fn emit_hir_if(
        &mut self,
        condition: &crate::hir::HirExpr,
        then_branch: &crate::hir::HirBlock,
        else_branch: Option<&crate::hir::HirBlock>,
        func: FunctionValue<'ctx>,
    ) -> Result<()> {
        let cond_val = self.emit_hir_expr(condition, func)?;
        let cond_bool = self.i64_to_bool(cond_val, "ifcond")?;
        let then_bb = self.context.append_basic_block(func, "then");
        let else_bb = self.context.append_basic_block(func, "else");
        let merge_bb = self.context.append_basic_block(func, "ifcont");
        self.builder.build_conditional_branch(cond_bool, then_bb, else_bb)
            .context("if branch")?;

        self.builder.position_at_end(then_bb);
        self.emit_hir_block(then_branch, func)?;
        if !self.current_block_has_terminator() {
            self.builder.build_unconditional_branch(merge_bb).context("then→merge")?;
        }

        self.builder.position_at_end(else_bb);
        if let Some(else_b) = else_branch {
            self.emit_hir_block(else_b, func)?;
        }
        if !self.current_block_has_terminator() {
            self.builder.build_unconditional_branch(merge_bb).context("else→merge")?;
        }

        self.builder.position_at_end(merge_bb);
        Ok(())
    }

    #[cfg(feature = "v1_30")]
    fn emit_hir_while(
        &mut self,
        condition: &crate::hir::HirExpr,
        body: &crate::hir::HirBlock,
        func: FunctionValue<'ctx>,
    ) -> Result<()> {
        let cond_bb = self.context.append_basic_block(func, "while.cond");
        let body_bb = self.context.append_basic_block(func, "while.body");
        let end_bb = self.context.append_basic_block(func, "while.end");
        self.builder.build_unconditional_branch(cond_bb).context("while→cond")?;

        self.builder.position_at_end(cond_bb);
        let cond_val = self.emit_hir_expr(condition, func)?;
        let cond_bool = self.i64_to_bool(cond_val, "whilecond")?;
        self.builder.build_conditional_branch(cond_bool, body_bb, end_bb)
            .context("while branch")?;

        self.builder.position_at_end(body_bb);
        self.loop_targets.push(LoopTarget { continue_block: cond_bb, break_block: end_bb });
        self.emit_hir_block(body, func)?;
        self.loop_targets.pop();
        if !self.current_block_has_terminator() {
            self.builder.build_unconditional_branch(cond_bb).context("while body→cond")?;
        }

        self.builder.position_at_end(end_bb);
        Ok(())
    }

    #[cfg(feature = "v1_30")]
    fn emit_hir_loop(
        &mut self,
        body: &crate::hir::HirBlock,
        func: FunctionValue<'ctx>,
    ) -> Result<()> {
        let body_bb = self.context.append_basic_block(func, "loop.body");
        let end_bb = self.context.append_basic_block(func, "loop.end");
        self.builder.build_unconditional_branch(body_bb).context("loop→body")?;

        self.builder.position_at_end(body_bb);
        self.loop_targets.push(LoopTarget { continue_block: body_bb, break_block: end_bb });
        self.emit_hir_block(body, func)?;
        self.loop_targets.pop();
        if !self.current_block_has_terminator() {
            self.builder.build_unconditional_branch(body_bb).context("loop body→body")?;
        }

        self.builder.position_at_end(end_bb);
        Ok(())
    }

    // ─── HIR Type Helpers ───

    #[cfg(feature = "v1_30")]
    fn hir_type_to_llvm(&self, type_ref: crate::types::TypeRef) -> Result<BasicTypeEnum<'ctx>> {
        let types = self.types.as_ref()
            .ok_or_else(|| anyhow!("hir_type_to_llvm: TypeRegistry not attached"))?;
        // v1.30 codegen uses a uniform i64 integer model: every HIR expression
        // produces an i64, so all integer-typed params/returns are i64 too. This
        // keeps call args, returns and arithmetic consistent. (Fixed-width int
        // semantics — true 32-bit wrapping — would require trunc/extend at each
        // boundary and is deferred.)
        match types.resolve(type_ref.id) {
            TypeKind::Primitive(PrimitiveType::Bool) => Ok(self.bool_type.into()),
            TypeKind::Primitive(PrimitiveType::F32) => Ok(self.context.f32_type().into()),
            TypeKind::Primitive(PrimitiveType::F64) => Ok(self.context.f64_type().into()),
            TypeKind::Primitive(PrimitiveType::Unit) => Ok(self.context.i8_type().into()),
            // All integer widths (I8..U64) collapse to i64 in this model.
            _ => Ok(self.i64_type.into()),
        }
    }

    #[cfg(feature = "v1_30")]
    fn unit_type_id(&self) -> TypeId {
        // Unit is represented as i8 (void)
        self.types.as_ref()
            .map(|t| t.primitive(PrimitiveType::Unit))
            .unwrap_or_else(|| TypeId(6)) // fallback
    }

    // ─── v1.38 A6: CallableRegistry Predeclaration ───

    /// Pre-declare all callable functions from the CallableRegistry.
    /// Must be called before codegen if CallableRegistry is attached.
    #[cfg(feature = "v1_30")]
    fn predeclare_callables(&mut self) -> Result<()> {
        if self.callables_predeclared {
            return Ok(()); // already done
        }
        let callables = match self.callables.as_ref() {
            Some(c) => c,
            None => return Ok(()), // no registry attached — nothing to do
        };
        // Iterate through all registered callables and declare them
        for (_idx, sig) in callables.signatures.iter().enumerate() {
            let name = &sig.name;
            if self.declared_funcs.contains_key(name) {
                continue; // already declared
            }
            // Determine whether the return type is void (Unit)
            let is_void = if let Some(types) = self.types.as_ref() {
                matches!(types.resolve(sig.return_type), crate::types::TypeKind::Primitive(crate::types::PrimitiveType::Unit))
            } else {
                false
            };
            // Build parameter types
            let mut param_types: Vec<BasicTypeEnum<'ctx>> = Vec::new();
            for _ in &sig.params {
                param_types.push(self.i64_type.into());
            }
            // Build function type
            let fn_type = if is_void {
                self.context.void_type().fn_type(&param_types.iter().map(|t| (*t).into()).collect::<Vec<inkwell::types::BasicMetadataTypeEnum>>(), sig.is_variadic)
            } else {
                let ret_type: BasicTypeEnum<'ctx> = self.i64_type.into();
                self.basic_type_fn_type(ret_type, &param_types, sig.is_variadic)
            };
            let func = self.module.add_function(name, fn_type, None);
            self.declared_funcs.insert(name.clone(), func);
        }
        self.callables_predeclared = true;
        Ok(())
    }

    // ─── v1.36 A5: Struct Registration ───

    /// Register a HIR struct declaration, creating its LLVM struct type.
    #[cfg(feature = "v1_30")]
    fn register_hir_struct(
        &mut self,
        struct_decl: &crate::hir::HirStructDecl,
    ) -> Result<()> {
        let field_types: Result<Vec<_>> = struct_decl.fields
            .iter()
            .map(|f| self.hir_type_to_llvm(f.ty))
            .collect();
        let field_types = field_types?;
        let struct_type = self.context.struct_type(&field_types, false);
        // TODO(inkwell-0.4.0): StructType::set_name() is not available; name set via symbol table only
        self.hir_struct_types.insert(struct_decl.symbol.0, struct_type);
        // Also store by name if we can resolve it
        self.hir_struct_names.insert(format!("Struct_{}", struct_decl.symbol.0), struct_decl.symbol.0);
        Ok(())
    }

    /// v1.42: Emit a struct constructor call: `StructName(field1, ...)` → struct value.
    /// Supports: Color(r,g,b,a) → packed u32, Vector2(x,y) → 8-byte struct,
    ///           Rectangle(x,y,w,h) → 16-byte struct.
    #[cfg(feature = "v1_30")]
    fn emit_hir_struct_constructor(
        &mut self,
        struct_name: &str,
        args: &[crate::hir::HirExpr],
        func: FunctionValue<'ctx>,
    ) -> Result<IntValue<'ctx>> {
        match struct_name {
            // ─── Color(r,g,b,a) → packed u32 → i64 ───
            "Color" if args.len() == 4 => {
                let mut packed: u32 = 0;
                for (i, arg) in args.iter().enumerate() {
                    match &arg.kind {
                        crate::hir::HirExprKind::Literal(crate::hir::LiteralAst::Integer(v))
                            if *v >= 0 && *v <= 255 =>
                        {
                            packed |= (*v as u32) << (24 - i * 8);
                        }
                        _ => {
                            let _val = self.emit_hir_expr(arg, func)?;
                            return Ok(self.i64_type.const_int(0, false));
                        }
                    }
                }
                Ok(self.i64_type.const_int(packed as u64, false))
            }

            // ─── Vector2(x: f32, y: f32) → 8-byte struct → i64 ───
            "Vector2" if args.len() == 2 => {
                // Use a 2×f32 struct instead of vec_type (inkwell 0.4.0 compat)
                let vec2_type = self.context.struct_type(
                    &[self.context.f32_type().into(), self.context.f32_type().into()], false);
                let alloca = self.create_entry_alloca_typed(
                    func,
                    &format!("vec2_{}_tmp", struct_name),
                    vec2_type.into(),
                );
                for (i, arg) in args.iter().enumerate() {
                    let val = self.emit_hir_expr(arg, func)?;
                    // Cast i64 arg to f32 via bitcast: truncate i64 → i32, then bitcast to f32
                    let i32_val = self.builder
                        .build_int_truncate(val, self.context.i32_type(),
                            &format!("vec2_i32_{}", i))
                        .unwrap_or(val);
                    let f32_val = self.builder.build_bitcast(
                        i32_val, self.context.f32_type(), &format!("vec2_f32_{}", i))
                        .context("Vector2 i32→f32 bitcast")?;
                    let field_ptr = unsafe {
                        self.builder.build_struct_gep(
                            vec2_type, alloca, i as u32,
                            &format!("vec2_field_{}", i))
                            .context("Vector2 field gep")?
                    };
                    self.builder.build_store(field_ptr, f32_val)
                        .context("Vector2 field store")?;
                }
                // For by-value return on x86_64: pack into i64
                let ptr_as_int = self.builder.build_ptr_to_int(
                    alloca, self.i64_type, "vec2_ptr")
                    .context("Vector2 ptr to int")?;
                Ok(ptr_as_int)
            }

            // ─── Rectangle(x, y, width, height: f32) → 16-byte struct ───
            "Rectangle" if args.len() == 4 => {
                let rect_type = self.context.struct_type(
                    &[self.context.f32_type().into(),
                      self.context.f32_type().into(),
                      self.context.f32_type().into(),
                      self.context.f32_type().into()], false);
                let alloca = self.create_entry_alloca_typed(
                    func, "rect_tmp", rect_type.into());
                for (i, arg) in args.iter().enumerate() {
                    let val = self.emit_hir_expr(arg, func)?;
                    // Cast i64 arg to f32: truncate i64 → i32, then bitcast to f32
                    let i32_val = self.builder
                        .build_int_truncate(val, self.context.i32_type(),
                            &format!("rect_i32_{}", i))
                        .unwrap_or(val);
                    let f32_val = self.builder.build_bitcast(
                        i32_val, self.context.f32_type(), &format!("rect_f32_{}", i))
                        .context("Rectangle i32→f32 bitcast")?;
                    let field_ptr = unsafe {
                        self.builder.build_struct_gep(rect_type, alloca, i as u32,
                            &format!("rect_field_{}", i))
                            .context("Rectangle field gep")?
                    };
                    self.builder.build_store(field_ptr, f32_val)
                        .context("Rectangle field store")?;
                }
                // Return pointer as i64 (pass-by-reference pattern)
                let ptr_as_int = self.builder.build_ptr_to_int(
                    alloca, self.i64_type, "rect_ptr")
                    .context("Rectangle ptr to int")?;
                Ok(ptr_as_int)
            }

            // ─── Generic user struct (registry layout) ───
            _ => {
                let layout = self.types.as_ref()
                    .and_then(|t| t.find_struct_by_name(struct_name).map(|(_, l)| l.clone()));
                let layout = match layout {
                    Some(l) => l,
                    None => return Ok(self.i64_type.const_int(0, false)),
                };
                let mut field_llvm: Vec<BasicTypeEnum<'ctx>> = Vec::with_capacity(layout.fields.len());
                for f in &layout.fields {
                    field_llvm.push(self.hir_type_to_llvm(crate::types::TypeRef { id: f.ty })?);
                }
                let struct_type = self.context.struct_type(&field_llvm, false);
                let alloca = self.create_entry_alloca_typed(
                    func, &format!("struct_{}_tmp", struct_name), struct_type.into());
                for (i, arg) in args.iter().enumerate() {
                    let val = self.emit_hir_expr(arg, func)?;
                    let field_ptr = unsafe {
                        self.builder.build_struct_gep(struct_type, alloca, i as u32,
                            &format!("field_{}", i))
                            .context("struct field gep")?
                    };
                    self.builder.build_store(field_ptr, val)
                        .context("struct field store")?;
                }
                let ptr_as_int = self.builder.build_ptr_to_int(
                    alloca, self.i64_type,
                    &format!("struct_{}_ptr", struct_name))
                    .context("struct ptr to int")?;
                Ok(ptr_as_int)
            }
        }
    }
}

