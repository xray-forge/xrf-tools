import { LTX_RULES } from "@/core/syntax/lib/syntax.ltx.rules";
import { LUA_RULES } from "@/core/syntax/lib/syntax.lua.rules";
import { SHADER_RULES } from "@/core/syntax/lib/syntax.shader.rules";
import { ESyntaxLanguage, ISyntaxRule } from "@/core/syntax/lib/syntax.types";
import { TYPESCRIPT_RULES } from "@/core/syntax/lib/syntax.typescript.rules";
import { XML_RULES } from "@/core/syntax/lib/syntax.xml.rules";

const SYNTAX_RULES: Record<ESyntaxLanguage, Array<ISyntaxRule>> = {
  [ESyntaxLanguage.PLAIN]: [],
  [ESyntaxLanguage.LTX]: LTX_RULES,
  [ESyntaxLanguage.LUA]: LUA_RULES,
  [ESyntaxLanguage.SHADER]: SHADER_RULES,
  [ESyntaxLanguage.TYPESCRIPT]: TYPESCRIPT_RULES,
  [ESyntaxLanguage.XML]: XML_RULES,
};

/**
 * Whether a grammar answers the same for a line read alone as for that line inside its file.
 */
const LINE_LOCAL_SYNTAX: Readonly<Record<ESyntaxLanguage, boolean>> = {
  [ESyntaxLanguage.PLAIN]: true,
  [ESyntaxLanguage.LTX]: true,
  [ESyntaxLanguage.LUA]: false,
  [ESyntaxLanguage.SHADER]: false,
  [ESyntaxLanguage.TYPESCRIPT]: false,
  [ESyntaxLanguage.XML]: false,
};

/**
 * Rules colouring one language, in the order they take precedence at a given position.
 *
 * @param language - Language to describe.
 * @returns Its ordered rules, empty for a language with nothing to colour.
 */
export function getSyntaxRules(language: ESyntaxLanguage): Array<ISyntaxRule> {
  return SYNTAX_RULES[language] ?? [];
}

/**
 * Whether one language can be coloured a line at a time.
 *
 * @param language - Language to describe.
 * @returns True when no rule of that language can match across a newline.
 */
export function isLineLocalSyntax(language: ESyntaxLanguage): boolean {
  return LINE_LOCAL_SYNTAX[language] ?? false;
}
