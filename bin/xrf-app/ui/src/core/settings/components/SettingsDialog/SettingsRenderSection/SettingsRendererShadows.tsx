import { useInjection } from "@wirestate/react";
import { CheckboxFormRow } from "@xrf/ui/form/CheckboxFormRow";
import { ChoiceFormRow } from "@xrf/ui/form/ChoiceFormRow";
import { SliderFormRow } from "@xrf/ui/form/SliderFormRow";
import { DetailSection } from "@xrf/ui/layout/DetailSection";
import { ReactElement } from "react";

import {
  formatCascadeBlend,
  formatCascadeWidths,
  formatShadowBias,
  formatShadowFilter,
  formatShadowSwayCascades,
  formatShadowSwayRate,
  RENDER_SHADOW_CASCADE_OPTIONS,
  RENDER_SHADOW_LIMITS,
  RENDER_SHADOW_RESOLUTION_OPTIONS,
} from "@/core/render/lib/features/render-shadow-choices";
import { RENDER_SHADOW_CASCADE_WIDTHS } from "@/core/render/lib/settings/render-feature-defaults";
import { TRenderShadowSettings } from "@/core/render/lib/settings/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** The sun's shadow: how many cascades, how fine, and how they are filtered and drawn. */
export function SettingsRendererShadows(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const shadows: TRenderShadowSettings = renderSettingsService.rendererFeatures.shadows;

  const onSet = useRendererOverride("shadows");

  return (
    <DetailSection
      title={"Sun shadows"}
      description={"Cascaded shadow maps. The game uses three cascades at 2048 texels."}
    >
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={"Sun shadows"}
          description={"Off, surfaces facing the sun are shaded by baked lighting alone."}
          isChecked={shadows.isEnabled}
          onChange={(isEnabled: boolean) => onSet({ isEnabled })}
        />

        <ChoiceFormRow
          label={"Cascades"}
          description={`Cascade count. Widths: ${formatCascadeWidths(RENDER_SHADOW_CASCADE_WIDTHS)}.`}
          options={RENDER_SHADOW_CASCADE_OPTIONS}
          value={String(shadows.cascades.length)}
          onChange={(count: string) => onSet({ cascades: RENDER_SHADOW_CASCADE_WIDTHS.slice(0, Number(count)) })}
        />

        <ChoiceFormRow
          label={"Map resolution (texels)"}
          description={"Texels per cascade map. Higher is sharper and uses more memory."}
          options={RENDER_SHADOW_RESOLUTION_OPTIONS}
          value={String(shadows.resolution)}
          onChange={(resolution: string) => onSet({ resolution: Number(resolution) })}
        />

        <SliderFormRow
          label={"Filter"}
          description={"PCF kernel radius in texels. 0 gives hard edges."}
          value={shadows.filter}
          {...RENDER_SHADOW_LIMITS.filter}
          format={formatShadowFilter}
          onChange={(filter: number) => onSet({ filter })}
        />

        <SliderFormRow
          label={"Normal offset"}
          description={"Offset along the surface normal, in texels. Prevents self-shadowing."}
          value={shadows.bias}
          {...RENDER_SHADOW_LIMITS.bias}
          format={formatShadowBias}
          onChange={(bias: number) => onSet({ bias })}
        />

        <SliderFormRow
          label={"Cascade blend"}
          description={
            "Transition band between cascades, as a share of cascade width. 0 switches abruptly, as the game does."
          }
          value={shadows.blend}
          {...RENDER_SHADOW_LIMITS.blend}
          format={formatCascadeBlend}
          onChange={(blend: number) => onSet({ blend })}
        />

        <CheckboxFormRow
          label={"Staggered updates"}
          description={
            "Updates far cascades every second and fourth frame. Exact for static scenes; keeps camera motion smooth."
          }
          isChecked={shadows.isStaggered}
          onChange={(isStaggered: boolean) => onSet({ isStaggered })}
        />

        <SliderFormRow
          label={"Sway updates"}
          description={"How often a second the cascades redraw swaying trees. Fewer is cheaper; their shadows step."}
          value={shadows.swayRate}
          {...RENDER_SHADOW_LIMITS.swayRate}
          format={formatShadowSwayRate}
          onChange={(swayRate: number) => onSet({ swayRate })}
        />

        <SliderFormRow
          label={"Swaying cascades"}
          description={"How many of the nearest cascades show trees swaying. Fewer is cheaper; far shadows hold still."}
          value={shadows.swayCascades}
          {...RENDER_SHADOW_LIMITS.swayCascades}
          format={formatShadowSwayCascades}
          onChange={(swayCascades: number) => onSet({ swayCascades })}
        />
      </div>
    </DetailSection>
  );
}
