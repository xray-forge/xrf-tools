import { describe, expect, it } from "@jest/globals";

import { LtxFileStructure, LtxResolvedIndexEntry } from "@/core/ipc/types/xrf-ltx-inspect";
import { ITreeNode } from "@/core/ui/tree/tree-node";

import {
  filterConfigsIndex,
  isListConfig,
  TConfigsIndexRow,
  toAuthoredIndex,
  toResolvedIndex,
} from "./configs-index-rows";

function getStructureOf(structure: Partial<LtxFileStructure>): LtxFileStructure {
  return {
    entryPoints: [],
    includes: [],
    parseError: null,
    path: "valid_item_sections.ltx",
    rootEntries: [],
    sections: [],
    ...structure,
  };
}

function getResolvedOf(name: string): LtxResolvedIndexEntry {
  return { fieldCount: 1, name, origin: null, parents: [] };
}

describe("configs index rows", () => {
  it("calls a config a list only when it declares keys and no section", () => {
    expect(isListConfig(null)).toBe(false);
    expect(isListConfig(getStructureOf({}))).toBe(false);
    expect(isListConfig(getStructureOf({ rootEntries: [{ hasValue: false, line: 1, name: "af_ear" }] }))).toBe(true);

    // Keys before a header in a config that has one is a defect the Problems panel reports, not a list: the index a
    // reader wants there is still the sections.
    expect(
      isListConfig(
        getStructureOf({
          rootEntries: [{ hasValue: true, line: 1, name: "loose" }],
          sections: [{ line: 2, name: "wpn_base", operation: "", parents: [], scheme: null }],
        })
      )
    ).toBe(false);
  });

  it("addresses a section by name and an entry by its line", () => {
    const sections: Array<ITreeNode<TConfigsIndexRow>> = toAuthoredIndex(
      getStructureOf({ sections: [{ line: 4, name: "wpn_base", operation: "", parents: [], scheme: null }] })
    );
    const entries: Array<ITreeNode<TConfigsIndexRow>> = toAuthoredIndex(
      getStructureOf({ rootEntries: [{ hasValue: false, line: 7, name: "af_ear" }] })
    );

    expect(sections[0].payload).toEqual({ kind: "section", name: "wpn_base" });
    expect(entries[0].payload).toEqual({ kind: "entry", line: 7, name: "af_ear" });
    expect(entries[0].label).toBe("af_ear");
  });

  it("names the unnamed section of a resolution rather than drawing an empty row", () => {
    const rows: Array<ITreeNode<TConfigsIndexRow>> = toResolvedIndex([getResolvedOf(""), getResolvedOf("wpn_base")]);

    expect(rows[0].label).toBe("(written before any section header)");
    // Addressed by the name the resolution holds it under, which is the empty one.
    expect(rows[0].payload).toEqual({ kind: "section", name: "" });
    expect(rows[1].label).toBe("wpn_base");
  });

  it("matches the name rather than the label, so the unnamed row leaves on any query", () => {
    const rows: Array<ITreeNode<TConfigsIndexRow>> = toResolvedIndex([getResolvedOf(""), getResolvedOf("wpn_base")]);

    expect(filterConfigsIndex(rows, "  ")).toHaveLength(2);
    expect(filterConfigsIndex(rows, "WPN")).toHaveLength(1);
    expect(filterConfigsIndex(rows, "written")).toHaveLength(0);
  });
});
