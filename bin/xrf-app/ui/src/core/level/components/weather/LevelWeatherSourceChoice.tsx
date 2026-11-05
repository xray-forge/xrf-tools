import { BaseComponentProps } from "@xrf/ui/element-types";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";
import { ReactElement, useMemo } from "react";

import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";

interface ILevelWeatherSourceChoiceProps extends BaseComponentProps {
  source: ELevelWeatherSource;
  /** Whether a cycle is ready to play. */
  isPlayable: boolean;
  onChange: (source: ELevelWeatherSource) => void;
}

/**
 * Whether the level's weather lights it, or the keyframe set by hand, in one row; the keyframe while no cycle plays.
 */
export function LevelWeatherSourceChoice({
  "data-testid": dataTestId = "level-weather-source-choice",
  id,
  className,
  source,
  isPlayable,
  onChange,
}: ILevelWeatherSourceChoiceProps): ReactElement {
  const options: ReadonlyArray<IChoiceFormRowOption<ELevelWeatherSource>> = useMemo(
    () => [
      { isDisabled: !isPlayable, label: "Weather", value: ELevelWeatherSource.WEATHER },
      { label: "Manual", value: ELevelWeatherSource.MANUAL },
    ],
    [isPlayable]
  );

  return (
    <RenderValueChoice
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Source"}
      description={"The level's weather cycle, or one keyframe set by hand in the weather popovers."}
      options={options}
      value={isPlayable ? source : ELevelWeatherSource.MANUAL}
      onChange={onChange}
    />
  );
}
