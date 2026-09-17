import { ESyntaxLanguage, highlightSyntax, ISyntaxSpan } from "@/core/syntax/lib";

/**
 * Colours one line of LTX text.
 *
 * Correct because `LTX_RULES` is line-local by construction: a comment runs to the end of its line, a
 * section header and a key are anchored at line start, and nothing in the grammar spans a newline. So
 * a line scanned alone gets the same answer it would get inside the whole file, and the listing above
 * this can colour a document without a round trip and without re-scanning what did not change.
 *
 * This is the lexical half only. The backend's structure record owns header and include lines outright,
 * because parents, resolution and scheme binding are not things a regex can know; a caller replaces the
 * spans of those lines with what the parser said. See `plans/configs-explorer.md` decision 4.
 *
 * @param line - One source line, without its terminator.
 * @returns Its spans; concatenating their `text` reproduces the line.
 */
export function toLexicalLine(line: string): Array<ISyntaxSpan> {
  return highlightSyntax(line, ESyntaxLanguage.LTX);
}
