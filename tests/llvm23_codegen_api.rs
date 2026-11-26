// =========================================================================
// LLVM 23 / Inkwell 0.11 codegen API compatibility regression tests.
//
// Inkwell 0.11 added `BasicTypeEnum::ScalableVectorType` and made
// `custom_width_int_type` fallible. Logicodex has no scalable-vector
// semantics, so both compatibility paths must fail closed with
// deterministic diagnostics while supported lowering stays unchanged.
// =========================================================================

use inkwell::context::Context;
use inkwell::types::{BasicType, BasicTypeEnum};
use logicodex::codegen::{basic_type_fn_type, wrap_width_int_type};

const SCALABLE_VECTOR_DIAGNOSTIC: &str =
    "unsupported codegen path: scalable vector types are not supported";

#[test]
fn scalable_vector_fn_type_fails_closed() {
    let context = Context::create();
    let module = context.create_module("scalable_vector_fail_closed");
    let functions_before = module.get_functions().count();
    let scalable: BasicTypeEnum = context.i64_type().scalable_vec_type(2).into();
    let params = [context.i64_type().as_basic_type_enum()];

    let first = basic_type_fn_type(scalable, &params, false)
        .expect_err("scalable vector return type must be rejected");
    let second = basic_type_fn_type(scalable, &params, false)
        .expect_err("scalable vector return type must be rejected");

    assert_eq!(first.to_string(), SCALABLE_VECTOR_DIAGNOSTIC);
    assert_eq!(second.to_string(), SCALABLE_VECTOR_DIAGNOSTIC);
    assert_eq!(module.get_functions().count(), functions_before);
}

#[test]
fn supported_fn_return_types_still_lower() {
    let context = Context::create();
    let params = [context.i64_type().as_basic_type_enum()];
    let supported: [BasicTypeEnum; 3] = [
        context.i64_type().into(),
        context.f64_type().into(),
        context.i64_type().vec_type(2).into(),
    ];
    for ret in supported {
        let fn_type =
            basic_type_fn_type(ret, &params, false).expect("supported return type must lower");
        assert_eq!(fn_type.get_return_type(), Some(ret));
        assert_eq!(fn_type.get_param_types().len(), params.len());
        assert!(!fn_type.is_var_arg());
    }
}

#[test]
fn invalid_width_fails_closed() {
    let context = Context::create();

    for bits in [1u32, 8, 16, 32] {
        let ty = wrap_width_int_type(&context, bits).expect("supported width must lower");
        assert_eq!(ty.get_bit_width(), bits);
    }

    let zero = wrap_width_int_type(&context, 0).expect_err("zero width must be rejected");
    assert_eq!(
        zero.to_string(),
        "wrap_to_width: invalid integer bit width 0"
    );

    // LLVM caps integer widths at 2^23 bits.
    let over_limit = 8_388_609u32;
    let first =
        wrap_width_int_type(&context, over_limit).expect_err("over-limit width must be rejected");
    let second =
        wrap_width_int_type(&context, over_limit).expect_err("over-limit width must be rejected");
    let expected = format!(
        "wrap_to_width: invalid integer bit width {}: LLVM only supports integers with bit widths between 1 and 8388608 (inclusive)",
        over_limit
    );
    assert_eq!(first.to_string(), expected);
    assert_eq!(second.to_string(), expected);
}
