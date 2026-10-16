import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";
import { act, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import {
  ERenderAntialiasing,
  ERenderBackend,
  ERenderDlssAvailability,
  ERenderDlssPreset,
  ERenderScale,
  ERenderTextureFiltering,
  ERenderTonemapMode,
  RenderAntialiasing,
  RenderBackendAvailability,
  RenderDlssAvailability,
  RenderScale,
} from "@/core/ipc/types/xrf-renderer";
import { LevelImageQualityAction } from "@/core/level/components/preview/toolbar/image/LevelImageQualityAction";
import { ILevelFeatureOptions } from "@/core/level/lib/features/level-feature-options";
import { ERenderResolution } from "@/core/render/lib/settings/render-resolution";
import { clearLocalStorage } from "@/core/storage";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  noteLevelFrame,
  renderLevelView,
  TLevelViewRenderResult,
} from "@/fixtures/utils/level-container";

/** What the settings draw with, what the GPU can do, and what the view already sets, as one case needs them. */
interface IImageQualitySetup {
  mode?: RenderAntialiasing;
  scale?: RenderScale;
  isRayReconstructed?: boolean;
  isSceneLinear?: boolean;
  isUpscaled?: boolean;
  dlss?: RenderDlssAvailability;
  rayReconstruction?: RenderDlssAvailability;
  features?: Partial<ILevelFeatureOptions>;
  backends?: Array<RenderBackendAvailability>;
}

/**
 * @param setup - The case: SMAA at the native scale on a GPU with DLSS, drawn smaller than the viewport, unless said.
 * @returns The rendered action and the services behind it.
 */
function renderAction(setup: IImageQualitySetup = {}): TLevelViewRenderResult {
  if (setup.backends) {
    setMockInvokeResponses({ ["plugin:render|list_backends"]: setup.backends });
  }

  const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();
  const isUpscaled: boolean = setup.isUpscaled ?? true;

  descriptor.renderSettings.setRendererOverrides({
    antialiasing: setup.mode ?? ERenderAntialiasing.SMAA,
    textureFiltering: ERenderTextureFiltering.ANISOTROPIC8,
    tonemap: setup.isSceneLinear === false ? ERenderTonemapMode.ENGINE : ERenderTonemapMode.ENHANCED,
    upscaling: {
      dlssPreset: ERenderDlssPreset.DEFAULT,
      isRayReconstructed: setup.isRayReconstructed ?? false,
      scale: setup.scale ?? ERenderScale.NATIVE,
      sharpening: 0.5,
    },
  });

  if (setup.features) {
    descriptor.view.setFeatures({ ...descriptor.view.features, ...setup.features });
  }

  noteLevelFrame(descriptor.render, {
    adapter: "Test GPU",
    dlss: setup.dlss ?? ERenderDlssAvailability.AVAILABLE,
    hasRayQueries: true,
    rayReconstruction: setup.rayReconstruction ?? ERenderDlssAvailability.AVAILABLE,
    renderWidth: isUpscaled ? 960 : 1920,
    width: 1920,
  });

  return renderLevelView(<LevelImageQualityAction />, descriptor);
}

/** Opens the popover by its settings gesture. */
async function openSettings(rendered: TLevelViewRenderResult): Promise<void> {
  await userEvent.pointer({ keys: "[MouseRight]", target: rendered.getByRole("button", { name: "Image quality" }) });
  await rendered.findByRole("dialog", { name: "Image quality" });
}

beforeEach(() => {
  clearLocalStorage();
});

afterEach(() => {
  resetMockInvoke();
});

describe("LevelImageQualityAction", () => {
  it("picks the view's own mode, and goes back to the settings'", async () => {
    const rendered = renderAction();
    const before: ILevelFeatureOptions = rendered.view.features;

    await openSettings(rendered);
    await userEvent.click(rendered.getByRole("button", { name: "FXAA" }));

    expect(rendered.view.features).toEqual({ ...before, antialiasing: ERenderAntialiasing.FXAA });

    await userEvent.click(rendered.getByRole("button", { name: "Back to the settings" }));

    expect(rendered.view.features).toEqual(before);
  });

  it("picks the view's own texture filtering", async () => {
    const rendered = renderAction();

    await openSettings(rendered);
    await userEvent.click(rendered.getByRole("button", { name: "16×" }));

    expect(rendered.view.features.textureFiltering).toBe(ERenderTextureFiltering.ANISOTROPIC16);
  });

  it("picks the view's own render scale over the settings'", async () => {
    const rendered = renderAction({ mode: ERenderAntialiasing.DLSS, scale: ERenderScale.QUALITY });

    await openSettings(rendered);

    expect(rendered.getByRole("button", { name: "Quality" })).toHaveAttribute("aria-pressed", "true");

    await userEvent.click(rendered.getByRole("button", { name: "Ultra performance" }));

    expect(rendered.view.features.upscaling).toEqual({ scale: ERenderScale.ULTRA_PERFORMANCE });
  });

  it("turns DLSS ray reconstruction on for the view, hiding the super resolution model it replaces", async () => {
    const rendered = renderAction({
      isRayReconstructed: true,
      mode: ERenderAntialiasing.DLSS,
      scale: ERenderScale.QUALITY,
    });

    await openSettings(rendered);

    expect(rendered.getByRole("checkbox", { name: "Ray reconstruction" })).toBeChecked();
    expect(rendered.queryByRole("button", { name: "Default" })).toBeNull();

    await userEvent.click(rendered.getByRole("checkbox", { name: "Ray reconstruction" }));

    expect(rendered.view.features.upscaling).toEqual({ isRayReconstructed: false });
  });

  it("greys DLSS ray reconstruction out with what it needs: the GPU's support, then the Enhanced tonemap", async () => {
    const rendered = renderAction({
      isRayReconstructed: true,
      mode: ERenderAntialiasing.DLSS,
      rayReconstruction: ERenderDlssAvailability.UNSUPPORTED,
      scale: ERenderScale.QUALITY,
    });

    await openSettings(rendered);

    expect(rendered.getByRole("checkbox", { name: "Ray reconstruction" })).toBeDisabled();
    expect(rendered.getByRole("checkbox", { name: "Ray reconstruction" })).not.toBeChecked();
    expect(rendered.getByRole("checkbox", { name: "Ray reconstruction" })).toHaveAccessibleDescription(
      "Needs an NVIDIA RTX GPU and a driver with DLSS ray reconstruction."
    );

    noteLevelFrame(rendered.render, {
      adapter: "Test GPU",
      dlss: ERenderDlssAvailability.AVAILABLE,
      rayReconstruction: ERenderDlssAvailability.AVAILABLE,
      renderWidth: 960,
      width: 1920,
    });
    act(() => rendered.renderSettings.setRendererOverrides({ tonemap: ERenderTonemapMode.ENGINE }));

    expect(rendered.getByRole("checkbox", { name: "Ray reconstruction" })).toBeDisabled();
    expect(rendered.getByRole("checkbox", { name: "Ray reconstruction" })).toHaveAccessibleDescription(
      "Needs the Enhanced tonemap: ray reconstruction works on linear light."
    );
  });

  it("shows FSR 2 standing in for DLSS where the GPU has none, DLSS greyed out with what it needs", async () => {
    const rendered = renderAction({
      dlss: ERenderDlssAvailability.UNSUPPORTED,
      mode: ERenderAntialiasing.DLSS,
      rayReconstruction: ERenderDlssAvailability.UNSUPPORTED,
    });

    await openSettings(rendered);

    expect(rendered.getByRole("button", { name: "FSR 2" })).toHaveAttribute("aria-pressed", "true");
    expect(rendered.getByRole("button", { name: "DLSS" })).toBeDisabled();
    expect(rendered.getByText("DLSS needs an NVIDIA RTX GPU and a current driver.")).toBeInTheDocument();
  });

  it("sets the view's own output resolution, with its scale offered while automatic", async () => {
    const rendered = renderAction({ mode: ERenderAntialiasing.TAA });

    await openSettings(rendered);

    expect(rendered.getByRole("slider", { name: "Output scale" })).toBeInTheDocument();

    await userEvent.click(rendered.getByRole("button", { name: "1440p" }));

    expect(rendered.view.features.output).toEqual({ resolution: ERenderResolution.HEIGHT_1440 });
    expect(rendered.queryByRole("slider", { name: "Output scale" })).toBeNull();
  });

  it("picks the view's own DLSS model while DLSS smooths it, and its own sharpening", async () => {
    const rendered = renderAction({
      features: { upscaling: { scale: ERenderScale.BALANCED } },
      mode: ERenderAntialiasing.DLSS,
      scale: ERenderScale.QUALITY,
    });

    await openSettings(rendered);

    expect(rendered.getByText(/NVIDIA's pick by quality/)).toBeInTheDocument();
    expect(rendered.getByRole("slider", { name: "Sharpening" })).toBeInTheDocument();

    await userEvent.click(rendered.getByRole("button", { name: "K" }));

    expect(rendered.view.features.upscaling).toEqual({ dlssPreset: ERenderDlssPreset.K, scale: ERenderScale.BALANCED });
  });

  it("draws every viewport with the graphics API picked, greying out one this machine cannot draw with", async () => {
    const rendered = renderAction({
      backends: [
        {
          backend: ERenderBackend.D3D12,
          gpus: [{ gpu: { vendor: 1, device: 1 }, name: "Test GPU", driver: "1.0" }],
          isForced: false,
          problem: null,
        },
        { backend: ERenderBackend.VULKAN, gpus: [], isForced: false, problem: "No Vulkan here" },
      ],
      dlss: ERenderDlssAvailability.UNSUPPORTED,
      isUpscaled: false,
      mode: ERenderAntialiasing.FXAA,
      rayReconstruction: ERenderDlssAvailability.UNSUPPORTED,
    });

    await openSettings(rendered);

    // The machine's graphics APIs are asked for as the action first draws.
    await waitFor(() => expect(rendered.getByRole("button", { name: "Vulkan" })).toBeDisabled());
    expect(rendered.getByRole("slider", { name: "Sharpening" })).toBeDisabled();
    expect(
      rendered.getByText("Sharpens only a frame drawn smaller than the viewport or resolved temporally.")
    ).toBeInTheDocument();

    await userEvent.click(rendered.getByRole("button", { name: "Direct3D 12" }));

    expect(rendered.renderSettings.renderBackend).toBe(ERenderBackend.D3D12);
  });

  it("is disabled while the settings smooth nothing", () => {
    const rendered = renderAction({ mode: ERenderAntialiasing.NONE });

    expect(rendered.getByRole("button", { name: "Image quality" })).toBeDisabled();
  });
});
