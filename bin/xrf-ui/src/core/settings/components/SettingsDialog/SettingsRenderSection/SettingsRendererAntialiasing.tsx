import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import {
  ERenderAntialiasing,
  ERenderScale,
  ERenderTonemapMode,
  RenderAntialiasing,
  RenderDlssPreset,
  RenderScale,
  RenderTextureFiltering,
} from "@/core/ipc/types/xrf-renderer";
import {
  formatOutputScale,
  formatSharpening,
  isTemporalAntialiasing,
  RENDER_ANTIALIASING_OPTIONS,
  RENDER_DLSS_PRESET_OPTIONS,
  RENDER_OUTPUT_SCALE_LIMITS,
  RENDER_RESOLUTION_OPTIONS,
  RENDER_SCALE_OPTIONS,
  RENDER_SHARPENING_LIMITS,
  RENDER_TEXTURE_FILTERING_OPTIONS,
} from "@/core/render/lib/features";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";
import { ERenderResolution } from "@/core/render/lib/settings/render-resolution";
import { SettingsService } from "@/core/settings/services/settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { ChoiceFormRow, IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow";
import { SliderFormRow } from "@/core/ui/form/SliderFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";

/** The game's texture qualities, `texture_lod`, by the Anomaly preset that sets each. */
const TEXTURE_LOD_OPTIONS: ReadonlyArray<IChoiceFormRowOption<string>> = [
  { label: "Full (extreme)", value: "0" },
  { label: "High", value: "1" },
  { label: "Medium (default)", value: "2" },
  { label: "Low", value: "3" },
  { label: "Lowest", value: "4" },
];

/**
 * What every viewport resolves its frame at and how much of that the scene is drawn at before it is upscaled to it,
 * side by side, then how the frame's edges are smoothed and its textures sampled.
 */
export function SettingsRendererAntialiasing(): ReactElement {
  const settingsService: SettingsService = useInjection(SettingsService);

  const features: IRenderFeatureSettings = settingsService.rendererFeatures;

  return (
    <DetailSection
      title={"Resolution and antialiasing"}
      description={"Output resolution, upscaling quality and edge smoothing."}
    >
      <div className={"mt-4 flex flex-col gap-6"}>
        <ChoiceFormRow
          label={"Output resolution"}
          description={
            "What every viewport's frame is resolved at before it is fitted to the viewport. Auto follows the " +
            "viewport's own size, scaled below."
          }
          options={RENDER_RESOLUTION_OPTIONS}
          value={settingsService.renderResolution}
          onChange={settingsService.setRenderResolution}
        />

        {settingsService.renderResolution === ERenderResolution.AUTO ? (
          <SliderFormRow
            label={"Output scale"}
            description={
              "Share of the viewport's size the frame is resolved at: below 100% upscales, above supersamples."
            }
            value={settingsService.renderOutputScale}
            {...RENDER_OUTPUT_SCALE_LIMITS}
            format={formatOutputScale}
            onChange={settingsService.setRenderOutputScale}
          />
        ) : null}

        <ChoiceFormRow
          label={"Upscaling quality"}
          description={
            "Share of the output resolution the scene is drawn at. TAA, FSR 2 and DLSS upscale temporally; other " +
            "modes use FSR 1."
          }
          options={RENDER_SCALE_OPTIONS}
          value={features.upscaling.scale}
          onChange={(scale: RenderScale) => settingsService.setRendererOverrides({ upscaling: { scale } })}
        />

        <ChoiceFormRow
          label={"Antialiasing"}
          description={
            "SMAA: sharp and stable. FXAA: cheaper, softer. TAA: temporal, also smooths foliage and thin geometry. " +
            "FSR 2: AMD's temporal upscaler, steadiest on fine detail. DLSS: NVIDIA's learned upscaler, on an RTX " +
            "GPU; FSR 2 stands in for it elsewhere."
          }
          options={RENDER_ANTIALIASING_OPTIONS}
          value={features.antialiasing}
          onChange={(antialiasing: RenderAntialiasing) => settingsService.setRendererOverrides({ antialiasing })}
        />

        <ChoiceFormRow
          label={"Texture filtering"}
          description={
            "How surfaces sample their textures. Anisotropic keeps ground and walls seen at grazing angles sharp; " +
            "trilinear blurs them, at a little less cost."
          }
          options={RENDER_TEXTURE_FILTERING_OPTIONS}
          value={features.textureFiltering}
          onChange={(textureFiltering: RenderTextureFiltering) =>
            settingsService.setRendererOverrides({ textureFiltering })
          }
        />

        <ChoiceFormRow
          label={"Texture quality"}
          description={
            "The game's `texture_lod`: how much coarser than stored the textures load. Each step past full drops " +
            "their largest levels as the engine does; changing it loads every texture again."
          }
          options={TEXTURE_LOD_OPTIONS}
          value={String(features.textureLod)}
          onChange={(value: string) => settingsService.setRendererOverrides({ textureLod: Number(value) })}
        />

        <ChoiceFormRow
          label={"DLSS model"}
          description={
            "Which of NVIDIA's models DLSS upscales with. Default picks per render scale: K up to Balanced, M at " +
            "Performance, L at Ultra performance. K is the sharpest and costliest; J ghosts slightly less and flickers " +
            "more."
          }
          options={RENDER_DLSS_PRESET_OPTIONS}
          value={features.upscaling.dlssPreset}
          isDisabled={features.antialiasing !== ERenderAntialiasing.DLSS || features.upscaling.isRayReconstructed}
          onChange={(dlssPreset: RenderDlssPreset) =>
            settingsService.setRendererOverrides({ upscaling: { dlssPreset } })
          }
        />

        <CheckboxFormRow
          label={"DLSS ray reconstruction"}
          description={
            "DLSS denoises and upscales in one network over each surface's albedo, normal and roughness, in its " +
            "model's place. Needs DLSS, the Enhanced tonemap and an RTX GPU whose driver has it; DLSS upscales " +
            "alone elsewhere."
          }
          isChecked={features.upscaling.isRayReconstructed}
          isDisabled={
            features.antialiasing !== ERenderAntialiasing.DLSS || features.tonemap !== ERenderTonemapMode.ENHANCED
          }
          onChange={(isRayReconstructed: boolean) =>
            settingsService.setRendererOverrides({ upscaling: { isRayReconstructed } })
          }
        />

        <SliderFormRow
          label={"Sharpening"}
          description={
            "RCAS sharpening of a frame drawn smaller than the viewport or resolved temporally (TAA, FSR 2, DLSS). " +
            "Inactive otherwise."
          }
          value={features.upscaling.sharpening}
          {...RENDER_SHARPENING_LIMITS}
          isDisabled={
            !isTemporalAntialiasing(features.antialiasing) &&
            features.upscaling.scale === ERenderScale.NATIVE &&
            settingsService.renderResolution === ERenderResolution.AUTO &&
            settingsService.renderOutputScale >= 1
          }
          format={formatSharpening}
          onChange={(sharpening: number) => settingsService.setRendererOverrides({ upscaling: { sharpening } })}
        />
      </div>
    </DetailSection>
  );
}
