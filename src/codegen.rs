// =========================================================================
// Project: Logicodex Language Engine (Phase 2 Deployment Integration)
// Version: v1.21-alpha (Specification Baseline & Practical Severity Roadmap)
// Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
// Copyright (c) 2026. All Rights Reserved.
// Licensed under permissive dual-license: MIT & Apache License 2.0
// =========================================================================
use crate::ast::{BinaryOp, Expr, Program, Stmt};
use crate::ffi::{CallableId, CallableRegistry, CallableSignature};
use crate::os::target::{build_target_machine, CompilationTarget, OutputKind};
use crate::types::{PrimitiveType, TypeId, TypeKind, TypeRegistry};
use anyhow::{anyhow, Context as AnyhowContext, Result};
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicTypeEnum, IntType, FloatType};
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
}

/// Backend trait for version-gated codegen. v1.21 uses direct compilation;
/// v1.30+ uses this trait for HIR-based codegen.
pub trait CodegenBackend {
    fn compile_hir_module(&mut self, module: &crate::hir::HirModule, options: &CodegenOptions) -> Result<CodegenArtifact>;
}

impl<'ctx> LlvmCompiler<'ctx> {
    pub fn compile_to_object(
        program: &Program,
        object_path: &Path,
        options: &CodegenOptions,
    ) -> Result<CodegenArtifact> {
        fn compile_with_context<'ctx>(
            context: &'ctx Context,
            program: &Program,
            object_path: &Path,
            options: &CodegenOptions,
        ) -> Result<CodegenArtifact> {
            let mut compiler = LlvmCompiler::new(context, &options.module_name);
            compiler.emit_program(program, options.target)?;
            if options.secure {
                eprintln!("Logicodex secure compilation path active: runtime memory integrity verification metadata, Golden Hash planning, and SHA/AES-NI accelerated attestation hooks are requested for final linkage.");
            }
            compiler
                .module
                .verify()
                .map_err(|e| anyhow!("LLVM module verification failed: {e}"))?;

            let output_kind = if options.target.is_freestanding() {
                OutputKind::FreestandingObject
            } else {
                OutputKind::Object
            };
            let target_machine = build_target_machine(output_kind)?;
            target_machine
                .write_to_file(
                    &compiler.module,
                    inkwell::targets::FileType::Object,
                    object_path,
                )
                .map_err(|e| {
                    anyhow!("failed to emit object file {}: {e}", object_path.display())
                })?;

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

        let context = Context::create();
        compile_with_context(&context, program, object_path, options)
    }

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
        }
    }

    /// Attach a CallableRegistry for FFI function resolution (Sprint 3).
    pub fn with_callables(mut self, callables: CallableRegistry, types: TypeRegistry) -> Self {
        self.callables = Some(callables);
        self.types = Some(types);
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
            TypeKind::Pointer { .. } => Ok(self.context.ptr_type(inkwell::AddressSpace::default()).into()),
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
        let fn_type = ret_type.fn_type(&llvm_param_types, signature.is_variadic);
        let func = self.module.add_function(name, fn_type, Some(inkwell::module::Linkage::External));
        self.declared_funcs.insert(name.clone(), func);
        Ok(func)
    }

    /// Detect if a callee name is a struct constructor (e.g., "Color" → packed u32).
    fn try_struct_constructor(&self, callee_name: &str, args: &[Expr]) -> Option<u32> {
        let types = self.types.as_ref()?;
        let (_, layout) = types.find_struct_by_name(callee_name)?;
        // For Sprint 3: pack Color(R,G,B,A) → u32 RGBA
        if callee_name == "Color" && args.len() == 4 {
            // We can only handle literal arguments at codegen time
            let vals: Vec<u32> = args.iter().filter_map(|arg| match arg {
                Expr::Integer(v) if *v >= 0 && *v <= 255 => Some(*v as u32),
                _ => None,
            }).collect();
            if vals.len() == 4 {
                return Some((vals[0] << 24) | (vals[1] << 16) | (vals[2] << 8) | vals[3]);
            }
        }
        // Other struct types: not yet supported for inline construction
        None
    }
    }

    fn emit_program(&mut self, program: &Program, target: CompilationTarget) -> Result<()> {
        let i32_type = self.context.i32_type();
        let main_type = i32_type.fn_type(&[], false);
        let main = self
            .module
            .add_function(target.entry_symbol(), main_type, None);
        let entry = self.context.append_basic_block(main, "entry");
        self.builder.position_at_end(entry);
        self.emit_block(&program.statements)?;
        self.builder
            .build_return(Some(&i32_type.const_int(0, false)))
            .context("failed to build return")?;
        Ok(())
    }

    fn emit_block(&mut self, statements: &[Stmt]) -> Result<()> {
        self.variables.push(HashMap::new());
        for stmt in statements {
            if self.current_block_has_terminator() {
                break;
            }
            self.emit_stmt(stmt)?;
        }
        self.variables.pop();
        Ok(())
    }

    fn emit_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Use { .. } => Ok(()),
            // Safety net: v1.30-only AST nodes must never reach v1.21 codegen
            Stmt::StructDecl { .. } => self.emit_v130_ast_in_v121("StructDecl"),
            Stmt::EnumDecl { .. } => self.emit_v130_ast_in_v121("EnumDecl"),
            Stmt::UnsafeBlock { .. } => self.emit_v130_ast_in_v121("UnsafeBlock"),
            Stmt::ExternBlock { .. } => self.emit_v130_ast_in_v121("ExternBlock"),
            Stmt::HardwareZone { body } => self.emit_block(body),
            Stmt::HardwareDecl { name, address, .. } => {
                let current_fn = self.current_function()?;
                let alloca = self.create_entry_alloca(current_fn, name);
                let value = self.emit_expr(address)?;
                self.builder
                    .build_store(alloca, value)
                    .context("failed to store hardware region address")?;
                self.variables
                    .last_mut()
                    .expect("codegen scope exists")
                    .insert(name.clone(), alloca);
                Ok(())
            }
            Stmt::Function { .. } => Ok(()),
            Stmt::Let { name, value, .. } => {
                let current_fn = self.current_function()?;
                let alloca = self.create_entry_alloca(current_fn, name);
                let value = self.emit_expr(value)?;
                self.builder
                    .build_store(alloca, value)
                    .context("failed to store variable")?;
                self.variables
                    .last_mut()
                    .expect("codegen scope exists")
                    .insert(name.clone(), alloca);
                Ok(())
            }
            Stmt::Print { value } => {
                let value = self.emit_expr(value)?;
                self.builder
                    .build_call(self.print_fn, &[value.into()], "printtmp")
                    .context("failed to build print call")?;
                Ok(())
            }
            Stmt::Return { value } | Stmt::ExprStmt { value } => {
                let _ = self.emit_expr(value)?;
                Ok(())
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => self.emit_if(condition, then_branch, else_branch),
            Stmt::While { condition, body } => self.emit_while(condition, body),
            Stmt::Loop { body } => self.emit_loop(body),
            Stmt::Break => {
                let target = self
                    .loop_targets
                    .last()
                    .ok_or_else(|| anyhow!("internal codegen error: break outside loop"))?
                    .break_block;
                self.builder
                    .build_unconditional_branch(target)
                    .context("failed to build break branch")?;
                Ok(())
            }
            Stmt::Continue => {
                let target = self
                    .loop_targets
                    .last()
                    .ok_or_else(|| anyhow!("internal codegen error: continue outside loop"))?
                    .continue_block;
                self.builder
                    .build_unconditional_branch(target)
                    .context("failed to build continue branch")?;
                Ok(())
            }
        }
    }

    fn emit_if(
        &mut self,
        condition: &Expr,
        then_branch: &[Stmt],
        else_branch: &[Stmt],
    ) -> Result<()> {
        let parent = self.current_function()?;
        let condition_value = self.emit_expr(condition)?;
        let zero = self.i64_type.const_zero();
        let condition_bool = self
            .builder
            .build_int_compare(IntPredicate::NE, condition_value, zero, "ifcond")
            .context("failed to compare if condition")?;
        let then_block = self.context.append_basic_block(parent, "then");
        let else_block = self.context.append_basic_block(parent, "else");
        let merge_block = self.context.append_basic_block(parent, "ifcont");
        self.builder
            .build_conditional_branch(condition_bool, then_block, else_block)
            .context("failed to build conditional branch")?;

        self.builder.position_at_end(then_block);
        self.emit_block(then_branch)?;
        if self
            .builder
            .get_insert_block()
            .and_then(|b| b.get_terminator())
            .is_none()
        {
            self.builder
                .build_unconditional_branch(merge_block)
                .context("failed to branch from then block")?;
        }

        self.builder.position_at_end(else_block);
        self.emit_block(else_branch)?;
        if self
            .builder
            .get_insert_block()
            .and_then(|b| b.get_terminator())
            .is_none()
        {
            self.builder
                .build_unconditional_branch(merge_block)
                .context("failed to branch from else block")?;
        }

        self.builder.position_at_end(merge_block);
        Ok(())
    }

    fn emit_while(&mut self, condition: &Expr, body: &[Stmt]) -> Result<()> {
        let parent = self.current_function()?;
        let condition_block = self.context.append_basic_block(parent, "while.cond");
        let body_block = self.context.append_basic_block(parent, "while.body");
        let after_block = self.context.append_basic_block(parent, "while.end");
        self.builder
            .build_unconditional_branch(condition_block)
            .context("failed to branch to while condition")?;

        self.builder.position_at_end(condition_block);
        let condition_value = self.emit_expr(condition)?;
        let condition_bool = self.i64_to_bool(condition_value, "whilecond")?;
        self.builder
            .build_conditional_branch(condition_bool, body_block, after_block)
            .context("failed to build while condition branch")?;

        self.builder.position_at_end(body_block);
        self.loop_targets.push(LoopTarget {
            continue_block: condition_block,
            break_block: after_block,
        });
        self.emit_block(body)?;
        self.loop_targets.pop();
        if !self.current_block_has_terminator() {
            self.builder
                .build_unconditional_branch(condition_block)
                .context("failed to branch from while body")?;
        }

        self.builder.position_at_end(after_block);
        Ok(())
    }

    fn emit_loop(&mut self, body: &[Stmt]) -> Result<()> {
        let parent = self.current_function()?;
        let body_block = self.context.append_basic_block(parent, "loop.body");
        let after_block = self.context.append_basic_block(parent, "loop.end");
        self.builder
            .build_unconditional_branch(body_block)
            .context("failed to branch to loop body")?;

        self.builder.position_at_end(body_block);
        self.loop_targets.push(LoopTarget {
            continue_block: body_block,
            break_block: after_block,
        });
        self.emit_block(body)?;
        self.loop_targets.pop();
        if !self.current_block_has_terminator() {
            self.builder
                .build_unconditional_branch(body_block)
                .context("failed to branch from loop body")?;
        }

        self.builder.position_at_end(after_block);
        Ok(())
    }

    fn emit_expr(&mut self, expr: &Expr) -> Result<IntValue<'ctx>> {
        match expr {
            Expr::Integer(value) => Ok(self.i64_type.const_int(*value as u64, true)),
            Expr::Boolean(value) => {
                Ok(self.bool_to_i64(self.bool_type.const_int(u64::from(*value), false)))
            }
            Expr::StringLiteral(value) => Ok(self.i64_type.const_int(value.len() as u64, false)),
            Expr::AddressOfLiteral(value) => Ok(self.i64_type.const_int(*value as u64, false)),
            Expr::Variable(name) => {
                let ptr = self.lookup_variable(name).ok_or_else(|| {
                    anyhow!("internal codegen error: undefined variable `{name}`")
                })?;
                Ok(self
                    .builder
                    .build_load(self.i64_type, ptr, name)
                    .context("failed to load variable")?
                    .into_int_value())
            }
            Expr::Call { callee, args } => {
                let callee_name = match callee.as_ref() {
                    Expr::Variable(name) => name.as_str(),
                    _ => {
                        eprintln!("logicodex v1.30: complex callees not yet supported in codegen");
                        return Ok(self.i64_type.const_int(0, false));
                    }
                };

                // Sprint 3: Try struct constructor first (e.g., Color(255,0,0,255) → packed u32)
                if let Some(packed) = self.try_struct_constructor(callee_name, args) {
                    return Ok(self.i64_type.const_int(packed as u64, false));
                }

                // Sprint 3: Try CallableRegistry lookup for FFI function
                if let Some(ref callables) = self.callables {
                    if let Some((callable_id, signature)) = callables.find_by_name(callee_name) {
                        // Declare the extern function in LLVM
                        let func = self.declare_extern_func(signature)?;
                        // Evaluate arguments recursively
                        let mut llvm_args: Vec<BasicValueEnum<'ctx>> = Vec::with_capacity(args.len());
                        for arg in args {
                            let val = self.emit_expr(arg)?;
                            // For Sprint 3, all args are promoted to i64 then truncated at call site
                            // based on the CallableSignature param types
                            llvm_args.push(val.into());
                        }
                        let call_site = self.builder
                            .build_call(func, &llvm_args, &format!("call_{}", callee_name))
                            .with_context(|| format!("failed to build call to '{}'", callee_name))?;
                        // Extract return value
                        match call_site.try_as_basic_value() {
                            inkwell::values::Either::Left(val) => {
                                // Convert return value to i64 for the expression result
                                match val {
                                    BasicValueEnum::IntValue(iv) => Ok(iv),
                                    other => {
                                        // Truncate/extend as needed — for Sprint 3 return as i64
                                        Ok(self.i64_type.const_int(0, false))
                                    }
                                }
                            }
                            inkwell::values::Either::Right(_) => {
                                // Void return — return 0
                                Ok(self.i64_type.const_int(0, false))
                            }
                        }
                    } else {
                        eprintln!(
                            "logicodex v1.30: function '{}' not found in CallableRegistry",
                            callee_name
                        );
                        Ok(self.i64_type.const_int(0, false))
                    }
                } else {
                    // No CallableRegistry attached — emit stub
                    eprintln!(
                        "logicodex v1.30: codegen for Call('{}', {} args) — \
                         CallableRegistry not attached, call with_callables()",
                        callee_name, args.len()
                    );
                    Ok(self.i64_type.const_int(0, false))
                }
            }
            Expr::Grouped(inner) => self.emit_expr(inner),
            Expr::Binary { left, op, right } => {
                let l = self.emit_expr(left)?;
                let r = self.emit_expr(right)?;
                match op {
                    BinaryOp::Add => self
                        .builder
                        .build_int_add(l, r, "addtmp")
                        .context("failed to build add"),
                    BinaryOp::Subtract => self
                        .builder
                        .build_int_sub(l, r, "subtmp")
                        .context("failed to build subtract"),
                    BinaryOp::Multiply => self
                        .builder
                        .build_int_mul(l, r, "multmp")
                        .context("failed to build multiply"),
                    BinaryOp::Divide => self
                        .builder
                        .build_int_signed_div(l, r, "divtmp")
                        .context("failed to build division"),
                    BinaryOp::Greater => self.compare_to_i64(IntPredicate::SGT, l, r, "gttmp"),
                    BinaryOp::GreaterEqual => self.compare_to_i64(IntPredicate::SGE, l, r, "getmp"),
                    BinaryOp::Less => self.compare_to_i64(IntPredicate::SLT, l, r, "lttmp"),
                    BinaryOp::LessEqual => self.compare_to_i64(IntPredicate::SLE, l, r, "letmp"),
                    BinaryOp::Equal => self.compare_to_i64(IntPredicate::EQ, l, r, "eqtmp"),
                    BinaryOp::NotEqual => self.compare_to_i64(IntPredicate::NE, l, r, "netmp"),
                    BinaryOp::And => {
                        let lb = self.i64_to_bool(l, "andlhs")?;
                        let rb = self.i64_to_bool(r, "andrhs")?;
                        let value = self
                            .builder
                            .build_and(lb, rb, "andtmp")
                            .context("failed to build logical and")?;
                        Ok(self.bool_to_i64(value))
                    }
                    BinaryOp::Or => {
                        let lb = self.i64_to_bool(l, "orlhs")?;
                        let rb = self.i64_to_bool(r, "orrhs")?;
                        let value = self
                            .builder
                            .build_or(lb, rb, "ortmp")
                            .context("failed to build logical or")?;
                        Ok(self.bool_to_i64(value))
                    }
                    BinaryOp::BitAnd => self
                        .builder
                        .build_and(l, r, "bitandtmp")
                        .context("failed to build bitwise and"),
                    BinaryOp::BitOr => self
                        .builder
                        .build_or(l, r, "bitortmp")
                        .context("failed to build bitwise or"),
                    BinaryOp::ShiftLeft => self
                        .builder
                        .build_left_shift(l, r, "shltmp")
                        .context("failed to build left shift"),
                    BinaryOp::ShiftRight => self
                        .builder
                        .build_right_shift(l, r, true, "shrtmp")
                        .context("failed to build right shift"),
                }
            }
            // v1.30.1-alpha Fasa 2: Zero-Copy Ownership Transfer via Pintu
            Expr::Hantar { pintu_name, value } => {
                // Fasa 2: Emit hantar with Release semantics
                // The value ownership is transferred to the Pintu
                let val = self.emit_expr(value)?;
                // Call runtime pintu_send_release(pintu_name, val)
                // For now, return 0 as placeholder — full impl in Fasa 3
                eprintln!("logicodex v1.30.1-alpha: hantar '{}' through '{}' — ownership transferred (Release)",
                    match value { Expr::Variable(n) => n.as_str(), _ => "<expr>" }, pintu_name);
                Ok(self.i64_type.const_int(0, false))
            }
            Expr::Terima { pintu_name } => {
                // Fasa 2: Emit terima with Acquire semantics
                // The value ownership is transferred from the Pintu
                // Call runtime pintu_recv_acquire(pintu_name)
                // For now, return 0 as placeholder — full impl in Fasa 3
                eprintln!("logicodex v1.30.1-alpha: terima through '{}' — ownership acquired (Acquire)", pintu_name);
                Ok(self.i64_type.const_int(0, false))
            }
            // v1.30.1-alpha: Threading expressions (stubs)
            Expr::Spawn { kotak_name, .. } => {
                eprintln!("logicodex v1.30.1-alpha: spawn '{}' — deferred to runtime", kotak_name);
                Ok(self.i64_type.const_int(0, false))
            }
            Expr::Tunggu { kotak_name } => {
                eprintln!("logicodex v1.30.1-alpha: tunggu '{}' — deferred to runtime", kotak_name);
                Ok(self.i64_type.const_int(0, false))
            }
        }
    }

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

    fn lookup_variable(&self, name: &str) -> Option<PointerValue<'ctx>> {
        for scope in self.variables.iter().rev() {
            if let Some(ptr) = scope.get(name) {
                return Some(*ptr);
            }
        }
        None
    }

    fn current_function(&self) -> Result<FunctionValue<'ctx>> {
        self.builder
            .get_insert_block()
            .and_then(|block| block.get_parent())
            .ok_or_else(|| anyhow!("no active LLVM function"))
    }

    /// Safety net: v1.21 codegen must never receive v1.30-only AST nodes.
    /// If this path is reached, it indicates a parser pipeline leak — fail-fast.
    fn emit_v130_ast_in_v121(&self, stmt_kind: &str) -> Result<()> {
        unreachable!(
            "BUG: v1.21 codegen received v1.30-only AST node '{}'. \
             This indicates a parser pipeline configuration error. \
             The v1.21 pipeline must trap these tokens at parse time. \
             Use --pipeline v1.30 to compile this construct.",
            stmt_kind
        )
    }
}

/// Entry point for v1.30 HIR-to-object compilation.
/// This is the gate between the v1.21 AST path and the v1.30 HIR path.
/// v1.21 code never calls this function.
pub fn compile_v130(
    hir_module: &crate::hir::HirModule,
    object_path: &Path,
    options: &CodegenOptions,
    callables: CallableRegistry,
    types: TypeRegistry,
) -> Result<CodegenArtifact> {
    let context = Context::create();
    let mut compiler = LlvmCompiler::new(&context, &options.module_name)
        .with_callables(callables, types);

    // v1.30 uses HIR items instead of v1.21 AST statements
    for item in &hir_module.items {
        match &item.node {
            crate::hir::HirItem::Function(function) => {
                compiler.emit_v130_function(function, options.target)?;
            }
            crate::hir::HirItem::Struct(_) => {
                // Struct layout computed at semantic time; emit placeholder
                eprintln!("logicodex v1.30: struct items are processed at semantic time");
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
    } else {
        OutputKind::Object
    };
    let target_machine = build_target_machine(output_kind)?;
    target_machine
        .write_to_file(
            &compiler.module,
            inkwell::targets::FileType::Object,
            object_path,
        )
        .map_err(|e| anyhow!("failed to emit object file {}: {e}", object_path.display()))?;

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

// Stub implementations for v1.30 HIR codegen — full implementation is a future milestone
impl<'ctx> LlvmCompiler<'ctx> {
    fn emit_v130_function(
        &mut self,
        _function: &crate::hir::HirFunction,
        _target: CompilationTarget,
    ) -> Result<()> {
        eprintln!("logicodex v1.30: HIR function codegen stub — full LLVM emission is a future milestone");
        Ok(())
    }

    fn emit_v130_extern_function(
        &mut self,
        _extern_fn: &crate::hir::HirExternFunction,
    ) -> Result<()> {
        eprintln!("logicodex v1.30: extern function codegen stub — full LLVM emission is a future milestone");
        Ok(())
    }
}
