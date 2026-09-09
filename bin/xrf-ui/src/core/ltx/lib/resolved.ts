import {
  LtxResolvedField,
  LtxResolvedFieldOrigin,
  LtxResolvedIndex,
  LtxResolvedIndexEntry,
  LtxResolvedSection,
} from "@/core/bindings/types/xrf-ltx-inspect";
import { ESyntaxToken, ISyntaxSpan } from "@/core/syntax/lib";
import { ICodeLine } from "@/core/ui/code/code-line";
import { Nullable } from "@/lib/types/general";

/** The unnamed section a resolution carries whatever was written before the first header in. */
const ROOT_SECTION: string = "";

/**
 * Where one section sits in the assembled document, gap included.
 *
 * Kept per section rather than recomputed from the index, because the index is what the layout was derived from and
 * deriving it twice is how the two answers drift apart.
 */
interface IResolvedPlacement {
  name: string;
  /** Line the header sits on. */
  firstLine: number;
  /** Last line the section owns, which is the gap after its fields. */
  lastLine: number;
}

/**
 * One resolution laid out as lines, and the two questions a viewport asks about that layout.
 */
export interface IResolvedDocument {
  /** Every line of the resolved document, numbered from one. */
  lines: ReadonlyArray<ICodeLine>;
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
 * Lay a resolution out as lines, filling in whichever section bodies have arrived.
 *
 * The height comes from the index alone: `fieldCount` says how many lines a section will take before anything is
 * fetched, so the whole document is laid out on the first answer and a page arriving later changes what a line says
 * and never where it sits. A section whose body has not arrived renders its fields as blank lines in the exact
 * positions they will occupy.
 *
 * Numbering is the assembled document's own, running from one, because a resolved document is not a file and has no
 * authored numbering to keep. That makes a number unique here, so keying anything by it is well defined - unlike an
 * excerpt of a real file, which is why `VirtualizedLines` does not assume uniqueness.
 *
 * Filtering is the caller's: the Resolved view of an included config passes an index whose `sections` were narrowed to
 * what that config declared. This builder never re-orders `index.sections`, which is the dialect's own output order.
 *
 * @param index - Every section the resolution holds, with the field count each one will take.
 * @param bodies - Section bodies that have arrived so far, by section name; absent names render as placeholders.
 * @returns The lines, and the queries a viewport and a sections panel address them by.
 */
export function toResolvedDocument(
  index: LtxResolvedIndex,
  bodies?: Nullable<ReadonlyMap<string, LtxResolvedSection>>
): IResolvedDocument {
  const lines: Array<ICodeLine> = [];
  const placements: Array<IResolvedPlacement> = [];
  const sectionLines: Map<string, number> = new Map();

  let number: number = 1;

  for (const entry of index.sections) {
    const firstLine: number = number;
    const body: Nullable<LtxResolvedSection> = bodies?.get(entry.name) ?? null;

    lines.push({ number: number++, spans: toHeaderSpans(entry) });

    // The index decides how many field lines there are, not the body: a body is what fills them in, and letting it
    // decide would move every line below it the moment a page arrived.
    for (let at: number = 0; at < entry.fieldCount; at += 1) {
      const field: Nullable<LtxResolvedField> = body?.fields[at] ?? null;

      lines.push({ number: number++, spans: field ? toFieldSpans(field, entry.name === ROOT_SECTION) : [] });
    }

    lines.push({ number: number++, spans: [] });

    placements.push({ firstLine, lastLine: number - 1, name: entry.name });

    // A section name is unique inside one resolution, but a malformed answer is not worth losing a document over, so
    // the first placement wins and a jump lands on the section a reader scrolls to first.
    if (!sectionLines.has(entry.name)) {
      sectionLines.set(entry.name, firstLine);
    }
  }

  return {
    getSectionLine: (name: string): Nullable<number> => sectionLines.get(name) ?? null,
    getSectionsInRange: (firstLine: number, lastLine: number): Array<string> =>
      selectPlacements(placements, firstLine, lastLine),
    lines,
  };
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
 * The sections overlapping a closed range of line numbers.
 *
 * Binary searched rather than scanned, because this runs on every scroll of a document that reaches 300,000 lines and
 * 23,000 sections. Placements are laid out in ascending order and do not overlap, so the first one whose last line has
 * reached `firstLine` is the first answer and the walk stops at the first header past `lastLine`.
 */
function selectPlacements(
  placements: ReadonlyArray<IResolvedPlacement>,
  firstLine: number,
  lastLine: number
): Array<string> {
  if (lastLine < firstLine) {
    return [];
  }

  let low: number = 0;
  let high: number = placements.length;

  while (low < high) {
    const middle: number = (low + high) >>> 1;

    if (placements[middle].lastLine < firstLine) {
      low = middle + 1;
    } else {
      high = middle;
    }
  }

  const names: Array<string> = [];

  for (let at: number = low; at < placements.length && placements[at].firstLine <= lastLine; at += 1) {
    names.push(placements[at].name);
  }

  return names;
}
