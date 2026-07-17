# Logicodex Reflex Engine Examples

This note documents the refreshed `examples/` compatibility set for the current single HIR compiler path. The `v1.30` CLI value is the canonical compatibility selector, while `v1.21` remains a deprecated compatibility alias to the same engine. The hidden `v130-check` command is retained only as a subsystem self-check name, not as a second compiler pipeline.

| File | Style | Main capability demonstrated | Compatibility note |
|---|---|---|---|
| `examples/01_tambah_pakar.ldx` | Expert | Arithmetic bindings and conditional flow | Uses `{ }`, `let`, `print`, and `if/else`. |
| `examples/01_tambah_pemula.ldx` | Malay beginner | Arithmetic bindings and conditional flow | Uses `MULA/TAMAT`, `BINA`, `PAPAR`, and `JIKA/MAKA/JIKALAU_TIDAK`. |
| `examples/02_fungsi_matematik.ldx` | Mixed expert and Malay | Function declarations, typed parameters, typed returns, and local bindings | Avoids function calls because call expressions are not yet implemented in the current AST/parser subset. |
| `examples/03_gelung_kiraan.ldx` | Expert | `while`, `loop`, `break`, and `continue` forms | Uses no reassignment because mutable assignment is not part of the current statement set. |
| `examples/04_bitwise_operasi.ldx` | Expert | Bitwise `&`, `|`, `<<`, and `>>` signal operations | Keeps operands numeric and explicitly typed as `U32`. |
| `examples/05_zon_perkakasan_reflex.ldx` | Malay beginner | Hardware zone, declared hardware provenance, address literal, and `PTR<U16>` binding | Keeps address and pointer operations inside `ZON_PERKAKASAN`. |
| `examples/06_logik_bersyarat.ldx` | Malay beginner | Nested conditionals with `&&`, `||`, equality, and boolean aliases | Uses `BENAR` and `PALSU` for implemented boolean literals. |

These files deliberately avoid recognized-but-blocked roadmap constructs such as `struct`, `enum`, `unsafe`, `extern`/`ffi`, and `resource`. Those tokens are still valuable for dictionary completeness, but they are not executable source constructs in the current compiler path.

The expected local validation sequence is:

```bash
cargo fmt --all -- --check
cargo check --locked
cargo test --locked
for file in examples/*.ldx; do
  cargo run --quiet -- check "$file"
  cargo run --quiet -- v130-check "$file"
done
```
