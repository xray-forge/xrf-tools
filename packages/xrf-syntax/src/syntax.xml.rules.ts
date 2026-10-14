import { ESyntaxToken, ISyntaxRule } from "#/syntax.types";

/**
 * XML, covering configs, UI layouts and dialogs.
 *
 * Attributes are found by the `=` that follows them rather than by tracking whether the scan is inside
 * a tag, so `word=` in text content would colour as one. Game XML does not do that in practice, and the
 * cost of being wrong is a differently coloured word in a read-only view.
 */
export const XML_RULES: Array<ISyntaxRule> = [
  { token: ESyntaxToken.COMMENT, pattern: "<!--[\\s\\S]*?-->" },
  { token: ESyntaxToken.DIRECTIVE, pattern: "<[!?][\\s\\S]*?>" },
  { token: ESyntaxToken.SECTION, pattern: "</?[A-Za-z_][\\w:.\\-]*" },
  { token: ESyntaxToken.STRING, pattern: "\"[^\"]*\"|'[^']*'" },
  { token: ESyntaxToken.KEY, pattern: "[A-Za-z_][\\w:.\\-]*(?=\\s*=)" },
  { token: ESyntaxToken.OPERATOR, pattern: "/?>|=" },
];
