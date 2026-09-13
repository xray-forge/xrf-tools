import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { AssetService } from "@/core/assets/services";
import { TextureDescription } from "@/core/ipc/types/xrf-app";
import { ITexturePreviewComparison } from "@/core/textures/lib/texture-preview";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { mockTextureDescription } from "@/fixtures/mocks/texture.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { AsyncState } from "@/lib/async-state";
import { Nullable } from "@/lib/types/general";

import { TexturePreview } from "./TexturePreview";

const SHAPED: TextureDescription = mockTextureDescription("ston\\ston_beton05", {
  base: { size: 2048, shape: { width: 256, height: 128, mipmapLevels: 9, format: "DXT5" } },
});

function renderPreview(
  selected: Nullable<TextureDescription>,
  isReading: boolean = false,
  comparison: Nullable<ITexturePreviewComparison> = null
): RenderResult {
  const { service, container } = mockInjectedService(TextureSelectionService, [AssetService]);

  service.selected = isReading ? AsyncState.loading(selected) : AsyncState.ready(selected);
  service.preview = AsyncState.ready(selected ? new ArrayBuffer(4) : null);

  return renderWithProviders(<TexturePreview comparison={comparison} />, { container });
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
    expect(getByText("Reading…")).toBeTruthy();
    expect(queryByText("256 x 128 · DXT5 · 9 mips")).toBeNull();
  });

  it("says nothing is open when nothing is", () => {
    const { getByText } = renderPreview(null);

    expect(getByText("No texture open")).toBeTruthy();
  });

  it("shows one picture when there is nothing to compare it with", () => {
    const { queryByTestId, getByTestId } = renderPreview(SHAPED);

    expect(getByTestId("texture-preview")).toBeTruthy();
    expect(queryByTestId("texture-image-pane-comparison")).toBeNull();
  });

  it("pairs the file with the encoding that would replace it", () => {
    // Side by side rather than one over the other: what a re-encode costs shows up as a difference between two
    // pictures of the same texels, and the captions have to say which is which.
    const { getByTestId, getByText } = renderPreview(SHAPED, false, {
      label: "BC7",
      preview: AsyncState.ready(new ArrayBuffer(4)),
    });

    expect(getByTestId("texture-image-pane-current")).toBeTruthy();
    expect(getByTestId("texture-image-pane-comparison")).toBeTruthy();
    expect(getByText("On disk — 256 x 128 · DXT5 · 9 mips")).toBeTruthy();
    expect(getByText("Would write — BC7")).toBeTruthy();
  });

  it("holds the pair open while the second picture is still being read", () => {
    // The pane is captioned before it has anything to draw, so choosing a format does not make the picture beside it
    // jump away and come back.
    const { getByTestId, getByText } = renderPreview(SHAPED, false, {
      label: "BC7",
      preview: AsyncState.loading(),
    });

    expect(getByTestId("texture-image-pane-current")).toBeTruthy();
    expect(getByText("Would write — BC7")).toBeTruthy();
  });
});
