# Logicodex Side View

**Logicodex Side View** ialah VS Code extension MVP untuk **current logicodex v 1.21 alpha**. Ia menyediakan syntax highlighting `.ldx`, snippets, dan panel sebelah kanan yang memaparkan **best-effort canonical expert preview** berdasarkan `core_map.json`.

> Preview ini bukan translator rasmi compiler. Ia hanya normalisasi token-level berdasarkan dictionary. Gunakan compiler Logicodex untuk validasi sebenar.

## Run Dalam VS Code

Buka repository Logicodex di VS Code, kemudian jalankan arahan berikut:

```bash
cd extensions/vscode-logicodex
npm install
npm run compile
```

Selepas itu tekan `F5` dalam VS Code untuk membuka **Extension Development Host**. Dalam window baharu, buka fail `.ldx`, kemudian jalankan command palette **Logicodex: Open Expert Side View**.

## Command

| Command | Tujuan |
|---|---|
| `Logicodex: Open Expert Side View` | Buka panel preview canonical expert di sebelah editor aktif. |
| `Logicodex: Refresh Expert Side View` | Refresh panel preview secara manual. |

## Settings

| Setting | Default | Kegunaan |
|---|---:|---|
| `logicodexSideView.debounceMs` | `300` | Delay refresh selepas edit. |
| `logicodexSideView.showLineNumbers` | `true` | Papar nombor baris preview. |
| `logicodexSideView.insertMissingBeginnerSemicolons` | `true` | Tambah semicolon preview untuk statement beginner yang sesuai. |
| `logicodexSideView.dictionaryPath` | `""` | Path custom kepada `core_map.json`. |

## Package Tar.gz

Untuk membina pakej tar.gz pembangunan:

```bash
npm run package:tar
```

Fail akan diletakkan di `dist/logicodex-side-view-0.1.0.tar.gz`.

## Scope MVP

Extension ini sengaja tidak menyentuh Rust dan tidak meniru parser compiler. Ia sesuai untuk demonstrasi UX, onboarding beginner, dan side-by-side teaching antara pseudo Melayu/English dengan expert canonical. Untuk translation rasmi, fasa seterusnya perlu menyediakan subcommand compiler seperti `translate --stdin --to expert`.
