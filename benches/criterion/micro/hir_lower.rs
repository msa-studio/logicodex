// =========================================================================
// Logicodex v1.45 — Layer 1 Micro-Benchmark: AST → HIR Lowering
//
// Measures: Expression lowering from AST to HIR (v1.30)
// Target: < 200ns mean, < 280ns p99
// Architecture: HIR as intermediate between AST and LLVM
// =========================================================================

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;

/// Simulated AST expression.
#[derive(Clone)]
enum AstExpr {
    Literal(i64),
    Variable(String),
    Binary { op: BinOp, left: Box<AstExpr>, right: Box<AstExpr> },
}

#[derive(Clone)]
enum BinOp { Add, Sub, Mul, Div }

/// Simulated HIR expression.
#[derive(Clone)]
enum HirExpr {
    Literal(i64),
    Local(u32),
    Binary { op: BinOp, left: Box<HirExpr>, right: Box<HirExpr> },
}

/// Lower AST to HIR (mirrors src/hir.rs lowering).
fn lower_expr(ast: &AstExpr, symbol_table: &mut Vec<String>) -> HirExpr {
    match ast {
        AstExpr::Literal(v) => HirExpr::Literal(*v),
        AstExpr::Variable(name) => {
            let id = lookup_or_insert(symbol_table, name);
            HirExpr::Local(id)
        }
        AstExpr::Binary { op, left, right } => {
            let l = lower_expr(left, symbol_table);
            let r = lower_expr(right, symbol_table);
            HirExpr::Binary {
                op: op.clone(),
                left: Box::new(l),
                right: Box::new(r),
            }
        }
    }
}

fn lookup_or_insert(table: &mut Vec<String>, name: &str) -> u32 {
    if let Some(idx) = table.iter().position(|n| n == name) {
        idx as u32
    } else {
        let idx = table.len();
        table.push(name.to_string());
        idx as u32
    }
}

/// Build a deep binary expression tree: (((1 + 2) + 3) + 4) ...
fn make_deep_ast(depth: usize) -> AstExpr {
    let mut expr = AstExpr::Literal(1);
    for i in 2..=depth {
        expr = AstExpr::Binary {
            op: BinOp::Add,
            left: Box::new(expr),
            right: Box::new(AstExpr::Literal(i as i64)),
        };
    }
    expr
}

fn bench_hir_lower(c: &mut Criterion) {
    let mut group = c.benchmark_group("hir_lower_latency");
    group.measurement_time(Duration::from_secs(5));
    group.warm_up_time(Duration::from_secs(1));
    group.sample_size(1000);

    // Varying expression depths
    for depth in [2, 4, 8, 16, 32] {
        let ast = make_deep_ast(depth);
        group.bench_with_input(
            BenchmarkId::new("depth", depth),
            &ast,
            |b, ast| {
                b.iter(|| {
                    let mut symbols = Vec::new();
                    let hir = lower_expr(black_box(ast), black_box(&mut symbols));
                    black_box(hir);
                });
            },
        );
    }

    // Simple literal (fastest path)
    group.bench_function("literal", |b| {
        let ast = AstExpr::Literal(42);
        b.iter(|| {
            let mut symbols = Vec::new();
            let hir = lower_expr(black_box(&ast), black_box(&mut symbols));
            black_box(hir);
        });
    });

    // Variable lookup (symbol table hit)
    group.bench_function("variable", |b| {
        let ast = AstExpr::Variable("counter".into());
        let mut prefill = Vec::new();
        prefill.push("counter".to_string());
        b.iter(|| {
            let mut symbols = prefill.clone();
            let hir = lower_expr(black_box(&ast), black_box(&mut symbols));
            black_box(hir);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_hir_lower);
criterion_main!(benches);
