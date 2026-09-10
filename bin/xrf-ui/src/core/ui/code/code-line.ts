import { ISyntaxSpan } from "@/core/syntax/lib";
import { Nullable } from "@/lib/types/general";

/**
 * What a gutter mark says about the line it sits beside.
 */
export enum ECodeLineMark {
  ERROR = "error",
  WARNING = "warning",
}

/**
 * One rendered source line.
 *
 * Its own module so a producer can build lines without pulling the list component in, which is what
 * lets a document be turned into lines by a worker, a service, or a test.
 *
 * `number` is what the gutter shows and what the list is addressed by, and it is deliberately not the
 * array index plus one: an excerpt keeps the numbering of the file it came from, and a resolved
 * document is assembled out of sections rather than read off one.
 */
export interface ICodeLine {
  number: number;
  spans: ReadonlyArray<ISyntaxSpan>;
  mark?: ECodeLineMark;
}

/**
 * A document a listing can draw, which answers for a line only when one is asked for.
 *
 * A source rather than an array because the documents this listing exists for are too large to hold as one: Anomaly's
 * `system.ltx` resolves to 551,000 lines, and materialising them costs about a million objects that the next answer
 * throws away. A screen shows forty of them. Every number here is a fact about the whole document - the count decides
 * the scroll height, the widest number decides the gutter - and only `getLine` is per line, so nothing about laying
 * the document out depends on how much of it has been built.
 *
 * A source is immutable in shape and may be mutable in content: a page arriving changes what `getLine` answers and
 * never how many lines there are or where one sits. A consumer that needs the listing redrawn when content lands hands
 * over a new source rather than mutating this one, which is what React re-renders on.
 */
export interface ICodeLineSource {
  /** Which document this is a view of, compared by identity. */
  layout: object;
  /** How many lines the document holds. */
  count: number;
  /** The widest number the gutter has to hold, which is what its width is measured from. */
  widestNumber: number;
  /**
   * The line at a position, built on demand.
   *
   * @param index - Position in the document, from zero.
   * @returns The line, which a source is free to build fresh on every ask.
   */
  getLine(index: number): ICodeLine;
  /**
   * Where the line displaying a number sits.
   *
   * @param number - Number as the gutter shows it.
   * @returns Its position, or -1 when this document does not display it.
   */
  indexOfLine(number: number): number;
}

/**
 * A source over lines that are already in hand.
 *
 * For documents small enough to hold whole - a config as authored is thousands of lines, not hundreds of thousands.
 * The number index is built on the first ask rather than up front, because most listings are never addressed by number
 * at all.
 *
 * @param lines - The lines, in the order they are displayed.
 * @returns A source over them.
 */
export function toCodeLineSource(lines: ReadonlyArray<ICodeLine>): ICodeLineSource {
  let indexOfNumber: Nullable<Map<number, number>> = null;
  let widest: number = 1;

  for (const line of lines) {
    if (line.number > widest) {
      widest = line.number;
    }
  }

  return {
    count: lines.length,
    getLine: (index: number): ICodeLine => lines[index],
    // The lines themselves: a caller holding an array holds the whole document, so the array is the document.
    layout: lines,
    indexOfLine: (number: number): number => {
      indexOfNumber ??= new Map(lines.map((line: ICodeLine, at: number) => [line.number, at]));

      return indexOfNumber.get(number) ?? -1;
    },
    widestNumber: widest,
  };
}

/**
 * The stretch of a listing that is currently on screen, by the numbers its lines display.
 *
 * Beside {@link ICodeLine} rather than on the component, for the same reason: what a viewport is showing is what
 * decides which section bodies a service has to fetch, and a service naming that range should not have to import a
 * React component to do it.
 *
 * Both ends are inclusive, and both are line numbers rather than positions - a listing addressed by index at its edges
 * and by number everywhere else would be two addressing schemes for one document.
 */
export interface ICodeLineRange {
  firstLine: number;
  lastLine: number;
}
