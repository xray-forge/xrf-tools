import { describe, expect, it } from "@jest/globals";

import { RenderMemoryReport } from "@/core/ipc/types/xrf-renderer";
import { LevelRendererResidentSection } from "@/core/level/components/panels/LevelRendererPanel/LevelRendererResidentSection";
import { renderWithProviders } from "@/fixtures/utils/render";

const MEMORY: RenderMemoryReport = { allocated: 2 * 1024 ** 3, reserved: 3 * 1024 ** 3, scene: 0, textures: 0 };

describe("LevelRendererResidentSection", () => {
  it("says what the GPU allocator holds of what it reserved", () => {
    const { getByText } = renderWithProviders(<LevelRendererResidentSection load={null} memory={MEMORY} />);

    expect(getByText(/of .* reserved/)).toBeInTheDocument();
  });

  it("says the GPU memory is not reported where the report holds none, or none at all", () => {
    const missing = { scene: 0, textures: 0 } as unknown as RenderMemoryReport;

    for (const memory of [{ ...MEMORY, allocated: null, reserved: null }, missing]) {
      const { getByText, unmount } = renderWithProviders(<LevelRendererResidentSection load={null} memory={memory} />);

      expect(getByText("Not reported")).toBeInTheDocument();
      unmount();
    }
  });
});
