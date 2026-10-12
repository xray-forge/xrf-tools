import { Button } from "@mui/material";
import { ReactElement } from "react";

import { ERenderDebandingMode, RenderDebandingQuality } from "@/core/ipc/types/xrf-renderer";
import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  explainRenderDebandingMode,
  formatDebandingRadius,
  RENDER_DEBANDING_LIMITS,
  RENDER_DEBANDING_QUALITY_OPTIONS,
} from "@/core/render/lib/features/render-debanding-choices";
import { TRenderDebandingSettings } from "@/core/render/lib/settings/render-feature-settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * Whether the sky's colour bands are smoothed over the image, at what quality and over how many pixels.
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
          <RenderValueChoice
            label={"Quality"}
            options={RENDER_DEBANDING_QUALITY_OPTIONS}
            value={value.quality}
            onChange={(quality: RenderDebandingQuality) => set({ quality })}
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
