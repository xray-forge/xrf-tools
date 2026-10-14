/**
 * Grammars the preview can colour.
 */
export enum ESyntaxLanguage {
  PLAIN = "plain",
  LTX = "ltx",
  LUA = "lua",
  SHADER = "shader",
  TYPESCRIPT = "typescript",
  XML = "xml",
}

/**
 * What a run of characters is, as far as colouring is concerned.
 *
 * Shared across every grammar so one palette covers all of them - `SECTION` is an LTX section header
 * and an XML tag name, `KEY` is an LTX key and an XML attribute. They are coloured alike because they
 * play the same role: the name of the thing whose value follows.
 */
export enum ESyntaxToken {
  PLAIN = "plain",
  COMMENT = "comment",
  STRING = "string",
  NUMBER = "number",
  KEYWORD = "keyword",
  TYPE = "type",
  DIRECTIVE = "directive",
  SECTION = "section",
  KEY = "key",
  OPERATOR = "operator",
}

/**
 * One coloured run. Concatenating `text` across every span of a result reproduces the input exactly.
 */
export interface ISyntaxSpan {
  token: ESyntaxToken;
  text: string;
}

/**
 * One grammar rule.
 *
 * `pattern` is regex source rather than a `RegExp` because the rules of a language are combined into a
 * single expression - scanning with one pass instead of trying every rule at every position. It must
 * therefore contain no capturing groups, since group indices are what identify the matching rule.
 */
export interface ISyntaxRule {
  token: ESyntaxToken;
  pattern: string;
}
