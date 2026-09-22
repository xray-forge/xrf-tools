import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { Nullable } from "@xrf/types";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { TArchiveBytes } from "@/core/archive/lib";
import { ImageDescriptor } from "@/core/ipc/types/xrf-app";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { AsyncState } from "@/lib/async-state";

import { ArchiveImagePreview } from "./ArchiveImagePreview";

const BYTES: TArchiveBytes = new Uint8Array([0x89, 0x50, 0x4e, 0x47]);

const DESCRIBED: ImageDescriptor = {
  shape: { width: 512, height: 256, format: "PNG" },
  mediaType: "image/png",
};

function renderPreview(descriptor: Nullable<ImageDescriptor> = DESCRIBED): RenderResult {
  const { service, container } = mockInjectedService(ArchivesService);

  service.content = AsyncState.ready(descriptor ? { kind: "image" as const, descriptor, bytes: BYTES } : null);

  return renderWithProviders(<ArchiveImagePreview />, { container });
}

describe("ArchiveImagePreview", () => {
  beforeEach(() => {
    jest.spyOn(URL, "createObjectURL").mockImplementation(() => "blob:picture");
    jest.spyOn(URL, "revokeObjectURL").mockImplementation(() => undefined);
  });

  it("names the size and the format the bytes turned out to be", () => {
    const { getByText } = renderPreview();

    expect(getByText("512 x 256 · PNG · image")).toBeTruthy();
  });

  it("draws the bytes as they stand rather than a decoded copy", () => {
    const { container } = renderPreview();

    expect(container.querySelector("img")?.getAttribute("src")).toBe("blob:picture");
  });

  it("gives up on a picture whose header will not parse rather than laying out against nothing", () => {
    // The viewport needs real dimensions. A `png` this cannot measure is corrupt, and saying so beats a blank frame.
    const { getByText } = renderPreview({ shape: null, mediaType: "image/png" });

    expect(getByText("Preview unavailable")).toBeTruthy();
    expect(getByText(/no header that could be read/)).toBeTruthy();
  });
});
