import { CoreMap } from './dictionary';

export interface NormalizeOptions {
  insertMissingBeginnerSemicolons: boolean;
}

export interface NormalizeResult {
  code: string;
  warnings: string[];
}

interface ReplacementRule {
  source: string;
  target: string;
  tokenName: string;
  wordLike: boolean;
}

const WORD_LIKE = /^[A-Za-z_][A-Za-z0-9_]*$/;

function escapeRegex(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function createReplacementRules(coreMap: CoreMap): ReplacementRule[] {
  const seen = new Set<string>();
  const rules: ReplacementRule[] = [];

  for (const [tokenName, token] of Object.entries(coreMap.tokens)) {
    const candidates = [token.primary_ms, ...(token.aliases ?? [])];
    for (const candidate of candidates) {
      if (!candidate || candidate === token.expert) {
        continue;
      }
      const key = `${candidate}\u0000${token.expert}`;
      if (seen.has(key)) {
        continue;
      }
      seen.add(key);
      rules.push({
        source: candidate,
        target: token.expert,
        tokenName,
        wordLike: WORD_LIKE.test(candidate)
      });
    }
  }

  rules.sort((a, b) => b.source.length - a.source.length || a.source.localeCompare(b.source));
  return rules;
}

function replaceOutsideStringsAndComments(input: string, rules: ReplacementRule[]): string {
  let output = '';
  let segment = '';
  let state: 'code' | 'string' | 'lineComment' | 'blockComment' = 'code';

  const flushSegment = (): void => {
    if (!segment) {
      return;
    }
    let transformed = segment;
    for (const rule of rules) {
      const pattern = rule.wordLike
        ? new RegExp(`(?<![A-Za-z0-9_])${escapeRegex(rule.source)}(?![A-Za-z0-9_])`, 'g')
        : new RegExp(escapeRegex(rule.source), 'g');
      transformed = transformed.replace(pattern, rule.target);
    }
    output += transformed;
    segment = '';
  };

  for (let i = 0; i < input.length; i += 1) {
    const current = input[i];
    const next = input[i + 1];

    if (state === 'code') {
      if (current === '"') {
        flushSegment();
        output += current;
        state = 'string';
        continue;
      }
      if (current === '/' && next === '/') {
        flushSegment();
        output += current + next;
        i += 1;
        state = 'lineComment';
        continue;
      }
      if (current === '/' && next === '*') {
        flushSegment();
        output += current + next;
        i += 1;
        state = 'blockComment';
        continue;
      }
      segment += current;
      continue;
    }

    output += current;

    if (state === 'string') {
      if (current === '\\' && next !== undefined) {
        output += next;
        i += 1;
        continue;
      }
      if (current === '"') {
        state = 'code';
      }
      continue;
    }

    if (state === 'lineComment') {
      if (current === '\n') {
        state = 'code';
      }
      continue;
    }

    if (state === 'blockComment' && current === '*' && next === '/') {
      output += next;
      i += 1;
      state = 'code';
    }
  }

  flushSegment();
  return output;
}

function needsPreviewSemicolon(line: string): boolean {
  const trimmed = line.trim();
  if (trimmed.length === 0) {
    return false;
  }
  if (trimmed.endsWith(';') || trimmed.endsWith('{') || trimmed.endsWith('}') || trimmed.endsWith('then') || trimmed.endsWith('else')) {
    return false;
  }
  return /^(let|print|return|break|continue)\b/.test(trimmed);
}

function insertBeginnerSemicolons(input: string): string {
  return input
    .split(/(\r?\n)/)
    .map((part) => {
      if (part === '\n' || part === '\r\n') {
        return part;
      }
      if (!needsPreviewSemicolon(part)) {
        return part;
      }
      return `${part};`;
    })
    .join('');
}

export function normalizeToExpertPreview(source: string, coreMap: CoreMap, options: NormalizeOptions): NormalizeResult {
  const rules = createReplacementRules(coreMap);
  let code = replaceOutsideStringsAndComments(source, rules);

  if (options.insertMissingBeginnerSemicolons) {
    code = insertBeginnerSemicolons(code);
  }

  const warnings = [
    'Best-effort preview only: this is token-level normalization from core_map.json; the current single HIR compiler path is authoritative.',
    'Run `logicodex check` or `cargo run -- check` for authoritative syntax and semantic validation.'
  ];

  return { code, warnings };
}
