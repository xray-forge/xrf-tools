import { describe, expect, it } from "@jest/globals";

import { ERenderBackend, RenderBackendAvailability, RenderGpu } from "@/core/ipc/types/xrf-renderer";
import {
  describeRenderGpu,
  fromRenderGpuKey,
  RENDER_GPU_AUTO,
  resolveRenderGpu,
  toRenderGpuChoice,
  toRenderGpuKey,
} from "@/core/render/lib/settings/render-gpu-choice";

const DISCRETE: RenderGpu = { vendor: 0x10de, device: 0x2c02 };
const INTEGRATED: RenderGpu = { vendor: 0x1002, device: 0x13c0 };

const BACKEND: RenderBackendAvailability = {
  backend: ERenderBackend.D3D12,
  gpus: [
    { gpu: DISCRETE, name: "Discrete", driver: "1.0" },
    { gpu: INTEGRATED, name: "Integrated", driver: null },
  ],
  isForced: false,
  problem: null,
};

describe("render GPU choice", () => {
  it("reads a stored choice, the automatic one for anything it does not know", () => {
    expect(toRenderGpuChoice({ vendor: 4318, device: 11266 })).toEqual({ vendor: 4318, device: 11266 });
    expect(toRenderGpuChoice({ vendor: "10de", device: 1 })).toBeNull();
    expect(toRenderGpuChoice(null)).toBeNull();
    expect(toRenderGpuChoice("auto")).toBeNull();
  });

  it("keys a choice for the settings row and reads it back", () => {
    expect(toRenderGpuKey(null)).toBe(RENDER_GPU_AUTO);
    expect(fromRenderGpuKey(toRenderGpuKey(INTEGRATED), BACKEND.gpus)).toEqual(INTEGRATED);
    expect(fromRenderGpuKey(RENDER_GPU_AUTO, BACKEND.gpus)).toBeNull();
  });

  it("draws on the chosen GPU where the backend has it, else on the automatic one", () => {
    expect(resolveRenderGpu(INTEGRATED, BACKEND)?.name).toBe("Integrated");
    expect(resolveRenderGpu(null, BACKEND)?.name).toBe("Discrete");
    expect(resolveRenderGpu({ vendor: 1, device: 2 }, BACKEND)?.name).toBe("Discrete");
    expect(resolveRenderGpu(null, undefined)).toBeNull();
  });

  it("names a GPU with its driver where the API reports one", () => {
    expect(describeRenderGpu(BACKEND.gpus[0])).toBe("Discrete, driver 1.0");
    expect(describeRenderGpu(BACKEND.gpus[1])).toBe("Integrated");
  });
});
