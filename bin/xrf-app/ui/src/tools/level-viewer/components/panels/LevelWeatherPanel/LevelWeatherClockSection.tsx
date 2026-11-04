import { WorldWeatherReport } from "@xrf/ipc/types/xrf-world";
import { Nullable } from "@xrf/types";
import { EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { Typography } from "@xrf/ui/material/typography";
import { ReactElement } from "react";

import { LevelWeatherTime } from "@/core/level/components/weather/LevelWeatherTime";
import { ILevelWeatherControl } from "@/core/level/lib/weather/level-weather-control";

interface ILevelWeatherClockSectionProps extends BaseComponentProps {
  /** Seconds since midnight. */
  time: number;
  /** Every keyframe's time, which the slider marks. */
  keyframes: ReadonlyArray<number>;
  /** Where the renderer's weather stood when it last reported, or null before it has. */
  report: Nullable<WorldWeatherReport>;
  control: ILevelWeatherControl;
  /** Whether Monolith's sun table stands the sun, as it does on its engine for a game that has one. */
  isSunTabled: boolean;
  /** Whether the keyframe set by hand lights the level, standing the sun by its own angles. */
  isManual: boolean;
  onSeek: (time: number) => void;
  onPlaying: (isPlaying: boolean) => void;
  onFactor: (factor: number) => void;
  onDynamicSun: (isDynamicSun: boolean) => void;
}

/**
 * The time of day as the toolbar's Weather popover shows it too (`LevelWeatherTime`), and how the sun stands for it.
 */
export function LevelWeatherClockSection({
  "data-testid": dataTestId = "level-weather-clock-section",
  id,
  className,
  time,
  keyframes,
  report,
  control,
  isSunTabled,
  isManual,
  onSeek,
  onPlaying,
  onFactor,
  onDynamicSun,
}: ILevelWeatherClockSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Time"}>
      <LevelWeatherTime
        time={time}
        keyframes={keyframes}
        report={report}
        control={control}
        onSeek={onSeek}
        onPlaying={onPlaying}
        onFactor={onFactor}
      />

      {isSunTabled ? (
        <Typography className={"block text-text-secondary"} variant={"caption"}>
          {isManual
            ? "The keyframe set by hand stands the sun by its own angles."
            : "The sun stands by the game's sun table, hour by hour."}
        </Typography>
      ) : (
        <CheckboxFormRow
          label={"Dynamic sun"}
          description={
            isManual
              ? "The keyframe set by hand stands the sun by its own angles"
              : control.isDynamicSun
                ? "Computed for the time of day, as OpenXRay does by default"
                : "At the angles the keyframes write"
          }
          isChecked={control.isDynamicSun && !isManual}
          isDisabled={isManual}
          onChange={onDynamicSun}
        />
      )}
    </EditorPanelSection>
  );
}
