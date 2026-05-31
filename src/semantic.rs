// =========================================================================
// Project: Logicodex Language Engine (Phase 2 Deployment Integration)
// Version: v1.21-alpha (Specification Baseline & Practical Severity Roadmap)
// Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
// Copyright (c) 2026. All Rights Reserved.
// Licensed under permissive dual-license: MIT & Apache License 2.0
// =========================================================================
pub mod coercion;
pub mod registry;
pub mod type_checker;

use crate::ast::{BinaryOp, Expr, Program, Stmt, Type};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeverityPolicy {
    Desktop,
    Embedded,
    Kernel,
}

impl SeverityPolicy {
    pub fn from_target(target: &str, secure: bool) -> Self {
        let lower = target.to_ascii_lowercase();
        if lower.contains("kernel") || secure {
            Self::Kernel
        } else if lower.contains("embedded")
            || lower.contains("mcu")
            || lower.contains("bare")
            || lower.contains("freestanding")
        {
            Self::Embedded
        } else {
            Self::Desktop
        }
    }
}

#[derive(Debug, Error)]
pub enum SemanticError {
    #[error("pembolehubah `{0}` sudah ditakrif dalam skop ini / variable `{0}` is already defined in this scope")]
    DuplicateVariable(String),
    #[error("pembolehubah `{0}` tidak ditakrif / variable `{0}` is not defined")]
    UndefinedVariable(String),
    #[error("fungsi `{0}` sudah ditakrif / function `{0}` is already defined")]
    DuplicateFunction(String),
    #[error("rantau perkakasan `{0}` sudah ditakrif / hardware region `{0}` is already defined")]
    DuplicateHardwareRegion(String),
    #[error("operator `{op}` memerlukan operand {expected} tetapi menerima {left} dan {right} / operator `{op}` requires {expected} operands but received {left} and {right}")]
    TypeMismatch {
        op: BinaryOp,
        expected: &'static str,
        left: Type,
        right: Type,
    },
    #[error("ikatan `{name}` diisytihar sebagai {declared} tetapi ekspresi mempunyai jenis {actual} / binding `{name}` was declared as {declared} but expression has type {actual}")]
    DeclaredTypeMismatch {
        name: String,
        declared: Type,
        actual: Type,
    },
    #[error(
        "syarat if mesti Bool tetapi menerima {0} / if condition must be Bool but received {0}"
    )]
    NonBooleanCondition(Type),
    #[error("pembahagian dengan sifar tetap ditolak oleh analisis statik / division by a constant zero is rejected by static analysis")]
    DivisionByZero,
    #[error("literal numerik {value} tidak muat dalam jenis diisytihar {ty} / numeric literal {value} does not fit in declared type {ty}")]
    NumericBounds { value: i64, ty: Type },
    #[error("alamat literal {0} tiada rantau provenance perkakasan yang diisytihar / literal address {0} has no declared hardware provenance region")]
    MissingProvenance(i64),
    #[error("nilai pointer untuk `{name}` mesti berasal daripada literal addr eksplisit atau rantau perkakasan / pointer value for `{name}` must originate from an explicit addr literal or hardware region")]
    InvalidPointerInitializer { name: String },
    #[error("literal alamat kosong ditolak di bawah polisi sasaran {policy:?}; isytihar rantau perkakasan dahulu / bare address literal is rejected under {policy:?} target policy; declare a hardware region first")]
    BareAddressRejected { policy: SeverityPolicy },
    #[error("KRITIKAL: Ralat Umum Tahap 1 - Percubaan Mutasi Perkakasan Tanpa Kebenaran Skop Zon Selamat / CRITICAL: General Error Level 1 - Attempted Hardware Mutation Without Safe Zone Scope Authorization.")]
    HardwareMutationOutsideZone,
    #[error("pernyataan return berada di luar fungsi / return statement is outside a function")]
    ReturnOutsideFunction,
    #[error("fungsi `{name}` memulangkan {expected} tetapi ekspresi pulangan mempunyai jenis {actual} / function `{name}` returns {expected} but returned expression has type {actual}")]
    ReturnTypeMismatch {
        name: String,
        expected: Type,
        actual: Type,
    },
    #[error("pernyataan break berada di luar loop / break statement is outside a loop")]
    BreakOutsideLoop,
    #[error("pernyataan continue berada di luar loop / continue statement is outside a loop")]
    ContinueOutsideLoop,
    // ─── Ketuk 1: Core Memory Model ───
    #[error("KRITIKAL: Akses indeks {index} melebihi kapasiti penimbal `{name}` ({capacity}) / CRITICAL: Index {index} exceeds buffer `{name}` capacity ({capacity})")]
    BufferOverflow {
        name: String,
        index: i64,
        capacity: i64,
    },
    #[error("KRITIKAL: Penggunaan penimbal `{name}` selepas pemindahan (move) / CRITICAL: Use of buffer `{name}` after ownership move")]
    UseAfterMove { name: String },
    #[error("penugasan kepada elemen {elem} memerlukan jenis {expected} tetapi menerima {actual} / assignment to element {elem} requires type {expected} but received {actual}")]
    ElementTypeMismatch {
        elem: String,
        expected: Type,
        actual: Type,
    },
    #[error("KRITIKAL: Pembolehubah `{name}` bukan Buffer yang berdaftar — perlu diisytiharkan dengan `let {name}: Buffer<T>` / CRITICAL: Variable `{name}` is not a registered Buffer — must be declared with `let {name}: Buffer<T>`")]
    NotABuffer { name: String },
    // ─── Ketuk 2: Result Abstraction ───
    #[error("KRITIKAL: `match` hanya boleh digunakan pada jenis `Result<T, E>`, tetapi menerima `{ty}` / CRITICAL: `match` can only be used on `Result<T, E>`, but received `{ty}`")]
    MatchOnNonResult { ty: Type },
    #[error("KRITIKAL: `match` tidak lengkap — perlu menangani `{missing}` / CRITICAL: `match` is non-exhaustive — must handle `{missing}`")]
    NonExhaustiveMatch { missing: String },
    // ─── Ketuk 3: File Handle ABI ───
    #[error("KRITIKAL: Handle fail `{name}` belum dibuka / CRITICAL: File handle `{name}` has not been opened")]
    HandleNotOpen { name: String },
    #[error("KRITIKAL: Operasi `{operation}` ditolak untuk handle `{name}` — kebenaran tidak mencukupi / CRITICAL: Operation `{operation}` denied for handle `{name}` — insufficient permission")]
    HandlePermissionDenied { name: String, operation: String },
    // ─── v1.30.1-alpha: Threading Foundation ───
    #[error("KRITIKAL: Actor `{name}` does not exist — mesti diisytiharkan sebelum digunakan / CRITICAL: Actor `{name}` does not exist — must be declared before use")]
    ActorNotFound { name: String },
    #[error("KRITIKAL: Channel from `{from}` to `{to}` is invalid — Kotak penerima tidak wujud / CRITICAL: Channel from `{from}` to `{to}` is invalid — receiving Kotak does not exist")]
    InvalidChannelTopology { from: String, to: String },
    #[error("KRITIKAL: Actor `{name}` is already declared / CRITICAL: Actor `{name}` is already declared")]
    DuplicateActor { name: String },
    #[error("KRITIKAL: `spawn` can only be used with an Actor name / CRITICAL: `spawn` can only be used with an Actor name")]
    SpawnNonActor,
    // v1.30.1-alpha Phase 2: Zero-Copy Ownership Transfer
    #[error("KRITIKAL: Variable `{name}` has already been sent through Channel — ownership telah dipindahkan / CRITICAL: Variable `{name}` has already been sent through Channel — ownership has been transferred")]
    UseAfterSend { name: String },
    // v1.30.1-alpha Phase 3: Backpressure + Scheduler
    #[error("KRITIKAL: Channel `{name}` penuh — hantar ditolak (backpressure) / CRITICAL: Channel `{name}` is full — send rejected (backpressure)")]
    ChannelFull { name: String },
    #[error("KRITIKAL: Timeout menunggu recv dari Channel `{name}` selepas {timeout_ms}ms / CRITICAL: Timeout waiting for recv from Channel `{name}` after {timeout_ms}ms")]
    RecvTimeout { name: String, timeout_ms: i64 },
    // v1.32.0-alpha: Static Capability Fabric — Zero Runtime Mediation
    #[error("KRITIKAL: Pelanggaran Kontrak Keupayaan — '{symbol}' memerlukan Gate '{gate}' tetapi tiada modul yang menyediakannya / CRITICAL: Capability Contract Violation — '{symbol}' requires Gate '{gate}' but no module provides it")]
    CapabilityContractViolation { symbol: String, gate: String },
    #[error("KRITIKAL: Peningkatan Keistimewaan Dikesan — '{symbol}' kini memerlukan Gate '{gate}' (akses sensitif) / CRITICAL: Privilege Escalation Detected — '{symbol}' now requires Gate '{gate}' (sensitive access)")]
    PrivilegeEscalation { symbol: String, gate: String },
    // ─── v1.42 P6: StrictAudioContext — Hardware-Safe Audio Guards ───
    #[error("KRITIKAL: AudioViolationIo — '{function}' dilarang dalam audio callback / CRITICAL: '{function}' is forbidden inside an audio callback")]
    AudioViolationIo { function: String },
    #[error("KRITIKAL: AudioViolationRecursion — '{function}' memanggil dir sendiri dalam audio callback / CRITICAL: '{function}' calls itself inside an audio callback")]
    AudioViolationRecursion { function: String },
    #[error("KRITIKAL: AudioViolationUnboundedLoop — loop tanpa batas dilarang dalam audio callback / CRITICAL: unbounded loop is forbidden inside an audio callback")]
    AudioViolationUnboundedLoop,
    #[error("KRITIKAL: AudioViolationForbiddenCall — '{function}' memanggil fungsi tidak selamat '{callee}' dalam audio callback / CRITICAL: '{function}' calls unsafe function '{callee}' in audio callback")]
    AudioViolationForbiddenCall { function: String, callee: String },
}

#[derive(Debug, Default)]
pub struct Analyzer {
    scopes: Vec<HashMap<String, Type>>,
    hardware_regions: HashMap<String, (Type, i64)>,
    hardware_addresses: HashSet<i64>,
    functions: HashMap<String, (Vec<Type>, Option<Type>)>,
    current_function: Option<(String, Option<Type>)>,
    loop_depth: u32,
    hw_zone_depth: u32,
    policy: SeverityPolicy,
    // Ketuk 1: Core Memory Model — Buffer ownership + provenance tracking
    /// Tracks which variables have been moved (ownership transferred).
    moved_vars: HashSet<String>,
    /// Buffer provenance: variable name → (element_type, declared_capacity)
    buffer_registry: HashMap<String, (Type, i64)>,
    // Ketuk 3: File Handle ABI — Handle lifecycle tracking
    /// Tracks open file handles: variable name → (handle_type, is_open)
    handle_registry: HashMap<String, (Type, bool)>,
    /// Tracks handle permissions: variable name → mode (Read/Write/ReadWrite)
    handle_permissions: HashMap<String, String>,
    // v1.30.1-alpha: Threading Foundation — Actor & Channel topology
    /// Registered Actor names
    actor_registry: HashSet<String>,
    /// Registered Channel: (from, to, message_type)
    channel_registry: Vec<(String, String, String)>,
    // v1.30.1-alpha Phase 2: Zero-Copy Ownership Transfer
    /// Variables moved via Channel hantar() — cannot use after send
    moved_via_channel: HashSet<String>,
    // v1.42 P6: StrictAudioContext — Audio callback safety tracking
    /// Set of function names that are audio callbacks (ISR-like)
    audio_callbacks: HashSet<String>,
    /// Currently analyzing an audio callback?
    in_audio_callback: bool,
}

impl Default for SeverityPolicy {
    fn default() -> Self {
        Self::Desktop
    }
}

impl Analyzer {
    #[allow(dead_code)]
    pub fn analyze(program: &Program) -> Result<(), SemanticError> {
        Self::analyze_with_policy(program, SeverityPolicy::Desktop)
    }

    pub fn analyze_for_target(
        program: &Program,
        target: &str,
        secure: bool,
    ) -> Result<(), SemanticError> {
        Self::analyze_with_policy(program, SeverityPolicy::from_target(target, secure))
    }

    pub fn analyze_with_policy(
        program: &Program,
        policy: SeverityPolicy,
    ) -> Result<(), SemanticError> {
        let mut analyzer = Self {
            scopes: vec![HashMap::new()],
            policy,
            moved_vars: HashSet::new(),
            buffer_registry: HashMap::new(),
            handle_registry: HashMap::new(),
            handle_permissions: HashMap::new(),
            actor_registry: HashSet::new(),
            channel_registry: Vec::new(),
            moved_via_channel: HashSet::new(),
            ..Self::default()
        };
        analyzer.block(&program.statements)
    }

    fn block(&mut self, statements: &[Stmt]) -> Result<(), SemanticError> {
        for stmt in statements {
            self.statement(stmt)?;
        }
        Ok(())
    }

    fn scoped_block(&mut self, statements: &[Stmt]) -> Result<(), SemanticError> {
        self.scopes.push(HashMap::new());
        let result = self.block(statements);
        // BUGFIX #3: Clear moved_vars and buffer_registry for variables going out of scope
        if let Some(scope) = self.scopes.last() {
            for name in scope.keys() {
                self.moved_vars.remove(name);
                self.buffer_registry.remove(name);
            }
        }
        self.scopes.pop();
        result
    }

    fn statement(&mut self, stmt: &Stmt) -> Result<(), SemanticError> {
        match stmt {
            Stmt::Use { .. } => Ok(()),
            Stmt::HardwareZone { body } => {
                self.hw_zone_depth += 1;
                let result = self.scoped_block(body);
                self.hw_zone_depth -= 1;
                result
            }
            Stmt::HardwareDecl { name, ty, address } => {
                if self.hardware_regions.contains_key(name) {
                    return Err(SemanticError::DuplicateHardwareRegion(name.clone()));
                }
                let Expr::AddressOfLiteral(addr) = address else {
                    return Err(SemanticError::InvalidPointerInitializer { name: name.clone() });
                };
                self.hardware_regions
                    .insert(name.clone(), (ty.clone(), *addr));
                self.hardware_addresses.insert(*addr);
                self.define(name, Type::Pointer(Box::new(ty.clone())))
            }
            Stmt::Function {
                name,
                params,
                return_type,
                body,
            } => {
                if self.functions.contains_key(name) {
                    return Err(SemanticError::DuplicateFunction(name.clone()));
                }
                self.functions.insert(
                    name.clone(),
                    (
                        params.iter().map(|p| p.ty.clone()).collect(),
                        return_type.clone(),
                    ),
                );
                self.scopes.push(HashMap::new());
                for param in params {
                    self.define(&param.name, param.ty.clone())?;
                }
                let previous = self
                    .current_function
                    .replace((name.clone(), return_type.clone()));
                let result = self.block(body);
                self.current_function = previous;
                self.scopes.pop();
                result
            }
            Stmt::Let {
                name,
                declared_type,
                value,
            } => {
                let inferred = self.expression(value)?;
                let ty = declared_type.clone().unwrap_or(inferred.clone());
                if let Some(declared) = declared_type {
                    self.check_assignment(name, declared, &inferred, value)?;
                }
                // BUGFIX #1: Register Buffer types in buffer_registry for provenance tracking
                if let Type::Buffer { element } = &ty {
                    // For now, capacity is not declared in type — runtime will enforce
                    // Future: Buffer<f32, 1024> syntax for compile-time capacity
                    self.register_buffer(name, *element.clone(), 0);
                }
                // Ketuk 3: Register FileHandle types in handle_registry
                if let Type::Opaque { name: type_name } = &ty {
                    if type_name == "FileHandle" {
                        // Detect mode from value — default to ReadWrite if not clear
                        let mode = match value {
                            Expr::Call { callee, args } => {
                                if let Expr::Variable(fn_name) = callee.as_ref() {
                                    if fn_name == "open" && args.len() >= 2 {
                                        if let Expr::Variable(mode) = &args[1] {
                                            mode.clone()
                                        } else { "ReadWrite".to_string() }
                                    } else { "ReadWrite".to_string() }
                                } else { "ReadWrite".to_string() }
                            }
                            _ => "ReadWrite".to_string(),
                        };
                        self.register_handle(name, &mode);
                    }
                }
                // BUGFIX #4: Detect ownership move — let buf2 = buf
                if let Expr::Variable(src_name) = value {
                    if self.is_moved(src_name) {
                        return Err(SemanticError::UseAfterMove {
                            name: src_name.clone(),
                        });
                    }
                    if self.buffer_registry.contains_key(src_name) {
                        // Moving a buffer: mark source as moved
                        self.mark_moved(src_name);
                    }
                }
                self.define(name, ty)
            }
            Stmt::Assign { target, value } => {
                let val_ty = self.expression(value)?;
                match target {
                    Expr::Index { base, index } => {
                        // BUGFIX #2: buf[index] = value assignment
                        let base_ty = self.expression(base)?;
                        match &base_ty {
                            Type::Slice { element } | Type::Buffer { element } => {
                                // Validate buffer provenance
                                if let Expr::Variable(buf_name) = base.as_ref() {
                                    self.validate_buffer_index(buf_name, index)?;
                                }
                                // Check element type compatibility
                                if !types_compatible(element, &val_ty) {
                                    return Err(SemanticError::ElementTypeMismatch {
                                        elem: format!("{}[{:?}]", base, index),
                                        expected: *element.clone(),
                                        actual: val_ty,
                                    });
                                }
                                Ok(())
                            }
                            other => Err(SemanticError::TypeMismatch {
                                op: BinaryOp::Add,
                                expected: "slice or buffer",
                                left: other.clone(),
                                right: other.clone(),
                            }),
                        }
                    }
                    Expr::Variable(name) => {
                        // Simple variable assignment
                        if self.is_moved(name) {
                            return Err(SemanticError::UseAfterMove {
                                name: name.clone(),
                            });
                        }
                        self.expression(target)?;
                        Ok(())
                    }
                    _ => {
                        self.expression(target)?;
                        Ok(())
                    }
                }
            }
            Stmt::Print { value } | Stmt::ExprStmt { value } => {
                self.expression(value)?;
                Ok(())
            }
            Stmt::Return { value } => {
                let actual = self.expression(value)?;
                let Some((function_name, expected)) = &self.current_function else {
                    return Err(SemanticError::ReturnOutsideFunction);
                };
                if let Some(expected) = expected {
                    if !types_compatible(expected, &actual) {
                        return Err(SemanticError::ReturnTypeMismatch {
                            name: function_name.clone(),
                            expected: expected.clone(),
                            actual,
                        });
                    }
                }
                Ok(())
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let ty = self.expression(condition)?;
                if ty != Type::Bool {
                    return Err(SemanticError::NonBooleanCondition(ty));
                }
                self.scoped_block(then_branch)?;
                self.scoped_block(else_branch)
            }
            Stmt::While { condition, body } => {
                let ty = self.expression(condition)?;
                if ty != Type::Bool {
                    return Err(SemanticError::NonBooleanCondition(ty));
                }
                self.loop_depth += 1;
                let result = self.scoped_block(body);
                self.loop_depth -= 1;
                result
            }
            Stmt::Loop { body } => {
                self.loop_depth += 1;
                let result = self.scoped_block(body);
                self.loop_depth -= 1;
                result
            }
            Stmt::Break => {
                if self.loop_depth == 0 {
                    Err(SemanticError::BreakOutsideLoop)
                } else {
                    Ok(())
                }
            }
            Stmt::Continue => {
                if self.loop_depth == 0 {
                    Err(SemanticError::ContinueOutsideLoop)
                } else {
                    Ok(())
                }
            }
            Stmt::Actor { name, body } => {
                // v1.30.1-alpha: Register Kotak in topology
                if self.actor_registry.contains(name) {
                    return Err(SemanticError::DuplicateActor { name: name.clone() });
                }
                self.actor_registry.insert(name.clone());
                // Validate Kotak body
                self.scopes.push(HashMap::new());
                let result = self.block(body);
                self.scopes.pop();
                // Collect Channel declarations within this Actor
                for stmt in body {
                    if let Stmt::Let { declared_type: Some(Type::Channel { from, to, message_type }), .. } = stmt {
                        self.channel_registry.push((from.clone(), to.clone(), message_type.clone()));
                    }
                }
                result
            }
            // v1.33.0-alpha: Service manifest validation
            Stmt::Service { name, port, requires, handler, policy } => {
                // Duplicate service name check
                if self.actor_registry.contains(name) {
                    // Service name clash dengan actor name
                    return Err(SemanticError::DuplicateActor { name: name.clone() });
                }
                // Port validation (0 = invalid)
                if *port == 0 {
                    return Err(SemanticError::CapabilityContractViolation {
                        symbol: name.clone(),
                        gate: "invalid port 0".to_string(),
                    });
                }
                // Well-known port check (require gate for ports < 1024)
                if *port < 1024 && requires.is_none() {
                    // Warning: well-known ports should have a requires gate
                    // Dalam Pantas mode: log warning sahaja
                    // Dalam Pakar mode: ini adalah error
                }
                // Policy validation
                let valid_policies = ["Block", "DropOldest", "Error"];
                if !valid_policies.contains(&policy.as_str()) {
                    return Err(SemanticError::CapabilityContractViolation {
                        symbol: name.clone(),
                        gate: format!("invalid policy '{}'", policy),
                    });
                }
                // Handler function must exist — deferred ke Pass 2 (tier2)
                Ok(())
            }
            Stmt::Match { value, arms } => {
                let val_ty = self.expression(value)?;
                if !val_ty.is_result() {
                    return Err(SemanticError::MatchOnNonResult { ty: val_ty });
                }
                let mut has_ok = false;
                let mut has_err = false;
                for arm in arms {
                    match &arm.pattern {
                        MatchPattern::Ok { .. } => has_ok = true,
                        MatchPattern::Err { .. } => has_err = true,
                        MatchPattern::Wildcard => { has_ok = true; has_err = true; }
                    }
                    self.scoped_block(&arm.body)?;
                }
                if !has_ok || !has_err {
                    return Err(SemanticError::NonExhaustiveMatch {
                        missing: if !has_ok { "Ok".to_string() } else { "Err".to_string() },
                    });
                }
                Ok(())
            }
        }
    }

    fn check_assignment(
        &self,
        name: &str,
        declared: &Type,
        actual: &Type,
        value: &Expr,
    ) -> Result<(), SemanticError> {
        if declared.is_pointer() {
            if matches!(value, Expr::AddressOfLiteral(_)) && self.hw_zone_depth == 0 {
                return Err(SemanticError::HardwareMutationOutsideZone);
            }
            match value {
                Expr::AddressOfLiteral(addr)
                    if self.hardware_addresses.contains(addr)
                        || self.policy == SeverityPolicy::Desktop =>
                {
                    Ok(())
                }
                Expr::AddressOfLiteral(_) => Err(SemanticError::BareAddressRejected {
                    policy: self.policy,
                }),
                Expr::Variable(source) if self.hardware_regions.contains_key(source) => Ok(()),
                _ => Err(SemanticError::InvalidPointerInitializer {
                    name: name.to_string(),
                }),
            }?;
        }
        if let Expr::Integer(value) = value {
            if !integer_fits(*value, declared) {
                return Err(SemanticError::NumericBounds {
                    value: *value,
                    ty: declared.clone(),
                });
            }
        }
        if !types_compatible(declared, actual) {
            return Err(SemanticError::DeclaredTypeMismatch {
                name: name.to_string(),
                declared: declared.clone(),
                actual: actual.clone(),
            });
        }
        Ok(())
    }

    // ─── Ketuk 1: Core Memory Model — Buffer ownership & provenance ───

    /// Register a buffer variable with its provenance (element type + capacity).
    fn register_buffer(&mut self, name: &str, element_type: Type, capacity: i64) {
        self.buffer_registry
            .insert(name.to_string(), (element_type, capacity));
    }

    /// Check if a variable has been moved (ownership transferred).
    fn is_moved(&self, name: &str) -> bool {
        self.moved_vars.contains(name)
    }

    /// Mark a variable as moved (ownership transferred).
    fn mark_moved(&mut self, name: &str) {
        self.moved_vars.insert(name.to_string());
    }

    /// Validate buffer index access: check against declared capacity.
    // ─── Ketuk 3: File Handle ABI ───

    fn register_handle(&mut self, name: &str, mode: &str) {
        self.handle_registry.insert(name.to_string(), (Type::Opaque { name: "FileHandle".to_string() }, true));
        self.handle_permissions.insert(name.to_string(), mode.to_string());
    }

    fn close_handle(&mut self, name: &str) {
        if let Some((_, open)) = self.handle_registry.get_mut(name) {
            *open = false;
        }
    }

    fn is_handle_open(&self, name: &str) -> bool {
        self.handle_registry.get(name).map(|(_, open)| *open).unwrap_or(false)
    }

    fn check_handle_permission(&self, name: &str, required: &str) -> bool {
        self.handle_permissions.get(name).map(|mode| {
            mode == "ReadWrite" || mode == required
        }).unwrap_or(false)
    }

    fn validate_buffer_index(
        &self,
        buf_name: &str,
        index_expr: &Expr,
    ) -> Result<(), SemanticError> {
        // Check use-after-move
        if self.is_moved(buf_name) {
            return Err(SemanticError::UseAfterMove {
                name: buf_name.to_string(),
            });
        }

        // Look up buffer provenance
        let (_, capacity) = self
            .buffer_registry
            .get(buf_name)
            .ok_or_else(|| SemanticError::NotABuffer {
                name: buf_name.to_string(),
            })?;

        // If index is a compile-time constant, check against capacity
        if let Expr::Integer(idx) = index_expr {
            if *idx < 0 || *idx >= *capacity {
                return Err(SemanticError::BufferOverflow {
                    name: buf_name.to_string(),
                    index: *idx,
                    capacity: *capacity,
                });
            }
        }
        // For runtime indices, we can't statically verify — emit a note
        // The runtime will enforce bounds via provenance_id
        Ok(())
    }

    fn expression(&self, expr: &Expr) -> Result<Type, SemanticError> {
        match expr {
            Expr::Integer(_) => Ok(Type::I64),
            Expr::Boolean(_) => Ok(Type::Bool),
            Expr::StringLiteral(_) => Ok(Type::String),
            Expr::Variable(name) => {
                // Phase 2: Check if variable was moved via Pintu hantar()
                if self.moved_via_channel.contains(name) {
                    return Err(SemanticError::UseAfterSend { name: name.clone() });
                }
                self.resolve(name)
            }
            Expr::AddressOfLiteral(addr) => {
                if self.hw_zone_depth == 0 {
                    return Err(SemanticError::HardwareMutationOutsideZone);
                }
                if self.policy != SeverityPolicy::Desktop && !self.hardware_addresses.contains(addr)
                {
                    return Err(SemanticError::MissingProvenance(*addr));
                }
                Ok(Type::Pointer(Box::new(Type::U16)))
            }
            // Ketuk 2: Result constructors
            Expr::Ok { value } => {
                let inner = self.expression(value)?;
                Ok(Type::Result { ok: Box::new(inner), err: Box::new(Type::I64) })
            }
            Expr::Err { value } => {
                let inner = self.expression(value)?;
                Ok(Type::Result { ok: Box::new(Type::I64), err: Box::new(inner) })
            }
            Expr::Index { base, index } => {
                // Ketuk 1: Buffer/Slice indexing — buf[idx]
                let base_ty = self.expression(base)?;
                let idx_ty = self.expression(index)?;

                // Index must be integer
                if !is_numeric(&idx_ty) {
                    return Err(SemanticError::TypeMismatch {
                        op: BinaryOp::Add,
                        expected: "integer index",
                        left: idx_ty.clone(),
                        right: idx_ty,
                    });
                }

                // Base must be slice or buffer
                match &base_ty {
                    Type::Slice { element } | Type::Buffer { element } => {
                        // Validate index against buffer provenance
                        if let Expr::Variable(buf_name) = base.as_ref() {
                            self.validate_buffer_index(buf_name, index)?;
                        }
                        Ok(*element.clone())
                    }
                    other => Err(SemanticError::TypeMismatch {
                        op: BinaryOp::Add,
                        expected: "slice or buffer",
                        left: other.clone(),
                        right: other.clone(),
                    }),
                }
            }
            Expr::MethodCall { object, method, args } => {
                // Ketuk 3: File Handle ABI — method call on opaque type
                // Validate handle is open
                if !self.is_handle_open(object) {
                    return Err(SemanticError::HandleNotOpen {
                        name: object.clone(),
                    });
                }
                // Validate permission for read/write operations
                match method.as_str() {
                    "read" | "Read" | "baca" => {
                        if !self.check_handle_permission(object, "Read") {
                            return Err(SemanticError::HandlePermissionDenied {
                                name: object.clone(),
                                operation: "read".to_string(),
                            });
                        }
                    }
                    "write" | "Write" | "tulis" => {
                        if !self.check_handle_permission(object, "Write") {
                            return Err(SemanticError::HandlePermissionDenied {
                                name: object.clone(),
                                operation: "write".to_string(),
                            });
                        }
                    }
                    "close" | "Close" | "tutup" => {
                        self.close_handle(object);
                    }
                    _ => {}
                }
                // Validate arguments
                for arg in args {
                    self.expression(arg)?;
                }
                // Return Result type for read/write operations
                match method.as_str() {
                    "read" | "baca" => Ok(Type::Result { ok: Box::new(Type::Slice { element: Box::new(Type::U32) }), err: Box::new(Type::Opaque { name: "IoError".to_string() }) }),
                    "write" | "tulis" => Ok(Type::Result { ok: Box::new(Type::I64), err: Box::new(Type::Opaque { name: "IoError".to_string() }) }),
                    "close" | "tutup" => Ok(Type::Opaque { name: "Unit".to_string() }),
                    _ => Ok(Type::Opaque { name: "Unknown".to_string() }),
                }
            }
            // v1.30.1-alpha: Threading expressions
            Expr::Spawn { actor_name, args } => {
                if !self.actor_registry.contains(actor_name) {
                    return Err(SemanticError::ActorNotFound { name: actor_name.clone() });
                }
                for arg in args { self.expression(arg)?; }
                Ok(Type::Opaque { name: "ThreadHandle".to_string() })
            }
            Expr::Send { channel_name, value } => {
                // Validate Pintu exists in registry
                let found = self.channel_registry.iter().any(|(f, t, _)| {
                    f == channel_name || t == channel_name
                });
                if !found {
                    // Check if it's a variable with Pintu type
                    let _ = self.resolve(channel_name); // Will error if not defined
                }
                // Phase 2: Zero-Copy Ownership Transfer
                // If value is a variable, mark it as moved via Pintu
                if let Expr::Variable(var_name) = value {
                    if self.moved_via_channel.contains(var_name) {
                        return Err(SemanticError::UseAfterSend { name: var_name.clone() });
                    }
                    self.moved_via_channel.insert(var_name.clone());
                }
                self.expression(value)?;
                Ok(Type::Opaque { name: "Unit".to_string() })
            }
            Expr::Recv { channel_name } => {
                // Return the message type of the Pintu
                if let Some((_, _, msg_type)) = self.channel_registry.iter().find(|(_, t, _)| t == channel_name) {
                    Ok(Type::Opaque { name: msg_type.clone() })
                } else {
                    // Fallback: check variable type
                    if let Ok(ty) = self.resolve(channel_name) {
                        Ok(ty)
                    } else {
                        Err(SemanticError::UndefinedVariable(channel_name.clone()))
                    }
                }
            }
            Expr::Join { actor_name } => {
                if !self.actor_registry.contains(actor_name) {
                    return Err(SemanticError::ActorNotFound { name: actor_name.clone() });
                }
                Ok(Type::Opaque { name: "Unit".to_string() })
            }
            // v1.30.1-alpha Phase 3: Backpressure + Scheduler
            Expr::TrySend { channel_name, value } => {
                // Validate channel exists
                let found = self.channel_registry.iter().any(|(f, t, _)| {
                    f == channel_name || t == channel_name
                });
                if !found {
                    let _ = self.resolve(channel_name)?;
                }
                // Ownership transfer (same as Send)
                if let Expr::Variable(var_name) = value {
                    if self.moved_via_channel.contains(var_name) {
                        return Err(SemanticError::UseAfterSend { name: var_name.clone() });
                    }
                    self.moved_via_channel.insert(var_name.clone());
                }
                self.expression(value)?;
                // Returns Result<bool, IoError>
                Ok(Type::Result { ok: Box::new(Type::Bool), err: Box::new(Type::Opaque { name: "IoError".to_string() }) })
            }
            Expr::TryRecv { channel_name } => {
                let found = self.channel_registry.iter().any(|(f, t, _)| {
                    f == channel_name || t == channel_name
                });
                if !found {
                    let _ = self.resolve(channel_name)?;
                }
                // Returns Option<T> — we don't know T at this point, use Opaque
                Ok(Type::Result { ok: Box::new(Type::Opaque { name: "T".to_string() }), err: Box::new(Type::Opaque { name: "None".to_string() }) })
            }
            Expr::Yield => {
                // Yield always succeeds, returns Unit
                Ok(Type::Opaque { name: "Unit".to_string() })
            }
            Expr::Sleep { duration_ms } => {
                let dur_ty = self.expression(duration_ms)?;
                if !is_numeric(&dur_ty) {
                    return Err(SemanticError::TypeMismatch {
                        op: BinaryOp::Add,
                        expected: "numeric duration (milliseconds)",
                        left: dur_ty.clone(),
                        right: dur_ty,
                    });
                }
                Ok(Type::Opaque { name: "Unit".to_string() })
            }
            Expr::TimeoutRecv { channel_name, timeout_ms } => {
                let found = self.channel_registry.iter().any(|(f, t, _)| {
                    f == channel_name || t == channel_name
                });
                if !found {
                    let _ = self.resolve(channel_name)?;
                }
                let to_ty = self.expression(timeout_ms)?;
                if !is_numeric(&to_ty) {
                    return Err(SemanticError::TypeMismatch {
                        op: BinaryOp::Add,
                        expected: "numeric timeout (milliseconds)",
                        left: to_ty.clone(),
                        right: to_ty,
                    });
                }
                // Returns Result<T, RecvTimeout>
                Ok(Type::Result { ok: Box::new(Type::Opaque { name: "T".to_string() }), err: Box::new(Type::Opaque { name: "RecvTimeout".to_string() }) })
            }
            Expr::Call { callee, args } => {
                // Sprint 2.5: Function/struct constructor call
                // Resolve the callee name, then validate arguments
                let callee_name = match callee.as_ref() {
                    Expr::Variable(name) => name.as_str(),
                    _ => "<complex callee>",
                };

                // Try to resolve as a type name (struct constructor)
                // or as a variable (function call)
                if let Ok(ty) = self.resolve(callee_name) {
                    // Struct constructor: Color(255, 0, 0, 255)
                    for arg in args {
                        let _ = self.expression(arg)?;
                    }
                    Ok(ty)
                } else {
                    // Function call: InitWindow(800, 600, "Hi")
                    for arg in args {
                        let _ = self.expression(arg)?;
                    }
                    Ok(Type::I64) // default return type (to be refined)
                }
            }
            Expr::Grouped(inner) => self.expression(inner),
            Expr::Binary { left, op, right } => {
                if *op == BinaryOp::Divide && matches!(right.as_ref(), Expr::Integer(0)) {
                    return Err(SemanticError::DivisionByZero);
                }
                let left_ty = self.expression(left)?;
                let right_ty = self.expression(right)?;
                match op {
                    BinaryOp::Add
                    | BinaryOp::Subtract
                    | BinaryOp::Multiply
                    | BinaryOp::Divide
                    | BinaryOp::BitAnd
                    | BinaryOp::BitOr
                    | BinaryOp::ShiftLeft
                    | BinaryOp::ShiftRight => {
                        if is_numeric(&left_ty) && is_numeric(&right_ty) {
                            Ok(promote_numeric(left_ty, right_ty))
                        } else {
                            Err(SemanticError::TypeMismatch {
                                op: *op,
                                expected: "numeric",
                                left: left_ty,
                                right: right_ty,
                            })
                        }
                    }
                    BinaryOp::Greater
                    | BinaryOp::GreaterEqual
                    | BinaryOp::Less
                    | BinaryOp::LessEqual => {
                        if is_numeric(&left_ty) && is_numeric(&right_ty) {
                            Ok(Type::Bool)
                        } else {
                            Err(SemanticError::TypeMismatch {
                                op: *op,
                                expected: "numeric",
                                left: left_ty,
                                right: right_ty,
                            })
                        }
                    }
                    BinaryOp::And | BinaryOp::Or => {
                        if left_ty == Type::Bool && right_ty == Type::Bool {
                            Ok(Type::Bool)
                        } else {
                            Err(SemanticError::TypeMismatch {
                                op: *op,
                                expected: "Bool",
                                left: left_ty,
                                right: right_ty,
                            })
                        }
                    }
                    BinaryOp::Equal | BinaryOp::NotEqual => {
                        if types_compatible(&left_ty, &right_ty) {
                            Ok(Type::Bool)
                        } else {
                            Err(SemanticError::TypeMismatch {
                                op: *op,
                                expected: "matching",
                                left: left_ty,
                                right: right_ty,
                            })
                        }
                    }
                }
            }
        }
    }

    fn define(&mut self, name: &str, ty: Type) -> Result<(), SemanticError> {
        let scope = self
            .scopes
            .last_mut()
            .expect("semantic analyzer must always have a scope");
        if scope.contains_key(name) {
            return Err(SemanticError::DuplicateVariable(name.to_string()));
        }
        scope.insert(name.to_string(), ty);
        Ok(())
    }

    fn resolve(&self, name: &str) -> Result<Type, SemanticError> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Ok(ty.clone());
            }
        }
        Err(SemanticError::UndefinedVariable(name.to_string()))
    }
}

fn is_numeric(ty: &Type) -> bool {
    matches!(
        ty,
        Type::I32 | Type::I64 | Type::U16 | Type::U32 | Type::F64
    )
}

fn promote_numeric(left: Type, right: Type) -> Type {
    if left == Type::F64 || right == Type::F64 {
        Type::F64
    } else if left == Type::I64 || right == Type::I64 {
        Type::I64
    } else if left == Type::U32 || right == Type::U32 {
        Type::U32
    } else if left == Type::I32 || right == Type::I32 {
        Type::I32
    } else {
        Type::U16
    }
}

fn types_compatible(expected: &Type, actual: &Type) -> bool {
    expected == actual
        || (is_numeric(expected) && is_numeric(actual))
        || (expected.is_pointer() && actual.is_pointer())
}

fn integer_fits(value: i64, ty: &Type) -> bool {
    match ty {
        Type::I32 => value >= i32::MIN as i64 && value <= i32::MAX as i64,
        Type::I64 => true,
        Type::U16 => value >= 0 && value <= u16::MAX as i64,
        Type::U32 => value >= 0 && value <= u32::MAX as i64,
        Type::F64 => true,
        Type::Bool => value == 0 || value == 1,
        Type::Pointer(_) | Type::String => false,
    }
}

// =========================================================================
// v1.42 P6: StrictAudioContext — Hardware-Safe Audio Guards
//
// Validates that audio callback functions (ISR-like) do not contain:
//   1. AudioViolationIo — I/O operations (Print, DrawText, InitWindow)
//   2. AudioViolationRecursion — self-calling (recursive callbacks)
//   3. AudioViolationUnboundedLoop — unbounded `loop { }` (watchdog risk)
//   4. AudioViolationForbiddenCall — calls to unsafe functions
//
// Pattern: `SetAudioStreamCallback(audio_func)` → mark `audio_func` as
// audio callback → validate `audio_func` body against all 4 violation types.
// =========================================================================

/// Functions that are forbidden inside audio callbacks (I/O operations).
const AUDIO_FORBIDDEN_IO: &[&str] = &["Print", "DrawText", "InitWindow", "ClearBackground"];

/// Functions considered unsafe for audio callbacks.
const AUDIO_FORBIDDEN_CALLS: &[&str] = &[
    "malloc", "free", "fopen", "fclose", "pthread_create", "spawn",
];

impl Analyzer {
    /// Mark a function as an audio callback. Called when
    /// `SetAudioStreamCallback(func_name)` is detected.
    pub fn register_audio_callback(&mut self, func_name: &str) {
        self.audio_callbacks.insert(func_name.to_string());
    }

    /// v1.42 P6: Validate all registered audio callbacks for safety violations.
    /// Call this after the full program has been analyzed.
    pub fn verify_audio_callbacks(&self, program: &Program) -> Result<(), SemanticError> {
        for callback_name in &self.audio_callbacks {
            // Find the function body
            if let Some(func_body) = self.find_function_body(program, callback_name) {
                self.verify_audio_safety(callback_name, func_body)?;
            }
        }
        Ok(())
    }

    /// v1.42 P6: Validate a single audio callback function body.
    /// Walks the AST and checks for all 4 violation types.
    fn verify_audio_safety(
        &self,
        func_name: &str,
        stmts: &[Stmt],
    ) -> Result<(), SemanticError> {
        for stmt in stmts {
            self.verify_audio_stmt(func_name, stmt)?;
        }
        Ok(())
    }

    fn verify_audio_stmt(
        &self,
        func_name: &str,
        stmt: &Stmt,
    ) -> Result<(), SemanticError> {
        match stmt {
            Stmt::Print { .. } => Err(SemanticError::AudioViolationIo {
                function: func_name.to_string(),
            }),
            Stmt::ExprStmt { value } => self.verify_audio_expr(func_name, value),
            Stmt::Let { value, .. } => self.verify_audio_expr(func_name, value),
            Stmt::If { condition, then_branch, else_branch } => {
                self.verify_audio_expr(func_name, condition)?;
                for s in then_branch { self.verify_audio_stmt(func_name, s)?; }
                for s in else_branch { self.verify_audio_stmt(func_name, s)?; }
                Ok(())
            }
            Stmt::While { condition, body } => {
                self.verify_audio_expr(func_name, condition)?;
                for s in body { self.verify_audio_stmt(func_name, s)?; }
                Ok(())
            }
            Stmt::Loop { body } => {
                // P6: Unbounded loop detected in audio callback
                Err(SemanticError::AudioViolationUnboundedLoop)
            }
            Stmt::For { body, .. } => {
                for s in body { self.verify_audio_stmt(func_name, s)?; }
                Ok(())
            }
            Stmt::Block(stmts) | Stmt::UnsafeBlock(stmts) | Stmt::HardwareZone(stmts) => {
                for s in stmts { self.verify_audio_stmt(func_name, s)?; }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn verify_audio_expr(
        &self,
        func_name: &str,
        expr: &Expr,
    ) -> Result<(), SemanticError> {
        match expr {
            Expr::Call { callee, .. } => {
                if let Expr::Variable(name) = callee.as_ref() {
                    // P6: Check for forbidden I/O functions
                    if AUDIO_FORBIDDEN_IO.contains(&name.as_str()) {
                        return Err(SemanticError::AudioViolationIo {
                            function: func_name.to_string(),
                        });
                    }
                    // P6: Check for recursion (self-calling)
                    if name == func_name {
                        return Err(SemanticError::AudioViolationRecursion {
                            function: func_name.to_string(),
                        });
                    }
                    // P6: Check for forbidden unsafe calls
                    if AUDIO_FORBIDDEN_CALLS.contains(&name.as_str()) {
                        return Err(SemanticError::AudioViolationForbiddenCall {
                            function: func_name.to_string(),
                            callee: name.clone(),
                        });
                    }
                }
                Ok(())
            }
            Expr::Binary { left, right, .. } => {
                self.verify_audio_expr(func_name, left)?;
                self.verify_audio_expr(func_name, right)
            }
            Expr::Unary { operand, .. } => self.verify_audio_expr(func_name, operand),
            Expr::Grouped(inner) => self.verify_audio_expr(func_name, inner),
            _ => Ok(()),
        }
    }

    /// Helper: find function body by name in the AST.
    fn find_function_body(&self, program: &Program, name: &str) -> Option<Vec<Stmt>> {
        for stmt in &program.statements {
            if let Stmt::Function { name: func_name, body, .. } = stmt {
                if func_name == name {
                    return Some(body.clone());
                }
            }
        }
        None
    }
}
