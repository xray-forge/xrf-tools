import { Button, List, ListItemButton, ListItemText, Typography } from "@mui/material";
import { IRendererWeatherEffectReport } from "@xrf/renderer";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { LevelWeatherCycle } from "@/core/ipc/types/xrf-app";
import { formatLevelWeatherTime } from "@/core/level/lib/weather/level-weather-time";
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
 * The game's weather effects, any of which plays over the cycle from the clock's time until it gives the cycle back.
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
      {playing ? (
        <div className={"mb-2 flex items-center justify-between gap-2"}>
          <Typography className={"min-w-0 wrap-anywhere"} variant={"body2"}>
            {`${playing.name}, ${formatLevelWeatherTime(playing.remaining, true)} of game time left`}
          </Typography>
          <Button size={"small"} disabled={isDisabled} onClick={() => onPlay(null)}>
            Stop
          </Button>
        </div>
      ) : null}

      {effects.length ? (
        <List dense disablePadding className={"-mx-panel-content"}>
          {effects.map((effect: LevelWeatherCycle) => (
            <ListItemButton
              key={effect.name}
              selected={effect.name === playing?.name}
              disabled={isDisabled}
              onClick={() => onPlay(effect.name)}
            >
              <ListItemText
                primary={effect.name}
                secondary={[
                  `${effect.keyframes.length} keyframes`,
                  effect.findings.length ? `${effect.findings.length} findings` : null,
                ]
                  .filter(Boolean)
                  .join(" · ")}
              />
            </ListItemButton>
          ))}
        </List>
      ) : (
        <Typography className={"block text-text-secondary"} variant={"caption"}>
          The game has no weather effects.
        </Typography>
      )}
    </EditorPanelSection>
  );
}
