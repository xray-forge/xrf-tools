import { ReactElement } from "react";

import {
  ILevelStreamStages,
  ILevelStreamSummary,
  LEVEL_STREAM_STAGES,
} from "@/core/level/lib/stream/level-stream-profile";
import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { formatDuration } from "@/lib/format/duration";

interface ILevelStreamReadingSectionProps {
  stream: ILevelStreamSummary;
}

/**
 * What reading a sector costs, stage by stage, and what answering the camera costs: nothing until something was read.
 */
export function LevelStreamReadingSection({ stream }: ILevelStreamReadingSectionProps): ReactElement {
  return (
    <>
      {stream.mean ? (
        <EditorPanelSection title={"Reading a sector"}>
          {LEVEL_STREAM_STAGES.map((stage: keyof ILevelStreamStages) => (
            <EditorPanelProperty key={stage} label={stage} value={formatDuration(stream.mean?.[stage] ?? 0)} />
          ))}
          <EditorPanelProperty
            label={"per texture file"}
            value={
              stream.last?.files
                ? `${formatDuration(stream.last.textures / stream.last.files)} · ${stream.last.files} files`
                : "none read"
            }
          />
          <EditorPanelProperty label={"Mean"} value={formatDuration(stream.mean.total)} />
          <EditorPanelProperty
            label={"Worst"}
            value={
              stream.worst
                ? `${formatDuration(stream.worst.total)} · sector ${stream.worst.sector}`
                : "nothing read yet"
            }
          />
          <EditorPanelProperty label={"Sectors read"} value={stream.sectors} />
        </EditorPanelSection>
      ) : null}

      {stream.planning.reports ? (
        <EditorPanelSection title={"Answering the camera"}>
          <EditorPanelProperty label={"Mean"} value={formatDuration(stream.planning.mean)} />
          <EditorPanelProperty label={"Reports"} value={stream.planning.reports} />
        </EditorPanelSection>
      ) : null}
    </>
  );
}
