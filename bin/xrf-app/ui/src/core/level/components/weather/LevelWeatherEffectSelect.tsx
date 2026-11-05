import { LevelWeatherCycle } from "@xrf/ipc/types/xrf-app-levels";
import { WorldWeatherEffectReport } from "@xrf/ipc/types/xrf-world";
import { Nullable } from "@xrf/types";
import { EditorPopoverNote, EditorPopoverSelectRow } from "@xrf/ui/editor/popover-row";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ListItemText } from "@xrf/ui/material/list";
import { MenuItem } from "@xrf/ui/material/menu";
import { ReactElement } from "react";

import {
  formatLevelWeatherFindings,
  formatLevelWeatherKeyframes,
} from "@/core/level/lib/weather/level-weather-cycle-choice";
import { formatLevelWeatherTime } from "@/core/level/lib/weather/level-weather-time";

/** What the select holds while no effect plays. */
const NONE: string = "";

interface ILevelWeatherEffectSelectProps extends BaseComponentProps {
  effects: ReadonlyArray<LevelWeatherCycle>;
  /** The effect playing, or null for none. */
  playing: Nullable<WorldWeatherEffectReport>;
  isDisabled?: boolean;
  onPlay: (name: Nullable<string>) => void;
}

/**
 * The game's weather effects, one of which plays over what lights the level until it gives it back; None stops it.
 * What is left of the one playing is said under it.
 */
export function LevelWeatherEffectSelect({
  "data-testid": dataTestId = "level-weather-effect-select",
  id,
  className,
  effects,
  playing,
  isDisabled = false,
  onPlay,
}: ILevelWeatherEffectSelectProps): ReactElement {
  return (
    <>
      <EditorPopoverSelectRow
        data-testid={dataTestId}
        id={id}
        className={className}
        label={"Effect"}
        value={playing?.name ?? NONE}
        isDisabled={isDisabled || !effects.length}
        renderValue={(value: string) => value || "None"}
        onChange={(value: string) => onPlay(value || null)}
      >
        <MenuItem dense value={NONE}>
          None
        </MenuItem>
        {effects.map((effect: LevelWeatherCycle) => (
          <MenuItem key={effect.name} dense value={effect.name}>
            <ListItemText
              primary={effect.name}
              secondary={[
                formatLevelWeatherKeyframes(effect.keyframes.length),
                formatLevelWeatherFindings(effect.findings.length),
              ]
                .filter(Boolean)
                .join(" · ")}
            />
          </MenuItem>
        ))}
      </EditorPopoverSelectRow>

      {playing ? (
        <EditorPopoverNote>{`${formatLevelWeatherTime(playing.remaining ?? 0, true)} of game time left`}</EditorPopoverNote>
      ) : null}
    </>
  );
}
