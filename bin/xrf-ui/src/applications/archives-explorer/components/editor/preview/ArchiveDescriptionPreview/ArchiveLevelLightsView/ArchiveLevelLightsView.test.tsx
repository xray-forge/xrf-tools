import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { ArchiveLevelLightsDescription } from "@/core/ipc/types/xrf-app";
import { mockArchiveLevelLightsDescription } from "@/fixtures/mocks/archive.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveLevelLightsView } from "./ArchiveLevelLightsView";

function renderView(description: ArchiveLevelLightsDescription = mockArchiveLevelLightsDescription()): RenderResult {
  return renderWithProviders(<ArchiveLevelLightsView description={description} />);
}

describe("ArchiveLevelLightsView", () => {
  it("separates what the compiler wrote from what the runtime takes", () => {
    // The two differ by more than half here, which is the fact worth leading with.
    const { getByText } = renderView();

    expect(getByText("2,203")).toBeTruthy();
    expect(getByText("918")).toBeTruthy();
    expect(getByText(/the only one CLight_DB::LoadHemi opens/)).toBeTruthy();
  });

  it("names the one chunk the engine opens and says of every other that it does not", () => {
    const { getByText, getAllByText } = renderView();

    expect(getByText("Opened by CLight_DB::LoadHemi, which keeps the 918 point ones")).toBeTruthy();
    expect(getAllByText(/so the runtime never reads this one/)).toHaveLength(2);
  });

  it("admits a payload that was not a run of lights rather than reporting it as none", () => {
    const { getByText } = renderView();

    expect(getByText("Not a run of lights")).toBeTruthy();
  });

  it("says how much world the lights stand in", () => {
    const { getByText } = renderView();

    expect(getByText("512.0 × 128.0 × 512.0 m")).toBeTruthy();
  });
});
