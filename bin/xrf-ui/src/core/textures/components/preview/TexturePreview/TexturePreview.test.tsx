import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { AssetService } from "@/core/assets/services";
import { TextureDescription } from "@/core/bindings/types/xrf-app";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { mockTextureDescription } from "@/fixtures/mocks/texture.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { Loadable } from "@/lib/loadable";
import { Nullable } from "@/lib/types/general";

import { TexturePreview } from "./TexturePreview";

const SHAPED: TextureDescription = mockTextureDescription("ston\\ston_beton05", {
  base: { size: 2048, shape: { width: 256, height: 128, mipmapLevels: 9, format: "DXT5" } },
});

function renderPreview(selected: Nullable<TextureDescription>, isReading: boolean = false): RenderResult {
  const { service, container } = mockInjectedService(TextureSelectionService, [AssetService]);

  service.selected = isReading ? Loadable.loading(selected) : Loadable.ready(selected);
  service.preview = Loadable.ready(selected ? new ArrayBuffer(4) : null);

  return renderWithProviders(<TexturePreview />, { container });
}

describe("TexturePreview", () => {
  beforeEach(() => {
    jest.spyOn(URL, "createObjectURL").mockImplementation(() => "blob:texture");
    jest.spyOn(URL, "revokeObjectURL").mockImplementation(() => undefined);
  });

  it("captions what the texture is once it has been read", () => {
    const { getByText } = renderPreview(SHAPED);

    expect(getByText("256 x 128 · DXT5 · 9 mips")).toBeTruthy();
  });

  it("keeps its frame while the next texture is read", () => {
    const { getByTestId, getByText, queryByText } = renderPreview(SHAPED, true);

    expect(getByTestId("texture-preview")).toBeTruthy();
    expect(getByText(/Reading/)).toBeTruthy();
    expect(queryByText("256 x 128 · DXT5 · 9 mips")).toBeNull();
  });

  it("says nothing is open when nothing is", () => {
    const { getByText } = renderPreview(null);

    expect(getByText("No texture open")).toBeTruthy();
  });
});
