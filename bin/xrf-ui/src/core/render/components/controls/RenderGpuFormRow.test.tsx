import { describe, expect, it, jest } from "@jest/globals";
import { render } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Nullable } from "@xrf/types";

import { ERenderBackend, RenderBackendAvailability, RenderGpu } from "@/core/ipc/types/xrf-renderer";
import { RENDER_BACKEND_AUTO } from "@/core/render/lib/settings/render-backend-choice";

import { RenderGpuFormRow } from "./RenderGpuFormRow";

const DISCRETE: RenderGpu = { vendor: 0x10de, device: 0x2c02 };
const INTEGRATED: RenderGpu = { vendor: 0x8086, device: 0xa780 };

const TWO_GPUS: ReadonlyArray<RenderBackendAvailability> = [
  {
    backend: ERenderBackend.D3D12,
    gpus: [
      { gpu: DISCRETE, name: "Test GPU", driver: "1.0" },
      { gpu: INTEGRATED, name: "Integrated GPU", driver: null },
    ],
    isForced: false,
    problem: null,
  },
];

describe("RenderGpuFormRow", () => {
  it("offers the automatic GPU and each GPU of the graphics API drawn with", async () => {
    const onChange = jest.fn<(gpu: Nullable<RenderGpu>) => void>();
    const view = render(
      <RenderGpuFormRow value={null} backend={RENDER_BACKEND_AUTO} availability={TWO_GPUS} onChange={onChange} />
    );

    expect(view.getByRole("group", { name: "GPU" })).toHaveAccessibleDescription(/Drawing on Test GPU, driver 1\.0\./);

    await userEvent.click(view.getByRole("button", { name: "Integrated GPU" }));

    expect(onChange).toHaveBeenCalledWith(INTEGRATED);

    view.rerender(
      <RenderGpuFormRow value={INTEGRATED} backend={RENDER_BACKEND_AUTO} availability={TWO_GPUS} onChange={onChange} />
    );

    await userEvent.click(view.getByRole("button", { name: "Automatic" }));

    expect(onChange).toHaveBeenLastCalledWith(null);
  });

  it("says the renderer uses the automatic GPU where the chosen one is missing", () => {
    const view = render(
      <RenderGpuFormRow
        value={{ vendor: 1, device: 2 }}
        backend={ERenderBackend.D3D12}
        availability={TWO_GPUS}
        onChange={jest.fn()}
      />
    );

    expect(view.getByRole("group", { name: "GPU" })).toHaveAccessibleDescription(
      /The chosen GPU is not available, so the renderer uses the automatic one\. Drawing on Test GPU/
    );
  });
});
