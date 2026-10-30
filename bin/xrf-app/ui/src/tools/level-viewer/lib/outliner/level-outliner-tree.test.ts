import { describe, expect, it } from "@jest/globals";
import { ELevelObjectId, ELevelObjectKind, LevelObject, LevelObjectsDescription } from "@xrf/ipc/types/xrf-app-levels";
import { ITreeNode } from "@xrf/ui/tree";

import { mockLevelObject, mockLevelVolumeObject } from "@/core/level/fixtures/level.mocks";
import {
  ILevelOutlinerFilter,
  listLevelOutlinerGroupIds,
  listLevelOutlinerObjects,
  listLevelOutlinerParentIds,
  TLevelOutlinerRow,
  toLevelOutlinerKindId,
  toLevelOutlinerObjectId,
  toLevelOutlinerTree,
} from "@/tools/level-viewer/lib/outliner/level-outliner-tree";

function mockSpawned(index: number, name: string, section: string, visual: number): LevelObject {
  return mockLevelObject({ id: { index, source: ELevelObjectId.SPAWN }, name }, { section, visual });
}

/** Three crates and a barrel among the props, a medkit among the items, and a restrictor. */
function mockDescription(): LevelObjectsDescription {
  return {
    objects: [
      mockSpawned(0, "crate_10", "box", 0),
      mockSpawned(1, "crate_2", "box", 0),
      mockSpawned(2, "barrel", "barrel", 1),
      { ...mockSpawned(3, "medkit", "medkit", 2), kind: ELevelObjectKind.ITEMS },
      mockSpawned(4, "crate_1", "box", 0),
      mockLevelVolumeObject(5),
    ],
    visuals: ["physics\\box", "physics\\barrel", "dynamics\\medkit"],
  };
}

function filter(text: string = "", kinds: Array<ELevelObjectKind> = []): ILevelOutlinerFilter {
  return { kinds: new Set(kinds), text };
}

function toLabels(nodes: ReadonlyArray<ITreeNode<TLevelOutlinerRow>> = []): Array<string> {
  return nodes.map((it: ITreeNode<TLevelOutlinerRow>) => it.label);
}

describe("toLevelOutlinerTree", () => {
  it("groups by family, kind and a spawned kind's section, counting each, sections and objects by name", () => {
    const tree: Array<ITreeNode<TLevelOutlinerRow>> = toLevelOutlinerTree(mockDescription(), filter());
    const [spawned, volumes] = tree;
    const props: ITreeNode<TLevelOutlinerRow> = spawned?.children?.[0] as ITreeNode<TLevelOutlinerRow>;
    const box: ITreeNode<TLevelOutlinerRow> = props.children?.[1] as ITreeNode<TLevelOutlinerRow>;

    expect(toLabels(tree)).toEqual(["Spawned", "Logic volumes"]);
    expect(spawned?.payload).toMatchObject({ count: 5, kind: "group" });
    expect(toLabels(spawned?.children)).toEqual(["Props", "Items"]);
    expect(props.id).toBe(toLevelOutlinerKindId(ELevelObjectKind.PROPS));
    expect(toLabels(props.children)).toEqual(["barrel", "box"]);
    expect(box.payload).toMatchObject({ count: 3, kind: "section" });
    expect(toLabels(box.children)).toEqual(["crate_1", "crate_2", "crate_10"]);
    expect(box.children?.[0]?.id).toBe("object:spawn:4");
    // A volume has no section: its objects stand under its kind.
    expect(toLabels(volumes?.children?.[0]?.children)).toEqual(["restrictor_5"]);
  });

  it("keeps what a filter matches by name, section or visual, and by kind, and no group left empty", () => {
    expect(toLabels(toLevelOutlinerTree(mockDescription(), filter("MEDKIT"))[0]?.children)).toEqual(["Items"]);
    expect(toLevelOutlinerTree(mockDescription(), filter("physics\\box"))[0]?.payload).toMatchObject({ count: 3 });
    expect(toLabels(toLevelOutlinerTree(mockDescription(), filter("", [ELevelObjectKind.RESTRICTORS])))).toEqual([
      "Logic volumes",
    ]);
    expect(toLevelOutlinerTree(mockDescription(), filter("nothing"))).toEqual([]);
  });

  it("names every group, which a filter opens, and every object in the order listed", () => {
    const tree: Array<ITreeNode<TLevelOutlinerRow>> = toLevelOutlinerTree(mockDescription(), filter());

    expect(listLevelOutlinerGroupIds(tree)).toEqual([
      "group:spawned",
      "kind:props",
      "section:props:barrel",
      "section:props:box",
      "kind:items",
      "section:items:medkit",
      "group:volumes",
      "kind:restrictors",
    ]);
    expect(listLevelOutlinerObjects(tree).map((object: LevelObject) => object.name)).toEqual([
      "barrel",
      "crate_1",
      "crate_2",
      "crate_10",
      "medkit",
      "restrictor_5",
    ]);
  });

  it("names the rows above an object, which open to show it", () => {
    const [crate] = mockDescription().objects;

    expect(listLevelOutlinerParentIds(crate as LevelObject)).toEqual([
      "group:spawned",
      "kind:props",
      "section:props:box",
    ]);
    expect(listLevelOutlinerParentIds(mockLevelVolumeObject(5))).toEqual(["group:volumes", "kind:restrictors"]);
    expect(toLevelOutlinerObjectId(mockLevelVolumeObject(5))).toBe("object:spawn:5");
  });
});
