import { describe, expect, it } from "@jest/globals";

import { LtxAnchoredFinding, LtxFindingKind } from "@/core/ipc/types/xrf-ltx-inspect";
import { toFindingMark, toFindingMarks, toOrderedFindings } from "@/core/ltx/lib/findings";
import { ECodeLineMark } from "@/core/ui/code/code-line";
import { Nullable } from "@/lib/types/general";

const ENTRY: string = "configs\\system.ltx";
const FILE: string = "configs\\items\\w_base.ltx";

/** A finding carrying only what a case is about. */
function findingOf(kind: LtxFindingKind, file: Nullable<string>, line: Nullable<number>): LtxAnchoredFinding {
  return {
    engineBehaviour: null,
    entry: ENTRY,
    field: null,
    file,
    kind,
    line,
    message: `${kind} at ${line}`,
    section: null,
  };
}

describe("toFindingMark", () => {
  it("should draw as a warning only what the engine loads anyway", () => {
    // The distinction the gutter makes: a dialect diagnostic is the case where the game starts and does something the
    // author did not intend, and everything else stops it.
    expect(toFindingMark(findingOf("dialect", FILE, 1))).toBe(ECodeLineMark.WARNING);
    expect(toFindingMark(findingOf("parse", FILE, 1))).toBe(ECodeLineMark.ERROR);
    expect(toFindingMark(findingOf("scheme", FILE, 1))).toBe(ECodeLineMark.ERROR);
    expect(toFindingMark(findingOf("include", FILE, 1))).toBe(ECodeLineMark.ERROR);
  });
});

describe("toFindingMarks", () => {
  it("should mark only the lines of the config being drawn", () => {
    const marks: ReadonlyMap<number, ECodeLineMark> = toFindingMarks(
      [findingOf("scheme", FILE, 4), findingOf("scheme", "configs\\items\\w_other.ltx", 9)],
      FILE
    );

    expect([...marks.entries()]).toEqual([[4, ECodeLineMark.ERROR]]);
  });

  it("should keep a finding with no line out of the gutter", () => {
    // The anchor could not place it, and the Problems panel is the surface that can show a finding without a line.
    expect(toFindingMarks([findingOf("dialect", FILE, null)], FILE).size).toBe(0);
  });

  it("should let the worst finding on a line decide its mark, whichever order they arrive in", () => {
    const warningFirst: ReadonlyMap<number, ECodeLineMark> = toFindingMarks(
      [findingOf("dialect", FILE, 7), findingOf("scheme", FILE, 7)],
      FILE
    );
    const errorFirst: ReadonlyMap<number, ECodeLineMark> = toFindingMarks(
      [findingOf("scheme", FILE, 7), findingOf("dialect", FILE, 7)],
      FILE
    );

    expect(warningFirst.get(7)).toBe(ECodeLineMark.ERROR);
    expect(errorFirst.get(7)).toBe(ECodeLineMark.ERROR);
  });
});

describe("toOrderedFindings", () => {
  it("should read by config and then down the file", () => {
    // Not by severity: the reader is looking for a place to go, and ordering by severity makes one file appear twice.
    const ordered: Array<LtxAnchoredFinding> = toOrderedFindings([
      findingOf("scheme", "configs\\z.ltx", 2),
      findingOf("scheme", FILE, 40),
      findingOf("dialect", FILE, 4),
    ]);

    expect(ordered.map((finding: LtxAnchoredFinding) => [finding.file, finding.line])).toEqual([
      [FILE, 4],
      [FILE, 40],
      ["configs\\z.ltx", 2],
    ]);
  });
});
