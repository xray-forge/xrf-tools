import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import {
  ERenderAntialiasing,
  ERenderBackend,
  ERenderDlssAvailability,
  ERenderDlssPreset,
  ERenderScale,
  ERenderTextureFiltering,
} from "@/core/ipc/types/xrf-renderer";
import { LevelImageQualityAction } from "@/core/level/components/preview/LevelImageQualityAction";
import { ILevelFeatureOptions } from "@/core/level/lib/features/level-feature-options";
import { RENDER_BACKEND_AUTO, TRenderBackendChoice } from "@/core/render/lib/settings/render-backend-choice";
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
        settingsSharpening={0.5}
        settingsDlssPreset={ERenderDlssPreset.DEFAULT}
        settingsRayReconstruction={false}
        isSceneLinear
        settingsResolution={ERenderResolution.AUTO}
        settingsOutputScale={1}
        backend={RENDER_BACKEND_AUTO}
        backends={null}
        isUpscaled
        dlss={ERenderDlssAvailability.AVAILABLE}
        rayReconstruction={ERenderDlssAvailability.AVAILABLE}
        features={mockLevelFeatureOptions()}
        onToggle={() => {}}
        onChange={onChange}
        onChangeBackend={() => {}}
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
        settingsSharpening={0.5}
        settingsDlssPreset={ERenderDlssPreset.DEFAULT}
        settingsRayReconstruction={false}
        isSceneLinear
        settingsResolution={ERenderResolution.AUTO}
        settingsOutputScale={1}
        backend={RENDER_BACKEND_AUTO}
        backends={null}
        isUpscaled
        dlss={ERenderDlssAvailability.AVAILABLE}
        rayReconstruction={ERenderDlssAvailability.AVAILABLE}
        features={mockLevelFeatureOptions()}
        onToggle={() => {}}
        onChange={onChange}
        onChangeBackend={() => {}}
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
        settingsSharpening={0.5}
        settingsDlssPreset={ERenderDlssPreset.DEFAULT}
        settingsRayReconstruction={false}
        isSceneLinear
        settingsResolution={ERenderResolution.AUTO}
        settingsOutputScale={1}
        backend={RENDER_BACKEND_AUTO}
        backends={null}
        isUpscaled
        dlss={ERenderDlssAvailability.AVAILABLE}
        rayReconstruction={ERenderDlssAvailability.AVAILABLE}
        features={mockLevelFeatureOptions()}
        onToggle={() => {}}
        onChange={onChange}
        onChangeBackend={() => {}}
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

  it("turns DLSS ray reconstruction on for the view, hiding the super resolution model it replaces", async () => {
    const onChange = jest.fn<(features: ILevelFeatureOptions) => void>();
    const { getByRole, findByRole, queryByRole } = renderWithProviders(
      <LevelImageQualityAction
        isOn
        settingsMode={ERenderAntialiasing.DLSS}
        settingsTextureFiltering={ERenderTextureFiltering.ANISOTROPIC8}
        settingsScale={ERenderScale.QUALITY}
        settingsSharpening={0.5}
        settingsDlssPreset={ERenderDlssPreset.DEFAULT}
        settingsRayReconstruction={true}
        isSceneLinear
        settingsResolution={ERenderResolution.AUTO}
        settingsOutputScale={1}
        backend={RENDER_BACKEND_AUTO}
        backends={null}
        isUpscaled
        dlss={ERenderDlssAvailability.AVAILABLE}
        rayReconstruction={ERenderDlssAvailability.AVAILABLE}
        features={mockLevelFeatureOptions()}
        onToggle={() => {}}
        onChange={onChange}
        onChangeBackend={() => {}}
      />
    );

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Image quality" }) });
    await findByRole("dialog", { name: "Image quality" });

    expect(getByRole("checkbox", { name: "Ray reconstruction" })).toBeChecked();
    expect(queryByRole("button", { name: "Default" })).toBeNull();

    await userEvent.click(getByRole("checkbox", { name: "Ray reconstruction" }));

    expect(onChange).toHaveBeenLastCalledWith({
      ...mockLevelFeatureOptions(),
      upscaling: { isRayReconstructed: false },
    });
  });

  it("greys DLSS ray reconstruction out with what it needs: the GPU's support, then the Enhanced tonemap", async () => {
    const backend: TRenderBackendChoice = RENDER_BACKEND_AUTO;
    const props = {
      isOn: true,
      settingsMode: ERenderAntialiasing.DLSS,
      settingsTextureFiltering: ERenderTextureFiltering.ANISOTROPIC8,
      settingsScale: ERenderScale.QUALITY,
      settingsSharpening: 0.5,
      settingsDlssPreset: ERenderDlssPreset.DEFAULT,
      settingsRayReconstruction: true,
      settingsResolution: ERenderResolution.AUTO,
      settingsOutputScale: 1,
      backend,
      backends: null,
      isUpscaled: true,
      dlss: ERenderDlssAvailability.AVAILABLE,
      features: mockLevelFeatureOptions(),
      onToggle: () => {},
      onChange: () => {},
      onChangeBackend: () => {},
    };
    const { getByRole, findByRole, rerender } = renderWithProviders(
      <LevelImageQualityAction {...props} isSceneLinear rayReconstruction={ERenderDlssAvailability.UNSUPPORTED} />
    );

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Image quality" }) });
    await findByRole("dialog", { name: "Image quality" });

    expect(getByRole("checkbox", { name: "Ray reconstruction" })).toBeDisabled();
    expect(getByRole("checkbox", { name: "Ray reconstruction" })).not.toBeChecked();
    expect(getByRole("checkbox", { name: "Ray reconstruction" })).toHaveAccessibleDescription(
      "Needs an NVIDIA RTX GPU and a driver with DLSS ray reconstruction."
    );

    rerender(
      <LevelImageQualityAction {...props} isSceneLinear={false} rayReconstruction={ERenderDlssAvailability.AVAILABLE} />
    );

    expect(getByRole("checkbox", { name: "Ray reconstruction" })).toBeDisabled();
    expect(getByRole("checkbox", { name: "Ray reconstruction" })).toHaveAccessibleDescription(
      "Needs the Enhanced tonemap: ray reconstruction works on linear light."
    );
  });

  it("shows FSR 2 standing in for DLSS where the GPU has none, DLSS greyed out with what it needs", async () => {
    const { getByRole, findByRole, getByText } = renderWithProviders(
      <LevelImageQualityAction
        isOn
        settingsMode={ERenderAntialiasing.DLSS}
        settingsTextureFiltering={ERenderTextureFiltering.ANISOTROPIC8}
        settingsScale={ERenderScale.NATIVE}
        settingsSharpening={0.5}
        settingsDlssPreset={ERenderDlssPreset.DEFAULT}
        settingsRayReconstruction={false}
        isSceneLinear
        settingsResolution={ERenderResolution.AUTO}
        settingsOutputScale={1}
        backend={RENDER_BACKEND_AUTO}
        backends={null}
        isUpscaled
        dlss={ERenderDlssAvailability.UNSUPPORTED}
        rayReconstruction={ERenderDlssAvailability.UNSUPPORTED}
        features={mockLevelFeatureOptions()}
        onToggle={() => {}}
        onChange={() => {}}
        onChangeBackend={() => {}}
      />
    );

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Image quality" }) });
    await findByRole("dialog", { name: "Image quality" });

    expect(getByRole("button", { name: "FSR 2" })).toHaveAttribute("aria-pressed", "true");
    expect(getByRole("button", { name: "DLSS" })).toBeDisabled();
    expect(getByText("DLSS needs an NVIDIA RTX GPU and a current driver.")).toBeInTheDocument();
  });

  it("sets the view's own output resolution, with its scale offered while automatic", async () => {
    const onChange = jest.fn<(features: ILevelFeatureOptions) => void>();
    const { getByRole, findByRole, queryByRole, rerender } = renderWithProviders(
      <LevelImageQualityAction
        isOn
        settingsMode={ERenderAntialiasing.TAA}
        settingsTextureFiltering={ERenderTextureFiltering.ANISOTROPIC8}
        settingsScale={ERenderScale.NATIVE}
        settingsSharpening={0.5}
        settingsDlssPreset={ERenderDlssPreset.DEFAULT}
        settingsRayReconstruction={false}
        isSceneLinear
        settingsResolution={ERenderResolution.AUTO}
        settingsOutputScale={1}
        backend={RENDER_BACKEND_AUTO}
        backends={null}
        isUpscaled
        dlss={ERenderDlssAvailability.AVAILABLE}
        rayReconstruction={ERenderDlssAvailability.AVAILABLE}
        features={mockLevelFeatureOptions()}
        onToggle={() => {}}
        onChange={onChange}
        onChangeBackend={() => {}}
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
        settingsSharpening={0.5}
        settingsDlssPreset={ERenderDlssPreset.DEFAULT}
        settingsRayReconstruction={false}
        isSceneLinear
        settingsResolution={ERenderResolution.AUTO}
        settingsOutputScale={1}
        backend={RENDER_BACKEND_AUTO}
        backends={null}
        isUpscaled
        dlss={ERenderDlssAvailability.AVAILABLE}
        rayReconstruction={ERenderDlssAvailability.AVAILABLE}
        features={mockLevelFeatureOptions({ output: { resolution: ERenderResolution.HEIGHT_1440 } })}
        onToggle={() => {}}
        onChange={onChange}
        onChangeBackend={() => {}}
      />
    );

    expect(queryByRole("slider", { name: "Output scale" })).toBeNull();
  });

  it("picks the view's own DLSS model while DLSS smooths it, and its own sharpening", async () => {
    const onChange = jest.fn<(features: ILevelFeatureOptions) => void>();
    const { getByRole, findByRole, getByText } = renderWithProviders(
      <LevelImageQualityAction
        isOn
        settingsMode={ERenderAntialiasing.DLSS}
        settingsTextureFiltering={ERenderTextureFiltering.ANISOTROPIC8}
        settingsScale={ERenderScale.QUALITY}
        settingsSharpening={0.5}
        settingsDlssPreset={ERenderDlssPreset.DEFAULT}
        settingsRayReconstruction={false}
        isSceneLinear
        settingsResolution={ERenderResolution.AUTO}
        settingsOutputScale={1}
        backend={RENDER_BACKEND_AUTO}
        backends={null}
        isUpscaled
        dlss={ERenderDlssAvailability.AVAILABLE}
        rayReconstruction={ERenderDlssAvailability.AVAILABLE}
        features={mockLevelFeatureOptions({ upscaling: { scale: ERenderScale.BALANCED } })}
        onToggle={() => {}}
        onChange={onChange}
        onChangeBackend={() => {}}
      />
    );

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Image quality" }) });
    await findByRole("dialog", { name: "Image quality" });

    expect(getByText(/NVIDIA's pick by quality/)).toBeInTheDocument();
    expect(getByRole("slider", { name: "Sharpening" })).toBeInTheDocument();

    await userEvent.click(getByRole("button", { name: "K" }));

    expect(onChange).toHaveBeenLastCalledWith({
      ...mockLevelFeatureOptions(),
      upscaling: { scale: ERenderScale.BALANCED, dlssPreset: ERenderDlssPreset.K },
    });
  });

  it("draws every viewport with the graphics API picked, greying out one this machine cannot draw with", async () => {
    const onChangeBackend = jest.fn<(backend: TRenderBackendChoice) => void>();
    const { getByRole, findByRole, getByText } = renderWithProviders(
      <LevelImageQualityAction
        isOn
        settingsMode={ERenderAntialiasing.FXAA}
        settingsTextureFiltering={ERenderTextureFiltering.ANISOTROPIC8}
        settingsScale={ERenderScale.NATIVE}
        settingsSharpening={0.5}
        settingsDlssPreset={ERenderDlssPreset.DEFAULT}
        settingsRayReconstruction={false}
        isSceneLinear
        settingsResolution={ERenderResolution.AUTO}
        settingsOutputScale={1}
        backend={RENDER_BACKEND_AUTO}
        backends={[
          { adapter: "Test GPU", backend: ERenderBackend.D3D12, problem: null },
          { adapter: null, backend: ERenderBackend.VULKAN, problem: "No Vulkan here" },
        ]}
        isUpscaled={false}
        dlss={ERenderDlssAvailability.UNSUPPORTED}
        rayReconstruction={ERenderDlssAvailability.UNSUPPORTED}
        features={mockLevelFeatureOptions()}
        onToggle={() => {}}
        onChange={() => {}}
        onChangeBackend={onChangeBackend}
      />
    );

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Image quality" }) });
    await findByRole("dialog", { name: "Image quality" });

    expect(getByRole("button", { name: "Vulkan" })).toBeDisabled();
    expect(getByRole("slider", { name: "Sharpening" })).toBeDisabled();
    expect(
      getByText("Sharpens only a frame drawn smaller than the viewport or resolved temporally.")
    ).toBeInTheDocument();

    await userEvent.click(getByRole("button", { name: "Direct3D 12" }));

    expect(onChangeBackend).toHaveBeenLastCalledWith(ERenderBackend.D3D12);
  });

  it("is disabled while the settings smooth nothing", () => {
    const { getByRole } = renderWithProviders(
      <LevelImageQualityAction
        isOn
        settingsMode={ERenderAntialiasing.NONE}
        settingsTextureFiltering={ERenderTextureFiltering.ANISOTROPIC8}
        settingsScale={ERenderScale.NATIVE}
        settingsSharpening={0.5}
        settingsDlssPreset={ERenderDlssPreset.DEFAULT}
        settingsRayReconstruction={false}
        isSceneLinear
        settingsResolution={ERenderResolution.AUTO}
        settingsOutputScale={1}
        backend={RENDER_BACKEND_AUTO}
        backends={null}
        isUpscaled
        dlss={ERenderDlssAvailability.AVAILABLE}
        rayReconstruction={ERenderDlssAvailability.AVAILABLE}
        features={mockLevelFeatureOptions()}
        onToggle={() => {}}
        onChange={() => {}}
        onChangeBackend={() => {}}
      />
    );

    expect(getByRole("button", { name: "Image quality" })).toBeDisabled();
  });
});
