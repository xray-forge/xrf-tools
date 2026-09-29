import { IRendererWeatherEffectReport } from "@xrf/renderer";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { LevelWeatherCycle } from "@/core/ipc/types/xrf-app";
import { LevelWeatherEffectSelect } from "@/core/level/components/weather/LevelWeatherEffectSelect";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelWeatherEffectsSectionProps extends BaseComponentProps {
  effects: ReadonlyArray<LevelWeatherCycle>;
  /** The effect playing, or null for none. */
  playing: Nullable<IRendererWeatherEffectReport>;
  isDisabled?: boolean;
  onPlay: (name: Nullable<string>) => void;
}

/**
 * The game's weather effects, any of which plays over what lights the level from the clock's time until it gives it
 * back.
 */
export function LevelWeatherEffectsSection({
  "data-testid": dataTestId = "level-weather-effects-section",
  id,
  className,
  effects,
  playing,
  isDisabled = false,
  onPlay,
}: ILevelWeatherEffectsSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Effects"}>
      <LevelWeatherEffectSelect effects={effects} playing={playing} isDisabled={isDisabled} onPlay={onPlay} />
    </EditorPanelSection>
  );
}
