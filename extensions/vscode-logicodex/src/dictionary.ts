import * as fs from 'fs';
import * as path from 'path';
import * as vscode from 'vscode';

export interface CoreMapToken {
  expert: string;
  primary_ms: string;
  aliases: string[];
  description?: string;
  beginner_line_terminated?: boolean;
  critical_policy?: Record<string, unknown>;
}

export interface CoreMap {
  schema_version: string;
  version: string;
  language: string;
  description?: string;
  policy?: Record<string, unknown>;
  tokens: Record<string, CoreMapToken>;
}

export interface LoadedDictionary {
  coreMap: CoreMap;
  sourcePath: string;
}

function readJsonFile(filePath: string): CoreMap | undefined {
  try {
    const raw = fs.readFileSync(filePath, 'utf8');
    const parsed = JSON.parse(raw) as CoreMap;
    if (!parsed.tokens || typeof parsed.tokens !== 'object') {
      return undefined;
    }
    return parsed;
  } catch {
    return undefined;
  }
}

function expandHome(inputPath: string): string {
  if (!inputPath.startsWith('~')) {
    return inputPath;
  }
  const home = process.env.HOME || process.env.USERPROFILE;
  if (!home) {
    return inputPath;
  }
  return path.join(home, inputPath.slice(1));
}

export function loadDictionary(context: vscode.ExtensionContext): LoadedDictionary {
  const configPath = vscode.workspace
    .getConfiguration('logicodexSideView')
    .get<string>('dictionaryPath', '')
    .trim();

  const candidates: string[] = [];

  if (configPath.length > 0) {
    candidates.push(path.resolve(expandHome(configPath)));
  }

  for (const folder of vscode.workspace.workspaceFolders ?? []) {
    candidates.push(path.join(folder.uri.fsPath, 'dict', 'core_map.json'));
  }

  candidates.push(context.asAbsolutePath(path.join('resources', 'core_map.json')));

  for (const candidate of candidates) {
    const coreMap = readJsonFile(candidate);
    if (coreMap) {
      return { coreMap, sourcePath: candidate };
    }
  }

  throw new Error('Unable to load Logicodex core_map.json from settings, workspace, or bundled resources.');
}
