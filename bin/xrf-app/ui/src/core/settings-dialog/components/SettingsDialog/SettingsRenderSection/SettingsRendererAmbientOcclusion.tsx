import { useInjection } from "@wirestate/react";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import { RenderSettingRows } from "@/core/render/components/settings/RenderSettingRows";
import {
  RENDER_AMBIENT_OCCLUSION_GTAO_SETTINGS,
  RENDER_AMBIENT_OCCLUSION_RTAO_SETTINGS,
  RENDER_AMBIENT_OCCLUSION_SETTINGS,
  RENDER_AMBIENT_OCCLUSION_VBAO_SETTINGS,
} from "@/core/render/lib/features/tracing/render-ambient-occlusion-choices";
import {
  IRenderFeatureSettings,
  TRenderAmbientOcclusionGtaoSettings,
  TRenderAmbientOcclusionRtaoSettings,
  TRenderAmbientOcclusionSettings,
  TRenderAmbientOcclusionVbaoSettings,
} from "@/core/render/lib/settings/choice/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** The screen's ambient occlusion: whether it is drawn, how far it reaches, how dark, and each method's search. */
export function SettingsRendererAmbientOcclusion(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const features: IRenderFeatureSettings = renderSettingsService.rendererFeatures;
  const occlusion: TRenderAmbientOcclusionSettings = features.ambientOcclusion;

  const onSet = useRendererOverride("ambientOcclusion");

  return (
    <DetailSection
      title={"Ambient occlusion"}
      description={
        "The game's SSAO, as GTAO or VBAO from the screen or RTAO traced, picked in a level's Ambient light popover."
      }
    >
      <div className={"mt-4 flex flex-col gap-6"}>
        <RenderSettingRows
          settings={RENDER_AMBIENT_OCCLUSION_SETTINGS}
          names={["isEnabled", "radius", "strength"]}
          group={occlusion}
          features={features}
          onSet={onSet}
        />

        <RenderSettingRows
          settings={RENDER_AMBIENT_OCCLUSION_GTAO_SETTINGS}
          names={["slices", "steps"]}
          group={occlusion.gtao}
          features={features}
          onSet={(gtao: Partial<TRenderAmbientOcclusionGtaoSettings>) => onSet({ gtao })}
        />

        <RenderSettingRows
          settings={RENDER_AMBIENT_OCCLUSION_VBAO_SETTINGS}
          names={["slices", "steps"]}
          group={occlusion.vbao}
          features={features}
          onSet={(vbao: Partial<TRenderAmbientOcclusionVbaoSettings>) => onSet({ vbao })}
        />

        <RenderSettingRows
          settings={RENDER_AMBIENT_OCCLUSION_RTAO_SETTINGS}
          names={["rays"]}
          group={occlusion.rtao}
          features={features}
          onSet={(rtao: Partial<TRenderAmbientOcclusionRtaoSettings>) => onSet({ rtao })}
        />
      </div>
    </DetailSection>
  );
}
