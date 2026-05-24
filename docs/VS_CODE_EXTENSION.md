# Logicodex VS Code Side View

This document explains how to use **Logicodex Side View** for **current Logicodex v1.21-alpha**. The extension is built as an MVP without changing the Rust compiler. It provides syntax highlighting, snippets, and a right-side panel that displays a **best-effort expert canonical shorthand preview** based on `dict/core_map.json`.

> This extension is a **best-effort preview**, not a compiler-backed translator. It normalizes Malay pseudocode and English pseudocode aliases to expert canonical shorthand, while real validation must still be performed through the Logicodex compiler.

## Repository Location

The extension lives under `extensions/vscode-logicodex/` so it can be developed with the main repository without touching `src/*.rs`.

| Path | Purpose |
|---|---|
| `extensions/vscode-logicodex/src/extension.ts` | VS Code entry point, command registration, Webview side-view, and auto-refresh. |
| `extensions/vscode-logicodex/src/dictionary.ts` | Loads `core_map.json` from settings, workspace `dict/core_map.json`, or the bundled snapshot. |
| `extensions/vscode-logicodex/src/previewNormalizer.ts` | Token-level normalizer from Malay/English pseudocode aliases to expert canonical shorthand. |
| `extensions/vscode-logicodex/syntaxes/logicodex.tmLanguage.json` | Basic syntax highlighting for `.ldx`. |
| `extensions/vscode-logicodex/snippets/logicodex.code-snippets` | Snippets for programs, functions, if/else, loop forms, and hardware zones. |
| `extensions/vscode-logicodex/resources/core_map.json` | Fallback dictionary snapshot used when the extension runs outside the repository workspace. |
| `extensions/vscode-logicodex/dist/*.tar.gz` | Tarball package for manual transfer or local installation. |

## Running in Visual Studio Code

The easiest way to try the extension is through the **Extension Development Host**. This does not require publishing to the Marketplace and is appropriate for local development.

| Step | Instruction |
|---|---|
| 1 | Open VS Code at the repository folder: `code /path/to/logicodex`. |
| 2 | Open the VS Code terminal and enter `extensions/vscode-logicodex`. |
| 3 | Run `npm install` if dependencies are not installed yet. |
| 4 | Run `npm run compile`. |
| 5 | Press `F5` in VS Code to open the **Extension Development Host**. |
| 6 | In the new window, open an `.ldx` file, for example `extensions/vscode-logicodex/examples/side_view_demo.ldx`. |
| 7 | Open the command palette with `Ctrl+Shift+P` or `Cmd+Shift+P`, then choose **Logicodex: Open Expert Side View**. |

After the panel opens, edit the `.ldx` file on the left side. The preview updates automatically after a short debounce. If needed, use **Logicodex: Refresh Expert Side View** for a manual refresh.

## Using the Tarball Package

The tarball package is a lightweight development artifact. It is not the same as a `.vsix`, but it is useful for sending the extension to another developer or extracting it into any development folder.

```bash
cd extensions/vscode-logicodex
npm install
npm run package:tar
mkdir -p /tmp/logicodex-vscode-extension
tar -xzf dist/logicodex-side-view-0.1.0.tar.gz -C /tmp/logicodex-vscode-extension
code /tmp/logicodex-vscode-extension/logicodex-side-view-0.1.0
```

Inside the extracted folder, run `npm install` and `npm run compile` if you want to launch an Extension Development Host from that copy.

## Preview Behavior

The extension reads the dictionary and builds an alias-to-expert-canonical map. For example, `MULA` previews as `{`, `TAMAT` previews as `}`, `BINA` previews as `let`, `PAPAR` previews as `print`, `FUNGSI` previews as `fn`, `JIKA` previews as `if`, `MAKA` previews as `then`, `MELAINKAN` previews as `else`, `PULANG` previews as `return`, `SELAGI` previews as `while`, `ULANG` previews as `loop`, `HENTI` previews as `break`, and `LANGKAU` previews as `continue`.

| Input pseudocode | Expert preview |
|---|---|
| `MULA` | `{` |
| `BINA seed: I64 = 21;` | `let seed: I64 = 21;` |
| `PAPAR seed;` | `print seed;` |
| `FUNGSI clamp(value: I64) -> I64 MULA` | `fn clamp(value: I64) -> I64 {` |
| `SELAGI seed < 30 MULA` | `while seed < 30 {` |
| `HENTI;` | `break;` |
| `LANGKAU;` | `continue;` |
| `PULANG value;` | `return value;` |
| `TAMAT` | `}` |

The normalizer avoids replacements inside string literals and comments so documentation text inside code is not aggressively changed. It can also add preview semicolons for newline-terminated Malay statement aliases such as `BINA`, `PAPAR`, and `PULANG`, following the legacy `logicodexSideView.insertMissingBeginnerSemicolons` setting.

## Settings

| Setting | Default | Purpose |
|---|---:|---|
| `logicodexSideView.debounceMs` | `300` | Wait time before the preview refreshes after edits. |
| `logicodexSideView.showLineNumbers` | `true` | Shows line numbers in the side-view. |
| `logicodexSideView.insertMissingBeginnerSemicolons` | `true` | Adds preview semicolons for suitable Malay statement aliases; the setting name is retained for MVP compatibility. |
| `logicodexSideView.dictionaryPath` | `""` | Custom path to `core_map.json`; when empty, the extension tries workspace `dict/core_map.json` and then the bundled fallback. |

Example `.vscode/settings.json`:

```json
{
  "logicodexSideView.dictionaryPath": "dict/core_map.json",
  "logicodexSideView.debounceMs": 200,
  "logicodexSideView.showLineNumbers": true,
  "logicodexSideView.insertMissingBeginnerSemicolons": true
}
```

## Deliberate MVP Limits

The extension does not implement the Logicodex parser in TypeScript because the real parser is in Rust. Therefore, it does not perform type checking, semantic checking, provenance checking, hardware-zone validation, or code generation. Its responsibility is UX: helping users quickly see the expert canonical shorthand that corresponds to Malay or English pseudocode aliases.

| Item | MVP status |
|---|---|
| Side-view expert canonical shorthand | Supported on a best-effort basis. |
| `.ldx` syntax highlighting | Supported. |
| Snippets | Supported. |
| Compiler-backed translation | Not supported yet because translation must come from the Rust compiler. |
| Live semantic diagnostics | Not supported yet. |
| `.vsix` Marketplace package | Not prepared yet; the current artifact is a tarball. |

## Real Validation

For real validation, use the Logicodex compiler from the repository root. Example:

```bash
cargo run --quiet -- check extensions/vscode-logicodex/examples/side_view_demo.ldx
```

If the extension should call the compiler automatically in the future, the recommended design is to add a separate command that runs `logicodex check` against the active file. That can still be done without changing Rust if the current subcommand is sufficient, but official translation should eventually come from a Rust compiler subcommand such as `translate --stdin --to expert`.

## Technical References

VS Code provides a Webview API for building custom views inside the editor, and this extension uses it with scripts disabled and a simple Content Security Policy so the panel renders static HTML only. The official VS Code documentation also recommends contribution points such as languages, grammars, snippets, commands, and configuration for typical editor integrations.

| Reference | Link |
|---|---|
| VS Code Webview API | <https://code.visualstudio.com/api/extension-guides/webview> |
| VS Code Contribution Points | <https://code.visualstudio.com/api/references/contribution-points> |
| VS Code Language Extensions | <https://code.visualstudio.com/api/language-extensions/overview> |
