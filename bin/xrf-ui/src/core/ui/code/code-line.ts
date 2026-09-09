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
