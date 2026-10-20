import { describe, expect, it } from "@jest/globals";

import { LevelRendererResidentSection } from "@/applications/level-viewer/components/panels/LevelRendererPanel/LevelRendererResidentSection";
import { RenderMemoryReport } from "@/core/ipc/types/xrf-renderer";
import { renderWithProviders } from "@/fixtures/utils/render";

const MEMORY: RenderMemoryReport = {
  allocated: 2 * 1024 ** 3,
  budget: 14 * 1024 ** 3,
  peakAllocated: 3 * 1024 ** 3,
  reserved: 3 * 1024 ** 3,
  sections: [],
  total: 16 * 1024 ** 3,
  used: 4 * 1024 ** 3,
};

describe("LevelRendererResidentSection", () => {
  it("says what the GPU allocator holds of what it reserved", () => {
    const { getByText } = renderWithProviders(<LevelRendererResidentSection load={null} memory={MEMORY} />);

    expect(getByText(/of .* reserved/)).toBeInTheDocument();
  });

  it("says what the process uses of the GPU's own memory, of its budget, and all of it", () => {
    const { getByText } = renderWithProviders(<LevelRendererResidentSection load={null} memory={MEMORY} />);

    expect(getByText(/used of .* budget/)).toBeInTheDocument();
    expect(getByText("Video memory total")).toBeInTheDocument();
  });

  it("says the GPU memory is not reported where the report holds none, or none at all", () => {
    const missing = { sections: [] } as unknown as RenderMemoryReport;
    const none: RenderMemoryReport = {
      ...MEMORY,
      allocated: null,
      budget: null,
      peakAllocated: null,
      reserved: null,
      total: null,
      used: null,
    };

    for (const memory of [none, missing]) {
      const { getAllByText, unmount } = renderWithProviders(
        <LevelRendererResidentSection load={null} memory={memory} />
      );

      expect(getAllByText("Not reported")).toHaveLength(4);
      unmount();
    }
  });
});
