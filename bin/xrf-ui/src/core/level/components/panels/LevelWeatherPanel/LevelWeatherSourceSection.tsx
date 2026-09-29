import { ToggleButton, ToggleButtonGroup, Typography } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelWeatherSourceSectionProps extends BaseComponentProps {
  source: ELevelWeatherSource;
  /** Whether a cycle is ready to play. */
  isPlayable: boolean;
  /** Why the last cycle asked for does not play, or null. */
  failure: Nullable<string>;
  isFirst?: boolean;
  onChange: (source: ELevelWeatherSource) => void;
}

/**
 * Whether the level's weather lights it, or the toolbar's sun, fog and wind.
 */
export function LevelWeatherSourceSection({
  "data-testid": dataTestId = "level-weather-source-section",
  id,
  className,
  source,
  isPlayable,
  failure,
  isFirst,
  onChange,
}: ILevelWeatherSourceSectionProps): ReactElement {
  const shown: ELevelWeatherSource = isPlayable ? source : ELevelWeatherSource.MANUAL;

  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Lit by"} isFirst={isFirst}>
      <ToggleButtonGroup
        exclusive
        fullWidth
        color={"primary"}
        size={"small"}
        value={shown}
        aria-label={"Lit by"}
        onChange={(_, next: Nullable<ELevelWeatherSource>) => {
          if (next !== null) {
            onChange(next);
          }
        }}
      >
        <ToggleButton value={ELevelWeatherSource.WEATHER} disabled={!isPlayable}>
          Weather
        </ToggleButton>
        <ToggleButton value={ELevelWeatherSource.MANUAL}>Manual</ToggleButton>
      </ToggleButtonGroup>

      {failure ? (
        <Typography className={"mt-2 block wrap-anywhere text-warning"} variant={"caption"}>
          {failure}
        </Typography>
      ) : null}

      {!failure && shown === ELevelWeatherSource.MANUAL ? (
        <Typography className={"mt-2 block text-text-secondary"} variant={"caption"}>
          The toolbar&apos;s sun, fog and wind light the level.
        </Typography>
      ) : null}
    </EditorPanelSection>
  );
}
