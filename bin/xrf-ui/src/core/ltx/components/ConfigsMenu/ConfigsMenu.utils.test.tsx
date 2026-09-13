import { describe, expect, it } from "@jest/globals";

import { LtxInventoryFile, LtxInventoryRole } from "@/core/bindings/types/xrf-ltx-inspect";
import { decorateConfigIcon } from "@/core/ltx/components/ConfigsMenu/ConfigsMenu.utils";
import { ITreeNode } from "@/core/ui/tree/tree-node";

function mockFileOf(role: LtxInventoryRole): LtxInventoryFile {
  return { isPhysical: true, path: "configs\\items\\w_ak74.ltx", role, source: "gamedata" };
}

function mockNodeOf(payload?: LtxInventoryFile): ITreeNode<LtxInventoryFile> {
  return { id: "file:configs\\items\\w_ak74.ltx", label: "w_ak74.ltx", payload };
}

describe("decorateConfigIcon", () => {
  it("should tell the three roles worth stopping on apart", () => {
    // Three hues, not three slots: the theme paints secondary and info as near-identical light blues, so a scheme and
    // a patch wearing the two would wear the same colour on a 17-pixel icon.
    expect(decorateConfigIcon(mockNodeOf(mockFileOf({ kind: "entryPoint" })))?.color).toBe("primary.main");
    expect(decorateConfigIcon(mockNodeOf(mockFileOf({ kind: "schemeFile" })))?.color).toBe("secondary.main");
    expect(decorateConfigIcon(mockNodeOf(mockFileOf({ kind: "attachment" })))?.color).toBe("success.main");
  });

  it("should say what each tint means, since a colour on its own says nothing", () => {
    expect(decorateConfigIcon(mockNodeOf(mockFileOf({ kind: "entryPoint" })))?.title).toContain("Entry point");
    expect(decorateConfigIcon(mockNodeOf(mockFileOf({ kind: "schemeFile" })))?.title).toContain("Scheme");
    expect(decorateConfigIcon(mockNodeOf(mockFileOf({ kind: "attachment" })))?.title).toContain("Patch");
  });

  it("should leave an included config neutral, since almost every config in a tree is one", () => {
    // A mark on nearly every row is a mark nobody reads.
    expect(decorateConfigIcon(mockNodeOf(mockFileOf({ kind: "included", by: ["configs\\system.ltx"] })))).toBeNull();
  });

  it("should leave a directory row neutral, which stands for no config at all", () => {
    expect(decorateConfigIcon(mockNodeOf())).toBeNull();
  });

  it("should reserve the severity colours for what is actually a problem", () => {
    // `warning` and `error` belong to the Problems panel; a role is not a problem, and an archived config is not one
    // either - it is the ordinary case on an install.
    const colors: Array<string> = [
      { kind: "entryPoint" } as const,
      { kind: "schemeFile" } as const,
      { kind: "attachment" } as const,
    ].map((role: LtxInventoryRole) => decorateConfigIcon(mockNodeOf(mockFileOf(role)))?.color ?? "");

    expect(colors.some((color: string) => color.startsWith("warning") || color.startsWith("error"))).toBe(false);
  });
});
