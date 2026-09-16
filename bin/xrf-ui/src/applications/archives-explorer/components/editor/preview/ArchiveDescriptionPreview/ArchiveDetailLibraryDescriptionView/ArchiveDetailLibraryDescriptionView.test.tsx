import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { ArchiveDetailLibraryDescription } from "@/core/ipc/types/xrf-app";
import { mockArchiveDetailLibraryDescription } from "@/fixtures/mocks/archive.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveDetailLibraryDescriptionView } from "./ArchiveDetailLibraryDescriptionView";

function renderView(
  description: ArchiveDetailLibraryDescription = mockArchiveDetailLibraryDescription()
): RenderResult {
  const { container } = mockInjectedService(ArchivesService);

  return renderWithProviders(<ArchiveDetailLibraryDescriptionView description={description} />, { container });
}

describe("ArchiveDetailLibraryDescriptionView", () => {
  it("leads with how much of the level is planted, not with how many cells the grid has", () => {
    const { getByText } = renderView();

    expect(getByText("87,842 slots · 71.3%")).toBeTruthy();
    expect(getByText("Of 123,201 slots, each 2 m square")).toBeTruthy();
  });

  it("says how much ground the grid spans", () => {
    const { getByText } = renderView();

    expect(getByText("351 × 351")).toBeTruthy();
    expect(getByText("Covering 702.0 × 702.0 m of ground")).toBeTruthy();
  });

  it("names the six-bit ceiling a library answers to", () => {
    const { getByText } = renderView();

    expect(getByText(/at most 63/)).toBeTruthy();
  });

  it("offers each object's texture as somewhere to go", () => {
    const { getByText } = renderView();

    expect(getByText("detail\\detail_grass").closest("button")).not.toBeNull();
  });

  it("falls back to the library index for an object naming no texture", () => {
    const { getByText } = renderView();

    expect(getByText("Object 2")).toBeTruthy();
  });

  it("counts corners rather than slots, and says so", () => {
    const { getByText } = renderView();

    expect(getByText("4,820 corners")).toBeTruthy();
    expect(getByText("1 corner")).toBeTruthy();
    expect(getByText(/one slot plants up to four objects/)).toBeTruthy();
  });

  it("says plainly when the grid never plants an object the library carries", () => {
    const { getByText } = renderView();

    expect(getByText("Never planted")).toBeTruthy();
  });

  it("qualifies an object with its mesh and scale, and names swaying only where it is off", () => {
    const { getByText, getAllByText, queryAllByText } = renderView();

    // Two of the three entries draw the same mesh, which is what a library is: a few materials replanted.
    expect(getAllByText("12 triangles over 24 vertices · scale 0.50–1.50")).toHaveLength(2);
    expect(getByText("0 triangles over 0 vertices · scale 0.50–1.50 · does not sway")).toBeTruthy();
    expect(queryAllByText(/· sways/)).toHaveLength(0);
  });

  it("reports a grid holding nothing without dividing by it", () => {
    const { getByText } = renderView(mockArchiveDetailLibraryDescription({ slots: 0, plantedSlots: 0 }));

    expect(getByText("No slots at all")).toBeTruthy();
  });
});
