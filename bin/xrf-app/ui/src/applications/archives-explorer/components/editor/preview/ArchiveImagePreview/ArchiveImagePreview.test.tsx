import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { ImageDescriptor } from "@xrf/ipc/types/xrf-app-archives";
import { Nullable } from "@xrf/types";

import {
  IArchivesExplorerMock,
  mockArchivesExplorer,
  mockShownArchiveFile,
} from "@/applications/archives-explorer/fixtures/archives-explorer.mocks";
import { mockArchiveFileDescriptor } from "@/core/archive/fixtures/archive.mocks";
import { TArchiveBytes } from "@/core/archive/lib";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveImagePreview } from "./ArchiveImagePreview";

const BYTES: TArchiveBytes = new Uint8Array([0x89, 0x50, 0x4e, 0x47]);

const DESCRIBED: ImageDescriptor = {
  shape: { width: 512, height: 256, format: "PNG" },
  mediaType: "image/png",
};

function renderPreview(descriptor: Nullable<ImageDescriptor> = DESCRIBED): RenderResult {
  const explorer: IArchivesExplorerMock = mockArchivesExplorer();

  mockShownArchiveFile(
    explorer,
    mockArchiveFileDescriptor(),
    descriptor ? { kind: "image" as const, descriptor, bytes: BYTES } : null
  );

  return renderWithProviders(<ArchiveImagePreview />, { container: explorer.container });
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
