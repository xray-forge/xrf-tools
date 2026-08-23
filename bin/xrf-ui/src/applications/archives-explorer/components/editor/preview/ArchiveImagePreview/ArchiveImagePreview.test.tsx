import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { ArchiveImagePreview } from "@/applications/archives-explorer/components/editor/preview/ArchiveImagePreview";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { TArchiveBytes } from "@/core/archive";
import { AssetTextureDescriptor } from "@/core/bindings/types/xrf-app";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { createLoadable } from "@/lib/loadable";
import { Nullable } from "@/lib/types/general";

const BYTES: TArchiveBytes = new Uint8Array([0x89, 0x50, 0x4e, 0x47]);

const MIPPED: AssetTextureDescriptor = {
  size: 2048,
  shape: { width: 256, height: 128, mipmapLevels: 9, format: "DXT5" },
};

/**
 * Renders the panel over content the service already holds, which is the state the preview reads.
 *
 * @param descriptor - Texture description to publish, or null for a selection that produced no content.
 * @returns The render result.
 */
function renderPreview(descriptor: Nullable<AssetTextureDescriptor>): RenderResult {
  const { service, container } = mockInjectedService(ArchivesService);

  service.content = createLoadable(descriptor ? { kind: "image" as const, descriptor, bytes: BYTES } : null);

  return renderWithProviders(<ArchiveImagePreview />, { container });
}

describe("ArchiveImagePreview", () => {
  beforeEach(() => {
    jest.spyOn(URL, "createObjectURL").mockImplementation(() => "blob:texture");
    jest.spyOn(URL, "revokeObjectURL").mockImplementation(() => undefined);
  });

  it("captions the source format and mip chain, not just the size", () => {
    const { getByText } = renderPreview(MIPPED);

    // Format and mip count come from the DDS header and do not survive the transcode, which is why they are described
    // rather than read back off the png.
    expect(getByText("256 x 128 · DXT5 · 9 mips")).toBeTruthy();
  });

  it("names a missing mip chain rather than counting it as one", () => {
    // A mipless texture has to be sampled with a linear filter or webgl renders it black, so "1 mip" would bury the
    // one fact worth spotting.
    const { getByText } = renderPreview({ size: 512, shape: { ...MIPPED.shape!, mipmapLevels: 1 } });

    expect(getByText(/no mips$/)).toBeTruthy();
  });

  it("paints the decoded bytes it was handed", () => {
    const { container } = renderPreview(MIPPED);

    expect(container.querySelector("img")?.getAttribute("src")).toBe("blob:texture");
  });

  it("shows nothing to preview when the header would not parse", () => {
    // The read succeeds and the descriptor comes back shapeless, so this state arrives as content rather than as an
    // error - the panel still has no dimensions to lay a viewport out against.
    const { getByText } = renderPreview({ size: 2048, shape: null });

    expect(getByText("Preview unavailable")).toBeTruthy();
  });

  it("shows nothing to preview when the texture was not read", () => {
    const { getByText } = renderPreview(null);

    expect(getByText("Preview unavailable")).toBeTruthy();
  });
});
