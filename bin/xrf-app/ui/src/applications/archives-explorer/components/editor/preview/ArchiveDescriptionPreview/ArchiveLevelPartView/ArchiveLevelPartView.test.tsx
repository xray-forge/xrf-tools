import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { mockArchiveLevelAiDescription, mockArchiveLevelCollisionDescription } from "@/fixtures/mocks/archive.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveLevelAiView } from "./ArchiveLevelAiView";
import { ArchiveLevelCollisionView } from "./ArchiveLevelCollisionView";

describe("ArchiveLevelCollisionView", () => {
  function renderView(): RenderResult {
    return renderWithProviders(<ArchiveLevelCollisionView description={mockArchiveLevelCollisionDescription()} />);
  }

  it("leads with what the mesh weighs, grouped so the digits are readable", () => {
    const { getByText } = renderView();

    expect(getByText("812,004")).toBeTruthy();
    expect(getByText("Over 420,690 vertices")).toBeTruthy();
  });

  it("says how much world the mesh covers", () => {
    const { getByText } = renderView();

    expect(getByText("512.0 × 128.0 × 512.0 m")).toBeTruthy();
  });

  it("says the mesh itself was not read, which is why a 41 MB file opens at all", () => {
    const { getByText } = renderView();

    expect(getByText(/only the header is read/)).toBeTruthy();
  });
});

describe("ArchiveLevelAiView", () => {
  function renderView(): RenderResult {
    return renderWithProviders(<ArchiveLevelAiView description={mockArchiveLevelAiDescription()} />);
  }

  it("leads with the node count and what one node covers", () => {
    const { getByText } = renderView();

    expect(getByText("1,204,331")).toBeTruthy();
    expect(getByText("0.70 m apart, 0.40 m tall")).toBeTruthy();
  });

  it("names the guid as what ties the grid to a spawn set", () => {
    // The same value appears as a spawn set's graph identity, which is what makes two builds recognisable as a pair.
    const { getByText } = renderView();

    expect(getByText("6a0f0a1e-0000-4000-8000-000000000002")).toBeTruthy();
    expect(getByText(/spawn set built against this grid carries the same value/)).toBeTruthy();
  });

  it("writes the vertical extent between the two ground ones, as X-Ray is Y-up", () => {
    const { getByText } = renderView();

    expect(getByText("512.0 × 128.0 × 512.0 m")).toBeTruthy();
  });
});
