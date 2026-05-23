# Logicodex VS Code Side View

Dokumen ini menerangkan cara menggunakan **Logicodex Side View** untuk **current logicodex v 1.21 alpha**. Extension ini dibina sebagai MVP tanpa mengubah kod Rust. Ia menyediakan syntax highlighting, snippets, dan panel sebelah kanan yang memaparkan bentuk **canonical expert preview** berdasarkan `dict/core_map.json`.

> Extension ini ialah **best-effort preview**, bukan compiler-backed translator. Ia melakukan normalisasi token daripada pseudo Melayu atau pseudo English kepada expert canonical, tetapi validasi sebenar masih perlu dibuat melalui compiler Logicodex.

## Kedudukan Dalam Repository

Extension diletakkan di bawah `extensions/vscode-logicodex/` supaya ia boleh dibangunkan bersama repository utama tanpa menyentuh `src/*.rs`.

| Path | Tujuan |
|---|---|
| `extensions/vscode-logicodex/src/extension.ts` | Entry point VS Code, command registration, Webview side-view, dan auto-refresh. |
| `extensions/vscode-logicodex/src/dictionary.ts` | Loader `core_map.json` daripada setting, workspace `dict/core_map.json`, atau snapshot bundled. |
| `extensions/vscode-logicodex/src/previewNormalizer.ts` | Normalizer token-level pseudo Melayu/English kepada expert canonical. |
| `extensions/vscode-logicodex/syntaxes/logicodex.tmLanguage.json` | Syntax highlighting asas untuk `.ldx`. |
| `extensions/vscode-logicodex/snippets/logicodex.code-snippets` | Snippets untuk program, fungsi, if/else, dan hardware zone. |
| `extensions/vscode-logicodex/resources/core_map.json` | Snapshot fallback dictionary semasa extension dipasang di luar workspace repo. |
| `extensions/vscode-logicodex/dist/*.tar.gz` | Pakej tar.gz untuk dipindahkan atau dipasang secara manual. |

## Cara Apply Di Visual Studio Code

Kaedah paling mudah untuk mencuba extension ini ialah menggunakan **Extension Development Host**. Ini tidak memerlukan publish ke Marketplace dan sesuai untuk development tempatan.

| Langkah | Arahan |
|---|---|
| 1 | Buka VS Code pada folder repository: `code /path/to/logicodex`. |
| 2 | Buka terminal VS Code dan masuk ke `extensions/vscode-logicodex`. |
| 3 | Jalankan `npm install` jika belum dibuat. |
| 4 | Jalankan `npm run compile`. |
| 5 | Tekan `F5` dalam VS Code untuk membuka **Extension Development Host**. |
| 6 | Dalam window baharu itu, buka fail `.ldx`, contohnya `extensions/vscode-logicodex/examples/side_view_demo.ldx`. |
| 7 | Jalankan command palette `Ctrl+Shift+P` atau `Cmd+Shift+P`, kemudian pilih **Logicodex: Open Expert Side View**. |

Selepas panel dibuka, edit fail `.ldx` di sebelah kiri. Preview akan dikemas kini secara automatik selepas debounce ringkas. Jika perlu, command **Logicodex: Refresh Expert Side View** boleh digunakan untuk refresh manual.

## Cara Guna Dari Pakej Tar.gz

Pakej tar.gz boleh digunakan sebagai artefak pembangunan ringkas. Ia tidak sama seperti `.vsix`, tetapi berguna untuk dihantar kepada delegate atau diekstrak ke mana-mana folder pembangunan.

```bash
cd extensions/vscode-logicodex
npm install
npm run package:tar
mkdir -p /tmp/logicodex-vscode-extension
tar -xzf dist/logicodex-side-view-0.1.0.tar.gz -C /tmp/logicodex-vscode-extension
code /tmp/logicodex-vscode-extension/logicodex-side-view-0.1.0
```

Dalam folder hasil extract, jalankan `npm install` dan `npm run compile` jika mahu menjalankan Extension Development Host daripada salinan tersebut.

## Behavior Preview

Extension membaca dictionary dan membina peta alias kepada expert canonical. Contohnya, `MULA` akan dipaparkan sebagai `{`, `TAMAT` sebagai `}`, `BINA` sebagai `let`, `PAPAR` sebagai `print`, `FUNGSI` sebagai `fn`, `JIKA` sebagai `if`, `MAKA` sebagai `then`, `MELAINKAN` sebagai `else`, dan `PULANG` sebagai `return`.

| Input pseudo | Preview expert |
|---|---|
| `MULA` | `{` |
| `BINA seed: I64 = 21;` | `let seed: I64 = 21;` |
| `PAPAR seed;` | `print seed;` |
| `FUNGSI clamp(value: I64) -> I64 MULA` | `fn clamp(value: I64) -> I64 {` |
| `PULANG value;` | `return value;` |
| `TAMAT` | `}` |

Normalizer mengelak replacement di dalam string literal dan comment supaya teks dokumentasi dalam kod tidak diubah secara agresif. Ia juga boleh menambah semicolon preview untuk statement beginner newline-terminated seperti `BINA`, `PAPAR`, dan `PULANG`, mengikut setting `logicodexSideView.insertMissingBeginnerSemicolons`.

## Settings

| Setting | Default | Kegunaan |
|---|---:|---|
| `logicodexSideView.debounceMs` | `300` | Masa menunggu sebelum preview refresh selepas edit. |
| `logicodexSideView.showLineNumbers` | `true` | Paparkan nombor baris dalam side-view. |
| `logicodexSideView.insertMissingBeginnerSemicolons` | `true` | Tambah semicolon preview untuk statement beginner yang sesuai. |
| `logicodexSideView.dictionaryPath` | `""` | Path custom kepada `core_map.json`; jika kosong, extension cuba workspace `dict/core_map.json`, kemudian bundled fallback. |

Contoh `.vscode/settings.json`:

```json
{
  "logicodexSideView.dictionaryPath": "dict/core_map.json",
  "logicodexSideView.debounceMs": 200,
  "logicodexSideView.showLineNumbers": true,
  "logicodexSideView.insertMissingBeginnerSemicolons": true
}
```

## Batasan Yang Sengaja Dikekalkan

Extension ini tidak melaksanakan parser Logicodex dalam TypeScript kerana parser sebenar berada dalam Rust. Oleh itu, ia tidak melakukan type checking, semantic checking, provenance checking, hardware-zone validation, atau code generation. Tanggungjawab extension ini ialah UX: membantu pengguna melihat padanan canonical expert dengan cepat semasa menulis pseudo Melayu atau pseudo English.

| Perkara | Status MVP |
|---|---|
| Side-view canonical expert | Disokong secara best-effort. |
| Syntax highlighting `.ldx` | Disokong. |
| Snippets | Disokong. |
| Compiler-backed translate | Belum disokong kerana Rust tidak disentuh. |
| Semantic diagnostics live | Belum disokong. |
| `.vsix` Marketplace package | Belum disediakan; artefak semasa ialah tar.gz. |

## Validasi Sebenar

Untuk validasi sebenar, gunakan compiler Logicodex daripada root repository. Contoh:

```bash
cargo run --quiet -- check extensions/vscode-logicodex/examples/side_view_demo.ldx
```

Jika mahu extension memanggil compiler secara automatik pada masa hadapan, reka bentuk yang disyorkan ialah menambah command berasingan yang menjalankan `logicodex check` terhadap fail aktif. Itu masih boleh dibuat tanpa mengubah Rust jika subcommand sedia ada mencukupi, tetapi untuk translation rasmi, subcommand Rust seperti `translate --stdin --to expert` tetap diperlukan pada fasa berikutnya.

## Rujukan Teknikal

VS Code menyediakan Webview API untuk membina view custom di dalam editor, dan extension ini menggunakannya dengan script disabled serta Content Security Policy ringkas supaya panel hanya memaparkan HTML statik. Dokumentasi rasmi VS Code juga menyarankan contribution points seperti languages, grammars, snippets, commands, dan configuration untuk integrasi editor yang biasa digunakan dalam extension.

| Rujukan | Pautan |
|---|---|
| VS Code Webview API | <https://code.visualstudio.com/api/extension-guides/webview> |
| VS Code Contribution Points | <https://code.visualstudio.com/api/references/contribution-points> |
| VS Code Language Extensions | <https://code.visualstudio.com/api/language-extensions/overview> |
