import { describe, expect, it } from "@jest/globals";
import { RenderMemoryReport } from "@xrf/ipc/types/xrf-renderer";

import { LevelRendererMemorySection } from "@/applications/level-viewer/components/panels/LevelRendererPanel/LevelRendererMemorySection";
import { renderWithProviders } from "@/fixtures/utils/render";

const MEMORY: RenderMemoryReport = {
  allocated: 1000,
  budget: null,
  peakAllocated: 1200,
  reserved: 2000,
  sections: [
    { bytes: 600, count: 3, name: "level texture" },
    { bytes: 400, count: 1, name: "graph transient" },
  ],
  total: null,
  used: null,
};

describe("LevelRendererMemorySection", () => {
  it("lists what the GPU allocator holds by name, with each share of all of it", () => {
    const { getByText } = renderWithProviders(<LevelRendererMemorySection memory={MEMORY} />);

    expect(getByText("level texture")).toBeInTheDocument();
    expect(getByText(/60\.0%/)).toBeInTheDocument();
    expect(getByText("graph transient")).toBeInTheDocument();
  });

  it("says the sections are not reported where the backend reports none", () => {
    const { getByText } = renderWithProviders(
      <LevelRendererMemorySection memory={{ ...MEMORY, allocated: null, sections: [] }} />
    );

    expect(getByText("Not reported")).toBeInTheDocument();
  });
});
