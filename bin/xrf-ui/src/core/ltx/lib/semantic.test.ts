import { describe, expect, it } from "@jest/globals";

import { LtxFileStructure } from "@/core/bindings/types/xrf-ltx-inspect";
import { toDocumentLines } from "@/core/ltx/lib/semantic";
import { ESyntaxToken, ISyntaxSpan } from "@/core/syntax/lib";
import { ECodeLineMark, ICodeLine } from "@/core/ui/code/code-line";

/** A structure carrying only what a case is about, with the rest of the record empty. */
function structureOf(structure: Partial<LtxFileStructure>): LtxFileStructure {
  return {
    entryPoints: [],
    includes: [],
    parseError: null,
    path: "configs\\system.ltx",
    sections: [],
    ...structure,
  };
}

/** Every span of one line, in order, as `token:text` pairs a failure can be read from. */
function spansOf(lines: Array<ICodeLine>, line: number): Array<string> {
  return (lines[line - 1]?.spans ?? []).map((span: ISyntaxSpan) => `${span.token}:${span.text}`);
}

describe("toDocumentLines", () => {
  it("should number every line from one, so a gutter mark and a finding agree", () => {
    const lines: Array<ICodeLine> = toDocumentLines(["[a]", "key = 1", ""], null);

    expect(lines.map((line: ICodeLine) => line.number)).toEqual([1, 2, 3]);
  });

  it("should colour a line the backend says nothing about with the lexical layer alone", () => {
    const lines: Array<ICodeLine> = toDocumentLines(["cost = 100 ; price"], structureOf({}));

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
    const lines: Array<ICodeLine> = toDocumentLines(
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
    const lines: Array<ICodeLine> = toDocumentLines(
      ["![wpn_ak74]"],
      structureOf({
        sections: [{ line: 1, name: "wpn_ak74", operation: "!", parents: [], scheme: null }],
      })
    );

    expect(spansOf(lines, 1)).toEqual([`${ESyntaxToken.SECTION}:![wpn_ak74]`]);
  });

  it("should keep the padding of a section name, which the engine treats as part of it", () => {
    const lines: Array<ICodeLine> = toDocumentLines(
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
    const lines: Array<ICodeLine> = toDocumentLines(
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
    const lines: Array<ICodeLine> = toDocumentLines(
      ["[a]", "!!!broken"],
      structureOf({ parseError: { line: 2, column: 1, message: "unexpected token" } }),
      new Map([[2, ECodeLineMark.ERROR]])
    );

    expect(lines[0].mark).toBeUndefined();
    expect(lines[1].mark).toBe(ECodeLineMark.ERROR);
  });

  it("should ignore a structure describing a line the text does not hold", () => {
    // Two reads of one file disagreeing is a defect worth surviving rather than crashing a viewer on.
    const lines: Array<ICodeLine> = toDocumentLines(
      ["[a]"],
      structureOf({
        sections: [{ line: 9, name: "a", operation: "", parents: [], scheme: null }],
      })
    );

    expect(lines).toHaveLength(1);
    expect(spansOf(lines, 1)).toEqual([`${ESyntaxToken.SECTION}:[a]`]);
  });
});
