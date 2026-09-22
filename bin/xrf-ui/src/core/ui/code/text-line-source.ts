import { Nullable } from "@xrf/types";

import { ESyntaxLanguage, ESyntaxToken, highlightSyntax, isLineLocalSyntax, ISyntaxSpan } from "@/core/syntax/lib";
import { ICodeLine, ICodeLineSource } from "@/core/ui/code/code-line";
import { EMPTY_ARRAY } from "@/lib/types/array";

/** Most text a grammar that is not line-local is coloured over. */
export const MAXIMUM_SCANNED_LENGTH: number = 4 * 1024 * 1024;

/**
 * A source over one document held as a single string, coloured as the listing asks for lines.
 *
 * @param content - The document's text, exactly as it was read.
 * @param language - Grammar to colour it with.
 * @param firstLine - Number the first line displays, for an excerpt lifted out of a larger file.
 * @returns A source over its lines.
 */
export function toTextLineSource(content: string, language: ESyntaxLanguage, firstLine: number = 1): ICodeLineSource {
  // Split verbatim. A `\r` left by a Windows-authored file draws as nothing in a `pre` run, and stripping it would
  // make what the listing shows differ from what the file holds and from what extracting it writes.
  const lines: Array<string> = content.split("\n");
  const isLineLocal: boolean = isLineLocalSyntax(language);

  // Scanned on the first line asked for rather than here, so a document opened and left alone costs nothing.
  let scanned: Nullable<Array<Array<ISyntaxSpan>>> = null;

  function getSpans(index: number): ReadonlyArray<ISyntaxSpan> {
    const text: string = lines[index];

    if (!text) {
      return EMPTY_ARRAY;
    }

    if (isLineLocal) {
      return highlightSyntax(text, language);
    }

    if (content.length > MAXIMUM_SCANNED_LENGTH) {
      return [{ token: ESyntaxToken.PLAIN, text }];
    }

    scanned ??= toScannedLines(content, lines.length, language);

    return scanned[index];
  }

  return {
    count: lines.length,
    getLine: (index: number): ICodeLine => ({ number: firstLine + index, spans: getSpans(index) }),
    // The lines themselves: a caller holding them holds the whole document, as `toCodeLineSource` does.
    layout: lines,
    indexOfLine: (number: number): number => {
      const at: number = number - firstLine;

      return at >= 0 && at < lines.length ? at : -1;
    },
    widestNumber: firstLine + Math.max(lines.length, 1) - 1,
  };
}

/**
 * Colour a document whose grammar crosses lines, and deal its spans out to the lines they cover.
 *
 * @param content - The document's text.
 * @param count - How many lines it was split into.
 * @param language - Grammar to colour it with.
 * @returns Spans per line, positionally matching that split.
 */
function toScannedLines(content: string, count: number, language: ESyntaxLanguage): Array<Array<ISyntaxSpan>> {
  const lines: Array<Array<ISyntaxSpan>> = Array.from({ length: count }, () => []);

  let at: number = 0;

  for (const span of highlightSyntax(content, language)) {
    const parts: Array<string> = span.text.split("\n");

    for (let index: number = 0; index < parts.length; index += 1) {
      // Every part after the first begins where a newline ended the line before it.
      if (index > 0) {
        at += 1;
      }

      if (parts[index] && at < count) {
        lines[at].push({ token: span.token, text: parts[index] });
      }
    }
  }

  return lines;
}
