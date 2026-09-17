import { getSyntaxRules } from "@/core/syntax/lib/syntax-rules";
import { ESyntaxLanguage, ESyntaxToken, ISyntaxRule, ISyntaxSpan } from "@/core/syntax/lib/syntax.types";
import { Nullable } from "@/lib/types/general";

const SCANNERS: Map<ESyntaxLanguage, Nullable<RegExp>> = new Map();

/**
 * Combine a language's rules into one expression, so the whole file is scanned in a single pass.
 *
 * @param rules - Ordered rules of one language.
 * @returns One expression whose capture group N corresponds to rule N, or null when there are none.
 */
function buildScanner(rules: Array<ISyntaxRule>): Nullable<RegExp> {
  if (!rules.length) {
    return null;
  }

  return new RegExp(rules.map((rule: ISyntaxRule) => `(${rule.pattern})`).join("|"), "gm");
}

/**
 * The scanner for a language, built once and kept.
 *
 * @param language - Language to scan with.
 * @returns Its expression, or null when the language has no rules.
 */
function getScanner(language: ESyntaxLanguage): Nullable<RegExp> {
  if (!SCANNERS.has(language)) {
    SCANNERS.set(language, buildScanner(getSyntaxRules(language)));
  }

  return SCANNERS.get(language) ?? null;
}

/**
 * Split source text into coloured runs.
 *
 * Order in the rule table is precedence: at any position the first rule that matches wins, which is why
 * comments and strings are declared before the operators and keywords they can contain.
 *
 * @param content - File contents to colour.
 * @param language - Grammar to colour it with.
 * @returns Spans covering the input exactly, so joining their text reproduces `content`.
 */
export function highlightSyntax(content: string, language: ESyntaxLanguage): Array<ISyntaxSpan> {
  const scanner: Nullable<RegExp> = getScanner(language);

  if (!content) {
    return [];
  }

  if (!scanner) {
    return [{ token: ESyntaxToken.PLAIN, text: content }];
  }

  const rules: Array<ISyntaxRule> = getSyntaxRules(language);
  const spans: Array<ISyntaxSpan> = [];

  let plainStart: number = 0;
  let match: Nullable<RegExpExecArray>;

  scanner.lastIndex = 0;

  while ((match = scanner.exec(content)) !== null) {
    // A rule that can match nothing would otherwise spin on one position forever.
    if (!match[0]) {
      scanner.lastIndex += 1;

      continue;
    }

    if (match.index > plainStart) {
      spans.push({ token: ESyntaxToken.PLAIN, text: content.slice(plainStart, match.index) });
    }

    spans.push({ token: rules[getMatchedRule(match)].token, text: match[0] });
    plainStart = match.index + match[0].length;
  }

  if (plainStart < content.length) {
    spans.push({ token: ESyntaxToken.PLAIN, text: content.slice(plainStart) });
  }

  return spans;
}

/**
 * Work out which rule produced a match from the capture group that filled.
 *
 * @param match - A match from a combined scanner.
 * @returns Index of the matching rule, falling back to the first when a group cannot be identified.
 */
function getMatchedRule(match: RegExpExecArray): number {
  for (let group = 1; group < match.length; group += 1) {
    if (match[group] !== undefined) {
      return group - 1;
    }
  }

  return 0;
}
