import { ReactElement } from "react";

import { ArchiveOmfPart } from "@/core/ipc/types/xrf-app";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";

interface IArchiveOmfPartitionSectionProps extends BaseComponentProps {
  parts: Array<ArchiveOmfPart>;
}

/**
 * The partition every cycle of the bank is routed through.
 */
export function ArchiveOmfPartitionSection({
  "data-testid": dataTestId = "archive-omf-partition-section",
  id,
  className,
  parts,
}: IArchiveOmfPartitionSectionProps): ReactElement {
  return (
    <EditorPanelSection
      data-testid={dataTestId}
      id={id}
      className={className}
      title={"Partition"}
      caption={"Parts in the order the bank declares them, which is the order their indices count in"}
    >
      {parts.map((part: ArchiveOmfPart, index: number) => (
        <ArchiveDescriptionRow
          key={`${index}-${part.name}`}
          label={part.name}
          value={`${part.bones.length} ${part.bones.length === 1 ? "bone" : "bones"} · ${part.cycles} ${
            part.cycles === 1 ? "cycle" : "cycles"
          }`}
          caption={part.bones.join(", ")}
        />
      ))}
    </EditorPanelSection>
  );
}
