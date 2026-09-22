import { Maybe } from "@xrf/types";

import { LtxFileStructure, LtxStructureInclude, LtxStructureSection } from "@/core/ipc/types/xrf-ltx-inspect";
import { toLexicalLine } from "@/core/ltx/lib/lexical";
import { ESyntaxToken, ISyntaxSpan } from "@/core/syntax/lib";
import { ECodeLineMark, ICodeLine, ICodeLineSource } from "@/core/ui/code/code-line";

/** Shared, because a config is mostly blank lines and each of them would otherwise allocate. */
const NO_SPANS: ReadonlyArray<ISyntaxSpan> = Object.freeze([]);

/**
 * Turn one config's text and structure into a source a viewer renders.
 *
 * Two layers, as an editor draws them. The lexical one colours every line from its own characters, locally and at
 * keystroke speed. The semantic one replaces the lines only the parser can be right about - a section header, an
 * include - because a regex cannot tell whether a parent resolves or whether an include reached anything, and the
 * document already knows.
 *
 * @param text - The config's lines, index `n` holding line `n + 1`.
 * @param structure - What the backend made of the same file, or null when nothing judged it.
 * @param marks - What is wrong, by the line it sits on. Findings are the one source of a gutter mark, so a listing
 *   never has two answers about the same line.
 * @returns A source over its lines, numbered from one.
 */
export function toDocumentLineSource(
  text: ReadonlyArray<string>,
  structure: LtxFileStructure | null,
  marks: ReadonlyMap<number, ECodeLineMark> = new Map()
): ICodeLineSource {
  const sections: ReadonlyMap<number, LtxStructureSection> = new Map(
    (structure?.sections ?? []).map((section: LtxStructureSection) => [section.line, section])
  );
  // Built after the sections and read before them, so a file declaring both on one line draws as it did when the
  // two passes ran in order.
  const includes: ReadonlyMap<number, LtxStructureInclude> = new Map(
    (structure?.includes ?? []).map((include: LtxStructureInclude) => [include.line, include])
  );

  function getSpans(number: number): ReadonlyArray<ISyntaxSpan> {
    const line: string = text[number - 1];
    const include: Maybe<LtxStructureInclude> = includes.get(number);

    if (include) {
      return toIncludeSpans(include, line ?? "");
    }

    const section: Maybe<LtxStructureSection> = sections.get(number);

    if (section) {
      return toSectionSpans(section, line ?? "");
    }

    return line ? toLexicalLine(line) : NO_SPANS;
  }

  return {
    count: text.length,
    getLine: (index: number): ICodeLine => ({
      mark: marks.get(index + 1),
      number: index + 1,
      spans: getSpans(index + 1),
    }),
    // The text itself, which is what a different config brings and what marks arriving do not.
    layout: text,
    indexOfLine: (number: number): number => (number >= 1 && number <= text.length ? number - 1 : -1),
    widestNumber: Math.max(text.length, 1),
  };
}

/**
 * A section header, coloured by what resolving it made of each part.
 *
 * A parent nothing declares is drawn as plain text rather than as a name, which is the only visible difference between
 * `[a]:base` in a tree that has `base` and one that does not - and in the second the engine silently inherits nothing.
 */
function toSectionSpans(section: LtxStructureSection, line: string): Array<ISyntaxSpan> {
  const bracketed: string = `${section.operation}[${section.name}]`;
  const at: number = line.indexOf(bracketed);

  if (at === -1) {
    return [{ token: ESyntaxToken.SECTION, text: line }];
  }

  const spans: Array<ISyntaxSpan> = [];

  if (at > 0) {
    spans.push({ token: ESyntaxToken.PLAIN, text: line.slice(0, at) });
  }

  spans.push({ token: ESyntaxToken.SECTION, text: bracketed });

  const rest: string = line.slice(at + bracketed.length);

  if (!rest) {
    return spans;
  }

  // Everything after the header is the parent list and whatever comment follows it. Each declared parent is coloured
  // by whether the resolution holds it; the separators between them stay operators.
  let cursor: number = 0;

  for (const parent of section.parents) {
    const found: number = rest.indexOf(parent.name, cursor);

    if (found === -1) {
      continue;
    }

    if (found > cursor) {
      spans.push({ token: ESyntaxToken.OPERATOR, text: rest.slice(cursor, found) });
    }

    spans.push({ token: parent.resolves ? ESyntaxToken.TYPE : ESyntaxToken.PLAIN, text: parent.name });
    cursor = found + parent.name.length;
  }

  if (cursor < rest.length) {
    spans.push({ token: ESyntaxToken.PLAIN, text: rest.slice(cursor) });
  }

  return spans;
}

/**
 * An include, coloured by whether it reached anything.
 */
function toIncludeSpans(include: LtxStructureInclude, line: string): Array<ISyntaxSpan> {
  return [
    {
      token: include.resolved.length ? ESyntaxToken.DIRECTIVE : ESyntaxToken.PLAIN,
      text: line,
    },
  ];
}
