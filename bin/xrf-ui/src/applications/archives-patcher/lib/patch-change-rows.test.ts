import { describe, expect, it } from "@jest/globals";

import { IPatchChangeRow, toPatchChangeRows } from "@/applications/archives-patcher/lib/patch-change-rows";
import { ArchivePatchChange, ArchivePatchResult } from "@/core/ipc/types/xrf-pack";

/** The table every side indexes into: a volume set at 0, a loose tree at 1. */
const ORIGINS: ArchivePatchResult["origins"] = [
  { kind: "archive", path: "C:\\db" },
  { kind: "directory", root: "C:\\t" },
];

function archived(size: number): ArchivePatchChange["base"] {
  return { origin: 0, size };
}

function loose(size: number): ArchivePatchChange["target"] {
  return { origin: 1, size };
}

function result(patch: Partial<ArchivePatchResult> = {}): ArchivePatchResult {
  return { added: [], modified: [], unchanged: 99, origins: ORIGINS, ...patch } as ArchivePatchResult;
}

describe("toPatchChangeRows", () => {
  it("carries both classes and leaves the unchanged count out of the table", () => {
    const rows: Array<IPatchChangeRow> = toPatchChangeRows(
      result({
        added: [{ name: "a.ltx", class: "added", base: null, target: loose(10) }],
        modified: [{ name: "b.ltx", class: "modified", base: archived(20), target: loose(30) }],
      })
    );

    expect(rows.map((row) => row.class)).toEqual(["added", "modified"]);
    expect(rows).toHaveLength(2);
  });

  it("sizes a row by the target, which is the payload the patch writes", () => {
    // The two sides of a modification differ, and the one worth showing is what lands in the volume.
    const rows: Array<IPatchChangeRow> = toPatchChangeRows(
      result({ modified: [{ name: "b.ltx", class: "modified", base: archived(20), target: loose(30) }] })
    );

    expect(rows[0]?.size).toBe(30);
  });

  it("reports the size as a number rather than a formatted string, so the column sorts", () => {
    // "9 KB" sorts above "1 MB" as text, which is exactly the ordering a size column exists to avoid.
    const rows: Array<IPatchChangeRow> = toPatchChangeRows(
      result({
        added: [
          { name: "small.ltx", class: "added", base: null, target: loose(9_000) },
          { name: "large.dds", class: "added", base: null, target: loose(1_000_000) },
        ],
      })
    );

    expect(rows.map((row) => row.size).sort((left, right) => left - right)).toEqual([9_000, 1_000_000]);
  });

  it("resolves each row against the report's origin table", () => {
    const rows: Array<IPatchChangeRow> = toPatchChangeRows(
      result({
        added: [{ name: "a.ltx", class: "added", base: null, target: loose(10) }],
        modified: [{ name: "b.ltx", class: "modified", base: archived(20), target: loose(30) }],
      })
    );

    expect(rows[0]?.origin).toBe("C:\\t");
    expect(rows[1]?.origin).toBe("C:\\t");
  });

  it("hands rows sharing an origin the same string rather than a copy each", () => {
    // The reason the table exists: the origin is resolved once per report, not once per row.
    const rows: Array<IPatchChangeRow> = toPatchChangeRows(
      result({
        added: [
          { name: "a.ltx", class: "added", base: null, target: loose(10) },
          { name: "b.ltx", class: "added", base: null, target: loose(20) },
        ],
      })
    );

    expect(rows[0]?.origin).toBe(rows[1]?.origin);
  });

  it("leaves the origin empty when an index names nothing, rather than rendering undefined", () => {
    // A row is worth showing even if the table cannot explain where it came from.
    const rows: Array<IPatchChangeRow> = toPatchChangeRows(
      result({
        origins: [],
        added: [{ name: "a.ltx", class: "added", base: null, target: loose(10) }],
      })
    );

    expect(rows[0]?.origin).toBe("");
  });
});
