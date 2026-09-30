import { describe, expect, it } from "@jest/globals";

import { ELevelSpawnCategory, LevelSpawnObjectsDescription } from "@/core/ipc/types/xrf-app";
import {
  listLevelSpawnGroupIds,
  TLevelSpawnTreeRow,
  toLevelSpawnCategoryId,
  toLevelSpawnObjectId,
  toLevelSpawnTree,
} from "@/core/level/lib/spawn/level-spawn-tree";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { mockLevelSpawnObject } from "@/fixtures/mocks/level.mocks";

/** Three crates and a barrel among the props, a medkit among the items, nothing among the weapons or lamps. */
function mockDescription(): LevelSpawnObjectsDescription {
  return {
    objects: [
      mockLevelSpawnObject({ index: 0, name: "crate_10", section: "box", visual: 0 }),
      mockLevelSpawnObject({ index: 1, name: "crate_2", section: "box", visual: 0 }),
      mockLevelSpawnObject({ index: 2, name: "barrel", section: "barrel", visual: 1 }),
      mockLevelSpawnObject({
        category: ELevelSpawnCategory.ITEMS,
        index: 3,
        name: "medkit",
        section: "medkit",
        visual: 2,
      }),
      mockLevelSpawnObject({ index: 4, name: "crate_1", section: "box", visual: 0 }),
    ],
    visuals: ["physics\\box", "physics\\barrel", "dynamics\\medkit"],
  };
}

function toLabels(nodes: ReadonlyArray<ITreeNode<TLevelSpawnTreeRow>> = []): Array<string> {
  return nodes.map((it: ITreeNode<TLevelSpawnTreeRow>) => it.label);
}

describe("toLevelSpawnTree", () => {
  it("groups by category then section, counting each group, sections and objects by name as a reader orders them", () => {
    const tree: Array<ITreeNode<TLevelSpawnTreeRow>> = toLevelSpawnTree(mockDescription(), "");
    const [props] = tree;
    const box: ITreeNode<TLevelSpawnTreeRow> = props.children?.[1] as ITreeNode<TLevelSpawnTreeRow>;

    expect(toLabels(tree)).toEqual(["Props", "Items"]);
    expect(props.id).toBe(toLevelSpawnCategoryId(ELevelSpawnCategory.PROPS));
    expect(props.payload).toMatchObject({ count: 4, kind: "category" });
    expect(toLabels(props.children)).toEqual(["barrel", "box"]);
    expect(box.payload).toMatchObject({ count: 3, kind: "section" });
    expect(toLabels(box.children)).toEqual(["crate_1", "crate_2", "crate_10"]);
    expect(box.children?.[0].id).toBe(toLevelSpawnObjectId(4));
  });

  it("keeps what a filter matches by name, section or visual, and no group left empty", () => {
    expect(toLabels(toLevelSpawnTree(mockDescription(), "MEDKIT"))).toEqual(["Items"]);
    expect(toLabels(toLevelSpawnTree(mockDescription(), "barrel")[0].children)).toEqual(["barrel"]);
    expect(toLevelSpawnTree(mockDescription(), "physics\\box")[0].payload).toMatchObject({ count: 3 });
    expect(toLevelSpawnTree(mockDescription(), "nothing")).toEqual([]);
  });

  it("names every group, which a filter opens", () => {
    const tree: Array<ITreeNode<TLevelSpawnTreeRow>> = toLevelSpawnTree(mockDescription(), "");

    expect(listLevelSpawnGroupIds(tree)).toEqual([
      "category:props",
      "section:props:barrel",
      "section:props:box",
      "category:items",
      "section:items:medkit",
    ]);
  });
});
