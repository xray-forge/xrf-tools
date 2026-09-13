import { describe, expect, it } from "@jest/globals";

import { XrayAsset } from "@/core/ipc/types/xrf-vfs";
import { mockTextureDescriptor } from "@/fixtures/mocks/visual.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

import { AssetTextureDetails } from "./AssetTextureDetails";

const ASSET: XrayAsset = {
  logicalPath: "textures\\wall.dds",
  container: { kind: "archive", path: "C:\\game\\textures.db" },
};

describe("AssetTextureDetails", () => {
  it("keeps a known file size when the header is unreadable", () => {
    const view = renderWithProviders(
      <AssetTextureDetails asset={ASSET} descriptor={mockTextureDescriptor({ size: 2048, shape: null })} />
    );

    expect(view.getByText("Archive").parentElement).toHaveTextContent("C:\\game\\textures.db");
    expect(view.getByText("Size").parentElement).toHaveTextContent("2 KB");
    expect(view.getByText("Format").parentElement).toHaveTextContent("Header unreadable");
    expect(view.queryByText("Dimensions")).not.toBeInTheDocument();
    expect(view.queryByText("Mipmaps")).not.toBeInTheDocument();
  });

  it("reports location without inventing metadata for an undescribed file", () => {
    const view = renderWithProviders(<AssetTextureDetails asset={ASSET} />);

    expect(view.getByText("Path").parentElement).toHaveTextContent(ASSET.logicalPath);
    expect(view.queryByText("Size")).not.toBeInTheDocument();
    expect(view.queryByText("Format")).not.toBeInTheDocument();
  });
});
