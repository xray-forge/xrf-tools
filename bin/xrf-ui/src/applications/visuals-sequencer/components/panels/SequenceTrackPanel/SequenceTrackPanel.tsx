import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { SequenceClipRow } from "@/applications/visuals-sequencer/components/panels/SequenceTrackPanel/SequenceClipRow";
import { ISequenceClip, VisualSequenceService } from "@/applications/visuals-sequencer/services/sequence";
import { VisualPanel, VisualPanelEmpty, VisualPanelRow, VisualPanelSection } from "@/core/visuals/components/panels";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatDuration } from "@/lib/format/duration";

/**
 * The ordered track: every clip in the order it plays, and what the whole of it adds up to.
 */
export function SequenceTrackPanel({
  "data-testid": dataTestId = "sequence-track-panel",
  id,
  className,
}: BaseComponentProps = {}): ReactElement {
  const service: VisualSequenceService = useInjection(VisualSequenceService);

  const clips: ReadonlyArray<ISequenceClip> = service.clips;

  if (!clips.length) {
    return (
      <VisualPanel data-testid={dataTestId} id={id} className={className} title={"Sequence"}>
        <VisualPanelEmpty label={"No clips yet. Add motions from the panel on the left to build a track."} />
      </VisualPanel>
    );
  }

  return (
    <VisualPanel data-testid={dataTestId} id={id} className={className} title={"Sequence"}>
      <VisualPanelSection
        title={`Track (${clips.length})`}
        caption={"Played in order, cutting at each boundary"}
        isFirst
      >
        {clips.map((clip: ISequenceClip, position: number) => (
          <SequenceClipRow key={clip.id} clip={clip} position={position} length={clips.length} />
        ))}
      </VisualPanelSection>

      <VisualPanelSection title={"Totals"} caption={"As the baked clips report themselves"}>
        <VisualPanelRow label={"Playable clips"} value={`${service.playableCount} / ${clips.length}`} />
        <VisualPanelRow label={"Duration"} value={formatDuration(Math.round(service.duration * 1000))} />
      </VisualPanelSection>
    </VisualPanel>
  );
}
