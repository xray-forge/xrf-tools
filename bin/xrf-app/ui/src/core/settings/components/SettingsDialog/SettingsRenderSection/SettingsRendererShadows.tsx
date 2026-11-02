import { useInjection } from "@wirestate/react";
import { ChoiceFormRow } from "@xrf/ui/form/choice-row";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import { RenderSettingRows } from "@/core/render/components/settings/RenderSettingRows";
import {
  RENDER_SHADOW_CASCADE_OPTIONS,
  RENDER_SHADOW_RESOLUTION_OPTIONS,
  RENDER_SHADOW_SETTINGS,
} from "@/core/render/lib/features/lighting/render-shadow-choices";
import { RENDER_SHADOW_CASCADE_WIDTHS } from "@/core/render/lib/settings/choice/render-feature-defaults";
import {
  IRenderFeatureSettings,
  TRenderShadowSettings,
} from "@/core/render/lib/settings/choice/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** The sun's shadow: how many cascades, how fine, and how they are filtered and drawn. */
export function SettingsRendererShadows(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const features: IRenderFeatureSettings = renderSettingsService.rendererFeatures;
  const shadows: TRenderShadowSettings = features.shadows;

  const onSet = useRendererOverride("shadows");

  return (
    <DetailSection
      title={"Sun shadows"}
      description={"Cascaded shadow maps. The game uses three cascades at 2048 texels."}
    >
      <div className={"mt-4 flex flex-col gap-6"}>
        <RenderSettingRows
          settings={RENDER_SHADOW_SETTINGS}
          names={["isEnabled"]}
          group={shadows}
          features={features}
          onSet={onSet}
        />

        <ChoiceFormRow
          label={RENDER_SHADOW_SETTINGS.cascades.label}
          description={RENDER_SHADOW_SETTINGS.cascades.description as string}
          options={RENDER_SHADOW_CASCADE_OPTIONS}
          value={String(shadows.cascades.length)}
          onChange={(count: string) => onSet({ cascades: RENDER_SHADOW_CASCADE_WIDTHS.slice(0, Number(count)) })}
        />

        <ChoiceFormRow
          label={RENDER_SHADOW_SETTINGS.resolution.label}
          description={RENDER_SHADOW_SETTINGS.resolution.description as string}
          options={RENDER_SHADOW_RESOLUTION_OPTIONS}
          value={String(shadows.resolution)}
          onChange={(resolution: string) => onSet({ resolution: Number(resolution) })}
        />

        <RenderSettingRows
          settings={RENDER_SHADOW_SETTINGS}
          names={["filter", "bias", "blend", "isStaggered", "swayRate", "swayCascades"]}
          group={shadows}
          features={features}
          onSet={onSet}
        />
      </div>
    </DetailSection>
  );
}
