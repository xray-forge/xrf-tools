import { describe, expect, it } from "@jest/globals";

import { ERenderBackend, RenderBackendAvailability } from "@/core/ipc/types/xrf-renderer";
import { RENDER_BACKEND_AUTO } from "@/core/render/lib/settings/render-backend-choice";

import { describeGraphics } from "./SettingsGraphicsSection.utils";

const BOTH: ReadonlyArray<RenderBackendAvailability> = [
  { adapter: "Test GPU", backend: ERenderBackend.D3D12, driver: "32.0.16.1714", isForced: false, problem: null },
  { adapter: null, backend: ERenderBackend.VULKAN, driver: null, isForced: false, problem: "No Vulkan here" },
];

describe("describeGraphics", () => {
  it("states each API's GPU and driver, or why it cannot draw, and the one drawn with", () => {
    expect(describeGraphics(RENDER_BACKEND_AUTO, BOTH, null)).toEqual([
      { label: "Direct3D 12", value: "Test GPU, driver 32.0.16.1714" },
      { label: "Vulkan", value: "Not available: No Vulkan here" },
      { label: "Drawing with", value: "Direct3D 12, chosen automatically" },
    ]);
  });

  it("says the environment asks for the API drawn with", () => {
    const forced: ReadonlyArray<RenderBackendAvailability> = BOTH.map((it: RenderBackendAvailability) => ({
      ...it,
      isForced: it.backend === ERenderBackend.D3D12,
    }));

    expect(describeGraphics(ERenderBackend.VULKAN, forced, null).at(-1)).toEqual({
      label: "Drawing with",
      value: "Direct3D 12, set by XRF_RENDER_BACKEND",
    });
  });

  it("names the DLSS runtime's version, and its absence from a build without it", () => {
    expect(describeGraphics(RENDER_BACKEND_AUTO, null, { dlss: "310.9.1.0" })).toEqual([
      { label: "DLSS", value: "310.9.1.0" },
    ]);
    expect(describeGraphics(RENDER_BACKEND_AUTO, null, { dlss: null })).toEqual([
      { label: "DLSS", value: "Not in this build" },
    ]);
  });
});
