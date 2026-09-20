import { Button } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useMemo } from "react";

import { LevelSunDescription } from "@/core/ipc/types/xrf-app";
import { DEFAULT_LEVEL_LIGHTING, ILevelLighting } from "@/core/level/lib/lighting/level-lighting";
import { toSunAngles } from "@/core/level/lib/lighting/level-sun";
import { LevelLoadService } from "@/core/level/services";
import { RenderValueSlider } from "@/core/render/components/controls";
import { RenderLightingControls } from "@/core/render/components/lighting";
import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { EditorPanel, EditorPanelEmpty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatPercent } from "@/lib/format/number";
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
      <EditorPanelSection title={"Light"} isFirst>
        <RenderLightingControls lighting={lighting} onChange={(next: IRenderLighting) => set(next)} />

        <Button size={"small"} disabled={!angles} onClick={onUseLevelSun}>
          {angles ? "Use the level's own sun" : "This level names no sun"}
        </Button>
      </EditorPanelSection>

      <EditorPanelSection title={"Baked"}>
        <RenderValueSlider
          label={"Occlusion"}
          value={lighting.hemiStrength}
          min={0}
          max={1}
          step={0.05}
          format={formatPercent}
          onChange={(hemiStrength: number) => set({ hemiStrength })}
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
