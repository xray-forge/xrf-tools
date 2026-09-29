import { Typography } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { LevelWeatherSeedNote } from "@/core/level/components/weather/LevelWeatherSeedNote";
import { LevelWeatherSourceChoice } from "@/core/level/components/weather/LevelWeatherSourceChoice";
import { ILevelWeatherSeed } from "@/core/level/lib/weather/level-weather-seed";
import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelWeatherSourceSectionProps extends BaseComponentProps {
  source: ELevelWeatherSource;
  /** Whether a cycle is ready to play. */
  isPlayable: boolean;
  /** Whether the keyframe set by hand lights the level. */
  isManual: boolean;
  /** What the keyframe set by hand was seeded from, or null for none the level played. */
  seed: Nullable<ILevelWeatherSeed>;
  /** Why the last cycle asked for does not play, or null. */
  failure: Nullable<string>;
  isFirst?: boolean;
  onChange: (source: ELevelWeatherSource) => void;
}

/**
 * Whether the level's weather lights it, or the keyframe set by hand, which the toolbar's popovers edit.
 */
export function LevelWeatherSourceSection({
  "data-testid": dataTestId = "level-weather-source-section",
  id,
  className,
  source,
  isPlayable,
  isManual,
  seed,
  failure,
  isFirst,
  onChange,
}: ILevelWeatherSourceSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Lit by"} isFirst={isFirst}>
      <LevelWeatherSourceChoice source={source} isPlayable={isPlayable} onChange={onChange} />

      {failure ? (
        <Typography className={"mt-2 block wrap-anywhere text-warning"} variant={"caption"}>
          {failure}
        </Typography>
      ) : null}

      {isManual ? (
        <LevelWeatherSeedNote
          className={"mt-2"}
          seed={seed}
          isPlayable={isPlayable}
          onBack={() => onChange(ELevelWeatherSource.WEATHER)}
        />
      ) : null}
    </EditorPanelSection>
  );
}
