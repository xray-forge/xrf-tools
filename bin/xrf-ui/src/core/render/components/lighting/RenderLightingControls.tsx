import { ReactElement, useCallback } from "react";

import { RenderLightingSlider } from "@/core/render/components/lighting/RenderLightingSlider";
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
      <RenderLightingSlider
        label={"Elevation"}
        value={lighting.sunElevation}
        {...RENDER_LIGHTING_LIMITS.sunElevation}
        format={formatDegrees}
        onChange={(sunElevation) => set({ sunElevation })}
      />

      <RenderLightingSlider
        label={"Azimuth"}
        value={lighting.sunAzimuth}
        {...RENDER_LIGHTING_LIMITS.sunAzimuth}
        format={formatDegrees}
        onChange={(sunAzimuth) => set({ sunAzimuth })}
      />

      <RenderLightingSlider
        label={"Light"}
        value={lighting.sunIntensity}
        {...RENDER_LIGHTING_LIMITS.sunIntensity}
        format={(value: number) => formatNumber(value, STRENGTH_DIGITS)}
        onChange={(sunIntensity) => set({ sunIntensity })}
      />

      <RenderLightingSlider
        label={"Ambient"}
        value={lighting.ambientIntensity}
        {...RENDER_LIGHTING_LIMITS.ambientIntensity}
        format={(value: number) => formatNumber(value, STRENGTH_DIGITS)}
        onChange={(ambientIntensity) => set({ ambientIntensity })}
      />
    </div>
  );
}
