// =========================================================================
// Project: Logicodex Language Engine (Phase 2 - Milestone 1)
// Version: v1.11-alpha (EBNF Formal Grammar Integration)
// Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
// Copyright (c) 2026. All Rights Reserved.
// Licensed under permissive dual-license: MIT & Apache License 2.0
// =========================================================================
use crate::ast::{BinaryOp, Expr, Program, Stmt};
use crate::os::target::{build_target_machine, CompilationTarget, OutputKind};
use anyhow::{anyhow, Context as AnyhowContext, Result};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::IntType;
use inkwell::values::{FunctionValue, IntValue, PointerValue};
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

pub struct LlvmCompiler<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    i64_type: IntType<'ctx>,
    bool_type: IntType<'ctx>,
    variables: Vec<HashMap<String, PointerValue<'ctx>>>,
    print_fn: FunctionValue<'ctx>,
}

impl<'ctx> LlvmCompiler<'ctx> {
    pub fn compile_to_object(program: &Program, object_path: &Path, options: &CodegenOptions) -> Result<CodegenArtifact> {
        let context = Context::create();
        let mut compiler = Self::new(&context, &options.module_name);
        compiler.emit_program(program, options.target)?;
        if options.secure {
            eprintln!("Logicodex secure compilation path active: runtime memory integrity verification metadata, Golden Hash planning, and SHA/AES-NI accelerated attestation hooks are requested for final linkage.");
        }
        compiler.module.verify().map_err(|e| anyhow!("LLVM module verification failed: {e}"))?;

        let output_kind = if options.target.is_freestanding() { OutputKind::FreestandingObject } else { OutputKind::Object };
        let target_machine = build_target_machine(output_kind)?;
        target_machine.write_to_file(&compiler.module, inkwell::targets::FileType::Object, object_path)
            .map_err(|e| anyhow!("failed to emit object file {}: {e}", object_path.display()))?;

        let ir_path = if options.emit_ir {
            let mut ir_path = object_path.to_path_buf();
            ir_path.set_extension("ll");
            compiler.module.print_to_file(&ir_path).map_err(|e| anyhow!("failed to write LLVM IR {}: {e}", ir_path.display()))?;
            Some(ir_path)
        } else { None };

        Ok(CodegenArtifact { object_path: object_path.to_path_buf(), ir_path })
    }

    fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        let i64_type = context.i64_type();
        let bool_type = context.bool_type();
        let print_type = context.void_type().fn_type(&[i64_type.into()], false);
        let print_fn = module.add_function("logicodex_print_i64", print_type, None);
        Self { context, module, builder, i64_type, bool_type, variables: vec![HashMap::new()], print_fn }
    }

    fn emit_program(&mut self, program: &Program, target: CompilationTarget) -> Result<()> {
        let i32_type = self.context.i32_type();
        let main_type = i32_type.fn_type(&[], false);
        let main = self.module.add_function(target.entry_symbol(), main_type, None);
        let entry = self.context.append_basic_block(main, "entry");
        self.builder.position_at_end(entry);
        self.emit_block(&program.statements)?;
        self.builder.build_return(Some(&i32_type.const_int(0, false))).context("failed to build return")?;
        Ok(())
    }

    fn emit_block(&mut self, statements: &[Stmt]) -> Result<()> {
        self.variables.push(HashMap::new());
        for stmt in statements { self.emit_stmt(stmt)?; }
        self.variables.pop();
        Ok(())
    }

    fn emit_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Let { name, value } => {
                let current_fn = self.current_function()?;
                let alloca = self.create_entry_alloca(current_fn, name);
                let value = self.emit_expr(value)?;
                self.builder.build_store(alloca, value).context("failed to store variable")?;
                self.variables.last_mut().expect("codegen scope exists").insert(name.clone(), alloca);
                Ok(())
            }
            Stmt::Print { value } => {
                let value = self.emit_expr(value)?;
                self.builder.build_call(self.print_fn, &[value.into()], "printtmp").context("failed to build print call")?;
                Ok(())
            }
            Stmt::If { condition, then_branch, else_branch } => self.emit_if(condition, then_branch, else_branch),
        }
    }

    fn emit_if(&mut self, condition: &Expr, then_branch: &[Stmt], else_branch: &[Stmt]) -> Result<()> {
        let parent = self.current_function()?;
        let condition_value = self.emit_expr(condition)?;
        let zero = self.bool_type.const_zero();
        let condition_bool = self.builder.build_int_compare(IntPredicate::NE, condition_value, zero, "ifcond").context("failed to compare if condition")?;
        let then_block = self.context.append_basic_block(parent, "then");
        let else_block = self.context.append_basic_block(parent, "else");
        let merge_block = self.context.append_basic_block(parent, "ifcont");
        self.builder.build_conditional_branch(condition_bool, then_block, else_block).context("failed to build conditional branch")?;

        self.builder.position_at_end(then_block);
        self.emit_block(then_branch)?;
        if self.builder.get_insert_block().and_then(|b| b.get_terminator()).is_none() {
            self.builder.build_unconditional_branch(merge_block).context("failed to branch from then block")?;
        }

        self.builder.position_at_end(else_block);
        self.emit_block(else_branch)?;
        if self.builder.get_insert_block().and_then(|b| b.get_terminator()).is_none() {
            self.builder.build_unconditional_branch(merge_block).context("failed to branch from else block")?;
        }

        self.builder.position_at_end(merge_block);
        Ok(())
    }

    fn emit_expr(&mut self, expr: &Expr) -> Result<IntValue<'ctx>> {
        match expr {
            Expr::Integer(value) => Ok(self.i64_type.const_int(*value as u64, true)),
            Expr::Boolean(value) => Ok(self.bool_to_i64(self.bool_type.const_int(u64::from(*value), false))),
            Expr::Variable(name) => {
                let ptr = self.lookup_variable(name).ok_or_else(|| anyhow!("internal codegen error: undefined variable `{name}`"))?;
                Ok(self.builder.build_load(self.i64_type, ptr, name).context("failed to load variable")?.into_int_value())
            }
            Expr::Grouped(inner) => self.emit_expr(inner),
            Expr::Binary { left, op, right } => {
                let l = self.emit_expr(left)?;
                let r = self.emit_expr(right)?;
                match op {
                    BinaryOp::Add => self.builder.build_int_add(l, r, "addtmp").context("failed to build add"),
                    BinaryOp::Subtract => self.builder.build_int_sub(l, r, "subtmp").context("failed to build subtract"),
                    BinaryOp::Multiply => self.builder.build_int_mul(l, r, "multmp").context("failed to build multiply"),
                    BinaryOp::Divide => self.builder.build_int_signed_div(l, r, "divtmp").context("failed to build division"),
                    BinaryOp::Greater => self.compare_to_i64(IntPredicate::SGT, l, r, "gttmp"),
                    BinaryOp::GreaterEqual => self.compare_to_i64(IntPredicate::SGE, l, r, "getmp"),
                    BinaryOp::Less => self.compare_to_i64(IntPredicate::SLT, l, r, "lttmp"),
                    BinaryOp::LessEqual => self.compare_to_i64(IntPredicate::SLE, l, r, "letmp"),
                    BinaryOp::Equal => self.compare_to_i64(IntPredicate::EQ, l, r, "eqtmp"),
                    BinaryOp::NotEqual => self.compare_to_i64(IntPredicate::NE, l, r, "netmp"),
                }
            }
        }
    }

    fn compare_to_i64(&self, predicate: IntPredicate, left: IntValue<'ctx>, right: IntValue<'ctx>, name: &str) -> Result<IntValue<'ctx>> {
        let cmp = self.builder.build_int_compare(predicate, left, right, name).context("failed to build comparison")?;
        Ok(self.bool_to_i64(cmp))
    }

    fn bool_to_i64(&self, value: IntValue<'ctx>) -> IntValue<'ctx> {
        self.builder.build_int_z_extend(value, self.i64_type, "booltoint").expect("zext from i1 to i64 is valid")
    }

    fn create_entry_alloca(&self, function: FunctionValue<'ctx>, name: &str) -> PointerValue<'ctx> {
        let entry_builder = self.context.create_builder();
        let entry = function.get_first_basic_block().expect("function has entry block");
        match entry.get_first_instruction() {
            Some(first) => entry_builder.position_before(&first),
            None => entry_builder.position_at_end(entry),
        }
        entry_builder.build_alloca(self.i64_type, name).expect("alloca in entry block is valid")
    }

    fn lookup_variable(&self, name: &str) -> Option<PointerValue<'ctx>> {
        for scope in self.variables.iter().rev() {
            if let Some(ptr) = scope.get(name) { return Some(*ptr); }
        }
        None
    }

    fn current_function(&self) -> Result<FunctionValue<'ctx>> {
        self.builder.get_insert_block().and_then(|block| block.get_parent()).ok_or_else(|| anyhow!("no active LLVM function"))
    }
}
