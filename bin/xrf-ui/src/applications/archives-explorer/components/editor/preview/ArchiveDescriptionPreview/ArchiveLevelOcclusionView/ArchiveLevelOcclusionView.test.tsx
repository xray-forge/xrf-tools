import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { ArchiveLevelHomDescription, ArchiveLevelSomDescription } from "@/core/ipc/types/xrf-app";
import { mockArchiveLevelHomDescription, mockArchiveLevelSomDescription } from "@/fixtures/mocks/archive.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveLevelHomView } from "./ArchiveLevelHomView";
import { ArchiveLevelSomView } from "./ArchiveLevelSomView";

describe("ArchiveLevelHomView", () => {
  function renderView(description: ArchiveLevelHomDescription = mockArchiveLevelHomDescription()): RenderResult {
    return renderWithProviders(<ArchiveLevelHomView description={description} />);
  }

  it("leads with the occluders and says plainly that nothing draws them", () => {
    const { getByText } = renderView();

    expect(getByText("1,860")).toBeTruthy();
    expect(getByText(/drawn by nothing itself/)).toBeTruthy();
  });

  it("says how much world the occluders span", () => {
    const { getByText } = renderView();

    expect(getByText("512.0 × 128.0 × 512.0 m")).toBeTruthy();
  });

  it("does not present a mesh carrying nothing as a box of no size", () => {
    const { getByText } = renderView(mockArchiveLevelHomDescription({ triangles: 0, bounds: null }));

    expect(getByText("Nothing to measure")).toBeTruthy();
  });
});

describe("ArchiveLevelSomView", () => {
  function renderView(description: ArchiveLevelSomDescription = mockArchiveLevelSomDescription()): RenderResult {
    return renderWithProviders(<ArchiveLevelSomView description={description} />);
  }

  it("counts what the file holds and says what the loader makes of it", () => {
    // A two-sided triangle becomes a second, reversed face, so the sound renderer ends up with more than are stored.
    const { getByText } = renderView();

    expect(getByText("218")).toBeTruthy();
    expect(getByText("96 occlude both ways, which the loader doubles into 314 faces")).toBeTruthy();
  });

  it("says a mesh occluding one way is taken as it is, rather than reporting a doubling of zero", () => {
    const { getByText } = renderView(mockArchiveLevelSomDescription({ twoSided: 0, faces: 218 }));

    expect(getByText(/takes them as they are/)).toBeTruthy();
  });

  it("gives the quietest and loudest faces, and names which end muffles nothing", () => {
    const { getByText } = renderView();

    expect(getByText("0.10 to 0.85")).toBeTruthy();
    expect(getByText(/1 muffles nothing/)).toBeTruthy();
  });

  it("writes a uniform mesh as one factor rather than as a range of one value", () => {
    const { getByText } = renderView(mockArchiveLevelSomDescription({ minimumOcclusion: 0.5, maximumOcclusion: 0.5 }));

    expect(getByText("0.50")).toBeTruthy();
  });
});
