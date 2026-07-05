# P0 Type Checker Hardening

Status: active P0 branch.

## P0-TC-A local function call validation

This branch hardens local function calls in `src/semantic.rs`.

Implemented:

- undefined local function calls are rejected
- local function argument count is validated
- local function argument types are validated
- local function call expressions now return the declared function return type
  instead of the previous default `I64`

Regression tests:

- `local_function_call_argument_count_mismatch_fails`
- `local_function_call_argument_type_mismatch_fails`
- `local_function_call_return_type_flows_to_declared_assignment`
- `local_function_call_with_matching_args_still_passes`

Deferred:

- extern/FFI callable registry typing
- builtin call policy cleanup
- generic/unknown call expression handling outside simple local function names
- richer structured diagnostic codes for semantic errors
