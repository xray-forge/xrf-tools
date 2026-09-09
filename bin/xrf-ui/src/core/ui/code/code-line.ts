import { ISyntaxSpan } from "@/core/syntax/lib";

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
