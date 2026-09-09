import { describe, expect, it } from "@jest/globals";

import { LtxInventoryFile, LtxInventoryRole } from "@/core/bindings/types/xrf-ltx-inspect";
import { ConfigsTreeLabel } from "@/core/ltx/components/ConfigsMenu/ConfigsTreeLabel";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { renderWithProviders } from "@/fixtures/utils/render";

/** One inventory row, with only the parts a case is about spelled out. */
function fileOf(role: LtxInventoryRole, isPhysical: boolean = true): LtxInventoryFile {
  return { isPhysical, path: "configs\\items\\w_ak74.ltx", role, source: "gamedata" };
}

/** The node the tree hands a label renderer. */
function nodeOf(payload?: LtxInventoryFile): ITreeNode<LtxInventoryFile> {
  return { id: "file:configs\\items\\w_ak74.ltx", label: "w_ak74.ltx", payload };
}

describe("ConfigsTreeLabel", () => {
  it("should render the config's name, which the tree shows nowhere else", () => {
    // `VirtualizedTree` renders this in place of the row's own text rather than beside it, so a label answering only
    // badges leaves a tree of badges with no file names - which is what a first cut did.
    const { getByText } = renderWithProviders(<ConfigsTreeLabel item={nodeOf(fileOf({ kind: "entryPoint" }))} />);

    expect(getByText("w_ak74.ltx")).toBeInTheDocument();
  });

  it("should mark a config nothing can write to in place", () => {
    // The case that matters on a real install: 3,238 of an Anomaly tree's 3,240 configs are read out of `db\\configs`,
    // where they read like any other config and no editor can ever replace them.
    const { getByText } = renderWithProviders(
      <ConfigsTreeLabel item={nodeOf(fileOf({ kind: "included", by: ["configs\\system.ltx"] }, false))} />
    );

    expect(getByText("archived")).toBeInTheDocument();
  });

  it("should mark what resolves on its own and what declares the rules", () => {
    const entry = renderWithProviders(<ConfigsTreeLabel item={nodeOf(fileOf({ kind: "entryPoint" }))} />);
    const scheme = renderWithProviders(<ConfigsTreeLabel item={nodeOf(fileOf({ kind: "schemeFile" }))} />);
    const patch = renderWithProviders(<ConfigsTreeLabel item={nodeOf(fileOf({ kind: "attachment" }))} />);

    expect(entry.getByText("entry")).toBeInTheDocument();
    expect(scheme.getByText("scheme")).toBeInTheDocument();
    expect(patch.getByText("patch")).toBeInTheDocument();
  });

  it("should leave an included config unmarked, since almost every config in a tree is one", () => {
    // A badge on nearly every row is a badge nobody reads.
    const { queryByText, getByText } = renderWithProviders(
      <ConfigsTreeLabel item={nodeOf(fileOf({ kind: "included", by: ["configs\\system.ltx"] }))} />
    );

    expect(getByText("w_ak74.ltx")).toBeInTheDocument();
    expect(queryByText("entry")).not.toBeInTheDocument();
    expect(queryByText("archived")).not.toBeInTheDocument();
  });

  it("should render a directory row, which stands for no config at all", () => {
    const { getByText } = renderWithProviders(<ConfigsTreeLabel item={{ id: "directory:configs", label: "configs" }} />);

    expect(getByText("configs")).toBeInTheDocument();
  });
});
