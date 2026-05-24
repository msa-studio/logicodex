#!/usr/bin/env python3
"""
Systematic Malay → English syntax rename for international acceptance.

Mappings:
  kotak      → actor
  pintu      → channel
  lahirkan   → spawn  (already English in AST)
  hantar     → send
  terima     → recv
  tunggu     → join

Internal field names also renamed:
  kotak_registry    → actor_registry
  pintu_registry    → channel_registry
  moved_via_pintu   → moved_via_channel
  pintu_name        → channel_name
  kotak_name        → actor_name
"""

from pathlib import Path
import sys

root = Path(__file__).resolve().parents[1]

# ============================================================
# Phase 1: TokenKind renames in lexer.rs
# ============================================================
lexer_replacements = [
    # TokenKind enum variants
    ('Kotak,', 'Actor,'),
    ('Pintu,', 'Channel,'),
    ('Lahirkan,', 'Spawn,'),
    ('Hantar,', 'Send,'),
    ('Terima,', 'Recv,'),
    ('Tunggu,', 'Join,'),
    # TokenKind:: references
    ('TokenKind::Kotak', 'TokenKind::Actor'),
    ('TokenKind::Pintu', 'TokenKind::Channel'),
    ('TokenKind::Lahirkan', 'TokenKind::Spawn'),
    ('TokenKind::Hantar', 'TokenKind::Send'),
    ('TokenKind::Terima', 'TokenKind::Recv'),
    ('TokenKind::Tunggu', 'TokenKind::Join'),
    # TryFrom identity strings
    ('"KOTAK"', '"ACTOR"'),
    ('"PINTU"', '"CHANNEL"'),
    ('"LAHIRKAN"', '"SPAWN"'),
    ('"HANTAR"', '"SEND"'),
    ('"TERIMA"', '"RECV"'),
    ('"TUNGGU"', '"JOIN"'),
    # default_aliases lexemes
    ('("KOTAK", TokenKind::Actor)', '("ACTOR", TokenKind::Actor)'),
    ('("Kotak", TokenKind::Actor)', '("Actor", TokenKind::Actor)'),
    ('("kotak", TokenKind::Actor)', '("actor", TokenKind::Actor)'),
    ('("PINTU", TokenKind::Channel)', '("CHANNEL", TokenKind::Channel)'),
    ('("Pintu", TokenKind::Channel)', '("Channel", TokenKind::Channel)'),
    ('("pintu", TokenKind::Channel)', '("channel", TokenKind::Channel)'),
    ('("LAHIRKAN", TokenKind::Spawn)', '("SPAWN", TokenKind::Spawn)'),
    ('("Lahirkan", TokenKind::Spawn)', '("Spawn", TokenKind::Spawn)'),
    ('("lahirkan", TokenKind::Spawn)', '("spawn", TokenKind::Spawn)'),
    ('("HANTAR", TokenKind::Send)', '("SEND", TokenKind::Send)'),
    ('("Hantar", TokenKind::Send)', '("Send", TokenKind::Send)'),
    ('("hantar", TokenKind::Send)', '("send", TokenKind::Send)'),
    ('("TERIMA", TokenKind::Recv)', '("RECV", TokenKind::Recv)'),
    ('("Terima", TokenKind::Recv)', '("Recv", TokenKind::Recv)'),
    ('("terima", TokenKind::Recv)', '("recv", TokenKind::Recv)'),
    ('("TUNGGU", TokenKind::Join)', '("JOIN", TokenKind::Join)'),
    ('("Tunggu", TokenKind::Join)', '("Join", TokenKind::Join)'),
    ('("tunggu", TokenKind::Join)', '("join", TokenKind::Join)'),
]

# ============================================================
# Phase 2: AST renames in ast.rs
# ============================================================
ast_replacements = [
    # Stmt variants
    ('Kotak {', 'Actor {'),
    # Expr variants
    ('Hantar {', 'Send {'),
    ('Terima {', 'Recv {'),
    ('Tunggu {', 'Join {'),
    # Field names
    ('pintu_name:', 'channel_name:'),
    ('kotak_name:', 'actor_name:'),
    # Type variant
    ('Pintu {', 'Channel {'),
    # Method names
    ('is_pintu', 'is_channel'),
    ('pintu_capability', 'channel_capability'),
    # Display
    ('"Pintu<', '"Channel<'),
    # Comments
    ('/// v1.30.1-alpha: Kotak', '/// v1.30.1-alpha: Actor'),
    ('/// Syntax: `kotak', '/// Syntax: `actor'),
    ('/// v1.30.1-alpha: Send value through Pintu', '/// v1.30.1-alpha: Send value through Channel'),
    ('/// Syntax: `pintu_data.hantar', '/// Syntax: `channel_data.send'),
    ('/// v1.30.1-alpha: Receive value from Pintu', '/// v1.30.1-alpha: Receive value from Channel'),
    ('/// Syntax: `pintu_data.terima', '/// Syntax: `channel_data.recv'),
    ('/// v1.30.1-alpha: Wait for Kotak', '/// v1.30.1-alpha: Wait for Actor'),
    ('/// Syntax: `tunggu', '/// Syntax: `join'),
    ('/// Pintu<T, U> — SPSC channel', '/// Channel<T, U> — SPSC channel'),
    ('/// Syntax: Pintu<', '/// Syntax: Channel<'),
]

# ============================================================
# Phase 3: Semantic renames
# ============================================================
semantic_replacements = [
    # Field names
    ('kotak_registry', 'actor_registry'),
    ('pintu_registry', 'channel_registry'),
    ('moved_via_pintu', 'moved_via_channel'),
    # Error references
    ('KotakNotFound', 'ActorNotFound'),
    ('InvalidPintuTopology', 'InvalidChannelTopology'),
    ('DuplicateKotak', 'DuplicateActor'),
    ('SpawnNonKotak', 'SpawnNonActor'),
    ('UseAfterHantar', 'UseAfterSend'),
    # Error messages - Malay part
    ('Kotak `{name}` tidak wujud', 'Actor `{name}` does not exist'),
    ('Pintu dari `{from}` ke `{to}` tidak sah', 'Channel from `{from}` to `{to}` is invalid'),
    ('Kotak `{name}` sudah diisytiharkan', 'Actor `{name}` is already declared'),
    ('`lahirkan` hanya boleh digunakan dengan nama Kotak', '`spawn` can only be used with an Actor name'),
    ('Pembolehubah `{name` sudah dihantar melalui Pintu', 'Variable `{name}` has already been sent through Channel'),
    # Error messages - English part
    ('Kotak `{name}` does not exist', 'Actor `{name}` does not exist'),
    ('Pintu from `{from}` to `{to}` is invalid', 'Channel from `{from}` to `{to}` is invalid'),
    ('Kotak `{name}` is already declared', 'Actor `{name}` is already declared'),
    ('`lahirkan` can only be used with a Kotak name', '`spawn` can only be used with an Actor name'),
    ('has already been sent through Pintu', 'has already been sent through Channel'),
    # Variable use in expressions
    ('pintu_name', 'channel_name'),
    ('kotak_name', 'actor_name'),
    # Comments
    ('v1.30.1-alpha: Threading Foundation — Kotak & Pintu', 'v1.30.1-alpha: Threading Foundation — Actor & Channel'),
    ('Registered Kotak names', 'Registered Actor names'),
    ('Registered Pintu', 'Registered Channel'),
    ('Variables moved via Pintu', 'Variables moved via Channel'),
    ('v1.30.1-alpha Fasa 2: Zero-Copy Ownership Transfer', 'v1.30.1-alpha Phase 2: Zero-Copy Ownership Transfer'),
    ('// Fasa 2: Check', '// Phase 2: Check'),
    ('// Fasa 2: Zero-Copy', '// Phase 2: Zero-Copy'),
    ('Kotak { name, body } =>', 'Actor { name, body } =>'),
    ('Stmt::Kotak', 'Stmt::Actor'),
    ('Type::Pintu', 'Type::Channel'),
    ('kotak_registry.contains', 'actor_registry.contains'),
    ('kotak_registry.insert', 'actor_registry.insert'),
    ('pintu_registry.iter', 'channel_registry.iter'),
    ('pintu_registry.push', 'channel_registry.push'),
    ('moved_via_pintu.contains', 'moved_via_channel.contains'),
    ('moved_via_pintu.insert', 'moved_via_channel.insert'),
]

# ============================================================
# Phase 4: Codegen renames
# ============================================================
codegen_replacements = [
    ('Expr::Hantar {', 'Expr::Send {'),
    ('Expr::Terima {', 'Expr::Recv {'),
    ('Expr::Tunggu {', 'Expr::Join {'),
    ('Expr::Spawn {', 'Expr::Spawn {'),
    ('pintu_name', 'channel_name'),
    ('kotak_name', 'actor_name'),
    ('hantar', 'send'),
    ('terima', 'recv'),
    ('tunggu', 'join'),
    ('Fasa 2', 'Phase 2'),
    ('ownership transferred (Release)', 'ownership transferred (Release)'),
    ('ownership acquired (Acquire)', 'ownership acquired (Acquire)'),
    ('// v1.30.1-alpha Fasa 2', '// v1.30.1-alpha Phase 2'),
    ('// v1.30.1-alpha: Threading', '// v1.30.1-alpha: Threading'),
]

# ============================================================
# Phase 5: Parser renames
# ============================================================
parser_replacements = [
    ('fn kotak_statement', 'fn actor_statement'),
    ('Stmt::Kotak {', 'Stmt::Actor {'),
    ('Kotak { name,', 'Actor { name,'),
    ('Hantar { pintu_name', 'Send { channel_name'),
    ('Terima { pintu_name', 'Recv { channel_name'),
    ('Tunggu { kotak_name', 'Join { actor_name'),
    ('Spawn { kotak_name', 'Spawn { actor_name'),
    ('pintu_name', 'channel_name'),
    ('kotak_name', 'actor_name'),
    # Check keyword strings
    ('"kotak"', '"actor"'),
    ('"lahirkan"', '"spawn"'),
    ('"hantar"', '"send"'),
    ('"terima"', '"recv"'),
    ('"tunggu"', '"join"'),
]


def apply_replacements(path, replacements, description):
    """Apply a list of (old, new) replacements to a file."""
    if not path.exists():
        print(f"  SKIP: {path} not found")
        return 0
    text = path.read_text(encoding='utf-8')
    original = text
    count = 0
    for old, new in replacements:
        if old in text:
            n = text.count(old)
            text = text.replace(old, new)
            count += n
    if text != original:
        path.write_text(text, encoding='utf-8')
        print(f"  {description}: {count} replacement(s)")
        return count
    print(f"  {description}: no changes needed")
    return 0


def apply_all_phases(path, all_replacements, description):
    """Apply all replacement phases to a file."""
    if not path.exists():
        print(f"  SKIP: {path} not found")
        return
    text = path.read_text(encoding='utf-8')
    original = text
    total = 0
    for phase_name, replacements in all_replacements:
        for old, new in replacements:
            if old in text:
                n = text.count(old)
                text = text.replace(old, new)
                total += n
    if text != original:
        path.write_text(text, encoding='utf-8')
    print(f"  {description}: {total} replacement(s)")


print("=" * 60)
print("Malay → English Syntax Rename")
print("=" * 60)

total = 0

# 1. lexer.rs
n = apply_replacements(root / "src" / "lexer.rs", lexer_replacements, "src/lexer.rs")
total += n

# 2. ast.rs
n = apply_replacements(root / "src" / "ast.rs", ast_replacements, "src/ast.rs")
total += n

# 3. parser.rs — needs lexer + ast + parser replacements
parser_all = [
    ("lexer", lexer_replacements),
    ("ast", ast_replacements),
    ("parser", parser_replacements),
]
apply_all_phases(root / "src" / "parser.rs", parser_all, "src/parser.rs")

# 4. semantic.rs — needs lexer + ast + semantic replacements
semantic_all = [
    ("lexer", lexer_replacements),
    ("ast", ast_replacements),
    ("semantic", semantic_replacements),
]
apply_all_phases(root / "src" / "semantic.rs", semantic_all, "src/semantic.rs")

# 5. codegen.rs — needs lexer + ast + codegen replacements
codegen_all = [
    ("lexer", lexer_replacements),
    ("ast", ast_replacements),
    ("codegen", codegen_replacements),
]
apply_all_phases(root / "src" / "codegen.rs", codegen_all, "src/codegen.rs")

# 6. Test files
test_replacements = [
    # Source strings (Malay keywords in test source)
    ('kotak ', 'actor '),
    ('Pintu<', 'Channel<'),
    ('pintu_', 'channel_'),
    ('pintu.', 'channel.'),
    ('lahirkan ', 'spawn '),
    ('hantar', 'send'),
    ('terima', 'recv'),
    ('tunggu ', 'join '),
    # AST references
    ('Stmt::Kotak', 'Stmt::Actor'),
    ('Expr::Hantar', 'Expr::Send'),
    ('Expr::Terima', 'Expr::Recv'),
    ('Expr::Tunggu', 'Expr::Join'),
    ('Type::Pintu', 'Type::Channel'),
    # Field names
    ('pintu_name', 'channel_name'),
    ('kotak_name', 'actor_name'),
    # Method names
    ('is_pintu', 'is_channel'),
    ('pintu_capability', 'channel_capability'),
    # Comments
    ('Kotak', 'Actor'),
    ('Pintu', 'Channel'),
    ('kotak', 'actor'),
    ('pintu', 'channel'),
    ('hantar', 'send'),
    ('terima', 'recv'),
    ('tunggu', 'join'),
    ('lahirkan', 'spawn'),
]

for test_file in ["threading_foundation.rs", "threading_fasa2.rs"]:
    n = apply_replacements(root / "tests" / test_file, test_replacements, f"tests/{test_file}")
    total += n

# 7. Library files
lib_replacements = [
    ('ring_hantar', 'ring_send'),
    ('ring_terima', 'ring_recv'),
    ('ring_kosong', 'ring_empty'),
    ('ring_penuh', 'ring_full'),
    ('ring_saiz', 'ring_size'),
    ('penimbal', 'buffer'),
    ('kapasiti', 'capacity'),
    ('kepala', 'head'),
    ('ekor', 'tail'),
    ('ring_baru', 'ring_new'),
    ('hantar', 'send'),
    ('terima', 'recv'),
    ('kosong', 'empty'),
    ('penuh', 'full'),
    ('saiz', 'size'),
    ('kotak', 'actor'),
    ('pintu', 'channel'),
    ('Kotak', 'Actor'),
    ('Pintu', 'Channel'),
    ('Hantar', 'Send'),
    ('Terima', 'Recv'),
    ('lahirkan', 'spawn'),
    ('Lahirkan', 'Spawn'),
    ('tunggu', 'join'),
    ('Tunggu', 'Join'),
]

n = apply_replacements(root / "lib" / "core" / "ring_buffer.ldx", lib_replacements, "lib/core/ring_buffer.ldx")
total += n

# Thread and sync .ldx files
for lib_file in ["thread.ldx", "sync.ldx"]:
    n = apply_replacements(root / "lib" / "core" / lib_file, lib_replacements, f"lib/core/{lib_file}")
    total += n

# 8. Validators
validator_replacements = [
    ('Kotak {', 'Actor {'),
    ('"Kotak"', '"Actor"'),
    ('"Pintu"', '"Channel"'),
    ('Pintu {', 'Channel {'),
    ('Expr::Hantar', 'Expr::Send'),
    ('Expr::Terima', 'Expr::Recv'),
    ('Expr::Tunggu', 'Expr::Join'),
    ('Stmt::Kotak', 'Stmt::Actor'),
    ('KotakNotFound', 'ActorNotFound'),
    ('DuplicateKotak', 'DuplicateActor'),
    ('InvalidPintuTopology', 'InvalidChannelTopology'),
    ('SpawnNonKotak', 'SpawnNonActor'),
    ('is_pintu', 'is_channel'),
    ('pintu_capability', 'channel_capability'),
    ('pintu_name', 'channel_name'),
    ('kotak_name', 'actor_name'),
    ('kotak', 'actor'),
    ('pintu', 'channel'),
    ('hantar', 'send'),
    ('terima', 'recv'),
    ('tunggu', 'join'),
    ('lahirkan', 'spawn'),
]

for val_file in ["validate_threading_foundation.py", "validate_threading_fasa2.py"]:
    n = apply_replacements(root / "scripts" / val_file, validator_replacements, f"scripts/{val_file}")
    total += n

# Also update other validators that reference threading
cross_validator_replacements = [
    ('KotakNotFound', 'ActorNotFound'),
    ('DuplicateKotak', 'DuplicateActor'),
    ('InvalidPintuTopology', 'InvalidChannelTopology'),
    ('SpawnNonKotak', 'SpawnNonActor'),
    ('UseAfterHantar', 'UseAfterSend'),
    ('moved_via_pintu', 'moved_via_channel'),
]

for val_file in [
    "validate_v121_executable_logic.py",
    "validate_v130_pipeline.py",
    "validate_sprint1_type_registry.py",
    "validate_sprint1_2_parser_types.py",
    "validate_sprint2_layout_engine.py",
    "validate_sprint2_5_struct_literals.py",
    "validate_sprint3_codegen_calls.py",
    "validate_demo_raylib_box.py",
    "validate_audio_engine.py",
    "validate_core_memory.py",
    "validate_buffer_bugfixes.py",
    "validate_result_abstraction.py",
    "validate_io_file_syscall.py",
]:
    apply_replacements(root / "scripts" / val_file, cross_validator_replacements, f"scripts/{val_file}")

print("\n" + "=" * 60)
print("Rename complete. Run validators to verify.")
print("=" * 60)
