import { Button } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useMemo } from "react";

import { LevelSunDescription } from "@/core/ipc/types/xrf-app";
import { LevelLightingSlider } from "@/core/level/components/panels/LevelLightingPanel/LevelLightingSlider";
import { DEFAULT_LEVEL_LIGHTING, ILevelLighting } from "@/core/level/lib/lighting/level-lighting";
import { toSunAngles } from "@/core/level/lib/lighting/level-sun";
import { LevelLoadService } from "@/core/level/services";
import { EditorPanel, EditorPanelEmpty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

interface ILevelLightingPanelProps extends BaseComponentProps {
  lighting: ILevelLighting;
  onChange: (lighting: ILevelLighting) => void;
}

/**
 * What the viewer is lighting the level with, which is the viewer's answer and not the level's.
 */
export function LevelLightingPanel({
  "data-testid": dataTestId = "level-lighting-panel",
  id,
  className,
  lighting,
  onChange,
}: ILevelLightingPanelProps): ReactElement {
  const service: LevelLoadService = useInjection(LevelLoadService);

  const sun: Nullable<LevelSunDescription> = service.level.value?.selected.value.sun ?? null;
  const angles = useMemo(() => toSunAngles(sun?.direction ?? null), [sun]);

  const set = useCallback((part: Partial<ILevelLighting>) => onChange({ ...lighting, ...part }), [lighting, onChange]);

  // The one thing about its lighting a level can answer for: the occlusion it carries was computed for this
  // direction, so lighting a preview from it is lighting it the way the compiler assumed.
  const onUseLevelSun = useCallback(() => {
    if (angles) {
      set({ sunAzimuth: Math.round(angles.azimuth), sunElevation: Math.round(angles.elevation) });
    }
  }, [angles, set]);

  if (!service.level.value) {
    return (
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Lighting"}>
        <EditorPanelEmpty label={"No level open. Open one to light it."} />
      </EditorPanel>
    );
  }

  return (
    <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Lighting"}>
      <EditorPanelSection title={"Sun"} isFirst>
        <LevelLightingSlider
          label={"Elevation"}
          value={lighting.sunElevation}
          min={-15}
          max={90}
          step={1}
          format={(value) => `${value}°`}
          onChange={(sunElevation) => set({ sunElevation })}
        />
        <LevelLightingSlider
          label={"Azimuth"}
          value={lighting.sunAzimuth}
          min={-180}
          max={180}
          step={1}
          format={(value) => `${value}°`}
          onChange={(sunAzimuth) => set({ sunAzimuth })}
        />
        <LevelLightingSlider
          label={"Intensity"}
          value={lighting.sunIntensity}
          min={0}
          max={5}
          step={0.05}
          format={(value) => value.toFixed(2)}
          onChange={(sunIntensity) => set({ sunIntensity })}
        />

        <Button size={"small"} disabled={!angles} onClick={onUseLevelSun}>
          {angles ? "Use the level's own sun" : "This level names no sun"}
        </Button>
      </EditorPanelSection>

      <EditorPanelSection title={"Ambient"}>
        <LevelLightingSlider
          label={"Intensity"}
          value={lighting.ambientIntensity}
          min={0}
          max={4}
          step={0.05}
          format={(value) => value.toFixed(2)}
          onChange={(ambientIntensity) => set({ ambientIntensity })}
        />
        <LevelLightingSlider
          label={"Baked occlusion"}
          value={lighting.hemiStrength}
          min={0}
          max={1}
          step={0.05}
          format={(value) => `${Math.round(value * 100)}%`}
          onChange={(hemiStrength) => set({ hemiStrength })}
        />
      </EditorPanelSection>

      <EditorPanelSection title={"Reset"}>
        <Button size={"small"} onClick={() => onChange(DEFAULT_LEVEL_LIGHTING)}>
          Back to the default light
        </Button>
      </EditorPanelSection>
    </EditorPanel>
  );
}
