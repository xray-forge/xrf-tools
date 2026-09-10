import { LtxFileStructure, LtxStructureInclude, LtxStructureSection } from "@/core/bindings/types/xrf-ltx-inspect";
import { toLexicalLines } from "@/core/ltx/lib/lexical";
import { ESyntaxToken, ISyntaxSpan } from "@/core/syntax/lib";
import { ECodeLineMark, ICodeLine } from "@/core/ui/code/code-line";

/**
 * Turn one config's text and structure into the lines a viewer renders.
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
 * @returns One entry per line, numbered from one.
 */
export function toDocumentLines(
  text: ReadonlyArray<string>,
  structure: LtxFileStructure | null,
  marks: ReadonlyMap<number, ECodeLineMark> = new Map()
): Array<ICodeLine> {
  const lexical: Array<Array<ISyntaxSpan>> = toLexicalLines(text);
  const lines: Array<ICodeLine> = lexical.map((spans: Array<ISyntaxSpan>, index: number) => ({
    number: index + 1,
    spans,
  }));

  for (const [line, mark] of marks) {
    markLine(lines, line, mark);
  }

  if (!structure) {
    return lines;
  }

  for (const section of structure.sections) {
    replaceLine(lines, section.line, toSectionSpans(section, text[section.line - 1] ?? ""));
  }

  for (const include of structure.includes) {
    replaceLine(lines, include.line, toIncludeSpans(include, text[include.line - 1] ?? ""));
  }

  return lines;
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

/**
 * Replaces one line's colouring, when the line exists.
 *
 * A structure describing a line the text does not hold is a disagreement between two reads of one file, and dropping
 * it is better than writing past the end of the document.
 */
function replaceLine(lines: Array<ICodeLine>, line: number, spans: Array<ISyntaxSpan>): void {
  const at: number = line - 1;

  if (at >= 0 && at < lines.length) {
    lines[at] = { ...lines[at], spans };
  }
}

/** Marks one line's gutter, when the line exists. */
function markLine(lines: Array<ICodeLine>, line: number, mark: ECodeLineMark): void {
  const at: number = line - 1;

  if (at >= 0 && at < lines.length) {
    lines[at] = { ...lines[at], mark };
  }
}
