import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { ERendererRenderScale } from "@xrf/renderer";

import { LevelRenderScaleAction } from "@/core/level/components/preview/LevelRenderScaleAction";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("LevelRenderScaleAction", () => {
  it("sets every viewport's render scale", async () => {
    const onChange = jest.fn<(scale: ERendererRenderScale) => void>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelRenderScaleAction scale={ERendererRenderScale.NATIVE} onChange={onChange} />
    );

    await userEvent.click(getByRole("button", { name: "Render scale" }));
    await findByRole("dialog", { name: "Render scale" });
    await userEvent.click(getByRole("button", { name: "Performance" }));

    expect(onChange).toHaveBeenLastCalledWith(ERendererRenderScale.PERFORMANCE);
  });
});
