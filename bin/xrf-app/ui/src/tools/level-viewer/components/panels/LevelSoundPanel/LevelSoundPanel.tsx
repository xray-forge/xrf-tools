import { useInjection } from "@wirestate/react";
import { WorldMusicCommand, WorldSoundReport, WorldSoundVoiceReport } from "@xrf/ipc/types/xrf-world";
import { Nullable } from "@xrf/types";
import {
  EditorPanelEmpty,
  EditorPanelProperty,
  EditorPanelSection,
  EditorPanelSectionEmpty,
} from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement, useCallback } from "react";

import { LevelLoadService } from "@/core/level/services/level-load";
import { LevelRenderService } from "@/core/level/services/level-render";
import { EditorPanel } from "@/core/shell/chrome/EditorPanel";
import { usePolledValue } from "@/lib/react/use-polled-value";
import { LevelSoundMusicSection } from "@/tools/level-viewer/components/panels/LevelSoundPanel/LevelSoundMusicSection";
import {
  formatLevelSoundAcoustics,
  formatLevelSoundAcousticsState,
  formatLevelSoundGain,
  formatLevelSoundSeconds,
  formatLevelSoundTimeOfDay,
  sortLevelSoundVoices,
} from "@/tools/level-viewer/lib/sound/level-sound-format";

/** How often the panel asks again what plays while it is open. */
const LEVEL_SOUND_REFRESH_MS: number = 500;

/**
 * The level's sound as it plays: whether it is heard, each sound with where it stands and how loud, the music, the
 * weather ambient's channels, and every sound that could not play and why.
 */
export function LevelSoundPanel({
  "data-testid": dataTestId = "level-sound-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const loadService: LevelLoadService = useInjection(LevelLoadService);
  const renderService: LevelRenderService = useInjection(LevelRenderService);

  // One question at a time: the world answers from the audio's thread, so a slow answer must not pile up behind it.
  const report: Nullable<WorldSoundReport> = usePolledValue(
    () => renderService.describeSound(),
    LEVEL_SOUND_REFRESH_MS
  );

  const onMusic = useCallback(
    (command: WorldMusicCommand): Promise<boolean> => renderService.commandMusic(command),
    [renderService]
  );

  if (!loadService.level.value || !report) {
    return (
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Sound"}>
        <EditorPanelEmpty label={loadService.level.value ? "Reading the level's sound." : "No level open."} />
      </EditorPanel>
    );
  }

  const voices: Array<WorldSoundVoiceReport> = sortLevelSoundVoices(report);

  return (
    <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Sound"}>
      <EditorPanelSection title={"Heard"}>
        <EditorPanelProperty
          label={"Listening"}
          value={report.isAudible ? "This viewport" : "Another viewport, this one plays on silent"}
        />
        <EditorPanelProperty label={"Game time"} value={formatLevelSoundTimeOfDay(report.gameTime)} />
        <EditorPanelProperty label={"Voices"} value={`${report.realVoices} of ${report.targets} targets`} />
        <EditorPanelProperty label={"Acoustics"} value={formatLevelSoundAcousticsState(report.acoustics)} />
        {report.mixerLoad !== null ? (
          <EditorPanelProperty label={"Mixer"} value={`${(report.mixerLoad * 100).toFixed(1)}% of the period`} />
        ) : null}
      </EditorPanelSection>

      <LevelSoundMusicSection music={report.music} onCommand={onMusic} />

      <EditorPanelSection title={report.ambient ? `Channels of ${report.ambient}` : "Channels"}>
        {report.channels.length === 0 ? (
          <EditorPanelSectionEmpty label={"The ambient playing has no sound channel."} />
        ) : (
          report.channels.map((channel) => (
            <EditorPanelProperty
              key={channel.channel}
              label={channel.channel}
              value={channel.wait > 0 ? `In ${formatLevelSoundSeconds(channel.wait)}` : "Now"}
            />
          ))
        )}
      </EditorPanelSection>

      <EditorPanelSection title={`Playing (${voices.length})`}>
        {voices.map((voice: WorldSoundVoiceReport) => {
          const acoustics: string = voice.isReal ? formatLevelSoundAcoustics(voice) : "";

          return (
            <EditorPanelProperty
              key={voice.id}
              label={voice.sound}
              value={`${voice.kind}, ${voice.distance !== null ? `${voice.distance.toFixed(1)} m` : "2D"}, ${
                voice.isReal ? "" : "simulated at "
              }${formatLevelSoundGain(voice)}${voice.restarts > 0 ? `, started over ${voice.restarts}×` : ""}${
                acoustics ? `; ${acoustics}` : ""
              }`}
            />
          );
        })}
      </EditorPanelSection>

      {report.refused.length > 0 ? (
        <EditorPanelSection title={`Refused (${report.refused.length})`}>
          {report.refused.map((refusal) => (
            <EditorPanelProperty key={refusal.sound} label={refusal.sound} value={refusal.reason} />
          ))}
        </EditorPanelSection>
      ) : null}
    </EditorPanel>
  );
}
