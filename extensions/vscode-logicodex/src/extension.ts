import * as path from 'path';
import * as vscode from 'vscode';
import { loadDictionary, LoadedDictionary } from './dictionary';
import { normalizeToExpertPreview } from './previewNormalizer';

let currentPanel: vscode.WebviewPanel | undefined;
let currentDocumentUri: vscode.Uri | undefined;
let refreshTimer: NodeJS.Timeout | undefined;

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

function renderCodeWithLineNumbers(code: string, showLineNumbers: boolean): string {
  const lines = code.split('\n');
  return lines
    .map((line, index) => {
      const lineNo = showLineNumbers ? `<span class="line-number">${index + 1}</span>` : '';
      return `<div class="code-line">${lineNo}<span class="code-text">${escapeHtml(line) || ' '}</span></div>`;
    })
    .join('');
}

function getDocumentForPreview(): vscode.TextDocument | undefined {
  if (currentDocumentUri) {
    const found = vscode.workspace.textDocuments.find((doc) => doc.uri.toString() === currentDocumentUri?.toString());
    if (found) {
      return found;
    }
  }

  const active = vscode.window.activeTextEditor?.document;
  if (active && (active.languageId === 'logicodex' || path.extname(active.fileName) === '.ldx')) {
    currentDocumentUri = active.uri;
    return active;
  }

  return undefined;
}

function webviewHtml(document: vscode.TextDocument, dictionary: LoadedDictionary): string {
  const config = vscode.workspace.getConfiguration('logicodexSideView');
  const showLineNumbers = config.get<boolean>('showLineNumbers', true);
  const insertMissingBeginnerSemicolons = config.get<boolean>('insertMissingBeginnerSemicolons', true);
  const result = normalizeToExpertPreview(document.getText(), dictionary.coreMap, { insertMissingBeginnerSemicolons });
  const title = escapeHtml(path.basename(document.fileName));
  const dictionarySource = escapeHtml(dictionary.sourcePath);
  const version = escapeHtml(dictionary.coreMap.version ?? 'unknown');
  const warningHtml = result.warnings.map((warning) => `<p>${escapeHtml(warning)}</p>`).join('');
  const codeHtml = renderCodeWithLineNumbers(result.code, showLineNumbers);

  return `<!doctype html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline';">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Logicodex Expert Preview</title>
  <style>
    :root {
      color-scheme: light dark;
      --border: var(--vscode-panel-border, #444);
      --muted: var(--vscode-descriptionForeground, #777);
      --warn-bg: rgba(255, 193, 7, 0.12);
      --warn-border: rgba(255, 193, 7, 0.55);
      --code-bg: var(--vscode-textCodeBlock-background, rgba(127, 127, 127, 0.12));
    }
    body {
      margin: 0;
      padding: 0;
      color: var(--vscode-foreground);
      background: var(--vscode-editor-background);
      font-family: var(--vscode-font-family);
    }
    header {
      border-bottom: 1px solid var(--border);
      padding: 0.8rem 1rem;
    }
    h1 {
      font-size: 1rem;
      margin: 0 0 0.3rem 0;
    }
    .meta {
      color: var(--muted);
      font-size: 0.78rem;
      line-height: 1.4;
    }
    .warning {
      margin: 1rem;
      padding: 0.8rem;
      border: 1px solid var(--warn-border);
      background: var(--warn-bg);
      border-radius: 6px;
      font-size: 0.82rem;
    }
    .warning p {
      margin: 0.15rem 0;
    }
    pre {
      margin: 1rem;
      padding: 1rem 0;
      overflow: auto;
      border-radius: 6px;
      background: var(--code-bg);
      border: 1px solid var(--border);
    }
    .code-line {
      white-space: pre;
      font-family: var(--vscode-editor-font-family, monospace);
      font-size: var(--vscode-editor-font-size, 13px);
      line-height: 1.45;
    }
    .line-number {
      display: inline-block;
      width: 3rem;
      padding-right: 0.75rem;
      color: var(--muted);
      text-align: right;
      user-select: none;
    }
    .code-text {
      padding-right: 1rem;
    }
  </style>
</head>
<body>
  <header>
    <h1>Logicodex Expert Preview: ${title}</h1>
    <div class="meta">current logicodex v ${version} · Dictionary: ${dictionarySource}</div>
  </header>
  <section class="warning">${warningHtml}</section>
  <pre>${codeHtml}</pre>
</body>
</html>`;
}

function refreshPreview(context: vscode.ExtensionContext): void {
  if (!currentPanel) {
    return;
  }

  const document = getDocumentForPreview();
  if (!document) {
    currentPanel.webview.html = '<p>No active .ldx Logicodex document found.</p>';
    return;
  }

  try {
    const dictionary = loadDictionary(context);
    currentPanel.title = `Logicodex Expert: ${path.basename(document.fileName)}`;
    currentPanel.webview.html = webviewHtml(document, dictionary);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    currentPanel.webview.html = `<p>${escapeHtml(message)}</p>`;
  }
}

function scheduleRefresh(context: vscode.ExtensionContext): void {
  const debounceMs = vscode.workspace.getConfiguration('logicodexSideView').get<number>('debounceMs', 300);
  if (refreshTimer) {
    clearTimeout(refreshTimer);
  }
  refreshTimer = setTimeout(() => refreshPreview(context), debounceMs);
}

export function activate(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.commands.registerCommand('logicodexSideView.openPreview', () => {
      const document = getDocumentForPreview();
      if (!document) {
        void vscode.window.showWarningMessage('Open a .ldx Logicodex file before opening the expert side-view.');
        return;
      }

      currentDocumentUri = document.uri;

      if (!currentPanel) {
        currentPanel = vscode.window.createWebviewPanel(
          'logicodexExpertSideView',
          `Logicodex Expert: ${path.basename(document.fileName)}`,
          vscode.ViewColumn.Beside,
          {
            enableScripts: false,
            retainContextWhenHidden: true
          }
        );

        currentPanel.onDidDispose(() => {
          currentPanel = undefined;
          currentDocumentUri = undefined;
        }, undefined, context.subscriptions);
      } else {
        currentPanel.reveal(vscode.ViewColumn.Beside);
      }

      refreshPreview(context);
    }),
    vscode.commands.registerCommand('logicodexSideView.refreshPreview', () => refreshPreview(context)),
    vscode.workspace.onDidChangeTextDocument((event) => {
      if (event.document.uri.toString() === currentDocumentUri?.toString()) {
        scheduleRefresh(context);
      }
    }),
    vscode.window.onDidChangeActiveTextEditor((editor) => {
      if (editor?.document.languageId === 'logicodex' || (editor && path.extname(editor.document.fileName) === '.ldx')) {
        currentDocumentUri = editor.document.uri;
        scheduleRefresh(context);
      }
    })
  );
}

export function deactivate(): void {
  if (refreshTimer) {
    clearTimeout(refreshTimer);
  }
}
