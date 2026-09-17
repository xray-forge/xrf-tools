import { describe, expect, it } from "@jest/globals";

import { LtxFileStructure } from "@/core/ipc/types/xrf-ltx-inspect";
import { toDocumentLineSource } from "@/core/ltx/lib/semantic";
import { ESyntaxToken, ISyntaxSpan } from "@/core/syntax/lib";
import { ECodeLineMark, ICodeLine, ICodeLineSource } from "@/core/ui/code/code-line";

/** A structure carrying only what a case is about, with the rest of the record empty. */
function structureOf(structure: Partial<LtxFileStructure>): LtxFileStructure {
  return {
    entryPoints: [],
    includes: [],
    parseError: null,
    path: "configs\\system.ltx",
    rootEntries: [],
    sections: [],
    ...structure,
  };
}

/**
 * Every line of a document, since these cases are about what one line ends up as rather than about drawing a window.
 *
 * @param text - The config's lines.
 * @param structure - What the backend made of it.
 * @param marks - Gutter marks by line.
 * @returns One entry per line.
 */
function documentLines(
  text: Array<string>,
  structure: LtxFileStructure | null,
  marks?: ReadonlyMap<number, ECodeLineMark>
): Array<ICodeLine> {
  const source: ICodeLineSource = toDocumentLineSource(text, structure, marks);

  return Array.from({ length: source.count }, (_, index: number) => source.getLine(index));
}

/** Every span of one line, in order, as `token:text` pairs a failure can be read from. */
function spansOf(lines: Array<ICodeLine>, line: number): Array<string> {
  return (lines[line - 1]?.spans ?? []).map((span: ISyntaxSpan) => `${span.token}:${span.text}`);
}

describe("toDocumentLineSource", () => {
  it("should number every line from one, so a gutter mark and a finding agree", () => {
    const lines: Array<ICodeLine> = documentLines(["[a]", "key = 1", ""], null);

    expect(lines.map((line: ICodeLine) => line.number)).toEqual([1, 2, 3]);
  });

  it("should colour a line the backend says nothing about with the lexical layer alone", () => {
    const lines: Array<ICodeLine> = documentLines(["cost = 100 ; price"], structureOf({}));

    expect(spansOf(lines, 1)).toEqual([
      `${ESyntaxToken.KEY}:cost`,
      `${ESyntaxToken.PLAIN}: `,
      `${ESyntaxToken.OPERATOR}:=`,
      `${ESyntaxToken.PLAIN}: `,
      `${ESyntaxToken.NUMBER}:100`,
      `${ESyntaxToken.PLAIN}: `,
      `${ESyntaxToken.COMMENT}:; price`,
    ]);
  });

  it("should draw a parent the resolution holds differently from one it does not", () => {
    // The whole reason the semantic layer exists on a header. Both names are spelled identically and a regex
    // cannot tell them apart, while the engine silently inherits nothing from the second.
    const lines: Array<ICodeLine> = documentLines(
      ["[wpn_ak74]:wpn_base,missing_base"],
      structureOf({
        sections: [
          {
            line: 1,
            name: "wpn_ak74",
            operation: "",
            parents: [
              { name: "wpn_base", resolves: true },
              { name: "missing_base", resolves: false },
            ],
            scheme: null,
          },
        ],
      })
    );

    expect(spansOf(lines, 1)).toEqual([
      `${ESyntaxToken.SECTION}:[wpn_ak74]`,
      `${ESyntaxToken.OPERATOR}::`,
      `${ESyntaxToken.TYPE}:wpn_base`,
      `${ESyntaxToken.OPERATOR}:,`,
      `${ESyntaxToken.PLAIN}:missing_base`,
    ]);
  });

  it("should keep a header's operation prefix inside the section span", () => {
    // `![wpn_ak74]` is one header under the patch dialect, not a section named `!wpn_ak74`.
    const lines: Array<ICodeLine> = documentLines(
      ["![wpn_ak74]"],
      structureOf({
        sections: [{ line: 1, name: "wpn_ak74", operation: "!", parents: [], scheme: null }],
      })
    );

    expect(spansOf(lines, 1)).toEqual([`${ESyntaxToken.SECTION}:![wpn_ak74]`]);
  });

  it("should keep the padding of a section name, which the engine treats as part of it", () => {
    const lines: Array<ICodeLine> = documentLines(
      ["[  wpn_base  ]"],
      structureOf({
        sections: [{ line: 1, name: "  wpn_base  ", operation: "", parents: [], scheme: null }],
      })
    );

    expect(spansOf(lines, 1)).toEqual([`${ESyntaxToken.SECTION}:[  wpn_base  ]`]);
  });

  it("should colour an include by what it reached without marking it", () => {
    // The colour is this layer's answer and the mark is not: a finding is what marks a line, so a gutter never has two
    // sources disagreeing about the same one.
    const lines: Array<ICodeLine> = documentLines(
      ['#include "items\\w_*.ltx"', '#include "missing.ltx"'],
      structureOf({
        includes: [
          { line: 1, statement: "items\\w_*.ltx", resolved: ["configs\\items\\w_ak74.ltx"] },
          { line: 2, statement: "missing.ltx", resolved: [] },
        ],
      })
    );

    expect(lines[0].mark).toBeUndefined();
    expect(lines[1].mark).toBeUndefined();
    expect(spansOf(lines, 1)).toEqual([`${ESyntaxToken.DIRECTIVE}:#include "items\\w_*.ltx"`]);
    expect(spansOf(lines, 2)).toEqual([`${ESyntaxToken.PLAIN}:#include "missing.ltx"`]);
  });

  it("should draw the marks it was given on the lines they name", () => {
    const lines: Array<ICodeLine> = documentLines(
      ["[a]", "!!!broken"],
      structureOf({ parseError: { line: 2, column: 1, message: "unexpected token" } }),
      new Map([[2, ECodeLineMark.ERROR]])
    );

    expect(lines[0].mark).toBeUndefined();
    expect(lines[1].mark).toBe(ECodeLineMark.ERROR);
  });

  it("should ignore a structure describing a line the text does not hold", () => {
    // Two reads of one file disagreeing is a defect worth surviving rather than crashing a viewer on.
    const lines: Array<ICodeLine> = documentLines(
      ["[a]"],
      structureOf({
        sections: [{ line: 9, name: "a", operation: "", parents: [], scheme: null }],
      })
    );

    expect(lines).toHaveLength(1);
    expect(spansOf(lines, 1)).toEqual([`${ESyntaxToken.SECTION}:[a]`]);
  });

  it("should address a line by the number it displays, and no other", () => {
    const source: ICodeLineSource = toDocumentLineSource(["[a]", "key = 1"], null);

    expect(source.indexOfLine(2)).toBe(1);
    expect(source.indexOfLine(0)).toBe(-1);
    expect(source.indexOfLine(3)).toBe(-1);
  });

  it("should colour a config far larger than one pass over it would be worth", () => {
    // What a line is coloured by is its own characters, so how long the file is decides nothing. Anomaly ships
    // `textures.ltx` at some 700 KB, which a document coloured in one pass had to give up on entirely.
    const text: Array<string> = new Array(80_000).fill("cost = 1000");
    const source: ICodeLineSource = toDocumentLineSource(text, null);

    expect(source.count).toBe(80_000);
    expect(spansOf([source.getLine(79_999)], 1)).toContain(`${ESyntaxToken.KEY}:cost`);
  });
});
