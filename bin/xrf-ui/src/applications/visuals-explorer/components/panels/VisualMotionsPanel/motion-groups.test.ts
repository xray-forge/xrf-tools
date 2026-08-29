import { describe, expect, it } from "@jest/globals";

import {
  getMotionNodeName,
  groupMotionNames,
  listMotionGroupIds,
  toMotionNodeId,
} from "@/applications/visuals-explorer/components/panels/VisualMotionsPanel/motion-groups";
import { ITreeNode } from "@/core/ui/tree/tree-node";

describe("groupMotionNames", () => {
  it("gathers a family under the token its names start with, counted", () => {
    const nodes: Array<ITreeNode<string>> = groupMotionNames([
      "norm_torso_0_aim_0",
      "norm_torso_0_aim_1",
      "cr_idle_0",
      "cr_idle_1",
    ]);

    expect(nodes.map((node: ITreeNode<string>) => node.label)).toEqual(["cr (2)", "norm (2)"]);
    expect(nodes[1].children?.map((child: ITreeNode<string>) => child.label)).toEqual([
      "norm_torso_0_aim_0",
      "norm_torso_0_aim_1",
    ]);
  });

  it("keeps a family in the order the backend listed it, since the names carry runs", () => {
    // Alphabetically `_10` sorts before `_2`, which would break every numbered run these sets are written in.
    const nodes: Array<ITreeNode<string>> = groupMotionNames(["norm_2", "norm_10", "norm_1"]);

    expect(nodes[0].children?.map((child: ITreeNode<string>) => child.label)).toEqual(["norm_2", "norm_10", "norm_1"]);
  });

  it("leaves a token holding one name as a row rather than a family of one", () => {
    // Six of the sixty tokens a stalker's motion sets produce hold exactly one name; a folder around each is a click
    // that reveals what it already said.
    const nodes: Array<ITreeNode<string>> = groupMotionNames(["ragdoll_0", "ragdoll_1", "$editor"]);

    expect(nodes[0]).toMatchObject({ id: toMotionNodeId("$editor"), label: "$editor", payload: "$editor" });
    expect(nodes[0].children).toBeUndefined();
    expect(nodes[1].label).toBe("ragdoll (2)");
  });

  it("cannot confuse a family with a motion spelled the same way", () => {
    const nodes: Array<ITreeNode<string>> = groupMotionNames(["idle", "idle_0", "idle_1"]);

    expect(new Set(nodes.map((node: ITreeNode<string>) => node.id)).size).toBe(nodes.length);
    expect(getMotionNodeName(nodes[0].id)).toBeNull();
  });

  it("names only the families a filter left, which is what a match expands", () => {
    const nodes: Array<ITreeNode<string>> = groupMotionNames(["cr_idle_0", "cr_idle_1", "$editor"]);

    expect(listMotionGroupIds(nodes)).toEqual([nodes[1].id]);
  });
});
