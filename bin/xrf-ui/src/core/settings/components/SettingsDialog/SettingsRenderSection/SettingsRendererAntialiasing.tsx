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
  explainRenderAntialiasing,
  isTemporalAntialiasing,
  RENDER_ANTIALIASING_OPTIONS,
  RENDER_TEXTURE_FILTERING_OPTIONS,
  RENDER_TEXTURE_LOD_OPTIONS,
} from "@/core/render/lib/features/render-antialiasing-choices";
import {
  formatOutputScale,
  RENDER_OUTPUT_SCALE_LIMITS,
  RENDER_RESOLUTION_OPTIONS,
} from "@/core/render/lib/features/render-display-choices";
import {
  describeDlssPreset,
  formatSharpening,
  RENDER_DLSS_PRESET_OPTIONS,
  RENDER_SCALE_OPTIONS,
  RENDER_SHARPENING_LIMITS,
} from "@/core/render/lib/features/render-upscaling-choices";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";
import { ERenderResolution } from "@/core/render/lib/settings/render-resolution";
import { RenderSettingsService } from "@/core/render/services/render-settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { ChoiceFormRow } from "@/core/ui/form/ChoiceFormRow";
import { SliderFormRow } from "@/core/ui/form/SliderFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";

/**
 * What every viewport resolves its frame at and how much of that the scene is drawn at before it is upscaled to it,
 * side by side, then how the frame's edges are smoothed and its textures sampled.
 */
export function SettingsRendererAntialiasing(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const features: IRenderFeatureSettings = renderSettingsService.rendererFeatures;

  return (
    <DetailSection
      title={"Resolution and antialiasing"}
      description={"Output resolution, upscaling quality and edge smoothing."}
    >
      <div className={"mt-4 flex flex-col gap-6"}>
        <ChoiceFormRow
          label={"Output resolution"}
          description={
            "The size every viewport's frame is resolved at before it is fitted. Auto follows the viewport, scaled below."
          }
          options={RENDER_RESOLUTION_OPTIONS}
          value={renderSettingsService.renderResolution}
          onChange={renderSettingsService.setRenderResolution}
        />

        {renderSettingsService.renderResolution === ERenderResolution.AUTO ? (
          <SliderFormRow
            label={"Output scale"}
            description={
              "Share of the viewport's size the frame is resolved at: below 100% upscales, above supersamples."
            }
            value={renderSettingsService.renderOutputScale}
            {...RENDER_OUTPUT_SCALE_LIMITS}
            format={formatOutputScale}
            onChange={renderSettingsService.setRenderOutputScale}
          />
        ) : null}

        <ChoiceFormRow
          label={"Upscaling quality"}
          description={
            "Share of the output resolution the scene is drawn at. TAA, FSR and DLSS upscale temporally; other " +
            "modes use FSR 1."
          }
          options={RENDER_SCALE_OPTIONS}
          value={features.upscaling.scale}
          onChange={(scale: RenderScale) => renderSettingsService.setRendererOverrides({ upscaling: { scale } })}
        />

        <ChoiceFormRow
          label={"Antialiasing"}
          description={explainRenderAntialiasing(features.antialiasing)}
          options={RENDER_ANTIALIASING_OPTIONS}
          value={features.antialiasing}
          onChange={(antialiasing: RenderAntialiasing) => renderSettingsService.setRendererOverrides({ antialiasing })}
        />

        <ChoiceFormRow
          label={"Texture filtering"}
          description={
            "How surfaces sample their textures: anisotropic keeps grazing ground and walls sharp; trilinear blurs them."
          }
          options={RENDER_TEXTURE_FILTERING_OPTIONS}
          value={features.textureFiltering}
          onChange={(textureFiltering: RenderTextureFiltering) =>
            renderSettingsService.setRendererOverrides({ textureFiltering })
          }
        />

        <ChoiceFormRow
          label={"Texture quality"}
          description={
            "Drops each texture's largest mip levels, one a step; a change loads every texture again. " +
            "Game setting: texture_lod."
          }
          options={RENDER_TEXTURE_LOD_OPTIONS}
          value={String(features.textureLod)}
          onChange={(value: string) => renderSettingsService.setRendererOverrides({ textureLod: Number(value) })}
        />

        <ChoiceFormRow
          label={"DLSS model"}
          description={describeDlssPreset(features.upscaling.dlssPreset)}
          options={RENDER_DLSS_PRESET_OPTIONS}
          value={features.upscaling.dlssPreset}
          isDisabled={features.antialiasing !== ERenderAntialiasing.DLSS || features.upscaling.isRayReconstructed}
          onChange={(dlssPreset: RenderDlssPreset) =>
            renderSettingsService.setRendererOverrides({ upscaling: { dlssPreset } })
          }
        />

        <CheckboxFormRow
          label={"DLSS ray reconstruction"}
          description={
            "Denoises the traced light as DLSS upscales. Needs the Enhanced tonemap and an RTX GPU with ray reconstruction."
          }
          isChecked={features.upscaling.isRayReconstructed}
          isDisabled={
            features.antialiasing !== ERenderAntialiasing.DLSS || features.tonemap !== ERenderTonemapMode.ENHANCED
          }
          onChange={(isRayReconstructed: boolean) =>
            renderSettingsService.setRendererOverrides({ upscaling: { isRayReconstructed } })
          }
        />

        <SliderFormRow
          label={"Sharpening"}
          description={
            "RCAS sharpening of a frame drawn smaller than the viewport or resolved temporally (TAA, FSR, DLSS). " +
            "Inactive otherwise."
          }
          value={features.upscaling.sharpening}
          {...RENDER_SHARPENING_LIMITS}
          isDisabled={
            !isTemporalAntialiasing(features.antialiasing) &&
            features.upscaling.scale === ERenderScale.NATIVE &&
            renderSettingsService.renderResolution === ERenderResolution.AUTO &&
            renderSettingsService.renderOutputScale >= 1
          }
          format={formatSharpening}
          onChange={(sharpening: number) => renderSettingsService.setRendererOverrides({ upscaling: { sharpening } })}
        />
      </div>
    </DetailSection>
  );
}
