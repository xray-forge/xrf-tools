import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { RenderLightingAction } from "@/core/render/components/lighting/RenderLightingAction";
import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { renderWithProviders } from "@/fixtures/utils/render";

const FALLBACK: IRenderLighting = {
  ambientColor: 0xffffff,
  ambientIntensity: 0.4,
  sunAzimuth: 30,
  sunColor: 0xffffff,
  sunElevation: 45,
  sunIntensity: 1,
};

describe("RenderLightingAction", () => {
  it("puts the surface's own light back from the heading, only while the light differs from it", async () => {
    const onChange = jest.fn();
    const { getByRole, queryByRole, rerender } = renderWithProviders(
      <RenderLightingAction lighting={FALLBACK} fallback={FALLBACK} onChange={onChange} />
    );
    const reset: { name: string } = { name: "Lighting: Back to the default light" };

    await userEvent.click(getByRole("button", { name: "Lighting" }));

    expect(queryByRole("button", reset)).toBeNull();

    rerender(
      <RenderLightingAction lighting={{ ...FALLBACK, sunElevation: 10 }} fallback={FALLBACK} onChange={onChange} />
    );
    await userEvent.click(getByRole("button", reset));

    expect(onChange).toHaveBeenCalledWith(FALLBACK);
  });
});
