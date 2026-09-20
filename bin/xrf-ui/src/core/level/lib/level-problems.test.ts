import { describe, expect, it } from "@jest/globals";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { ELevelProblemRule, listLevelProblems } from "@/core/level/lib/level-problems";
import { ILoadedSector } from "@/core/level/lib/level-sector-set";
import { ISectorViews } from "@/core/level/lib/level-sector-views";
import { IEditorProblem } from "@/core/shell/editor/EditorProblemsPanel";
import { mockSurfaceDescriptor } from "@/fixtures/mocks/visual.mocks";

function sectorOf(sector: number, skipped: ISectorViews["skipped"]): ReadonlyMap<number, ILoadedSector> {
  return new Map([[sector, { views: { skipped } } as ILoadedSector]]);
}

describe("listLevelProblems", () => {
  it("says nothing about a level that read cleanly", () => {
    expect(listLevelProblems([], [mockSurfaceDescriptor()], new Map())).toEqual([]);
  });

  it("names every texture the set could not answer for", () => {
    const problems: Array<IEditorProblem> = listLevelProblems(
      [{ reason: "The file in the mounted roots is 2x2", reference: "trees\\frond" }],
      [],
      new Map()
    );

    expect(problems).toEqual([
      {
        message: "The file in the mounted roots is 2x2",
        rule: ELevelProblemRule.TEXTURE,
        subject: "trees\\frond",
      },
    ]);
  });

  // An entry naming no shader is not a problem: a level's table carries them, and holding their place is what keeps
  // every later entry on the surface it belongs to.
  it("passes over a table entry that declares nothing", () => {
    const surfaces: Array<XraySurfaceDescriptor> = [mockSurfaceDescriptor({ declaration: { kind: "undeclared" } })];

    expect(listLevelProblems([], surfaces, new Map())).toEqual([]);
  });

  it("names the table entry a class was not modelled for, by its index", () => {
    const surfaces: Array<XraySurfaceDescriptor> = [
      mockSurfaceDescriptor(),
      mockSurfaceDescriptor({ declaration: { class: "S_SET", kind: "unmodelled" } }),
    ];
    const problems: Array<IEditorProblem> = listLevelProblems([], surfaces, new Map());

    expect(problems).toHaveLength(1);
    expect(problems[0].subject).toBe("shader table entry 1");
    expect(problems[0].message).toContain("S_SET");
    expect(problems[0].rule).toBe(ELevelProblemRule.SURFACE);
  });

  it("says a library that could not be read once, with the reason it gave", () => {
    const surfaces: Array<XraySurfaceDescriptor> = [
      mockSurfaceDescriptor({ declaration: { kind: "unreadable", reason: "truncated chunk" } }),
    ];

    expect(listLevelProblems([], surfaces, new Map())[0].message).toContain("truncated chunk");
  });

  // Geometry the packer could not read is simply absent from the picture, which is the hardest kind of wrong to
  // notice: nothing is drawn oddly, something is not drawn at all.
  it("names what a resident sector could not pack, and why it counts as missing", () => {
    const sectors: ReadonlyMap<number, ILoadedSector> = sectorOf(4, [
      { cause: "unsupported", drawable: 91, reason: "progressive geometry" },
    ]);
    const problems: Array<IEditorProblem> = listLevelProblems([], [], sectors);

    expect(problems[0].subject).toBe("sector 4, visual 91");
    expect(problems[0].message).toContain("does not model");
    expect(problems[0].rule).toBe(ELevelProblemRule.DRAWABLE);
  });

  it("orders the three sources, so one reading is always in the same place", () => {
    const problems: Array<IEditorProblem> = listLevelProblems(
      [{ reason: "missing", reference: "stone" }],
      [mockSurfaceDescriptor({ declaration: { kind: "undefined" } })],
      sectorOf(0, [{ cause: "malformed", drawable: 1, reason: "bad range" }])
    );

    expect(problems.map((it) => it.rule)).toEqual([
      ELevelProblemRule.TEXTURE,
      ELevelProblemRule.SURFACE,
      ELevelProblemRule.DRAWABLE,
    ]);
  });
});
