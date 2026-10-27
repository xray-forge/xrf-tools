import { ArchiveDetailEntry } from "@xrf/ipc/types/xrf-app-archives";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";

import { ArchiveDetailObjectRow } from "./ArchiveDetailObjectRow";

interface IArchiveDetailObjectsSectionProps extends BaseComponentProps {
  entries: Array<ArchiveDetailEntry>;
}

/**
 * The library the grid plants from, in the order the file numbers it.
 */
export function ArchiveDetailObjectsSection({
  "data-testid": dataTestId = "archive-detail-objects-section",
  id,
  className,
  entries,
}: IArchiveDetailObjectsSectionProps): ReactElement {
  return (
    <EditorPanelSection
      data-testid={dataTestId}
      id={id}
      className={className}
      title={`Objects (${entries.length})`}
      caption={"Planted corners rather than slots: one slot plants up to four objects"}
    >
      {entries.map((entry: ArchiveDetailEntry) => (
        <ArchiveDetailObjectRow key={entry.index} entry={entry} />
      ))}
    </EditorPanelSection>
  );
}
