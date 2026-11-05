import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { RenderSettingEntries } from "@/core/render/components/settings/RenderSettingEntries";
import { RENDER_PARALLAX_SETTINGS } from "@/core/render/lib/features/world/render-parallax-choices";
import { TRenderParallaxSettings } from "@/core/render/lib/settings/choice/render-feature-settings";

/**
 * How this view's surfaces are displaced by their height: the engine's steep parallax, or the enhanced march, its
 * terrain's reach, then the level surfaces', then how it shades. Their reset is the section's heading's.
 */
export function LevelParallaxSettings({
  "data-testid": dataTestId = "level-parallax-settings",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const { settings, features, drawn } = useLevelRenderFeatures();
  const { set } = useLevelFeatureOverride("parallax");

  const value: TRenderParallaxSettings = toLevelModeFeature("parallax", settings, features);

  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <RenderSettingEntries
        settings={RENDER_PARALLAX_SETTINGS}
        names={[
          "mode",
          "isSteep",
          "steps",
          "range",
          "height",
          "surfaceRange",
          "surfaceHeight",
          "isRefined",
          "shadow",
          "blending",
          "cavity",
        ]}
        group={value}
        features={drawn}
        onSet={set}
      />
    </div>
  );
}
