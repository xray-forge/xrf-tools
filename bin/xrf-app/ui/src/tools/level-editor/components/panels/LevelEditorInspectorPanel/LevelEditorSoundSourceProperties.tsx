import { ABSENT_VALUE, formatHex, formatNumber } from "@xrf/format";
import { ESceneObjectProperties } from "@xrf/ipc/types/xrf-app-levels";
import { Vector2d } from "@xrf/ipc/types/xrf-math";
import { Nullable } from "@xrf/types";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { ReactElement } from "react";

import { formatSceneMetres, formatScenePoint } from "@/tools/level-editor/lib/inspector/scene-object-format";
import { TSceneObjectPropertiesOf } from "@/tools/level-editor/lib/inspector/scene-object-kinds";

const SECONDS_PER_DAY: number = 24 * 60 * 60;

/** A span of seconds, from and to, a decimal each. */
function formatSecondsWindow(window: Vector2d): string {
  return `${formatNumber(window.x, 1)} to ${formatNumber(window.y, 1)} s`;
}

/** A time of the game day, from its seconds past midnight, as a clock writes it. */
function formatDayTime(seconds: Nullable<number>): string {
  if (seconds === null || !Number.isFinite(seconds)) {
    return ABSENT_VALUE;
  }

  const minutes: number = Math.floor((((seconds % SECONDS_PER_DAY) + SECONDS_PER_DAY) % SECONDS_PER_DAY) / 60);

  return `${String(Math.floor(minutes / 60)).padStart(2, "0")}:${String(minutes % 60).padStart(2, "0")}`;
}

interface ILevelEditorSoundSourcePropertiesProps {
  properties: TSceneObjectPropertiesOf<ESceneObjectProperties.SOUND_SOURCE>;
}

/**
 * The sound a source plays, where it stands, how loud and how far it is heard, and when and how long it plays.
 */
export function LevelEditorSoundSourceProperties({ properties }: ILevelEditorSoundSourcePropertiesProps): ReactElement {
  return (
    <EditorPanelSection title={"Sound source"}>
      <EditorPanelProperty label={"Sound"} value={properties.sound} isMonospace />
      <EditorPanelProperty label={"Sound position"} value={formatScenePoint(properties.position)} />
      <EditorPanelProperty label={"Volume"} value={formatNumber(properties.volume, 2)} />
      <EditorPanelProperty label={"Frequency"} value={formatNumber(properties.frequency, 2)} />
      <EditorPanelProperty
        label={"Heard from"}
        value={`${formatSceneMetres(properties.minDistance)} to ${formatSceneMetres(properties.maxDistance)}`}
      />
      <EditorPanelProperty label={"AI hear it within"} value={formatSceneMetres(properties.maxAiDistance)} />
      <EditorPanelProperty
        label={"Plays"}
        value={`${formatDayTime(properties.activeTime.x)} to ${formatDayTime(properties.activeTime.y)}`}
      />
      <EditorPanelProperty label={"Each play"} value={formatSecondsWindow(properties.playTime)} />
      <EditorPanelProperty label={"Pause"} value={formatSecondsWindow(properties.randomPause)} />
      <EditorPanelProperty label={"Sound flags"} value={formatHex(properties.flags)} isMonospace />
    </EditorPanelSection>
  );
}
