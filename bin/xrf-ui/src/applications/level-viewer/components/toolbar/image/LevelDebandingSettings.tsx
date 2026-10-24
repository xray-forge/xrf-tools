import { Button } from "@mui/material";
import { ERenderDebandingMode } from "@xrf/ipc/types/xrf-renderer";
import { ReactElement } from "react";

import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  explainRenderDebandingMode,
  formatDebandingPasses,
  formatDebandingRadius,
  RENDER_DEBANDING_LIMITS,
} from "@/core/render/lib/features/render-debanding-choices";
import { TRenderDebandingSettings } from "@/core/render/lib/settings/render-feature-settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * Whether the sky's colour bands are smoothed over the image, how many times and over how many pixels.
 */
export function LevelDebandingSettings({
  "data-testid": dataTestId = "level-debanding-settings",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const { settings, features } = useLevelRenderFeatures();
  const { set, reset } = useLevelFeatureOverride("debanding");

  const value: TRenderDebandingSettings = toLevelModeFeature("debanding", settings, features);
  const isDebanded: boolean = value.mode === ERenderDebandingMode.ENHANCED;

  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <CheckboxFormRow
        label={"Debanding"}
        description={explainRenderDebandingMode(value.mode)}
        isChecked={isDebanded}
        onChange={(isOn: boolean) => set({ mode: isOn ? ERenderDebandingMode.ENHANCED : ERenderDebandingMode.ENGINE })}
      />

      {isDebanded ? (
        <>
          <RenderValueSlider
            label={"Debanding passes"}
            value={value.passes}
            {...RENDER_DEBANDING_LIMITS.passes}
            format={formatDebandingPasses}
            onChange={(passes: number) => set({ passes })}
          />

          <RenderValueSlider
            label={"Debanding radius"}
            value={value.radius}
            {...RENDER_DEBANDING_LIMITS.radius}
            format={formatDebandingRadius}
            onChange={(radius: number) => set({ radius })}
          />
        </>
      ) : null}

      <Button size={"small"} aria-label={"Reset the debanding to the preset"} onClick={reset}>
        Reset to the preset
      </Button>
    </div>
  );
}
