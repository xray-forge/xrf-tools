import { useInjection } from "@wirestate/react";
import { ERenderScale, RenderAntialiasing, RenderTextureFiltering } from "@xrf/ipc/types/xrf-renderer";
import { ChoiceFormRow } from "@xrf/ui/form/choice-row";
import { SliderFormRow } from "@xrf/ui/form/slider-row";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import { RenderSettingRow } from "@/core/render/components/settings/RenderSettingRow";
import { RenderSettingRows } from "@/core/render/components/settings/RenderSettingRows";
import {
  explainRenderAntialiasing,
  isTemporalAntialiasing,
  RENDER_ANTIALIASING_OPTIONS,
  RENDER_TEXTURE_FILTERING_OPTIONS,
  RENDER_TEXTURE_LOD_OPTIONS,
} from "@/core/render/lib/features/image/render-antialiasing-choices";
import {
  formatOutputScale,
  RENDER_OUTPUT_SCALE_LIMITS,
  RENDER_RESOLUTION_OPTIONS,
} from "@/core/render/lib/features/image/render-display-choices";
import { RENDER_UPSCALING_SETTINGS } from "@/core/render/lib/features/image/render-upscaling-choices";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { ERenderResolution } from "@/core/render/lib/settings/device/render-resolution";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/**
 * What every viewport resolves its frame at and how much of that the scene is drawn at before it is upscaled to it,
 * side by side, then how the frame's edges are smoothed and its textures sampled.
 */
export function SettingsRendererAntialiasing(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const features: IRenderFeatureSettings = renderSettingsService.rendererFeatures;

  const onSetUpscaling = useRendererOverride("upscaling");

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

        <SliderFormRow
          label={"Output scale"}
          description={
            "Share of the viewport's size the frame is resolved at: below 100% upscales, above supersamples."
          }
          value={renderSettingsService.renderOutputScale}
          {...RENDER_OUTPUT_SCALE_LIMITS}
          isDisabled={renderSettingsService.renderResolution !== ERenderResolution.AUTO}
          format={formatOutputScale}
          onChange={renderSettingsService.setRenderOutputScale}
        />

        <RenderSettingRow
          settings={RENDER_UPSCALING_SETTINGS}
          name={"scale"}
          group={features.upscaling}
          features={features}
          onSet={onSetUpscaling}
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

        <RenderSettingRows
          settings={RENDER_UPSCALING_SETTINGS}
          names={["dlssPreset", "rayReconstructionPreset", "isRayReconstructed"]}
          group={features.upscaling}
          features={features}
          onSet={onSetUpscaling}
        />

        <RenderSettingRow
          settings={RENDER_UPSCALING_SETTINGS}
          name={"sharpening"}
          group={features.upscaling}
          features={features}
          isDisabled={
            !isTemporalAntialiasing(features.antialiasing) &&
            features.upscaling.scale === ERenderScale.NATIVE &&
            renderSettingsService.renderResolution === ERenderResolution.AUTO &&
            renderSettingsService.renderOutputScale >= 1
          }
          onSet={onSetUpscaling}
        />
      </div>
    </DetailSection>
  );
}
