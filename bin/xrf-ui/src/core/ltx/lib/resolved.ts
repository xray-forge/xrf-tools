import {
  LtxResolvedField,
  LtxResolvedFieldOrigin,
  LtxResolvedIndexEntry,
  LtxResolvedSection,
} from "@/core/bindings/types/xrf-ltx-inspect";
import { ESyntaxToken, ISyntaxSpan } from "@/core/syntax/lib";
import { ICodeLine, ICodeLineSource } from "@/core/ui/code/code-line";
import { Nullable } from "@/lib/types/general";

/** The unnamed section a resolution carries whatever was written before the first header in. */
const ROOT_SECTION: string = "";

/**
 * One section and where it sits in the assembled document, gap included.
 *
 * The section travels with its placement rather than beside it in a second array: everything downstream needs both -
 * the name to fetch a body by, the field count to tell a field line from the gap, the parents to draw the header - and
 * two arrays held in step is an invariant every reader would have to keep.
 */
interface IResolvedPlacement {
  entry: LtxResolvedIndexEntry;
  /** Line the header sits on. */
  firstLine: number;
  /** Last line the section owns, which is the gap after its fields. */
  lastLine: number;
}

/**
 * Where a resolution's lines sit, and the questions a viewport asks about that.
 *
 * The layout is everything the index decides and nothing the bodies do: how many lines there are, which section owns
 * each of them, and where a section's header sits. It is built once per resolution, and a page of bodies landing
 * produces a new source over the same layout rather than a new layout.
 */
export interface IResolvedLayout {
  /** How many lines the resolution comes to. */
  lineCount: number;
  /**
   * A view of these lines filled in with whichever bodies have arrived.
   *
   * @param bodies - Section bodies by name; absent names draw as the blank lines they will occupy.
   * @returns A source that builds a line only when the listing asks for one.
   */
  toSource(bodies?: Nullable<ReadonlyMap<string, LtxResolvedSection>>): ICodeLineSource;
  /**
   * Names of the sections holding any line in the given closed range, in the order the index lists them.
   *
   * This is what a viewport turns into a fetch, so it names every section the window touches even partly - a header
   * scrolled just off the top still has its fields on screen.
   */
  getSectionsInRange(firstLine: number, lastLine: number): Array<string>;
  /** The line the named section's header sits on, or null when this document does not hold it. */
  getSectionLine(name: string): Nullable<number>;
}

/**
 * Lay a resolution out, without building a line of it.
 *
 * The height comes from the index alone: `fieldCount` says how many lines a section will take before anything is
 * fetched, so the whole document is laid out on the first answer and a page arriving later changes what a line says
 * and never where it sits.
 *
 * Nothing here materialises a line, and that is the point at this scale. Anomaly's `configs\system.ltx` resolves to
 * 11,870 sections holding 527,000 fields - 551,000 lines. Building them costs a million objects and their collection,
 * per page of bodies that lands, for a screen that shows forty. What is built instead is one entry per section, and
 * `toSource` hands the listing a document it can read a line at a time.
 *
 * Numbering is the assembled document's own, running from one, because a resolved document is not a file and has no
 * authored numbering to keep. That makes a number unique here and equal to its position plus one, so an address into
 * the document is arithmetic - unlike an excerpt of a real file, which is why `VirtualizedLines` does not assume it.
 *
 * Filtering is the caller's: the Resolved view of an included config passes the sections that config declared. This
 * never re-orders them, since the order it is given is the dialect's own output order.
 *
 * @param sections - Every section the view shows, with the field count each one will take.
 * @returns Where those sections' lines sit, and what a viewport asks about them.
 */
export function toResolvedLayout(sections: ReadonlyArray<LtxResolvedIndexEntry>): IResolvedLayout {
  const placements: Array<IResolvedPlacement> = [];
  const sectionLines: Map<string, number> = new Map();

  let number: number = 1;

  for (const entry of sections) {
    const firstLine: number = number;

    // A header, a line per field the index counted, and the blank line that separates one section from the next.
    number += entry.fieldCount + 2;

    placements.push({ entry, firstLine, lastLine: number - 1 });

    // A section name is unique inside one resolution, but a malformed answer is not worth losing a document over, so
    // the first placement wins and a jump lands on the section a reader scrolls to first.
    if (!sectionLines.has(entry.name)) {
      sectionLines.set(entry.name, firstLine);
    }
  }

  const lineCount: number = number - 1;

  return {
    getSectionLine: (name: string): Nullable<number> => sectionLines.get(name) ?? null,
    getSectionsInRange: (firstLine: number, lastLine: number): Array<string> =>
      selectPlacements(placements, firstLine, lastLine),
    lineCount,
    toSource: (bodies?: Nullable<ReadonlyMap<string, LtxResolvedSection>>): ICodeLineSource => ({
      count: lineCount,
      getLine: (index: number): ICodeLine => toLine(placements, bodies, index),
      // Every source this layout hands out is a view of one document, and the placements are what say so.
      layout: placements,
      // The numbering is the document's own and runs from one, so an address is arithmetic rather than a lookup.
      indexOfLine: (line: number): number => (line >= 1 && line <= lineCount ? line - 1 : -1),
      widestNumber: lineCount,
    }),
  };
}

/**
 * The line at one position, built from the section that owns it.
 *
 * Every line of a resolved document is one of three things, and which one it is follows from where it sits inside its
 * section: the header, one of the fields the index counted, or the blank line that separates it from the next. A field
 * whose body has not arrived draws blank in the place it will occupy, so nothing moves when the page lands.
 *
 * @param placements - Every section of the document and where it sits.
 * @param bodies - Bodies that have arrived, by section name.
 * @param index - Position in the document, from zero.
 * @returns The line at that position.
 */
function toLine(
  placements: ReadonlyArray<IResolvedPlacement>,
  bodies: Nullable<ReadonlyMap<string, LtxResolvedSection>> | undefined,
  index: number
): ICodeLine {
  const number: number = index + 1;
  const placement: Nullable<IResolvedPlacement> = placements[findPlacement(placements, number)] ?? null;

  // A position outside the document, which a listing asks for while a shorter one replaces a taller.
  if (!placement || number < placement.firstLine) {
    return { number, spans: [] };
  }

  const entry: LtxResolvedIndexEntry = placement.entry;
  const offset: number = number - placement.firstLine;

  if (offset === 0) {
    return { number, spans: toHeaderSpans(entry) };
  }

  // Past the last field is the blank line the section ends with.
  if (offset > entry.fieldCount) {
    return { number, spans: [] };
  }

  const field: Nullable<LtxResolvedField> = bodies?.get(entry.name)?.fields[offset - 1] ?? null;

  return { number, spans: field ? toFieldSpans(field, entry.name === ROOT_SECTION) : [] };
}

/**
 * Why one resolved field holds what it holds, in the words `ltx inspect` uses for the same record.
 *
 * One wording for both surfaces on purpose: a person who reads `inherited from [wpn_base] in configs\weapons.ltx` in
 * a terminal and the same field in the explorer is being told about one fact, and two spellings of it would read as
 * two different findings.
 *
 * @param origin - What the resolution recorded about the field.
 * @param isRootSection - Whether the field belongs to the unnamed root section, whose per-field file is not exact.
 * @returns One line of prose, without punctuation of its own.
 */
export function describeResolvedFieldOrigin(origin: LtxResolvedFieldOrigin, isRootSection: boolean = false): string {
  switch (origin.kind) {
    case "declared":
      // The root section is the one section that merges across configs instead of colliding, so its per-field file is
      // the first config merged rather than the one the field is written in. Naming it would be a claim this record
      // cannot stand behind (`plans/configs-explorer.md`, caveats).
      return origin.file && !isRootSection ? `written here, in ${origin.file}` : "written here";

    case "inherited":
      return origin.file
        ? `inherited from [${origin.section}] in ${origin.file}`
        : `inherited from [${origin.section}]`;

    case "loaded":
      return origin.operation
        ? `set by ${origin.file} ('${origin.operation}', depth ${origin.depth})`
        : `set by ${origin.file} (depth ${origin.depth})`;

    default:
      // Reached when a resolution was produced without asking for provenance. Said rather than left blank, because a
      // view inventing "written here" for a field nothing recorded is the one wrong answer available here.
      return "origin not recorded";
  }
}

/**
 * A section header, as the resolved document draws it.
 *
 * Parents are coloured as names rather than judged: whether one resolves is the Authored view's question, asked of the
 * config that declared the header, and a section that reached this index resolved.
 */
function toHeaderSpans(entry: LtxResolvedIndexEntry): Array<ISyntaxSpan> {
  if (entry.name === ROOT_SECTION) {
    // `[]` is a section nobody wrote. What this line stands for is the fields written before the first header, and it
    // says so rather than inventing a name for them.
    return [{ text: "; fields written before any section header", token: ESyntaxToken.COMMENT }];
  }

  const spans: Array<ISyntaxSpan> = [{ text: `[${entry.name}]`, token: ESyntaxToken.SECTION }];

  entry.parents.forEach((parent: string, at: number) => {
    spans.push({ text: at === 0 ? ":" : ",", token: ESyntaxToken.OPERATOR });
    spans.push({ text: parent, token: ESyntaxToken.TYPE });
  });

  return spans;
}

/**
 * One resolved field, and the dim note saying where its value came from.
 *
 * The value is drawn plain rather than run through the lexical rules the Authored view uses. A resolved value has had
 * its comment stripped already, so a `;` inside one is data, and `LTX_RULES` would grey out the rest of the line as a
 * comment. The origin note is a comment because that is what it is - text this view adds beside the data - and
 * spelling it with a leading `;` keeps a copied line legal LTX instead of turning the note into a second value.
 */
function toFieldSpans(field: LtxResolvedField, isRootSection: boolean): Array<ISyntaxSpan> {
  return [
    { text: field.key, token: ESyntaxToken.KEY },
    { text: " ", token: ESyntaxToken.PLAIN },
    { text: "=", token: ESyntaxToken.OPERATOR },
    { text: ` ${field.value}`, token: ESyntaxToken.PLAIN },
    { text: `  ; ${describeResolvedFieldOrigin(field.origin, isRootSection)}`, token: ESyntaxToken.COMMENT },
  ];
}

/**
 * The first placement whose last line has reached the given line, which is the one holding it.
 *
 * Binary searched rather than scanned, because both callers run per rendered line and per scroll of a document that
 * reaches half a million lines across twelve thousand sections. Placements are laid out in ascending order and do not
 * overlap, so this is a lower bound and nothing else.
 *
 * @param placements - Sections as laid out, in ascending order.
 * @param line - Line number to find.
 * @returns Position of the placement, which is `placements.length` when the line sits past the last one.
 */
function findPlacement(placements: ReadonlyArray<IResolvedPlacement>, line: number): number {
  let low: number = 0;
  let high: number = placements.length;

  while (low < high) {
    const middle: number = (low + high) >>> 1;

    if (placements[middle].lastLine < line) {
      low = middle + 1;
    } else {
      high = middle;
    }
  }

  return low;
}

/**
 * The sections overlapping a closed range of line numbers.
 *
 * The walk starts at the placement holding `firstLine` and stops at the first header past `lastLine`.
 */
function selectPlacements(
  placements: ReadonlyArray<IResolvedPlacement>,
  firstLine: number,
  lastLine: number
): Array<string> {
  if (lastLine < firstLine) {
    return [];
  }

  const low: number = findPlacement(placements, firstLine);
  const names: Array<string> = [];

  for (let at: number = low; at < placements.length && placements[at].firstLine <= lastLine; at += 1) {
    names.push(placements[at].entry.name);
  }

  return names;
}
