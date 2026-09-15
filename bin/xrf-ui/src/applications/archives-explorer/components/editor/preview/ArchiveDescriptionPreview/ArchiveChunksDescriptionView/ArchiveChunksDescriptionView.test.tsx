import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { ArchiveChunksDescription } from "@/core/ipc/types/xrf-app";
import { mockArchiveChunksDescription } from "@/fixtures/mocks/archive.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveChunksDescriptionView } from "./ArchiveChunksDescriptionView";

function renderView(description: ArchiveChunksDescription = mockArchiveChunksDescription()): RenderResult {
  return renderWithProviders(<ArchiveChunksDescriptionView description={description} />);
}

describe("ArchiveChunksDescriptionView", () => {
  it("says up front that nothing reads the format", () => {
    const { getByText } = renderView();

    expect(getByText(/No reader claims this format/)).toBeTruthy();
  });

  it("counts every chunk of the tree and how deep it goes", () => {
    const { getByText } = renderView();

    expect(getByText("4")).toBeTruthy();
    expect(getByText("Nested 2 deep, counting every level")).toBeTruthy();
  });

  it("words a flat container as flat rather than as one level deep", () => {
    const { getByText } = renderView(
      mockArchiveChunksDescription({
        chunks: [{ id: 0x1100, size: 512, isCompressed: false, children: [] }],
        nodes: 1,
        depth: 1,
      })
    );

    expect(getByText(/A flat sequence/)).toBeTruthy();
  });

  it("shows an id as the number it is rather than naming it", () => {
    const { getByText } = renderView();

    // Naming a chunk would mean knowing the format, which is the premise this view denies.
    expect(getByText("0x0001")).toBeTruthy();
    expect(getByText("0x1100")).toBeTruthy();
  });

  it("draws a nested payload under the chunk that holds it", () => {
    const { getByText } = renderView();

    expect(getByText("0x0010")).toBeTruthy();
    expect(getByText("0x0011")).toBeTruthy();
  });

  it("marks a compressed chunk, whose payload is not what it stands for", () => {
    const { getByText } = renderView(
      mockArchiveChunksDescription({
        chunks: [{ id: 0x7, size: 128, isCompressed: true, children: [] }],
        nodes: 1,
        depth: 1,
      })
    );

    expect(getByText("compressed")).toBeTruthy();
  });
});
