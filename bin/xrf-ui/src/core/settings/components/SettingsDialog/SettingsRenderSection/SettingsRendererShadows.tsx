import { useInjection } from "@wirestate/react";
import { IRendererShadowSettings, RENDERER_SHADOW_CASCADE_WIDTHS } from "@xrf/renderer";
import { ReactElement } from "react";

import {
  formatCascadeBlend,
  formatShadowBias,
  formatShadowFilter,
  formatShadowReach,
  RENDER_SHADOW_CASCADE_OPTIONS,
  RENDER_SHADOW_LIMITS,
  RENDER_SHADOW_RESOLUTION_OPTIONS,
} from "@/core/render/lib/features";
import { SettingsService } from "@/core/settings/services/settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { ChoiceFormRow } from "@/core/ui/form/ChoiceFormRow";
import { SliderFormRow } from "@/core/ui/form/SliderFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";

import { useRendererOverride } from "./use-renderer-override";

/** The sun's shadow: how many cascades, how fine, and how they are filtered and drawn. */
export function SettingsRendererShadows(): ReactElement {
  const settingsService: SettingsService = useInjection(SettingsService);

  const shadows: IRendererShadowSettings = settingsService.rendererFeatures.shadows;

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
          description={"Cascade count. Widths: 20, 40, 160 and 480 m."}
          options={RENDER_SHADOW_CASCADE_OPTIONS}
          value={String(shadows.cascades.length)}
          onChange={(count: string) => onSet({ cascades: RENDERER_SHADOW_CASCADE_WIDTHS.slice(0, Number(count)) })}
        />

        <ChoiceFormRow
          label={"Map resolution"}
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

        <SliderFormRow
          label={"Caster reach"}
          description={"Distance toward the sun beyond a cascade that still casts into it, in metres."}
          value={shadows.reach}
          {...RENDER_SHADOW_LIMITS.reach}
          format={formatShadowReach}
          onChange={(reach: number) => onSet({ reach })}
        />

        <CheckboxFormRow
          label={"Staggered updates"}
          description={
            "Updates far cascades every second and fourth frame. Exact for static scenes; keeps camera motion smooth."
          }
          isChecked={shadows.isStaggered}
          onChange={(isStaggered: boolean) => onSet({ isStaggered })}
        />
      </div>
    </DetailSection>
  );
}
