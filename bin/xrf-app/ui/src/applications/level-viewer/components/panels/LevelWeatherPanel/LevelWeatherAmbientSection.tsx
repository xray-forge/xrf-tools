import { Button, Typography } from "@mui/material";
import { WorldAmbientReport } from "@xrf/ipc/types/xrf-world";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelWeatherAmbientSectionProps extends BaseComponentProps {
  /** Where the ambient effects near the camera stand, null while no weather plays or the particles are unread. */
  ambient: Nullable<WorldAmbientReport>;
  /** Whether the view plays them at all. */
  isPlayed: boolean;
  onPlay: () => void;
}

/**
 * @param milliseconds - Real milliseconds.
 * @returns Them rounded up to whole seconds.
 */
function formatWholeSecondsOf(milliseconds: number): string {
  return `${Math.ceil(milliseconds / 1000)} s`;
}

/**
 * @param wait - Real milliseconds until the next may start.
 * @param isPlaying - Whether one plays now, which the next waits to be gone.
 * @returns When the next starts, as the engine waits for both.
 */
function toNext(wait: number, isPlaying: boolean): string {
  if (wait > 0) {
    return `In ${formatWholeSecondsOf(wait)}`;
  }

  return isPlaying ? "Once this one is gone" : "Now";
}

/**
 * The weather's ambient effects: what plays near the camera and how long it has left, when the next may start, and a
 * way to play one at once.
 */
export function LevelWeatherAmbientSection({
  "data-testid": dataTestId = "level-weather-ambient-section",
  id,
  className,
  ambient,
  isPlayed,
  onPlay,
}: ILevelWeatherAmbientSectionProps): ReactElement {
  if (!isPlayed || !ambient) {
    return (
      <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Ambient effects"}>
        <Typography className={"block text-text-secondary"} variant={"caption"}>
          {isPlayed
            ? "None until the weather plays and the level's particles are read."
            : "Switched off in the Particles menu."}
        </Typography>
      </EditorPanelSection>
    );
  }

  const { effect, isIndoors, wait } = ambient;
  const remaining: number = effect?.remaining ?? 0;

  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Ambient effects"}>
      <EditorPanelProperty label={"Playing"} value={effect ? `${effect.name}, ${effect.particles}` : "Nothing"} />
      {effect ? (
        <EditorPanelProperty label={"Left"} value={remaining > 0 ? formatWholeSecondsOf(remaining) : "Dying out"} />
      ) : null}
      <EditorPanelProperty label={"Next"} value={toNext(wait, effect !== null)} />
      <EditorPanelProperty label={"Camera"} value={isIndoors ? "Indoors, none start" : "Outdoors"} />

      <Button className={"mt-2"} size={"small"} disabled={isIndoors} onClick={onPlay}>
        Play one now
      </Button>
    </EditorPanelSection>
  );
}
