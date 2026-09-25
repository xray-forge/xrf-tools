import { default as AirIcon } from "@mui/icons-material/Air";
import { Button } from "@mui/material";
import { ReactElement, useCallback } from "react";

import { ILevelLighting } from "@/core/level/lib/lighting/level-lighting";
import { DEFAULT_LEVEL_WIND, ILevelWind, LEVEL_WIND_LIMITS } from "@/core/level/lib/lighting/level-wind";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatNumber } from "@/lib/format/number";

interface ILevelWindActionProps extends BaseComponentProps {
  isOn: boolean;
  lighting: ILevelLighting;
  onToggle: () => void;
  onChange: (lighting: ILevelLighting) => void;
}

/**
 * Whether the trees sway in the wind, as the game sways them, and how far and how fast.
 */
export function LevelWindAction({
  "data-testid": dataTestId = "level-wind-action",
  id,
  className,
  isOn,
  lighting,
  onToggle,
  onChange,
}: ILevelWindActionProps): ReactElement {
  const set = useCallback((part: Partial<ILevelWind>) => onChange({ ...lighting, ...part }), [lighting, onChange]);

  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Wind"}
      description={
        isOn ? `Trees sway, ${formatNumber(lighting.windAmplitude, 3)} of their height` : "Wind off, trees stand still"
      }
      icon={<AirIcon />}
      isOn={isOn}
      toggleLabel={"Sway the trees"}
      onToggle={onToggle}
    >
      <RenderValueSlider
        label={"Lean"}
        value={lighting.windAmplitude}
        {...LEVEL_WIND_LIMITS.windAmplitude}
        format={(value: number) => formatNumber(value, 3)}
        onChange={(windAmplitude: number) => set({ windAmplitude })}
      />

      <RenderValueSlider
        label={"Pace"}
        value={lighting.windSpeed}
        {...LEVEL_WIND_LIMITS.windSpeed}
        format={(value: number) => `${formatNumber(value, 1)}×`}
        onChange={(windSpeed: number) => set({ windSpeed })}
      />

      <Button size={"small"} onClick={() => set(DEFAULT_LEVEL_WIND)}>
        Back to the game&apos;s wind
      </Button>
    </EditorPopoverToggle>
  );
}
