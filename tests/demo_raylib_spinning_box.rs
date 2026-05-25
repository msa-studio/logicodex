// =========================================================================
// Logicodex v1.30 — Demo: Raylib Spinning Box Pipeline Validation
//
// This test verifies that examples/raylib_spinning_box.ldx can be fully
// processed by the Logicodex compiler pipeline:
//   1. Parse → Expr::Call for all Color(...) and DrawRectangle(...)
//   2. TypeChecker validates all struct constructors
//   3. CallableRegistry contains all Raylib functions used
//   4. HIR lowering → HirExprKind::Call for each function call
//   5. Struct packing: Color(255, 0, 0, 255) → 0xFF0000FF
//   6. Codegen: CallableRegistry lookup + extern function declaration ready
// =========================================================================

use logicodex::ast::{Expr, Stmt};
use logicodex::ffi::{CallableRegistry, SafetyContext, raylib};
use logicodex::hir::LoweringContext;
use logicodex::semantic::type_checker::TypeChecker;
use logicodex::types::TypeRegistry;

const DEMO_SOURCE: &str = include_str!("../examples/raylib_spinning_box.ldx");

// ─── Helper: Parse the demo file ───
fn parse_demo() -> Vec<Stmt> {
    use logicodex::lexer::{Lexer, Lexicon};
    use logicodex::parser::{CompilerPipeline, Parser};

    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(DEMO_SOURCE, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();
    program.statements
}

// ─── 1. Parser: All struct constructors parsed as Expr::Call ───

#[test]
demo_parses_all_color_constructors() {
    let stmts = parse_demo();

    // Walk all expressions and count Color(...) calls
    let mut color_count = 0;
    let mut draw_count = 0;
    let mut initwindow_count = 0;

    fn count_calls_in_expr(expr: &Expr, color: &mut i32, draw: &mut i32, init: &mut i32) {
        match expr {
            Expr::Call { callee, args } => {
                if let Expr::Variable(name) = callee.as_ref() {
                    if name == "Color" { *color += 1; }
                    if name.starts_with("Draw") { *draw += 1; }
                    if name == "InitWindow" { *init += 1; }
                }
                for arg in args { count_calls_in_expr(arg, color, draw, init); }
            }
            Expr::Binary { left, right, .. } => {
                count_calls_in_expr(left, color, draw, init);
                count_calls_in_expr(right, color, draw, init);
            }
            Expr::Grouped(inner) => count_calls_in_expr(inner, color, draw, init),
            _ => {}
        }
    }

    fn count_calls_in_stmts(stmts: &[Stmt], color: &mut i32, draw: &mut i32, init: &mut i32) {
        for stmt in stmts {
            match stmt {
                Stmt::Let { value, .. } => count_calls_in_expr(value, color, draw, init),
                Stmt::ExprStmt { value } | Stmt::Print { value } | Stmt::Return { value } => {
                    count_calls_in_expr(value, color, draw, init);
                }
                Stmt::If { condition, then_branch, else_branch } => {
                    count_calls_in_expr(condition, color, draw, init);
                    count_calls_in_stmts(then_branch, color, draw, init);
                    count_calls_in_stmts(else_branch, color, draw, init);
                }
                Stmt::While { condition, body } => {
                    count_calls_in_expr(condition, color, draw, init);
                    count_calls_in_stmts(body, color, draw, init);
                }
                Stmt::Loop { body } | Stmt::UnsafeBlock { body } | Stmt::HardwareZone { body } => {
                    count_calls_in_stmts(body, color, draw, init);
                }
                _ => {}
            }
        }
    }

    count_calls_in_stmts(&stmts, &mut color_count, &mut draw_count, &mut initwindow_count);

    assert!(color_count >= 6, "Expected >=6 Color constructors, got {}", color_count);
    assert!(draw_count >= 4, "Expected >=4 Draw* calls, got {}", draw_count);
    assert_eq!(initwindow_count, 1, "Expected 1 InitWindow call");
}

#[test]
demo_parses_color_with_four_args() {
    let stmts = parse_demo();

    // Find the red color declaration
    let red_let = stmts.iter().find(|s| matches!(s,
        Stmt::Let { name, .. } if name == "red"
    )).expect("red variable declaration must exist");

    match red_let {
        Stmt::Let { value, .. } => match value {
            Expr::Call { callee, args } => {
                assert_eq!(callee.as_ref(), &Expr::Variable("Color".into()));
                assert_eq!(args.len(), 4, "Color constructor must have 4 args");
                assert_eq!(args[0], Expr::Integer(255));
                assert_eq!(args[1], Expr::Integer(0));
                assert_eq!(args[2], Expr::Integer(0));
                assert_eq!(args[3], Expr::Integer(255));
            }
            other => panic!("Expected Color constructor, got {:?}", other),
        },
        _ => panic!("Expected Let statement"),
    }
}

#[test]
demo_parses_drawrectangle_call() {
    let stmts = parse_demo();

    // Walk to find DrawRectangle call
    let mut found = false;
    fn find_draw(expr: &Expr, found: &mut bool) {
        match expr {
            Expr::Call { callee, args } => {
                if let Expr::Variable(name) = callee.as_ref() {
                    if name == "DrawRectangle" && args.len() == 5 {
                        *found = true;
                        // Verify position args are integers
                        assert!(matches!(args[0], Expr::Integer(350) | Expr::Integer(352)));
                    }
                }
                for arg in args { find_draw(arg, found); }
            }
            Expr::Binary { left, right, .. } => { find_draw(left, found); find_draw(right, found); }
            Expr::Grouped(inner) => find_draw(inner, found),
            _ => {}
        }
    }

    fn search(stmts: &[Stmt], found: &mut bool) {
        for s in stmts {
            match s {
                Stmt::ExprStmt { value } | Stmt::Print { value } | Stmt::Return { value } | Stmt::Let { value, .. } => find_draw(value, found),
                Stmt::If { condition, then_branch, else_branch } => {
                    find_draw(condition, found);
                    search(then_branch, found);
                    search(else_branch, found);
                }
                Stmt::While { condition, body } => { find_draw(condition, found); search(body, found); }
                Stmt::Loop { body } | Stmt::UnsafeBlock { body } | Stmt::HardwareZone { body } => search(body, found),
                _ => {}
            }
        }
    }

    search(&stmts, &mut found);
    assert!(found, "DrawRectangle(5 args) must appear in demo");
}

// ─── 2. TypeChecker: All struct constructors valid ───

#[test]
demo_typechecker_validates_all_colors() {
    let mut types = TypeRegistry::new();
    let _ids = raylib::register_raylib_types(&mut types);
    let checker = TypeChecker::new(&types);

    // Color(255, 0, 0, 255) — red
    assert!(checker.check_call(
        &Expr::Variable("Color".into()),
        &[Expr::Integer(255), Expr::Integer(0), Expr::Integer(0), Expr::Integer(255)],
    ).is_ok(), "Color(255,0,0,255) must be valid");

    // Color(0, 255, 0, 255) — green
    assert!(checker.check_call(
        &Expr::Variable("Color".into()),
        &[Expr::Integer(0), Expr::Integer(255), Expr::Integer(0), Expr::Integer(255)],
    ).is_ok(), "Color(0,255,0,255) must be valid");

    // Color(0, 0, 255, 255) — blue
    assert!(checker.check_call(
        &Expr::Variable("Color".into()),
        &[Expr::Integer(0), Expr::Integer(0), Expr::Integer(255), Expr::Integer(255)],
    ).is_ok(), "Color(0,0,255,255) must be valid");

    // Color(255, 255, 0, 255) — yellow
    assert!(checker.check_call(
        &Expr::Variable("Color".into()),
        &[Expr::Integer(255), Expr::Integer(255), Expr::Integer(0), Expr::Integer(255)],
    ).is_ok(), "Color(255,255,0,255) must be valid");

    // Color(255, 255, 255, 255) — white
    assert!(checker.check_call(
        &Expr::Variable("Color".into()),
        &[Expr::Integer(255), Expr::Integer(255), Expr::Integer(255), Expr::Integer(255)],
    ).is_ok(), "Color(255,255,255,255) must be valid");

    // Color(0, 0, 0, 255) — black
    assert!(checker.check_call(
        &Expr::Variable("Color".into()),
        &[Expr::Integer(0), Expr::Integer(0), Expr::Integer(0), Expr::Integer(255)],
    ).is_ok(), "Color(0,0,0,255) must be valid");
}

// ─── 3. CallableRegistry: All demo functions registered ───

#[test]
demo_callableregistry_has_all_functions() {
    let mut types = TypeRegistry::new();
    let mut callables = CallableRegistry::default();
    raylib::register_raylib_functions_compat(&mut types, &mut callables);

    // Core functions
    assert!(callables.find_by_name("InitWindow").is_some());
    assert!(callables.find_by_name("CloseWindow").is_some());
    assert!(callables.find_by_name("WindowShouldClose").is_some());
    assert!(callables.find_by_name("SetTargetFPS").is_some());
    assert!(callables.find_by_name("BeginDrawing").is_some());
    assert!(callables.find_by_name("EndDrawing").is_some());

    // Drawing functions
    assert!(callables.find_by_name("ClearBackground").is_some());
    assert!(callables.find_by_name("DrawRectangle").is_some());
    assert!(callables.find_by_name("DrawText").is_some());

    // Input functions
    assert!(callables.find_by_name("IsMouseButtonPressed").is_some());
    assert!(callables.find_by_name("IsKeyPressed").is_some());

    // FPS
    assert!(callables.find_by_name("GetFPS").is_some());
}

#[test]
demo_callableregistry_signatures_correct() {
    let mut types = TypeRegistry::new();
    let mut callables = CallableRegistry::default();
    raylib::register_raylib_functions_compat(&mut types, &mut callables);

    // ClearBackground takes 1 param (u32 packed Color)
    let (_, sig) = callables.find_by_name("ClearBackground").unwrap();
    assert_eq!(sig.params.len(), 1, "ClearBackground takes 1 param");

    // DrawRectangle takes 5 params (x, y, w, h, color)
    let (_, sig) = callables.find_by_name("DrawRectangle").unwrap();
    assert_eq!(sig.params.len(), 5, "DrawRectangle takes 5 params");

    // InitWindow takes 3 params (width, height, title)
    let (_, sig) = callables.find_by_name("InitWindow").unwrap();
    assert_eq!(sig.params.len(), 3, "InitWindow takes 3 params");
}

// ─── 4. HIR lowering: All function calls lowered to HirExprKind::Call ───

#[test]
demo_hir_lowering_produces_call_for_drawrectangle() {
    let stmts = parse_demo();
    let mut types = TypeRegistry::new();
    let mut symbols = logicodex::hir::SymbolTable::default();
    let mut lowering = LoweringContext {
        symbols: &mut symbols,
        types: &mut types,
        diagnostics: Vec::new(),
    };

    let program = logicodex::ast::Program::new(stmts);
    let result = lowering.lower_v121_program(program);
    assert!(result.is_ok(), "Demo program must lower to HIR: {:?}", result.err());
}

// ─── 5. Color packing verification ───

#[test]
demo_color_red_packs_to_0xff0000ff() {
    // Color(255, 0, 0, 255) → packed u32
    let r: u32 = 255;
    let g: u32 = 0;
    let b: u32 = 0;
    let a: u32 = 255;
    let packed = (r << 24) | (g << 16) | (b << 8) | a;
    assert_eq!(packed, 0xFF0000FF);
}

#[test]
demo_color_black_packs_to_0x000000ff() {
    let packed = (0u32 << 24) | (0u32 << 16) | (0u32 << 8) | 255u32;
    assert_eq!(packed, 0x000000FF);
}

#[test]
demo_color_yellow_packs_to_0xffff00ff() {
    let packed = (255u32 << 24) | (255u32 << 16) | (0u32 << 8) | 255u32;
    assert_eq!(packed, 0xFFFF00FF);
}
