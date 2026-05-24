# Logicodex Side View

**Logicodex Side View** is a VS Code extension MVP for **current Logicodex v1.21-alpha**. It provides `.ldx` syntax highlighting, snippets, and a right-side panel that displays a **best-effort expert canonical shorthand preview** based on `core_map.json`.

> This preview is not the official compiler translator. It only performs token-level normalization based on the dictionary. Use the Logicodex compiler for real validation.

## Run in VS Code

Open the Logicodex repository in VS Code, then run the following commands:

```bash
cd extensions/vscode-logicodex
npm install
npm run compile
```

After that, press `F5` in VS Code to open the **Extension Development Host**. In the new window, open an `.ldx` file, then run the **Logicodex: Open Expert Side View** command from the command palette.

## Commands

| Command | Purpose |
|---|---|
| `Logicodex: Open Expert Side View` | Opens the expert canonical shorthand preview panel beside the active editor. |
| `Logicodex: Refresh Expert Side View` | Manually refreshes the preview panel. |

## Settings

| Setting | Default | Purpose |
|---|---:|---|
| `logicodexSideView.debounceMs` | `300` | Refresh delay after edits. |
| `logicodexSideView.showLineNumbers` | `true` | Shows preview line numbers. |
| `logicodexSideView.insertMissingBeginnerSemicolons` | `true` | Adds preview semicolons for suitable Malay statement aliases; the setting name is retained for MVP compatibility. |
| `logicodexSideView.dictionaryPath` | `""` | Custom path to `core_map.json`. |

## Tarball Package

To build the development tarball package:

```bash
npm run package:tar
```

The file is written to `dist/logicodex-side-view-0.1.0.tar.gz`.

## MVP Scope

The extension deliberately avoids changing Rust and does not duplicate the compiler parser. It is suitable for UX demonstrations, Malay/English pseudocode alias onboarding, and side-by-side teaching between Malay/English pseudocode and expert canonical shorthand. Official translation should eventually be provided by a compiler subcommand such as `translate --stdin --to expert`.
