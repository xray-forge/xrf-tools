import { default as FoggyIcon } from "@mui/icons-material/Foggy";
import { Button } from "@mui/material";
import { ReactElement, useCallback } from "react";

import { DEFAULT_LEVEL_FOG, ILevelFog, LEVEL_FOG_LIMITS } from "@/core/level/lib/lighting/level-fog";
import { ILevelLighting } from "@/core/level/lib/lighting/level-lighting";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatNumber, formatPercent } from "@/lib/format/number";

interface ILevelFogActionProps extends BaseComponentProps {
  isOn: boolean;
  lighting: ILevelLighting;
  onToggle: () => void;
  onChange: (lighting: ILevelLighting) => void;
}

/**
 * Whether the noon fog closes the level in, and where and how thickly it does.
 */
export function LevelFogAction({
  "data-testid": dataTestId = "level-fog-action",
  id,
  className,
  isOn,
  lighting,
  onToggle,
  onChange,
}: ILevelFogActionProps): ReactElement {
  const set = useCallback((part: Partial<ILevelFog>) => onChange({ ...lighting, ...part }), [lighting, onChange]);

  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Fog"}
      description={isOn ? `Fog total at ${lighting.fogDistance} m` : "Fog off"}
      icon={<FoggyIcon />}
      isOn={isOn}
      toggleLabel={"Draw the fog"}
      onToggle={onToggle}
    >
      <RenderValueSlider
        label={"Distance"}
        value={lighting.fogDistance}
        {...LEVEL_FOG_LIMITS.fogDistance}
        format={(value: number) => `${value} m`}
        onChange={(fogDistance: number) => set({ fogDistance })}
      />

      <RenderValueSlider
        label={"Density"}
        value={lighting.fogDensity}
        {...LEVEL_FOG_LIMITS.fogDensity}
        format={formatPercent}
        onChange={(fogDensity: number) => set({ fogDensity })}
      />

      <RenderValueSlider
        label={"Brightness"}
        value={lighting.fogIntensity}
        {...LEVEL_FOG_LIMITS.fogIntensity}
        format={(value: number) => formatNumber(value, 2)}
        onChange={(fogIntensity: number) => set({ fogIntensity })}
      />

      <Button size={"small"} onClick={() => set(DEFAULT_LEVEL_FOG)}>
        Back to the noon fog
      </Button>
    </EditorPopoverToggle>
  );
}
