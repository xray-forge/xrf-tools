import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { ERenderAntialiasing, ERenderScale, ERenderTextureFiltering } from "@/core/ipc/types/xrf-renderer";
import { LevelImageQualityAction } from "@/core/level/components/preview/LevelImageQualityAction";
import { ILevelFeatureOptions } from "@/core/level/lib/features/level-feature-options";
import { ERenderResolution } from "@/core/render/lib/settings/render-resolution";
import { mockLevelFeatureOptions } from "@/fixtures/mocks/level.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("LevelImageQualityAction", () => {
  it("picks the view's own mode, and goes back to the settings'", async () => {
    const onChange = jest.fn<(features: ILevelFeatureOptions) => void>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelImageQualityAction
        isOn={true}
        settingsMode={ERenderAntialiasing.SMAA}
        settingsTextureFiltering={ERenderTextureFiltering.ANISOTROPIC8}
        settingsScale={ERenderScale.NATIVE}
        settingsResolution={ERenderResolution.AUTO}
        settingsOutputScale={1}
        hasDlss
        features={mockLevelFeatureOptions()}
        onToggle={() => {}}
        onChange={onChange}
      />
    );

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Image quality" }) });
    await findByRole("dialog", { name: "Image quality" });
    await userEvent.click(getByRole("button", { name: "FXAA" }));

    expect(onChange).toHaveBeenLastCalledWith({
      ...mockLevelFeatureOptions(),
      antialiasing: ERenderAntialiasing.FXAA,
    });

    await userEvent.click(getByRole("button", { name: "Back to the settings" }));

    expect(onChange).toHaveBeenLastCalledWith(mockLevelFeatureOptions());
  });

  it("picks the view's own texture filtering", async () => {
    const onChange = jest.fn<(features: ILevelFeatureOptions) => void>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelImageQualityAction
        isOn
        settingsMode={ERenderAntialiasing.SMAA}
        settingsTextureFiltering={ERenderTextureFiltering.ANISOTROPIC8}
        settingsScale={ERenderScale.NATIVE}
        settingsResolution={ERenderResolution.AUTO}
        settingsOutputScale={1}
        hasDlss
        features={mockLevelFeatureOptions()}
        onToggle={() => {}}
        onChange={onChange}
      />
    );

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Image quality" }) });
    await findByRole("dialog", { name: "Image quality" });
    await userEvent.click(getByRole("button", { name: "16×" }));

    expect(onChange).toHaveBeenLastCalledWith({
      ...mockLevelFeatureOptions(),
      textureFiltering: ERenderTextureFiltering.ANISOTROPIC16,
    });
  });

  it("picks the view's own render scale, and goes back to the settings'", async () => {
    const onChange = jest.fn<(features: ILevelFeatureOptions) => void>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelImageQualityAction
        isOn
        settingsMode={ERenderAntialiasing.DLSS}
        settingsTextureFiltering={ERenderTextureFiltering.ANISOTROPIC8}
        settingsScale={ERenderScale.QUALITY}
        settingsResolution={ERenderResolution.AUTO}
        settingsOutputScale={1}
        hasDlss
        features={mockLevelFeatureOptions()}
        onToggle={() => {}}
        onChange={onChange}
      />
    );

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Image quality" }) });
    await findByRole("dialog", { name: "Image quality" });

    expect(getByRole("button", { name: "Quality" })).toHaveAttribute("aria-pressed", "true");

    await userEvent.click(getByRole("button", { name: "Ultra performance" }));

    expect(onChange).toHaveBeenLastCalledWith({
      ...mockLevelFeatureOptions(),
      upscaling: { scale: ERenderScale.ULTRA_PERFORMANCE },
    });
  });

  it("shows FSR 2 standing in for DLSS where the GPU has none, and offers no DLSS", async () => {
    const { getByRole, findByRole, queryByRole } = renderWithProviders(
      <LevelImageQualityAction
        isOn
        settingsMode={ERenderAntialiasing.DLSS}
        settingsTextureFiltering={ERenderTextureFiltering.ANISOTROPIC8}
        settingsScale={ERenderScale.NATIVE}
        settingsResolution={ERenderResolution.AUTO}
        settingsOutputScale={1}
        hasDlss={false}
        features={mockLevelFeatureOptions()}
        onToggle={() => {}}
        onChange={() => {}}
      />
    );

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Image quality" }) });
    await findByRole("dialog", { name: "Image quality" });

    expect(getByRole("button", { name: "FSR 2" })).toHaveAttribute("aria-pressed", "true");
    expect(queryByRole("button", { name: "DLSS" })).toBeNull();
  });

  it("sets the view's own output resolution, with its scale offered while automatic", async () => {
    const onChange = jest.fn<(features: ILevelFeatureOptions) => void>();
    const { getByRole, findByRole, queryByRole, rerender } = renderWithProviders(
      <LevelImageQualityAction
        isOn
        settingsMode={ERenderAntialiasing.TAA}
        settingsTextureFiltering={ERenderTextureFiltering.ANISOTROPIC8}
        settingsScale={ERenderScale.NATIVE}
        settingsResolution={ERenderResolution.AUTO}
        settingsOutputScale={1}
        hasDlss
        features={mockLevelFeatureOptions()}
        onToggle={() => {}}
        onChange={onChange}
      />
    );

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Image quality" }) });
    await findByRole("dialog", { name: "Image quality" });

    expect(getByRole("slider", { name: "Output scale" })).toBeInTheDocument();

    await userEvent.click(getByRole("button", { name: "1440p" }));

    expect(onChange).toHaveBeenLastCalledWith({
      ...mockLevelFeatureOptions(),
      output: { resolution: ERenderResolution.HEIGHT_1440 },
    });

    rerender(
      <LevelImageQualityAction
        isOn
        settingsMode={ERenderAntialiasing.TAA}
        settingsTextureFiltering={ERenderTextureFiltering.ANISOTROPIC8}
        settingsScale={ERenderScale.NATIVE}
        settingsResolution={ERenderResolution.AUTO}
        settingsOutputScale={1}
        hasDlss
        features={mockLevelFeatureOptions({ output: { resolution: ERenderResolution.HEIGHT_1440 } })}
        onToggle={() => {}}
        onChange={onChange}
      />
    );

    expect(queryByRole("slider", { name: "Output scale" })).toBeNull();
  });

  it("is disabled while the settings smooth nothing", () => {
    const { getByRole } = renderWithProviders(
      <LevelImageQualityAction
        isOn
        settingsMode={ERenderAntialiasing.NONE}
        settingsTextureFiltering={ERenderTextureFiltering.ANISOTROPIC8}
        settingsScale={ERenderScale.NATIVE}
        settingsResolution={ERenderResolution.AUTO}
        settingsOutputScale={1}
        hasDlss
        features={mockLevelFeatureOptions()}
        onToggle={() => {}}
        onChange={() => {}}
      />
    );

    expect(getByRole("button", { name: "Image quality" })).toBeDisabled();
  });
});
