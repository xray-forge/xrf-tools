import { useInjection } from "@wirestate/react";
import { WorldSoundReport, WorldSoundVoiceReport } from "@xrf/ipc/types/xrf-world";
import { Nullable } from "@xrf/types";
import { EditorPanelEmpty, EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Typography } from "@xrf/ui/material/typography";
import { ReactElement, useEffect, useState } from "react";

import { LevelLoadService } from "@/core/level/services/level-load";
import { LevelRenderService } from "@/core/level/services/level-render";
import { EditorPanel } from "@/core/shell/chrome/EditorPanel";
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
 * Asks what the level's scene plays while mounted, again every {@link LEVEL_SOUND_REFRESH_MS}.
 *
 * @param renderService - The viewport the level is shown in.
 * @returns The last answer, null until one came or while no level is shown.
 */
function useLevelSoundReport(renderService: LevelRenderService): Nullable<WorldSoundReport> {
  const [report, setReport] = useState<Nullable<WorldSoundReport>>(null);

  useEffect(() => {
    let isMounted: boolean = true;

    function ask(): void {
      void renderService.describeSound().then((next: Nullable<WorldSoundReport>) => {
        if (isMounted) {
          setReport(next);
        }
      });
    }

    ask();

    const timer: ReturnType<typeof setInterval> = setInterval(ask, LEVEL_SOUND_REFRESH_MS);

    return () => {
      isMounted = false;
      clearInterval(timer);
    };
  }, [renderService]);

  return report;
}

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

  const report: Nullable<WorldSoundReport> = useLevelSoundReport(renderService);

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

      <EditorPanelSection title={"Music"}>
        {report.music === null || report.music.tracks === 0 ? (
          <Typography className={"block text-text-secondary"} variant={"caption"}>
            The level names no music.
          </Typography>
        ) : (
          <EditorPanelProperty
            label={report.music.track ?? "Next"}
            value={
              report.music.track
                ? formatLevelSoundSeconds(report.music.position)
                : `In ${formatLevelSoundSeconds(report.music.wait)}`
            }
          />
        )}
      </EditorPanelSection>

      <EditorPanelSection title={report.ambient ? `Channels of ${report.ambient}` : "Channels"}>
        {report.channels.length === 0 ? (
          <Typography className={"block text-text-secondary"} variant={"caption"}>
            The ambient playing has no sound channel.
          </Typography>
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
