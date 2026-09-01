import { ESyntaxLanguage } from "@/core/syntax/lib/syntax.types";
import { getFileExtension } from "@/lib/path/extension";

/**
 * Extension to grammar, for the file kinds a game archive holds.
 *
 * `.s` is Lua rather than a shader language despite living in `shaders/`: those files script the render
 * pipeline through Lua bindings. `.cmd` is a build batch file and is left plain - there are a handful of
 * them in the whole game, which does not pay for a grammar.
 */
const SYNTAX_LANGUAGE_BY_EXTENSION: Record<string, ESyntaxLanguage> = {
  ltx: ESyntaxLanguage.LTX,
  script: ESyntaxLanguage.LUA,
  lua: ESyntaxLanguage.LUA,
  s: ESyntaxLanguage.LUA,
  ps: ESyntaxLanguage.SHADER,
  vs: ESyntaxLanguage.SHADER,
  gs: ESyntaxLanguage.SHADER,
  hs: ESyntaxLanguage.SHADER,
  ds: ESyntaxLanguage.SHADER,
  cs: ESyntaxLanguage.SHADER,
  h: ESyntaxLanguage.SHADER,
  hlsl: ESyntaxLanguage.SHADER,
  ts: ESyntaxLanguage.TYPESCRIPT,
  tsx: ESyntaxLanguage.TYPESCRIPT,
  js: ESyntaxLanguage.TYPESCRIPT,
  json: ESyntaxLanguage.TYPESCRIPT,
  xml: ESyntaxLanguage.XML,
};

/**
 * Pick the grammar for a file from its name.
 *
 * @param filename - Archive relative path or bare name, in either slash style.
 * @returns The grammar to colour it with, or `PLAIN` when its extension means nothing here.
 */
export function getSyntaxLanguage(filename: string): ESyntaxLanguage {
  return SYNTAX_LANGUAGE_BY_EXTENSION[getFileExtension(filename)] ?? ESyntaxLanguage.PLAIN;
}
