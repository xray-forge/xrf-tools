import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { ArchiveLevelGeomDescription } from "@/core/ipc/types/xrf-app";
import { mockArchiveLevelGeomDescription } from "@/fixtures/mocks/archive.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveLevelGeomView } from "./ArchiveLevelGeomView";

function renderView(description: ArchiveLevelGeomDescription = mockArchiveLevelGeomDescription()): RenderResult {
  return renderWithProviders(<ArchiveLevelGeomView description={description} />);
}

describe("ArchiveLevelGeomView", () => {
  it("leads with what the level costs to draw, grouped so the digits are readable", () => {
    const { getByText } = renderView();

    expect(getByText("1,403,836")).toBeTruthy();
    expect(getByText("Over 1,204,331 vertices")).toBeTruthy();
  });

  it("says the payload was never read, which is why a 143 MB file opens at all", () => {
    const { getByText } = renderView();

    expect(getByText("137 MB")).toBeTruthy();
    expect(getByText(/only the shape is read/)).toBeTruthy();
  });

  it("groups 33 buffers into the two layouts the whole corpus uses", () => {
    // A level carries up to 55 vertex buffers and only ever two strides between them, so the layouts are the answer
    // and the buffers are not.
    const { getByText } = renderView();

    expect(getByText("Vertex layouts (2)")).toBeTruthy();
    expect(getByText("32 bytes · 6 elements")).toBeTruthy();
    expect(getByText("24 buffers · 980,114 vertices")).toBeTruthy();
    expect(getByText("12 bytes · 1 element")).toBeTruthy();
  });

  it("names the detail twin as itself and says why it carries no progressive meshes", () => {
    const { getByText } = renderView(
      mockArchiveLevelGeomDescription({ isDetail: true, progressiveMeshes: 0, detailLevels: 0 })
    );

    expect(getByText("Detail geometry")).toBeTruthy();
    expect(getByText(/renderer reads them from level.geom alone/)).toBeTruthy();
  });

  it("says plainly when a level drops no detail with distance, rather than reporting a zero", () => {
    const { getByText } = renderView(mockArchiveLevelGeomDescription({ progressiveMeshes: 0, detailLevels: 0 }));

    expect(getByText("Nothing on this level drops detail with distance")).toBeTruthy();
  });
});
