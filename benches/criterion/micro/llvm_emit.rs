// =========================================================================
// Logicodex v1.45 — Layer 1 Micro-Benchmark: LLVM IR Generation
//
// Measures: LLVM IR instruction emission (v1.30 codegen)
// Target: < 500ns mean, < 620ns p99 for i64 + i64
// Architecture: inkwell-based LLVM codegen
// =========================================================================

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use inkwell::context::Context;
use inkwell::OptimizationLevel;
use std::time::Duration;

/// Build a minimal LLVM module with a single binary operation.
/// Mirrors `src/codegen.rs::emit_expr(Binary)`.
fn emit_binary_op(ctx: &Context, op_name: &str) {
    let module = ctx.create_module("bench");
    let builder = ctx.create_builder();

    let i64_ty = ctx.i64_type();
    let fn_ty = i64_ty.fn_type(&[i64_ty.into(), i64_ty.into()], false);
    let func = module.add_function(op_name, fn_ty, None);
    let entry = ctx.append_basic_block(func, "entry");
    builder.position_at_end(entry);

    let a = func.get_nth_param(0).unwrap().into_int_value();
    let b = func.get_nth_param(1).unwrap().into_int_value();

    let result = builder.build_int_add(a, b, "add").unwrap();
    builder.build_return(Some(&result)).unwrap();

    // Verify the module (this is the expensive part)
    module.verify().unwrap();
}

fn bench_llvm_emit(c: &mut Criterion) {
    let ctx = Context::create();

    let mut group = c.benchmark_group("llvm_emit_latency");
    group.measurement_time(Duration::from_secs(8)); // LLVM is slower — need more time
    group.warm_up_time(Duration::from_secs(2));
    group.sample_size(500);

    // i64 + i64 (most common operation)
    group.bench_function("i64_add", |b| {
        let ctx = Context::create();
        b.iter(|| {
            emit_binary_op(black_box(&ctx), "add_i64");
        });
    });

    // i64 * i64 (mul is more expensive)
    group.bench_function("i64_mul", |b| {
        let ctx = Context::create();
        b.iter(|| {
            let module = ctx.create_module("bench");
            let builder = ctx.create_builder();
            let i64_ty = ctx.i64_type();
            let fn_ty = i64_ty.fn_type(&[i64_ty.into(), i64_ty.into()], false);
            let func = module.add_function("mul_i64", fn_ty, None);
            let entry = ctx.append_basic_block(func, "entry");
            builder.position_at_end(entry);
            let a = func.get_nth_param(0).unwrap().into_int_value();
            let b = func.get_nth_param(1).unwrap().into_int_value();
            let result = builder.build_int_mul(a, b, "mul").unwrap();
            builder.build_return(Some(&result)).unwrap();
            module.verify().unwrap();
        });
    });

    // f64 + f64 (floating point)
    group.bench_function("f64_add", |b| {
        let ctx = Context::create();
        b.iter(|| {
            let module = ctx.create_module("bench");
            let builder = ctx.create_builder();
            let f64_ty = ctx.f64_type();
            let fn_ty = f64_ty.fn_type(&[f64_ty.into(), f64_ty.into()], false);
            let func = module.add_function("add_f64", fn_ty, None);
            let entry = ctx.append_basic_block(func, "entry");
            builder.position_at_end(entry);
            let a = func.get_nth_param(0).unwrap().into_float_value();
            let b = func.get_nth_param(1).unwrap().into_float_value();
            let result = builder.build_float_add(a, b, "fadd").unwrap();
            builder.build_return(Some(&result)).unwrap();
            module.verify().unwrap();
        });
    });

    // Function call (more realistic: declare extern + call)
    group.bench_function("extern_call_DrawText", |b| {
        let ctx = Context::create();
        b.iter(|| {
            let module = ctx.create_module("bench");
            let builder = ctx.create_builder();
            let i32_ty = ctx.i32_type();
            let i64_ty = ctx.i64_type();
            let void_ty = ctx.void_type();

            // Declare extern DrawText(i32, i32, i32, i64)
            let dt_ty = void_ty.fn_type(
                &[i32_ty.into(), i32_ty.into(), i32_ty.into(), i64_ty.into()], false);
            let dt = module.add_function("DrawText", dt_ty, None);
            dt.set_linkage(inkwell::module::Linkage::External);

            // Create caller
            let caller_ty = void_ty.fn_type(&[], false);
            let caller = module.add_function("caller", caller_ty, None);
            let entry = ctx.append_basic_block(caller, "entry");
            builder.position_at_end(entry);

            let args = &[
                i32_ty.const_int(10, false).into(),
                i32_ty.const_int(20, false).into(),
                i32_ty.const_int(30, false).into(),
                i64_ty.const_int(0xFF0000FF, false).into(),
            ];
            builder.build_call(dt, args, "call").unwrap();
            builder.build_return(None).unwrap();
            module.verify().unwrap();
        });
    });

    group.finish();
}

criterion_group!(benches, bench_llvm_emit);
criterion_main!(benches);
