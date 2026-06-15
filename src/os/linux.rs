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

    # ---------------------------------------------------------------------
    # logicodex_yield() -> i64
    # Cooperative yield via the sched_yield(2) syscall (x86_64 nr = 24).
    # Takes no arguments; always returns 0.
    # ---------------------------------------------------------------------
    .global logicodex_yield
    .type logicodex_yield, @function
logicodex_yield:
    mov $24, %rax          # sched_yield
    syscall
    xor %rax, %rax         # return 0
    ret

    # ---------------------------------------------------------------------
    # logicodex_sleep(ms: i64) -> i64
    # Sleep for `ms` milliseconds via nanosleep(2) (x86_64 nr = 35).
    # Builds a `struct timespec { i64 tv_sec; i64 tv_nsec; }` on the stack:
    #   tv_sec  = ms / 1000
    #   tv_nsec = (ms % 1000) * 1_000_000
    # Always returns 0 (interrupted sleeps are not resumed in this phase).
    # ---------------------------------------------------------------------
    .global logicodex_sleep
    .type logicodex_sleep, @function
logicodex_sleep:
    push %rbp
    mov %rsp, %rbp
    sub $16, %rsp          # room for timespec (16 bytes)
    mov %rdi, %rax         # rax = ms
    cqo                    # sign-extend rax into rdx:rax
    mov $1000, %rcx
    idiv %rcx             # rax = ms/1000 (sec), rdx = ms%1000 (rem ms)
    mov %rax, -16(%rbp)    # tv_sec
    mov %rdx, %rax         # rax = remainder ms
    mov $1000000, %rcx
    imul %rcx, %rax        # rax = rem_ms * 1_000_000 (nsec)
    mov %rax, -8(%rbp)     # tv_nsec
    lea -16(%rbp), %rdi    # rdi = &timespec
    xor %rsi, %rsi         # rsi = NULL (no remainder out)
    mov $35, %rax          # nanosleep
    syscall
    xor %rax, %rax         # return 0
    leave
    ret
"#
}
