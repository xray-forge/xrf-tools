import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent, RenderResult } from "@testing-library/react";

import { RenderLightingControls } from "@/core/render/components/lighting/RenderLightingControls";
import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { renderWithProviders } from "@/fixtures/utils/render";

const LIGHTING: IRenderLighting = {
  ambientColor: 0xffffff,
  ambientIntensity: 1.5,
  sunAzimuth: 35,
  sunColor: 0xffffff,
  sunElevation: 45,
  sunIntensity: 2,
};

function renderControls(onChange: (lighting: IRenderLighting) => void = jest.fn()): RenderResult {
  return renderWithProviders(<RenderLightingControls lighting={LIGHTING} onChange={onChange} />);
}

describe("RenderLightingControls", () => {
  it("offers the four values a preview is lit by", () => {
    const { getByRole } = renderControls();

    expect(getByRole("slider", { name: "Elevation" })).toHaveValue("45");
    expect(getByRole("slider", { name: "Azimuth" })).toHaveValue("35");
    expect(getByRole("slider", { name: "Light" })).toHaveValue("2");
    expect(getByRole("slider", { name: "Ambient" })).toHaveValue("1.5");
  });

  it("says what each value currently reads, which is the only place a unit is stated", () => {
    const { getByText } = renderControls();

    expect(getByText("45°")).toBeInTheDocument();
    expect(getByText("2.00")).toBeInTheDocument();
  });

  // The whole value travels, so a surface holding one of these never has to merge a partial answer.
  it("answers with the whole lighting, changed in one place", () => {
    const onChange = jest.fn<(lighting: IRenderLighting) => void>();
    const { getByRole } = renderControls(onChange);

    fireEvent.change(getByRole("slider", { name: "Light" }), { target: { value: "4" } });

    expect(onChange).toHaveBeenCalledWith({ ...LIGHTING, sunIntensity: 4 });
  });

  it("keeps the light within the range every surface offers", () => {
    const { getByRole } = renderControls();
    const elevation: HTMLElement = getByRole("slider", { name: "Elevation" });

    expect(elevation).toHaveAttribute("min", "-15");
    expect(elevation).toHaveAttribute("max", "90");
  });
});
