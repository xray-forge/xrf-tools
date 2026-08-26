import { describe, expect, it } from "@jest/globals";

import { VisualBone } from "@/core/bindings/types/xrf-visual";
import { selectAddonBones, selectHiddenBoneIndices } from "@/core/visuals/lib/visual-bones";
import { mockVisualBone } from "@/fixtures/mocks/visual.mocks";

/** A weapon skeleton in the shape ogf stores one: a body, its parts, and a lens hanging off the scope. */
const WEAPON: Array<VisualBone> = [
  mockVisualBone({ name: "wpn_body", parent: "" }),
  mockVisualBone({ name: "wpn_scope", parent: "wpn_body" }),
  mockVisualBone({ name: "wpn_scope_lens", parent: "wpn_scope" }),
  mockVisualBone({ name: "wpn_silencer", parent: "wpn_body" }),
  mockVisualBone({ name: "magazin", parent: "wpn_body" }),
];

describe("selectHiddenBoneIndices", () => {
  it("hides nothing when nothing is selected", () => {
    expect(selectHiddenBoneIndices(WEAPON, new Set())).toEqual(new Set());
  });

  it("hides a bone and everything parented to it", () => {
    // The lens hangs off the scope, and the engine hides recursively: leaving it behind would strand it in mid air.
    expect(selectHiddenBoneIndices(WEAPON, new Set(["wpn_scope"]))).toEqual(new Set([1, 2]));
  });

  it("hides several selections at once, without hiding their siblings", () => {
    expect(selectHiddenBoneIndices(WEAPON, new Set(["wpn_silencer", "magazin"]))).toEqual(new Set([3, 4]));
  });

  it("ignores a name this skeleton does not carry", () => {
    // What makes a selection survive a model change: the previous model's scope simply matches nothing here.
    expect(selectHiddenBoneIndices(WEAPON, new Set(["wpn_launcher"]))).toEqual(new Set());
  });

  it("terminates on a skeleton whose parents form a loop", () => {
    const looped: Array<VisualBone> = [
      mockVisualBone({ name: "first", parent: "second" }),
      mockVisualBone({ name: "second", parent: "first" }),
      mockVisualBone({ name: "self", parent: "self" }),
    ];

    expect(selectHiddenBoneIndices(looped, new Set(["first"]))).toEqual(new Set([0, 1]));
    expect(selectHiddenBoneIndices(looped, new Set(["self"]))).toEqual(new Set([2]));
  });
});

describe("selectAddonBones", () => {
  it("reports the addon bones a skeleton carries, in engine order", () => {
    expect(selectAddonBones(WEAPON)).toEqual(["wpn_scope", "wpn_silencer"]);
  });

  it("reports both spellings of the grenade launcher bone", () => {
    const bones: Array<VisualBone> = [
      mockVisualBone({ name: "wpn_grenade_launcher" }),
      mockVisualBone({ name: "wpn_launcher" }),
    ];

    expect(selectAddonBones(bones)).toEqual(["wpn_launcher", "wpn_grenade_launcher"]);
  });

  it("reports none for a skeleton that wears nothing", () => {
    expect(selectAddonBones([mockVisualBone({ name: "bip01" })])).toEqual([]);
  });
});
