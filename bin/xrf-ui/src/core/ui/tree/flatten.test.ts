import { describe, expect, it } from "@jest/globals";

import { flattenTree, IFlatTreeRow } from "@/core/ui/tree/flatten";
import { IPathTreeItem, LOGICAL_PATH_SEPARATOR, parsePathTree, toDirectoryItemId } from "@/core/ui/tree/path-tree";

/** Joins segments with the engine separator, so no test carries an escaped literal of it. */
function at(...segments: Array<string>): string {
  return segments.join(LOGICAL_PATH_SEPARATOR);
}

function tree(): Array<IPathTreeItem<string>> {
  return parsePathTree(
    [
      { path: at("meshes", "weapons", "ak74.ogf"), payload: "a" },
      { path: at("meshes", "weapons", "pm.ogf"), payload: "b" },
      { path: at("meshes", "actor.ogf"), payload: "c" },
      { path: "readme.txt", payload: "d" },
    ],
    LOGICAL_PATH_SEPARATOR
  );
}

function labels(rows: Array<IFlatTreeRow<string>>): Array<string> {
  return rows.map((it: IFlatTreeRow<string>) => it.item.label);
}

describe("flattenTree", () => {
  it("shows only root rows when nothing is expanded", () => {
    const rows: Array<IFlatTreeRow<string>> = flattenTree(tree(), new Set());

    expect(labels(rows)).toEqual(["meshes", "readme.txt"]);
  });

  it("reveals children of an expanded directory in display order", () => {
    const rows: Array<IFlatTreeRow<string>> = flattenTree(tree(), new Set([toDirectoryItemId("meshes")]));

    // Directories sort before files, so `weapons` precedes `actor.ogf`.
    expect(labels(rows)).toEqual(["meshes", "weapons", "actor.ogf", "readme.txt"]);
  });

  it("descends only through directories that are themselves revealed", () => {
    const rows: Array<IFlatTreeRow<string>> = flattenTree(
      tree(),
      new Set([toDirectoryItemId(at("meshes", "weapons"))])
    );

    // The nested directory is expanded, but its parent is not, so nothing of it is on screen.
    expect(labels(rows)).toEqual(["meshes", "readme.txt"]);
  });

  it("carries the depth, parent and sibling position each row needs", () => {
    const rows: Array<IFlatTreeRow<string>> = flattenTree(
      tree(),
      new Set([toDirectoryItemId("meshes"), toDirectoryItemId(at("meshes", "weapons"))])
    );

    expect(
      rows.map((it: IFlatTreeRow<string>) => [it.item.label, it.depth, it.posInSet, it.setSize, it.parentId])
    ).toEqual([
      ["meshes", 0, 1, 2, null],
      ["weapons", 1, 1, 2, toDirectoryItemId("meshes")],
      ["ak74.ogf", 2, 1, 2, toDirectoryItemId(at("meshes", "weapons"))],
      ["pm.ogf", 2, 2, 2, toDirectoryItemId(at("meshes", "weapons"))],
      ["actor.ogf", 1, 2, 2, toDirectoryItemId("meshes")],
      ["readme.txt", 0, 2, 2, null],
    ]);
  });

  it("marks an empty directory as having nothing to reveal", () => {
    const items: Array<IPathTreeItem<string>> = [
      { id: toDirectoryItemId("empty"), label: "empty", path: "empty", kind: "directory", children: [] },
    ];
    const rows: Array<IFlatTreeRow<string>> = flattenTree(items, new Set([toDirectoryItemId("empty")]));

    expect(rows[0].hasChildren).toBe(false);
    expect(rows[0].isExpanded).toBe(false);
  });
});
