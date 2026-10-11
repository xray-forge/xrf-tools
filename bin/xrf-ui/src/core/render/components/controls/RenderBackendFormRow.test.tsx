import { describe, expect, it, jest } from "@jest/globals";
import { render } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { ERenderBackend, RenderBackendAvailability } from "@/core/ipc/types/xrf-renderer";
import { RENDER_BACKEND_AUTO, TRenderBackendChoice } from "@/core/render/lib/settings/render-backend-choice";

import { RenderBackendFormRow } from "./RenderBackendFormRow";

const VULKAN_ONLY: ReadonlyArray<RenderBackendAvailability> = [
  { adapter: null, backend: ERenderBackend.D3D12, driver: null, isForced: false, problem: "No D3D12 adapter" },
  { adapter: "Test GPU", backend: ERenderBackend.VULKAN, driver: "1.0", isForced: false, problem: null },
];

describe("RenderBackendFormRow", () => {
  it("offers only the graphics APIs this machine can draw with, greying out the rest", async () => {
    const onChange = jest.fn<(choice: TRenderBackendChoice) => void>();
    const view = render(
      <RenderBackendFormRow value={RENDER_BACKEND_AUTO} availability={VULKAN_ONLY} onChange={onChange} />
    );

    expect(view.getByRole("button", { name: "Direct3D 12" })).toBeDisabled();
    expect(view.getByRole("group", { name: "Graphics API" })).toHaveAccessibleDescription(
      /Drawing with Vulkan on Test GPU\./
    );

    await userEvent.click(view.getByRole("button", { name: "Vulkan" }));

    expect(onChange).toHaveBeenCalledWith(ERenderBackend.VULKAN);
  });

  it("says the renderer fell back where the remembered API is not available here", () => {
    const view = render(
      <RenderBackendFormRow value={ERenderBackend.D3D12} availability={VULKAN_ONLY} onChange={jest.fn()} />
    );

    expect(view.getByRole("group", { name: "Graphics API" })).toHaveAccessibleDescription(
      /Direct3D 12 is not available here, so the renderer fell back\. Drawing with Vulkan on Test GPU\./
    );
  });

  it("says the environment asks for an API over the choice", () => {
    const forced: ReadonlyArray<RenderBackendAvailability> = VULKAN_ONLY.map((it: RenderBackendAvailability) => ({
      ...it,
      isForced: it.backend === ERenderBackend.VULKAN,
    }));
    const view = render(
      <RenderBackendFormRow value={ERenderBackend.D3D12} availability={forced} onChange={jest.fn()} />
    );

    expect(view.getByRole("group", { name: "Graphics API" })).toHaveAccessibleDescription(
      /XRF_RENDER_BACKEND overrides this choice with Vulkan\. Drawing with Vulkan on Test GPU\./
    );
  });

  it("offers every API before the renderer was asked which work", () => {
    const view = render(<RenderBackendFormRow value={RENDER_BACKEND_AUTO} availability={null} onChange={jest.fn()} />);

    expect(view.getByRole("button", { name: "Direct3D 12" })).toBeEnabled();
    expect(view.getByRole("button", { name: "Vulkan" })).toBeEnabled();
  });
});
