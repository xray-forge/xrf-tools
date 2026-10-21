import { describe, expect, it } from "@jest/globals";

import { ERenderBackend, RenderBackendAvailability } from "@/core/ipc/types/xrf-renderer";
import {
  getForcedRenderBackend,
  listRenderBackendChoices,
  RENDER_BACKEND_AUTO,
  resolveRenderBackend,
  toRenderBackend,
  toRenderBackendChoice,
} from "@/core/render/lib/settings/render-backend-choice";

const BOTH: ReadonlyArray<RenderBackendAvailability> = [
  {
    backend: ERenderBackend.D3D12,
    gpus: [{ gpu: { vendor: 1, device: 1 }, name: "GPU", driver: "1.0" }],
    isForced: false,
    problem: null,
  },
  {
    backend: ERenderBackend.VULKAN,
    gpus: [{ gpu: { vendor: 1, device: 1 }, name: "GPU", driver: "1.0" }],
    isForced: false,
    problem: null,
  },
];
const VULKAN_ONLY: ReadonlyArray<RenderBackendAvailability> = [
  { backend: ERenderBackend.D3D12, gpus: [], isForced: false, problem: "No D3D12 adapter" },
  {
    backend: ERenderBackend.VULKAN,
    gpus: [{ gpu: { vendor: 1, device: 1 }, name: "GPU", driver: "1.0" }],
    isForced: false,
    problem: null,
  },
];

describe("render backend choice", () => {
  it("reads a stored choice, the automatic one for anything it does not know", () => {
    expect(toRenderBackendChoice("vulkan")).toBe(ERenderBackend.VULKAN);
    expect(toRenderBackendChoice("metal")).toBe(ERenderBackend.METAL);
    expect(toRenderBackendChoice("opengl")).toBe(RENDER_BACKEND_AUTO);
    expect(toRenderBackendChoice(null)).toBe(RENDER_BACKEND_AUTO);
  });

  it("offers the backends the renderer probed on this platform, and the one asked for even where it is not one", () => {
    const metal: ReadonlyArray<RenderBackendAvailability> = [{ ...BOTH[0], backend: ERenderBackend.METAL }];

    expect(listRenderBackendChoices(BOTH, RENDER_BACKEND_AUTO)).toEqual([
      RENDER_BACKEND_AUTO,
      ERenderBackend.D3D12,
      ERenderBackend.VULKAN,
    ]);
    expect(listRenderBackendChoices(metal, RENDER_BACKEND_AUTO)).toEqual([RENDER_BACKEND_AUTO, ERenderBackend.METAL]);
    expect(listRenderBackendChoices(metal, ERenderBackend.VULKAN)).toEqual([
      RENDER_BACKEND_AUTO,
      ERenderBackend.VULKAN,
      ERenderBackend.METAL,
    ]);
    expect(listRenderBackendChoices(null, RENDER_BACKEND_AUTO)).toEqual([RENDER_BACKEND_AUTO]);
  });

  it("asks the renderer for the backend named, none for the automatic choice", () => {
    expect(toRenderBackend(ERenderBackend.D3D12)).toBe(ERenderBackend.D3D12);
    expect(toRenderBackend(RENDER_BACKEND_AUTO)).toBeNull();
  });

  it("draws with the backend asked for where it is available, else the first that is, as the renderer falls back", () => {
    expect(resolveRenderBackend(ERenderBackend.VULKAN, BOTH)).toBe(ERenderBackend.VULKAN);
    expect(resolveRenderBackend(RENDER_BACKEND_AUTO, BOTH)).toBe(ERenderBackend.D3D12);
    expect(resolveRenderBackend(ERenderBackend.D3D12, VULKAN_ONLY)).toBe(ERenderBackend.VULKAN);
    expect(resolveRenderBackend(ERenderBackend.D3D12, [])).toBeNull();
  });

  it("draws with the backend the environment forces over the choice, as the renderer starts on it", () => {
    const forced: ReadonlyArray<RenderBackendAvailability> = BOTH.map((it: RenderBackendAvailability) => ({
      ...it,
      isForced: it.backend === ERenderBackend.VULKAN,
    }));

    expect(getForcedRenderBackend(forced)).toBe(ERenderBackend.VULKAN);
    expect(getForcedRenderBackend(BOTH)).toBeNull();
    expect(resolveRenderBackend(ERenderBackend.D3D12, forced)).toBe(ERenderBackend.VULKAN);
  });
});
