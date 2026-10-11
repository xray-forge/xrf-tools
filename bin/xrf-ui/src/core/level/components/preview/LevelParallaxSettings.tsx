import { Button } from "@mui/material";
import { formatNumber, formatPercent } from "@xrf/format";
import { ReactElement, useCallback } from "react";

import { ERenderParallaxMode, RenderParallaxMode } from "@/core/ipc/types/xrf-renderer";
import { ILevelFeatureOptions, TLevelParallaxOptions } from "@/core/level/lib/features/level-feature-options";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { IRenderChoiceOption } from "@/core/render/lib/features/render-choice-option";
import { TRenderParallaxSettings } from "@/core/render/lib/settings/render-feature-settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { BaseComponentProps } from "@/lib/dom/element-types";

const MODE_OPTIONS: ReadonlyArray<IRenderChoiceOption<RenderParallaxMode>> = [
  { label: "Engine", value: ERenderParallaxMode.ENGINE },
  { label: "Enhanced", value: ERenderParallaxMode.ENHANCED },
];

interface ILevelParallaxSettingsProps extends BaseComponentProps {
  /** The parallax this view draws with: the settings' and what it sets over them. */
  value: TRenderParallaxSettings;
  /** What the view sets over the settings. */
  features: ILevelFeatureOptions;
  onChange: (features: ILevelFeatureOptions) => void;
}

/** How this view's surfaces are displaced by their height: the engine's steep parallax, or the enhanced terrain march. */
export function LevelParallaxSettings({
  "data-testid": dataTestId = "level-parallax-settings",
  id,
  className,
  value,
  features,
  onChange,
}: ILevelParallaxSettingsProps): ReactElement {
  const set = useCallback(
    (part: Partial<TLevelParallaxOptions>) => onChange({ ...features, parallax: { ...features.parallax, ...part } }),
    [features, onChange]
  );

  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <RenderValueChoice
        label={"Mode"}
        options={MODE_OPTIONS}
        value={value.mode}
        onChange={(mode: RenderParallaxMode) => set({ mode })}
      />

      <CheckboxFormRow
        label={"Steep parallax"}
        description={"The engine's on surfaces whose texture asks for it"}
        isChecked={value.isSteep}
        onChange={(isSteep: boolean) => set({ isSteep })}
      />

      {value.mode === ERenderParallaxMode.ENHANCED ? (
        <>
          <RenderValueSlider
            label={"Steps"}
            value={value.steps}
            min={4}
            max={48}
            step={1}
            format={(steps: number) => formatNumber(steps, 0)}
            onChange={(steps: number) => set({ steps })}
          />

          <RenderValueSlider
            label={"Range"}
            value={value.range}
            min={5}
            max={60}
            step={1}
            format={(range: number) => `${formatNumber(range, 0)} m`}
            onChange={(range: number) => set({ range })}
          />

          <RenderValueSlider
            label={"Height"}
            value={value.height}
            min={0.005}
            max={0.12}
            step={0.005}
            format={(height: number) => formatNumber(height, 3)}
            onChange={(height: number) => set({ height })}
          />

          <RenderValueSlider
            label={"Surface height"}
            value={value.surfaceHeight}
            min={0.005}
            max={0.06}
            step={0.005}
            format={(surfaceHeight: number) => formatNumber(surfaceHeight, 3)}
            onChange={(surfaceHeight: number) => set({ surfaceHeight })}
          />

          <RenderValueSlider
            label={"Surface range"}
            value={value.surfaceRange}
            min={4}
            max={100}
            step={1}
            format={(surfaceRange: number) => `${formatNumber(surfaceRange, 0)} m`}
            onChange={(surfaceRange: number) => set({ surfaceRange })}
          />

          <CheckboxFormRow
            label={"Refined"}
            isChecked={value.isRefined}
            onChange={(isRefined: boolean) => set({ isRefined })}
          />

          <RenderValueSlider
            label={"Self-shadowing"}
            value={value.shadow}
            min={0}
            max={1}
            step={0.05}
            format={formatPercent}
            onChange={(shadow: number) => set({ shadow })}
          />

          <RenderValueSlider
            label={"Height blending"}
            value={value.blending}
            min={0}
            max={1}
            step={0.05}
            format={formatPercent}
            onChange={(blending: number) => set({ blending })}
          />

          <RenderValueSlider
            label={"Cavities"}
            value={value.cavity}
            min={0}
            max={2}
            step={0.05}
            format={formatPercent}
            onChange={(cavity: number) => set({ cavity })}
          />
        </>
      ) : null}

      <Button size={"small"} onClick={() => onChange({ ...features, parallax: {} })}>
        Back to the settings
      </Button>
    </div>
  );
}
