import {
  ESyntaxLanguage,
  ESyntaxToken,
  highlightSyntax,
  ISyntaxSpan,
  MAXIMUM_HIGHLIGHT_LENGTH,
} from "@/core/syntax/lib";

/**
 * Colours LTX text one line at a time.
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
 * @param lines - Source lines, without their terminators.
 * @returns Spans per line, positionally matching the input; concatenating one line's `text` reproduces it.
 */
export function toLexicalLines(lines: ReadonlyArray<string>): Array<Array<ISyntaxSpan>> {
  // The same budget `highlightSyntax` applies to a file, applied to the document as a whole. The reason
  // differs: the listing renders only a window, so DOM nodes are no longer what grows - the span array
  // itself is, and a resolved root is the one document here big enough for that to matter.
  let budget: number = MAXIMUM_HIGHLIGHT_LENGTH;

  for (const line of lines) {
    budget -= line.length;

    if (budget < 0) {
      return lines.map((it: string) => (it ? [{ token: ESyntaxToken.PLAIN, text: it }] : []));
    }
  }

  return lines.map((line: string) => highlightSyntax(line, ESyntaxLanguage.LTX));
}
