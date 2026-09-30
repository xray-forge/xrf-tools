import { describe, expect, it } from "@jest/globals";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { ELevelProblemRule, ILevelProblemSources, listLevelProblems } from "@/core/level/lib/problems";
import { ILevelSectorSkip } from "@/core/level/lib/sector/level-sector-report";
import { ISectorViews } from "@/core/level/lib/sector/level-sector-views";
import { EMPTY_LEVEL_SPAWN_REPORT } from "@/core/level/lib/spawn/level-spawn-report";
import { IEditorProblem } from "@/core/shell/editor/EditorProblemsPanel";
import { mockSurfaceDescriptor } from "@/fixtures/mocks/visual.mocks";

function sectorOf(sector: number, skipped: ISectorViews["skipped"]): ReadonlyArray<ILevelSectorSkip> {
  return skipped.map((skip) => ({ sector, skip }));
}

function mockSources(overrides: Partial<ILevelProblemSources> = {}): ILevelProblemSources {
  return { skipped: [], spawn: EMPTY_LEVEL_SPAWN_REPORT, surfaces: [], textures: [], ...overrides };
}

describe("listLevelProblems", () => {
  it("says nothing about a level that read cleanly", () => {
    expect(listLevelProblems(mockSources({ surfaces: [mockSurfaceDescriptor()] }))).toEqual([]);
  });

  it("names every texture the set could not answer for", () => {
    const problems: Array<IEditorProblem> = listLevelProblems(
      mockSources({ textures: [{ reason: "The file in the mounted roots is 2x2", reference: "trees\\frond" }] })
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

    expect(listLevelProblems(mockSources({ surfaces }))).toEqual([]);
  });

  it("names the table entry a class was not modelled for, by its index", () => {
    const surfaces: Array<XraySurfaceDescriptor> = [
      mockSurfaceDescriptor(),
      mockSurfaceDescriptor({ declaration: { class: "S_SET", kind: "unmodelled" } }),
    ];
    const problems: Array<IEditorProblem> = listLevelProblems(mockSources({ surfaces }));

    expect(problems).toHaveLength(1);
    expect(problems[0].subject).toBe("shader table entry 1");
    expect(problems[0].message).toContain("S_SET");
    expect(problems[0].rule).toBe(ELevelProblemRule.SURFACE);
  });

  it("says a library that could not be read once, with the reason it gave", () => {
    const surfaces: Array<XraySurfaceDescriptor> = [
      mockSurfaceDescriptor({ declaration: { kind: "unreadable", reason: "truncated chunk" } }),
    ];

    expect(listLevelProblems(mockSources({ surfaces }))[0].message).toContain("truncated chunk");
  });

  // Geometry the packer could not read is simply absent from the picture, which is the hardest kind of wrong to
  // notice: nothing is drawn oddly, something is not drawn at all.
  it("names what a resident sector could not pack, and why it counts as missing", () => {
    const sectors: ReadonlyArray<ILevelSectorSkip> = sectorOf(4, [
      { cause: "unsupported", drawable: 91, reason: "progressive geometry" },
    ]);
    const problems: Array<IEditorProblem> = listLevelProblems(mockSources({ skipped: sectors }));

    expect(problems[0].subject).toBe("sector 4, visual 91");
    expect(problems[0].message).toContain("does not model");
    expect(problems[0].rule).toBe(ELevelProblemRule.DRAWABLE);
  });

  it("names a spawned visual that could not be read, whose objects are absent", () => {
    const problems: Array<IEditorProblem> = listLevelProblems(
      mockSources({
        spawn: { ...EMPTY_LEVEL_SPAWN_REPORT, failures: [{ name: "physics\\box", reason: "Failed to read visual" }] },
      })
    );

    expect(problems).toEqual([
      {
        message: "Its objects are not drawn: Failed to read visual",
        rule: ELevelProblemRule.SPAWN,
        subject: "physics\\box",
      },
    ]);
  });

  // A spawn that cannot be read leaves the level without a spawned object, which the picture alone never says.
  it("names the spawn itself where it could not be read, before any visual", () => {
    const problems: Array<IEditorProblem> = listLevelProblems(
      mockSources({
        spawn: {
          ...EMPTY_LEVEL_SPAWN_REPORT,
          failure: "Failed to read 'spawns\\all.spawn': truncated chunk",
          failures: [{ name: "box", reason: "missing" }],
        },
      })
    );

    expect(problems.map((it) => it.subject)).toEqual(["spawns\\all.spawn", "box"]);
    expect(problems[0].message).toBe("No spawned object is drawn: Failed to read 'spawns\\all.spawn': truncated chunk");
  });

  it("orders the four sources, so one reading is always in the same place", () => {
    const problems: Array<IEditorProblem> = listLevelProblems({
      skipped: sectorOf(0, [{ cause: "malformed", drawable: 1, reason: "bad range" }]),
      spawn: { ...EMPTY_LEVEL_SPAWN_REPORT, failures: [{ name: "box", reason: "missing" }] },
      surfaces: [mockSurfaceDescriptor({ declaration: { kind: "undefined" } })],
      textures: [{ reason: "missing", reference: "stone" }],
    });

    expect(problems.map((it) => it.rule)).toEqual([
      ELevelProblemRule.TEXTURE,
      ELevelProblemRule.SURFACE,
      ELevelProblemRule.DRAWABLE,
      ELevelProblemRule.SPAWN,
    ]);
  });
});
