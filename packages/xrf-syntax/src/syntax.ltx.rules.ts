import { ESyntaxToken, ISyntaxRule } from "#/syntax.types";

/**
 * X-Ray configuration.
 *
 * Not quite the ini every highlighter ships: a section header carries its inherited parents after a
 * colon (`[wpn_ak74]:wpn_base`), and `#include` pulls another file in. Both are the first thing worth
 * seeing in a config, and both are what a stock ini grammar gets wrong.
 */
export const LTX_RULES: Array<ISyntaxRule> = [
  { token: ESyntaxToken.COMMENT, pattern: "(?:;|//)[^\\n]*" },
  { token: ESyntaxToken.DIRECTIVE, pattern: "^[ \\t]*#[A-Za-z_]+" },
  { token: ESyntaxToken.SECTION, pattern: "^[ \\t]*\\[[^\\]\\n]*\\]" },
  // Everything after `]:` on a section line names a parent section rather than a value.
  { token: ESyntaxToken.TYPE, pattern: "(?<=\\]:)[^;\\n]*" },
  { token: ESyntaxToken.STRING, pattern: '"[^"\\n]*"' },
  // A key is whatever precedes the `=` on its line, which is the only way to tell it from a value.
  { token: ESyntaxToken.KEY, pattern: "^[ \\t]*[A-Za-z_$][^=;\\n]*?(?=[ \\t]*=)" },
  { token: ESyntaxToken.NUMBER, pattern: "\\b\\d+(?:\\.\\d+)?\\b" },
  { token: ESyntaxToken.OPERATOR, pattern: "[=,:]" },
];
