import { default as WbSunnyIcon } from "@mui/icons-material/WbSunny";
import { Button } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useMemo } from "react";

import { LevelSunDescription } from "@/core/ipc/types/xrf-app";
import { DEFAULT_LEVEL_LIGHTING, ILevelLighting } from "@/core/level/lib/lighting/level-lighting";
import { toSunAngles } from "@/core/level/lib/lighting/level-sun";
import { LevelLoadService } from "@/core/level/services";
import { RenderLightingControls } from "@/core/render/components/lighting/RenderLightingControls";
import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatDegrees } from "@/lib/format/angle";

interface ILevelSunActionProps extends BaseComponentProps {
  /** Whether the sun is drawn in the sky, which is what the toggle turns over; the light it casts stays. */
  isOn: boolean;
  lighting: ILevelLighting;
  onToggle: () => void;
  onChange: (lighting: ILevelLighting) => void;
}

/**
 * Where the sun stands and how strongly it and the ambient light the level, and whether it is drawn in the sky.
 */
export function LevelSunAction({
  "data-testid": dataTestId = "level-sun-action",
  id,
  className,
  isOn,
  lighting,
  onToggle,
  onChange,
}: ILevelSunActionProps): ReactElement {
  const service: LevelLoadService = useInjection(LevelLoadService);

  const sun: Nullable<LevelSunDescription> = service.level.value?.selected.value.sun ?? null;
  const angles = useMemo(() => toSunAngles(sun?.direction ?? null), [sun]);

  const set = useCallback((part: Partial<IRenderLighting>) => onChange({ ...lighting, ...part }), [lighting, onChange]);

  // The one thing about its lighting a level can answer for: the occlusion it carries was computed for this
  // direction, so lighting a preview from it is lighting it the way the compiler assumed.
  const onUseLevelSun = useCallback(() => {
    if (angles) {
      set({ sunAzimuth: Math.round(angles.azimuth), sunElevation: Math.round(angles.elevation) });
    }
  }, [angles, set]);

  const onReset = useCallback(
    () =>
      set({
        ambientColor: DEFAULT_LEVEL_LIGHTING.ambientColor,
        ambientIntensity: DEFAULT_LEVEL_LIGHTING.ambientIntensity,
        sunAzimuth: DEFAULT_LEVEL_LIGHTING.sunAzimuth,
        sunColor: DEFAULT_LEVEL_LIGHTING.sunColor,
        sunElevation: DEFAULT_LEVEL_LIGHTING.sunElevation,
        sunIntensity: DEFAULT_LEVEL_LIGHTING.sunIntensity,
      }),
    [set]
  );

  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Sun"}
      description={`Sun ${formatDegrees(lighting.sunElevation)} up at ${formatDegrees(lighting.sunAzimuth)}`}
      icon={<WbSunnyIcon />}
      isOn={isOn}
      toggleLabel={"Show the sun in the sky"}
      onToggle={onToggle}
    >
      <RenderLightingControls lighting={lighting} onChange={set} />

      <Button size={"small"} disabled={!angles} onClick={onUseLevelSun}>
        {angles ? "Use the level's own sun" : "This level names no sun"}
      </Button>

      <Button size={"small"} onClick={onReset}>
        Back to the default light
      </Button>
    </EditorPopoverToggle>
  );
}
