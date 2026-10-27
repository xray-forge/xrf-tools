import { useInjection } from "@wirestate/react";
import { filterByName } from "@xrf/search";
import { Nullable } from "@xrf/types";
import { EditorFilterInput } from "@xrf/ui/editor/filter-input";
import { EditorPanelEmpty } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Typography } from "@xrf/ui/material";
import { ReactElement, useMemo, useState } from "react";

import { SequenceMotionRow } from "@/applications/visuals-sequencer/components/panels/SequenceMotionsPanel/SequenceMotionRow";
import { VisualSequenceService } from "@/applications/visuals-sequencer/services/visual-sequence";
import { VisualsSequencerService } from "@/applications/visuals-sequencer/services/visuals-sequencer";
import { EditorPanel } from "@/core/shell/editor/EditorPanel";

/** How many matches are drawn at once. */
const SHOWN_LIMIT: number = 200;

/**
 * Every motion the open visual can play, filtered by name, each one addable to the track.
 */
export function SequenceMotionsPanel({
  "data-testid": dataTestId = "sequence-motions-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const sequencerService: VisualsSequencerService = useInjection(VisualsSequencerService);
  const sequenceService: VisualSequenceService = useInjection(VisualSequenceService);

  const [filter, setFilter] = useState<string>("");

  const listed: Nullable<Array<string>> = sequencerService.motions.value;
  const names: Array<string> = listed ?? [];

  const matched: Array<string> = useMemo(() => {
    return filterByName(listed ?? [], filter, (name: string) => name);
  }, [filter, listed]);

  /** How many clips already name each motion, so a track built by clicking the same row twice says so. */
  const used: Map<string, number> = useMemo(() => {
    const counts: Map<string, number> = new Map();

    for (const clip of sequenceService.clips) {
      counts.set(clip.motion, (counts.get(clip.motion) ?? 0) + 1);
    }

    return counts;
  }, [sequenceService.clips]);

  if (sequencerService.motions.isLoading) {
    return (
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Motions"}>
        <EditorPanelEmpty label={"Listing motions. Every animation file the visual references is read once."} />
      </EditorPanel>
    );
  }

  if (!names.length) {
    return (
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Motions"}>
        <EditorPanelEmpty
          label={
            sequencerService.motions.error?.message ??
            "No motions. This visual references no animation files, so there is nothing to sequence."
          }
        />
      </EditorPanel>
    );
  }

  return (
    <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Motions"}>
      <div className={"sticky top-0 z-1 border-b border-divider surface-frame px-4 py-3"}>
        <EditorFilterInput
          query={filter}
          placeholder={"Filter motions"}
          ariaLabel={"Filter motions"}
          onQueryChange={setFilter}
        />
      </div>

      <div className={"p-2"}>
        {matched.slice(0, SHOWN_LIMIT).map((name: string) => (
          <SequenceMotionRow key={name} motion={name} usageCount={used.get(name) ?? 0} onAdd={sequenceService.add} />
        ))}

        {matched.length === 0 ? (
          <EditorPanelEmpty label={`No motion of the ${names.length} this visual plays matches that.`} />
        ) : null}

        {matched.length > SHOWN_LIMIT ? (
          <Typography className={"block p-2 text-text-disabled"} variant={"caption"}>
            {`Showing ${SHOWN_LIMIT} of ${matched.length} matches. Narrow the filter to reach the rest.`}
          </Typography>
        ) : null}
      </div>
    </EditorPanel>
  );
}
