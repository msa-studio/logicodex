// =========================================================================
// Project: Logicodex Language Engine
// Pipeline: single HIR compilation engine (.ldx -> AST -> HIR -> LLVM)
// Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
// Copyright (c) 2026. All Rights Reserved.
// Licensed under permissive dual-license: MIT & Apache License 2.0
// =========================================================================
pub fn runtime_assembly() -> &'static str {
    r#"
    .text
    .globl logicodex_print_i64
    .def logicodex_print_i64; .scl 2; .type 32; .endef
logicodex_print_i64:
    pushq %rbp
    movq %rsp, %rbp
    subq $96, %rsp
    leaq -40(%rbp), %rdx
    movb $10, 31(%rdx)
    movq %rcx, %rax
    movq $1, %r10
    cmpq $0, %rax
    jne .Lwin_convert
    movb $48, 30(%rdx)
    leaq 30(%rdx), %rdx
    movl $2, %r8d
    jmp .Lwin_console
.Lwin_convert:
    movq $0, %r11
    cmpq $0, %rax
    jge .Lwin_digits
    negq %rax
    movq $1, %r11
.Lwin_digits:
    leaq 31(%rdx), %r9
    movq $10, %r10
.Lwin_loop:
    xorq %rdx, %rdx
    divq %r10
    addb $48, %dl
    decq %r9
    movb %dl, (%r9)
    cmpq $0, %rax
    jne .Lwin_loop
    cmpq $0, %r11
    je .Lwin_measure
    decq %r9
    movb $45, (%r9)
.Lwin_measure:
    leaq -40(%rbp), %rdx
    addq $32, %rdx
    subq %r9, %rdx
    movq %r9, %rdx
    movl $32, %r8d
.Lwin_console:
    movq $-11, %rcx
    call GetStdHandle
    movq %rax, %rcx
    leaq -8(%rbp), %r9
    movq $0, 32(%rsp)
    call WriteFile
    addq $96, %rsp
    popq %rbp
    ret
"#
}
