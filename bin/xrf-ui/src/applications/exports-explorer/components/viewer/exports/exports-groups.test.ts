import { describe, expect, it } from "@jest/globals";

import { ExportDescriptor } from "@/core/bindings/types/xrf-export";
import { TCallableExportDescriptor } from "@/core/exports";
import { getFileItemPath, IPathTreeItem, toDirectoryItemId, TREE_ROOT_ID } from "@/core/ui/tree/path-tree";

import { exportGroupsToTree, groupExports, ROOT_EXPORT_GROUP_ID } from "./exports-groups";

function callable(name: string, description: string | null = null): TCallableExportDescriptor {
  return {
    kind: "callable",
    name,
    description,
    parameters: [],
    returns: { typing: "void", description: null },
    source: { path: `${name}.ts`, line: 1, column: 1, endLine: 4 },
  };
}

describe("groupExports", () => {
  it("groups by the first dot and keeps root externs in a separate group", () => {
    const groups = groupExports([
      callable("xr_effects.run"),
      callable("start"),
      callable("dialogs_zaton.quest.answer"),
      callable("xr_effects.stop"),
    ]);

    expect(groups.map((group) => [group.id, group.label])).toEqual([
      [ROOT_EXPORT_GROUP_ID, "~"],
      ["group:namespace:dialogs_zaton", "dialogs_zaton"],
      ["group:namespace:xr_effects", "xr_effects"],
    ]);
    expect(groups[0]?.declarations.map((declaration) => declaration.name)).toEqual(["start"]);
    expect(groups[1]?.declarations.map((declaration) => declaration.name)).toEqual(["dialogs_zaton.quest.answer"]);
    expect(groups[2]?.declarations.map((declaration) => declaration.name)).toEqual([
      "xr_effects.run",
      "xr_effects.stop",
    ]);
  });
});

describe("exportGroupsToTree", () => {
  it("renders a namespace as a directory and a declaration as a leaf carrying its descriptor", () => {
    const items: Array<IPathTreeItem<ExportDescriptor>> = exportGroupsToTree(
      groupExports([callable("xr_effects.run"), callable("xr_effects.stop")])
    );

    expect(items).toHaveLength(1);
    expect(items[0].id).toBe(toDirectoryItemId("xr_effects"));
    expect(items[0].kind).toBe("directory");
    // The count travels in the label, as it did before the tree changed shape.
    expect(items[0].label).toBe("xr_effects (2)");

    const children: Array<IPathTreeItem<ExportDescriptor>> =
      items[0].kind === "directory" ? items[0].children : [];

    expect(children.map((it) => it.kind)).toEqual(["file", "file"]);
    expect(children.map((it) => getFileItemPath(it.id))).toEqual(["xr_effects.run", "xr_effects.stop"]);
    expect(children[0].kind === "file" ? children[0].payload.name : null).toBe("xr_effects.run");
  });

  it("puts root declarations under the synthetic root id, which is what its label already said", () => {
    const items: Array<IPathTreeItem<ExportDescriptor>> = exportGroupsToTree(groupExports([callable("start")]));

    expect(items[0].id).toBe(TREE_ROOT_ID);
    expect(items[0].label).toBe("~ (1)");
  });

  it("answers no file path for a namespace, which is what disables selecting one", () => {
    const items: Array<IPathTreeItem<ExportDescriptor>> = exportGroupsToTree(
      groupExports([callable("xr_conditions.is_alive")])
    );

    expect(getFileItemPath(items[0].id)).toBeNull();
  });
});
