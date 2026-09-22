import { Nullable } from "@xrf/types";

import { EXrayExtension } from "@/core/ipc/types/xrf-extension";
import { getXrayExtension } from "@/core/path/extension";
import { ESyntaxLanguage } from "@/core/syntax/lib/syntax.types";

/**
 * Extension to grammar, for the file kinds a game archive holds.
 *
 * `.s` is Lua rather than a shader language despite living in `shaders/`: those files script the render
 * pipeline through Lua bindings. `.s_` is the same kind of file under the spelling it is authored with beside
 * one - `XrayAssetType::of` groups the two - so it reads with the same grammar. `.cmd` is a build batch file
 * and is left plain - there are a handful of them in the whole game, which does not pay for a grammar.
 *
 * Partial over the vocabulary rather than exhaustive: most declared spellings name binary formats that no
 * highlighter reads, and listing each one as plain would say nothing.
 */
const SYNTAX_LANGUAGE_BY_EXTENSION: Readonly<Partial<Record<EXrayExtension, ESyntaxLanguage>>> = {
  [EXrayExtension.LTX]: ESyntaxLanguage.LTX,
  [EXrayExtension.SCRIPT]: ESyntaxLanguage.LUA,
  [EXrayExtension.LUA]: ESyntaxLanguage.LUA,
  [EXrayExtension.S]: ESyntaxLanguage.LUA,
  [EXrayExtension.S_]: ESyntaxLanguage.LUA,
  [EXrayExtension.PS]: ESyntaxLanguage.SHADER,
  [EXrayExtension.VS]: ESyntaxLanguage.SHADER,
  [EXrayExtension.GS]: ESyntaxLanguage.SHADER,
  [EXrayExtension.HS]: ESyntaxLanguage.SHADER,
  [EXrayExtension.DS]: ESyntaxLanguage.SHADER,
  [EXrayExtension.CS]: ESyntaxLanguage.SHADER,
  [EXrayExtension.H]: ESyntaxLanguage.SHADER,
  [EXrayExtension.HLSL]: ESyntaxLanguage.SHADER,
  [EXrayExtension.TS]: ESyntaxLanguage.TYPESCRIPT,
  [EXrayExtension.JSON]: ESyntaxLanguage.TYPESCRIPT,
  [EXrayExtension.XML]: ESyntaxLanguage.XML,
};

/**
 * Pick the grammar for a file from its name.
 *
 * @param filename - Archive relative path or bare name, in either slash style.
 * @returns The grammar to colour it with, or `PLAIN` when its extension means nothing here.
 */
export function getSyntaxLanguage(filename: string): ESyntaxLanguage {
  const extension: Nullable<EXrayExtension> = getXrayExtension(filename);

  return (extension === null ? undefined : SYNTAX_LANGUAGE_BY_EXTENSION[extension]) ?? ESyntaxLanguage.PLAIN;
}
