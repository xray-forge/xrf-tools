import { describe, expect, it } from "@jest/globals";
import { ERenderBackend, RenderBackendAvailability } from "@xrf/ipc/types/xrf-renderer";

import { RENDER_BACKEND_AUTO } from "@/core/render/lib/settings/render-backend-choice";

import { describeGraphics } from "./SettingsGraphicsSection.utils";

const BOTH: ReadonlyArray<RenderBackendAvailability> = [
  {
    backend: ERenderBackend.D3D12,
    gpus: [{ gpu: { vendor: 1, device: 1 }, name: "Test GPU", driver: "32.0.16.1714" }],
    isForced: false,
    problem: null,
  },
  { backend: ERenderBackend.VULKAN, gpus: [], isForced: false, problem: "No Vulkan here" },
];

describe("describeGraphics", () => {
  it("states each API's GPU and driver, or why it cannot draw, and the one drawn with", () => {
    expect(describeGraphics(RENDER_BACKEND_AUTO, null, BOTH, null)).toEqual([
      { label: "Direct3D 12", value: "Test GPU, driver 32.0.16.1714" },
      { label: "Vulkan", value: "Not available: No Vulkan here" },
      { label: "Drawing with", value: "Direct3D 12 on Test GPU, chosen automatically" },
    ]);
  });

  it("says the environment asks for the API drawn with", () => {
    const forced: ReadonlyArray<RenderBackendAvailability> = BOTH.map((it: RenderBackendAvailability) => ({
      ...it,
      isForced: it.backend === ERenderBackend.D3D12,
    }));

    expect(describeGraphics(ERenderBackend.VULKAN, null, forced, null).at(-1)).toEqual({
      label: "Drawing with",
      value: "Direct3D 12 on Test GPU, set by XRF_RENDER_BACKEND",
    });
  });

  it("names the DLSS runtime's version, and its absence from a build without it", () => {
    expect(describeGraphics(RENDER_BACKEND_AUTO, null, null, { dlss: "310.9.1.0" })).toEqual([
      { label: "DLSS", value: "310.9.1.0" },
    ]);
    expect(describeGraphics(RENDER_BACKEND_AUTO, null, null, { dlss: null })).toEqual([
      { label: "DLSS", value: "Not in this build" },
    ]);
  });
});
