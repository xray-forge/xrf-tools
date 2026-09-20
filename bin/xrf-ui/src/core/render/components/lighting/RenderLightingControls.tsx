import { ReactElement, useCallback } from "react";

import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { IRenderLighting, RENDER_LIGHTING_LIMITS } from "@/core/render/lib/lighting/render-lighting";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatDegrees } from "@/lib/format/angle";
import { formatNumber } from "@/lib/format/number";

/** Decimals a strength reads with, which is read against the other strength rather than in any unit. */
const STRENGTH_DIGITS: number = 2;

interface IRenderLightingControlsProps extends BaseComponentProps {
  lighting: IRenderLighting;
  onChange: (lighting: IRenderLighting) => void;
}

/**
 * The four values every preview is lit by, wherever one offers them.
 */
export function RenderLightingControls({
  "data-testid": dataTestId = "render-lighting-controls",
  id,
  className,
  lighting,
  onChange,
}: IRenderLightingControlsProps): ReactElement {
  const set = useCallback((part: Partial<IRenderLighting>) => onChange({ ...lighting, ...part }), [lighting, onChange]);

  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <RenderValueSlider
        label={"Elevation"}
        value={lighting.sunElevation}
        {...RENDER_LIGHTING_LIMITS.sunElevation}
        format={formatDegrees}
        onChange={(sunElevation) => set({ sunElevation })}
      />

      <RenderValueSlider
        label={"Azimuth"}
        value={lighting.sunAzimuth}
        {...RENDER_LIGHTING_LIMITS.sunAzimuth}
        format={formatDegrees}
        onChange={(sunAzimuth) => set({ sunAzimuth })}
      />

      <RenderValueSlider
        label={"Light"}
        value={lighting.sunIntensity}
        {...RENDER_LIGHTING_LIMITS.sunIntensity}
        format={(value: number) => formatNumber(value, STRENGTH_DIGITS)}
        onChange={(sunIntensity) => set({ sunIntensity })}
      />

      <RenderValueSlider
        label={"Ambient"}
        value={lighting.ambientIntensity}
        {...RENDER_LIGHTING_LIMITS.ambientIntensity}
        format={(value: number) => formatNumber(value, STRENGTH_DIGITS)}
        onChange={(ambientIntensity) => set({ ambientIntensity })}
      />
    </div>
  );
}
