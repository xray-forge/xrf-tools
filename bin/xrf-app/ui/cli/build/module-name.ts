import * as path from "node:path";

/**
 * Token authors write where they want their module's name.
 */
export const MODULE_NAME_TOKEN: string = "__MODULE_NAME__";

const TOKEN_PATTERN: RegExp = new RegExp(String.raw`\b${MODULE_NAME_TOKEN}\b`, "g");

/**
 * The name a module is known by in logs: its base name without extensions.
 *
 * A file rather than a class, because a bundle keeps neither. Base name rather than the path from `src`, because a tag
 * has to stay short enough to scan a console with - `archives.service` reads, the full route does not. One class per
 * file is the repository's own convention, so the two identify the same thing.
 *
 * An `index` names nothing on its own, so it borrows the directory it sits in.
 *
 * @param filename - Absolute path of the module being compiled.
 * @returns Name to substitute for the token.
 */
export function getModuleName(filename: string): string {
  const normalized: string = filename.replaceAll("\\", "/").split("?")[0];
  const base: string = path.basename(normalized).replace(/\.[cm]?[jt]sx?$/, "");

  return base === "index" ? path.basename(path.dirname(normalized)) : base;
}

/**
 * Replaces every module-name token in one file's source.
 *
 * Deliberately a text substitution rather than an AST rewrite: nothing is transformed unless an author wrote the token,
 * so the pass cannot surprise code it was not pointed at.
 *
 * @param code - Source of the module.
 * @param filename - Absolute path of the module being compiled.
 * @returns The source with tokens replaced, or the original when it carries none.
 */
export function replaceModuleName(code: string, filename: string): string {
  if (!code.includes(MODULE_NAME_TOKEN)) {
    return code;
  }

  return code.replace(TOKEN_PATTERN, JSON.stringify(getModuleName(filename)));
}
