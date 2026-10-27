import { describe, expect, it } from "@jest/globals";
import { ITreeNode } from "@xrf/ui/tree";

import { TLevelSurfaceTreeRow, toLevelSurfaceTree } from "@/applications/level-viewer/lib/surface/level-surface-tree";
import { ILevelSurfaceSummary, listLevelSurfaces } from "@/core/level/lib/surface/level-surface-summary";
import { mockSurfaceDescriptor } from "@/core/visuals/fixtures/visual.mocks";

function mockSummaries(): Array<ILevelSurfaceSummary> {
  return listLevelSurfaces([
    mockSurfaceDescriptor({ shader: "lod_10", textures: ["trees\\pine"] }),
    mockSurfaceDescriptor({ shader: "lod_2", textures: ["trees\\birch"] }),
    mockSurfaceDescriptor({ shader: "lod_2", textures: [] }),
  ]);
}

function toLabels(nodes: ReadonlyArray<ITreeNode<TLevelSurfaceTreeRow>> = []): Array<string> {
  return nodes.map((it: ITreeNode<TLevelSurfaceTreeRow>) => it.label);
}

describe("toLevelSurfaceTree", () => {
  it("groups entries by shader as a reader orders them, each by its id and texture", () => {
    const tree: Array<ITreeNode<TLevelSurfaceTreeRow>> = toLevelSurfaceTree(mockSummaries(), "");

    expect(toLabels(tree)).toEqual(["lod_2", "lod_10"]);
    expect(tree[0].payload).toMatchObject({ count: 2, kind: "shader" });
    expect(toLabels(tree[0].children)).toEqual(["1 · trees\\birch", "2 · no texture"]);
  });

  it("keeps what a filter matches by shader, texture or the whole id", () => {
    expect(toLabels(toLevelSurfaceTree(mockSummaries(), "PINE"))).toEqual(["lod_10"]);
    expect(toLabels(toLevelSurfaceTree(mockSummaries(), "2")[0].children)).toEqual([
      "1 · trees\\birch",
      "2 · no texture",
    ]);
    expect(toLabels(toLevelSurfaceTree(mockSummaries(), "0"))).toEqual(["lod_10"]);
  });
});
