import { ReactElement } from "react";

import { ArchiveAnmDescription } from "@/core/ipc/types/xrf-app";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatSeconds } from "@/lib/format/duration";
import { formatNumber } from "@/lib/format/number";

import { NOT_DECLARED } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { describeKeyedReach } from "./ArchiveAnmDescriptionView.utils";

/** What every shipped animation is authored at, named only where a file departs from it. */
const DEFAULT_FPS: number = 30;

interface IArchiveAnmMotionSectionProps extends BaseComponentProps {
  description: ArchiveAnmDescription;
}

/**
 * What the animation is, taken over the whole of it.
 */
export function ArchiveAnmMotionSection({
  "data-testid": dataTestId = "archive-anm-motion-section",
  id,
  className,
  description,
}: IArchiveAnmMotionSectionProps): ReactElement {
  const { frameStart, frameEnd, frames, fps } = description;

  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Motion"} isFirst>
      <ArchiveDescriptionRow
        label={"Saved as"}
        value={description.name ?? NOT_DECLARED}
        isMonospace={Boolean(description.name)}
        caption={description.name ? null : "Most animations carry no name of their own, and are loaded by path"}
      />

      <ArchiveDescriptionRow
        label={"Plays for"}
        value={formatSeconds(description.durationSeconds)}
        caption={`Frames ${frameStart} to ${frameEnd} inclusive, which is ${frames} at ${formatNumber(fps, 0)} fps`}
      />

      <ArchiveDescriptionRow
        label={"Keys"}
        value={`${description.keys}`}
        caption={describeKeyedReach(description) ?? "Across the six channels below"}
      />

      <ArchiveDescriptionRow
        label={"Version"}
        value={`${description.version}`}
        caption={
          fps === null || fps === DEFAULT_FPS
            ? null
            : `Authored at ${formatNumber(fps, 2)} fps rather than the usual 30`
        }
      />
    </EditorPanelSection>
  );
}
