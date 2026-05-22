// =========================================================================
// Project: Logicodex Language Engine (Phase 2 - Milestone 1)
// Version: v1.11-alpha (EBNF Formal Grammar Integration)
// Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
// Copyright (c) 2026. All Rights Reserved.
// Licensed under permissive dual-license: MIT & Apache License 2.0
// =========================================================================
pub fn runtime_assembly() -> &'static str {
    r#"
    .text
    .global logicodex_print_i64
    .type logicodex_print_i64, @function
logicodex_print_i64:
    push %rbp
    mov %rsp, %rbp
    sub $64, %rsp
    mov %rdi, %rax
    lea -2(%rbp), %rsi
    movb $10, (%rsi)
    mov $1, %rcx
    cmp $0, %rax
    jne .Lconvert
    movb $48, -3(%rbp)
    lea -3(%rbp), %rsi
    mov $2, %rdx
    jmp .Lwrite
.Lconvert:
    mov $0, %r8
    cmp $0, %rax
    jge .Ldigits
    neg %rax
    mov $1, %r8
.Ldigits:
    mov $10, %r9
.Lloop:
    xor %rdx, %rdx
    div %r9
    add $48, %dl
    dec %rsi
    mov %dl, (%rsi)
    inc %rcx
    cmp $0, %rax
    jne .Lloop
    cmp $0, %r8
    je .Lprepare
    dec %rsi
    movb $45, (%rsi)
    inc %rcx
.Lprepare:
    mov %rcx, %rdx
.Lwrite:
    mov $1, %rax
    mov $1, %rdi
    syscall
    leave
    ret
"#
}
