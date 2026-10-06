import { describe, expect, it } from "@jest/globals";

import { ERenderBackend, RenderBackendAvailability } from "@/core/ipc/types/xrf-renderer";
import {
  RENDER_BACKEND_AUTO,
  resolveRenderBackend,
  toRenderBackend,
  toRenderBackendChoice,
} from "@/core/render/lib/settings/render-backend-choice";

const BOTH: ReadonlyArray<RenderBackendAvailability> = [
  { adapter: "GPU", backend: ERenderBackend.D3D12, problem: null },
  { adapter: "GPU", backend: ERenderBackend.VULKAN, problem: null },
];
const VULKAN_ONLY: ReadonlyArray<RenderBackendAvailability> = [
  { adapter: null, backend: ERenderBackend.D3D12, problem: "No D3D12 adapter" },
  { adapter: "GPU", backend: ERenderBackend.VULKAN, problem: null },
];

describe("render backend choice", () => {
  it("reads a stored choice, the automatic one for anything it does not know", () => {
    expect(toRenderBackendChoice("vulkan")).toBe(ERenderBackend.VULKAN);
    expect(toRenderBackendChoice("metal")).toBe(RENDER_BACKEND_AUTO);
    expect(toRenderBackendChoice(null)).toBe(RENDER_BACKEND_AUTO);
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
});
